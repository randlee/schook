#!/usr/bin/env python3
"""Capture Claude WorktreeCreate payloads and optionally block unauthorized roots."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

from _capture_common import write_capture


REDIRECT_MESSAGE = (
    "WorktreeCreate hook rejected: use /sc-git-worktree instead of "
    "EnterWorktree directly"
)


def main() -> int:
    raw_text = sys.stdin.read()
    write_capture("worktree-create", raw_text)

    try:
        payload = json.loads(raw_text or "{}")
    except Exception:
        print("WorktreeCreate hook rejected: invalid JSON payload", file=sys.stderr)
        return 1

    allowed_root = os.environ.get("SCHOOK_WORKTREE_ALLOWED_ROOT", "").strip()
    create_root = os.environ.get("SCHOOK_WORKTREE_CREATE_ROOT", "").strip()
    if not allowed_root and not create_root:
        return 0

    cwd = Path(payload.get("cwd", "")).expanduser()
    allowed = Path(allowed_root).expanduser().resolve() if allowed_root else None

    try:
        resolved_cwd = cwd.resolve()
    except Exception:
        print("WorktreeCreate hook rejected: invalid cwd", file=sys.stderr)
        return 1

    if allowed and not (allowed == resolved_cwd or allowed in resolved_cwd.parents):
        print(REDIRECT_MESSAGE, file=sys.stderr)
        return 1

    if create_root:
        name = str(payload.get("name", "")).strip()
        if not name:
            print("WorktreeCreate hook rejected: missing worktree name", file=sys.stderr)
            return 1
        target = Path(create_root).expanduser().resolve() / name
        target.mkdir(parents=True, exist_ok=True)
        print(target)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
