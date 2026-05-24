#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


TARGETS = {
    "local-cutover": ["cargo", "run", "-p", "sc-hooks-cli", "--", "install"],
}


def main(argv: list[str]) -> int:
    repo_root = Path(__file__).resolve().parent.parent
    target = argv[1] if len(argv) > 1 else "local-cutover"

    command = TARGETS.get(target)
    if command is None:
        valid = ", ".join(sorted(TARGETS))
        print(f"unknown install target: {target}", file=sys.stderr)
        print(f"expected one of: {valid}", file=sys.stderr)
        return 2

    return subprocess.run(command, cwd=repo_root).returncode


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
