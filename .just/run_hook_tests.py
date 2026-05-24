#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


HOOK_TESTS = {
    "claude": ["pytest", "test-harness/hooks/claude/tests/", "-q"],
    "codex": ["pytest", "test-harness/hooks/codex/tests/", "-q"],
    "gemini": ["pytest", "test-harness/hooks/gemini/tests/", "-q"],
}


def main(argv: list[str]) -> int:
    root = Path(__file__).resolve().parent.parent
    provider = argv[1] if len(argv) > 1 else ""
    command = HOOK_TESTS.get(provider)
    if command is None:
        valid = ", ".join(sorted(HOOK_TESTS))
        print(f"unknown hooks provider: {provider}", file=sys.stderr)
        print(f"expected one of: {valid}", file=sys.stderr)
        return 2
    return subprocess.run(command, cwd=root).returncode


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
