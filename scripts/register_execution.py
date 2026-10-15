#!/usr/bin/env python3
"""Register a reviewed manual observation or resume an identical staged publication."""

import argparse
from pathlib import Path
import sys

if __package__:
    from .validate_register import ROOT, read_yaml
    from .validation_runtime import publish_run, read_artifact_map
else:
    from validate_register import ROOT, read_yaml
    from validation_runtime import publish_run, read_artifact_map


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--evidence-dir', type=Path, required=True)
    parser.add_argument('--artifact-map')
    parser.add_argument('--artifact-root', type=Path)
    parser.add_argument('--resume', action='store_true', help='complete publication of byte-equivalent staged records; never change an existing record')
    args = parser.parse_args()
    try:
        run = read_yaml(args.run)
        evidence = [read_yaml(path) for path in sorted(args.evidence_dir.glob('*.yaml'))]
        publish_run(args.root.resolve(), run, evidence, read_artifact_map(args.artifact_map), args.artifact_root, args.resume)
    except Exception as exc:
        print(f'Execution registration failed: {exc}', file=sys.stderr)
        return 1
    print(f"Registered {run['id']} with recorded outcome {run['outcome']}; no validation acceptance inferred.")
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
