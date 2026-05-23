#!/usr/bin/env python3

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[4]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from test_harness.hooks.codex.debounce import write_capture


def main() -> int:
    raw_text = sys.stdin.read()
    write_capture("session-start", raw_text)

    forward_path = os.environ.get("SCHOOK_CODEX_SESSION_START_FORWARD_PATH", "").strip()
    if forward_path:
        subprocess.run(
            [sys.executable, forward_path],
            input=raw_text,
            text=True,
            check=False,
            env=os.environ.copy(),
        )
    return 0


if __name__ == "__main__":
    # Codex can replay SessionStart hooks; capture + forward behavior must remain idempotent.
    raise SystemExit(main())
