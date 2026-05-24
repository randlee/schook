#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


WORKSPACE_TEST = [
    ["cargo", "build", "--workspace"],
    ["cargo", "test", "--workspace"],
]

HOOK_TESTS = {
    "claude": ["pytest", "test-harness/hooks/claude/tests/", "-q"],
    "codex": ["pytest", "test-harness/hooks/codex/tests/", "-q"],
    "gemini": ["pytest", "test-harness/hooks/gemini/tests/", "-q"],
}


def main(argv: list[str]) -> int:
    repo_root = Path(__file__).resolve().parent.parent
    target = argv[1] if len(argv) > 1 else "workspace"
    provider = argv[2] if len(argv) > 2 else ""

    if target == "workspace":
        for command in WORKSPACE_TEST:
            completed = subprocess.run(command, cwd=repo_root)
            if completed.returncode != 0:
                return completed.returncode
        return 0

    if target == "hooks":
        command = HOOK_TESTS.get(provider)
        if command is None:
            valid = ", ".join(sorted(HOOK_TESTS))
            print(f"unknown hooks provider: {provider}", file=sys.stderr)
            print(f"expected one of: {valid}", file=sys.stderr)
            return 2
        return subprocess.run(command, cwd=repo_root).returncode

    print(f"unknown test target: {target}", file=sys.stderr)
    print("expected one of: workspace, hooks", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
