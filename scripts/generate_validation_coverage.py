#!/usr/bin/env python3
"""Compatibility entrypoint for the complete validation documentation generator."""

if __package__:
    from .generate_validation_docs import main
else:
    from generate_validation_docs import main

if __name__ == '__main__':
    raise SystemExit(main())
