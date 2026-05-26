#!/usr/bin/env python3

from __future__ import annotations

import json
import os
from datetime import datetime, timezone
from pathlib import Path


def main() -> int:
    target = os.environ.get("SCHOOK_CODEX_DEBOUNCE_RECORD_PATH", "").strip()
    if target:
        path = Path(target).expanduser().resolve()
    else:
        state_root = os.environ.get("SCHOOK_CODEX_HOOK_STATE_ROOT", "").strip()
        if state_root:
            path = Path(state_root).expanduser().resolve() / "tool-invocations.jsonl"
        else:
            path = Path.cwd() / ".temp" / "codex-hook-test" / "tool-invocations.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "cwd": os.getcwd(),
        "ts": datetime.now(timezone.utc).isoformat(),
    }
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(payload, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
