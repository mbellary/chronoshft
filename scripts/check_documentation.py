#!/usr/bin/env python3
"""Run the same fail-fast documentation and registry gate locally and in CI."""

import argparse
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
CHECKS = (
    ('Registry schemas, references and mandatory coverage', ('scripts/validate_register.py', '--require-tests')),
    ('Git history and immutable ledgers', ('scripts/check_validation_history.py',)),
    ('Pinned execution, artifact integrity and decision evidence', ('scripts/verify_evidence.py',)),
    ('Generated Lab pages and navigation', ('scripts/generate_validation_docs.py', '--check')),
    ('Registry, documentation and CI regression checks', ('-m', 'unittest', 'discover', '-s', 'tests', '-p', 'test_validation*.py')),
    ('Strict MkDocs build', ('-m', 'mkdocs', 'build', '--strict')),
    ('Rendered local links and anchors', ('scripts/check_documentation_links.py',)),
)


def run_checks(root=ROOT):
    root = root.resolve()
    for index, (label, arguments) in enumerate(CHECKS, 1):
        print(f'[{index}/{len(CHECKS)}] {label}', flush=True)
        try:
            result = subprocess.run([sys.executable, *arguments], cwd=root, check=False)
        except OSError as exc:
            print(f'{label}: could not start check: {exc}', file=sys.stderr)
            return 1
        if result.returncode:
            print(f'{label}: failed; remaining checks were not run.', file=sys.stderr)
            return result.returncode if result.returncode > 0 else 1
    print('Documentation and registry gates passed. Experiment execution and validation acceptance are separate gates.')
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT, help='repository root (default: script repository)')
    return run_checks(parser.parse_args().root)


if __name__ == '__main__':
    raise SystemExit(main())
