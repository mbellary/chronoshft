#!/usr/bin/env python3
"""Build EXP-001, then run exactly one declared case with distinct tool failures."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CASES = {"fifo_exactly_once": "TST-001", "capacity_before_write": "TST-002",
         "wraparound_partial_drains": "TST-003", "capacity_configuration": "TST-004"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("case", choices=CASES)
    case = parser.parse_args().case
    report_dir = ROOT / os.environ.get("CHRONOSHFT_REPORT_DIR", "target/validation-reports")
    (report_dir / f"{CASES[case]}.txt").unlink(missing_ok=True)
    command = ["cargo", "test", "--offline", "--locked", "--manifest-path", "experiments/Cargo.toml",
               "-p", "exp-001-spsc-queue", "--test", "queue_contracts", "--no-run", "--message-format=json"]
    print("Build:", " ".join(command), flush=True)
    try:
        build = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
        print(build.stderr, end="", file=sys.stderr)
        executables = []
        for line in build.stdout.splitlines():
            item = json.loads(line)
            if item.get("reason") == "compiler-message":
                print(item["message"].get("rendered", ""), end="", file=sys.stderr)
            if item.get("reason") == "compiler-artifact" and item.get("target", {}).get("name") == "queue_contracts" and item.get("executable"):
                executables.append(item["executable"])
        if build.returncode or len(executables) != 1:
            print("Tool error: build failed or test executable is ambiguous", file=sys.stderr)
            return 2
        listing = subprocess.run([executables[0], "--list"], cwd=ROOT, capture_output=True, text=True, check=False)
        if listing.returncode or f"{case}: test" not in listing.stdout.splitlines():
            print("Tool error: requested test is missing", file=sys.stderr)
            return 2
        command = [executables[0], case, "--exact", "--nocapture"]
        print("Execute:", " ".join(command), flush=True)
        result = subprocess.run(command, cwd=ROOT, check=False)
        # Rust test-harness assertion failure is 101. Build/launch/list failures
        # are separately reported as 2, never mislabeled as hypothesis failure.
        return result.returncode if result.returncode in (0, 101) else 2
    except (OSError, ValueError) as exc:
        print(f"Tool error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
