#!/usr/bin/env python3
"""Print actual compiler and package-manager identities for experiment capture."""
import subprocess

for command in (["rustc", "--version", "--verbose"], ["cargo", "--version", "--verbose"]):
    print(" ".join(command), flush=True)
    subprocess.run(command, check=True)
