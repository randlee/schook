#!/usr/bin/env python3
"""Thin wrapper that preserves the plan path while delegating to the package."""

from test_harness.hooks.run_schema_drift import main


if __name__ == "__main__":
    raise SystemExit(main())
