#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


WORKSPACE_TEST = [["cargo", "build", "--workspace"], ["cargo", "test", "--workspace"]]
HOOK_PROVIDERS = {"claude", "codex", "gemini"}


def main(argv: list[str]) -> int:
    root = Path(__file__).resolve().parent.parent
    target = argv[1] if len(argv) > 1 else "workspace"
    provider = argv[2] if len(argv) > 2 else ""

    if target == "workspace":
        if provider:
            raise SystemExit("workspace test does not accept a provider")
        for command in WORKSPACE_TEST:
            completed = subprocess.run(command, cwd=root)
            if completed.returncode != 0:
                return completed.returncode
        return 0

    if target == "hooks":
        if provider not in HOOK_PROVIDERS:
            valid = ", ".join(sorted(HOOK_PROVIDERS))
            raise SystemExit(f"hooks test requires one of: {valid}")
        command = [sys.executable, str(root / ".just" / "run_hook_tests.py"), provider]
        return subprocess.run(command, cwd=root).returncode

    raise SystemExit("expected test target: workspace or hooks")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
