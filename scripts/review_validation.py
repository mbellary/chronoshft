#!/usr/bin/env python3
"""Prepare an unapproved review package; verify or explicitly register a human decision."""

import argparse
from pathlib import Path
import sys

if __package__:
    from .validate_register import ROOT, read_yaml
    from .validation_runtime import (PinnedDefinitions, git, immutable_write, load_records,
        definition, reference, read_artifact_map, verify_decision, yaml_text)
else:
    from validate_register import ROOT, read_yaml
    from validation_runtime import (PinnedDefinitions, git, immutable_write, load_records,
        definition, reference, read_artifact_map, verify_decision, yaml_text)


def prepare(root, validation_id, decision_id, output):
    records = load_records(root)
    val = records[validation_id]
    commit = git(root, 'rev-parse', 'HEAD').decode().strip()
    pinned = PinnedDefinitions(root).record(reference(validation_id, val['revision'], commit))
    runs = [run for run in records.values() if run['id'].startswith('RUN-') and run['experiment']['id'] in {ref['id'] for ref in pinned['coverage']['experiments']}]
    source_commits = {run['experiment']['registry_commit'] for run in runs}
    if len(source_commits) == 1:
        observed_commit = next(iter(source_commits))
        observed = PinnedDefinitions(root).record(reference(validation_id, val['revision'], observed_commit))
        if definition(observed) == definition(val):
            commit = observed_commit
    evidence = sorted({identity for run in runs for identity in run['evidence_ids']})
    candidate = {'schema_version': 1, 'id': decision_id, 'validation': reference(validation_id, val['revision'], commit),
                 'disposition': None, 'rationale': None, 'execution_ids': [run['id'] for run in runs], 'evidence_ids': evidence,
                 'coverage_assessment': None, 'evidence_assessment': [{'requirement': requirement, 'evidence_ids': [], 'rationale': None} for requirement in val['required_evidence']],
                 'unresolved_findings': [], 'applicability': {'implementation_commit': None, 'workload': None, 'hardware': None, 'dependencies': None, 'exclusions': [], 'limitations': []},
                 'reviewer': None, 'reviewed_at': None, 'architecture_spec_actions': [], 'follow_up': [], 'supersedes': val['current_decision_id']}
    draft = {'draft': True, 'instructions': 'Unapproved package. Select applicable runs/evidence; complete every field and the evidence assessment. A human reviewer must attest the decision. Set draft: false only after that review.',
             'decision': candidate, 'mandatory_experiments': val['coverage']['experiments'], 'mandatory_tests': val['coverage']['tests']}
    output = output.resolve()
    if output.is_relative_to((root / 'validation/decisions').resolve()):
        raise ValueError('Unapproved drafts must remain outside the decision registry')
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('x') as stream:
        stream.write(yaml_text(draft))
    return draft


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    commands = parser.add_subparsers(dest='mode', required=True)
    draft = commands.add_parser('prepare')
    draft.add_argument('--validation', required=True)
    draft.add_argument('--decision-id', required=True)
    draft.add_argument('--output', type=Path, required=True)
    review = commands.add_parser('check')
    review.add_argument('--candidate', type=Path, required=True)
    review.add_argument('--artifact-map')
    review.add_argument('--artifact-root', type=Path)
    review.add_argument('--register-reviewed', action='store_true', help='explicitly attest that the named human review happened; append the decision without changing lifecycle/current decision')
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        if args.mode == 'prepare':
            prepare(root, args.validation, args.decision_id, args.output)
            print('Prepared an unapproved review package outside the registry.')
            return 0
        candidate = read_yaml(args.candidate)
        if 'decision' in candidate:
            if candidate.get('draft') is not False:
                raise ValueError('Draft review packages cannot be registered or treated as approved')
            candidate = candidate['decision']
        records = load_records(root)
        resolver = PinnedDefinitions(root)
        verify_decision(root, candidate, resolver, records, read_artifact_map(args.artifact_map), args.artifact_root, require_current=True)
        if args.register_reviewed:
            if records[candidate['validation']['id']]['lifecycle']['state'] != 'reviewing':
                raise ValueError('The validation must explicitly be in reviewing state before decision registration')
            if candidate['id'] in records:
                raise ValueError('Decision IDs cannot be overwritten or reused')
            predecessor = candidate['supersedes']
            if predecessor and (predecessor not in records or records[predecessor].get('validation', {}).get('id') != candidate['validation']['id']):
                raise ValueError('Supersession must reference a decision for the same validation')
            immutable_write(root / 'validation/decisions' / f"{candidate['id']}.yaml", yaml_text(candidate))
            print('Registered the supplied human decision. Current-decision selection and lifecycle changes remain explicit record edits.')
        else:
            print('Decision checks passed; no decision or approval was registered.')
        return 0
    except Exception as exc:
        print(f'Review failed: {exc}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
