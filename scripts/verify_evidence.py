#!/usr/bin/env python3
"""Verify retained content, pinned execution definitions and recorded decisions."""

import argparse
import os
from pathlib import Path
import sys

if __package__:
    from .validate_register import ROOT
    from .validation_runtime import (PinnedDefinitions, load_records, read_artifact_map,
        git, reference, validate_record, verify_decision, verify_evidence, verify_run)
    from .validate_register import read_yaml
else:
    from validate_register import ROOT
    from validation_runtime import (PinnedDefinitions, load_records, read_artifact_map,
        git, reference, validate_record, verify_decision, verify_evidence, verify_run)
    from validate_register import read_yaml


def verify_all(root, artifact_map=None, artifact_root=None):
    records, resolver = load_records(root), PinnedDefinitions(root)
    errors = []
    for path in sorted((root / 'validation/execution-plans').glob('*.yaml')):
        try:
            plan = read_yaml(path)
            validate_record(root, 'capture-plan', plan)
            ref = dict(plan['experiment'])
            if ref['registry_commit'] == 'HEAD':
                ref['registry_commit'] = git(root, 'rev-parse', 'HEAD').decode().strip()
            experiment = resolver.record(ref)
            expected = {item['id']: item['revision'] for item in experiment['tests']}
            selected = [test['id'] for test in plan['tests']]
            if len(selected) != len(set(selected)):
                raise ValueError('Duplicate capture-plan test identity')
            for item in plan['tests']:
                if expected.get(item['id']) != item['revision']:
                    raise ValueError('Capture plan references a foreign/stale test')
                resolver.record(reference(item['id'], item['revision'], ref['registry_commit']))
        except Exception as exc:
            errors.append(f'{path.relative_to(root)}: {exc}')
    for record in records.values():
        try:
            if record['id'].startswith('EVD-'):
                verify_evidence(root, record, resolver, records, artifact_map, artifact_root)
            elif record['id'].startswith('RUN-'):
                verify_run(root, record, resolver, records, artifact_map, artifact_root)
            elif record['id'].startswith('VDEC-'):
                verify_decision(root, record, resolver, records, artifact_map, artifact_root)
            if record.get('supersedes'):
                predecessor = records.get(record['supersedes'])
                if predecessor is None or predecessor['id'].split('-')[0] != record['id'].split('-')[0] or predecessor['id'] == record['id']:
                    raise ValueError('Invalid supersession predecessor')
                visited, cursor = {record['id']}, predecessor
                while cursor.get('supersedes'):
                    if cursor['id'] in visited:
                        raise ValueError('Supersession cycle detected')
                    visited.add(cursor['id'])
                    cursor = records[cursor['supersedes']]
        except Exception as exc:
            errors.append(f"{record['id']}: {exc}")
    return errors, records


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--artifact-map', default=os.environ.get('VALIDATION_ARTIFACT_MAP'))
    parser.add_argument('--artifact-root', type=Path, help='local mirror root containing repo:// paths')
    args = parser.parse_args()
    try:
        errors, records = verify_all(args.root.resolve(), read_artifact_map(args.artifact_map), args.artifact_root)
    except Exception as exc:
        print(f'Evidence verification failed: {exc}', file=sys.stderr)
        return 1
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    counts = {prefix: sum(identity.startswith(prefix + '-') for identity in records) for prefix in ('RUN', 'EVD', 'VDEC')}
    print(f"Verified {counts['RUN']} executions, {counts['EVD']} artifact manifests and {counts['VDEC']} decisions. No technical approval is inferred.")
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
