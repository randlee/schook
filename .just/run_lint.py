#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


TARGETS = {
    "fmt": ["just", "_lint-fmt"],
    "clippy": ["just", "_lint-clippy"],
    "modules": ["just", "_lint-modules"],
    "deny": ["just", "_lint-deny"],
    "shear": ["just", "_lint-shear"],
    "version": ["just", "_lint-version"],
    "manifests": ["just", "_lint-manifests"],
    "spell": ["just", "_lint-spell"],
    "pytests": ["just", "_lint-pytests"],
    "boundary": ["just", "_lint-sc-boundary"],
    "portability": ["just", "_lint-portability"],
}

ALL_ORDER = (
    "fmt",
    "clippy",
    "modules",
    "deny",
    "shear",
    "version",
    "manifests",
    "spell",
    "pytests",
    "boundary",
    "portability",
)


def main(argv: list[str]) -> int:
    repo_root = Path(__file__).resolve().parent.parent
    target = argv[1] if len(argv) > 1 else "all"

    if target == "all":
        for name in ALL_ORDER:
            completed = subprocess.run(TARGETS[name], cwd=repo_root)
            if completed.returncode != 0:
                return completed.returncode
        return 0

    command = TARGETS.get(target)
    if command is None:
        valid = ", ".join(("all", *TARGETS.keys()))
        print(f"unknown lint target: {target}", file=sys.stderr)
        print(f"expected one of: {valid}", file=sys.stderr)
        return 2

    return subprocess.run(command, cwd=repo_root).returncode


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
