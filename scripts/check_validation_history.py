#!/usr/bin/env python3
"""Compare registry changes with a trusted Git baseline and protect immutable ledgers."""

import argparse
import json
import os
from pathlib import Path
import sys

if __package__:
    from .validation_runtime import definition, git, load_text
    from .validate_register import ROOT, read_yaml
else:
    from validation_runtime import definition, git, load_text
    from validate_register import ROOT, read_yaml

IMMUTABLE = ('validation/executions/', 'validation/evidence-manifests/', 'validation/decisions/',
             'validation/revisions/', 'validation/artifacts/')
SOURCE = 'docs/assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md'
BASELINE_TABLE = 'docs/00-governance/technical-validation.md'


def automatic_base(root):
    event_path, event_name = os.environ.get('GITHUB_EVENT_PATH'), os.environ.get('GITHUB_EVENT_NAME')
    if event_path and event_name:
        event = json.loads(Path(event_path).read_text())
        if event_name == 'pull_request':
            return event['pull_request']['base']['sha']
        if event_name == 'push':
            before = event['before']
            return None if set(before) == {'0'} else before
        if event_name == 'workflow_dispatch':
            try:
                return git(root, 'rev-parse', '--verify', 'HEAD^').decode().strip()
            except ValueError:
                return None
    return 'HEAD'


def prefix(old, new):
    return new[:len(old)] == old


def check_history(root, base):
    if base is None:
        return []
    commit = git(root, 'rev-parse', '--verify', base + '^{commit}').decode().strip()
    paths = git(root, 'ls-tree', '-r', '--name-only', commit).decode().splitlines()
    errors = []
    for relative in paths:
        path = root / relative
        if relative.startswith(IMMUTABLE) or relative in {'validation/baseline.yaml', SOURCE}:
            if not path.is_file() or path.read_bytes() != git(root, 'show', f'{commit}:{relative}'):
                errors.append(f'{relative}: immutable content changed or was removed; append a correction with a new identity')
        if relative.startswith('validation/register/') and relative.endswith('.yaml'):
            if not path.is_file():
                errors.append(f'{relative}: stable registry records cannot be removed')
                continue
            old = load_text(git(root, 'show', f'{commit}:{relative}'))
            new = read_yaml(path)
            if old['id'] != new['id'] or new['revision'] < old['revision']:
                errors.append(f'{relative}: identity changed or revision decreased')
                continue
            if old.get('origin') != new.get('origin'):
                errors.append(f'{relative}: historical provenance was rewritten')
            if not prefix(old['revision_history'], new['revision_history']):
                errors.append(f'{relative}: recorded revision history was rewritten')
            if old['revision'] == new['revision'] and definition(old) != definition(new):
                errors.append(f'{relative}: definition changes require a new revision')
            if new['revision'] > old['revision']:
                folder = relative.split('/')[2]
                archive = root / 'validation/revisions' / folder / old['id'] / f"r{old['revision']}.yaml"
                if not archive.is_file():
                    errors.append(f'{relative}: previous definition snapshot is missing')
                else:
                    prior = read_yaml(archive)
                    if definition(prior) != definition(old) or not prefix(old['revision_history'], prior['revision_history']):
                        errors.append(f'{relative}: previous definition snapshot differs from the Git baseline')
                    if 'lifecycle' in old and not prefix(old['lifecycle']['history'], prior['lifecycle']['history']):
                        errors.append(f'{relative}: snapshot rewrote lifecycle history')
            if 'lifecycle' in old:
                reset = (relative.startswith('validation/register/experiments/') and new['revision'] > old['revision']
                         and old['lifecycle']['state'] == 'completed' and new['lifecycle']['state'] == 'proposed')
                if not reset and not prefix(old['lifecycle']['history'], new['lifecycle']['history']):
                    errors.append(f'{relative}: lifecycle transitions were rewritten instead of appended')
            if old.get('current_decision_id') and old['current_decision_id'] != new.get('current_decision_id'):
                if old['current_decision_id'] not in new.get('prior_decision_ids', []):
                    errors.append(f'{relative}: previous decision must remain in prior_decision_ids')
    if BASELINE_TABLE in paths:
        old_rows = [line for line in git(root, 'show', f'{commit}:{BASELINE_TABLE}').decode().splitlines() if line.startswith('| VAL-')]
        current_rows = (root / BASELINE_TABLE).read_text().splitlines() if (root / BASELINE_TABLE).exists() else []
        if any(row not in current_rows for row in old_rows):
            errors.append(f'{BASELINE_TABLE}: original research rows were changed or removed')
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--base-ref', help='trusted baseline; defaults to event base SHA in CI or HEAD locally')
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        base = args.base_ref or automatic_base(root)
        errors = check_history(root, base)
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(f'History verification failed: {exc}', file=sys.stderr)
        return 1
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print('Registry history and immutable ledgers match the Git baseline.' if base else 'Initial repository event: no earlier Git baseline; current schema/coverage checks still apply.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
