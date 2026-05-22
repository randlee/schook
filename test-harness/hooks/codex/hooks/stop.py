#!/usr/bin/env python3

from __future__ import annotations

import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[4]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from test_harness.hooks.codex.debounce import schedule_stop


if __name__ == "__main__":
    schedule_stop(sys.stdin.read())
    raise SystemExit(0)
