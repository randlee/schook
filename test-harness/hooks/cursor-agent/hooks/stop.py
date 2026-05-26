#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import sys
from datetime import UTC, datetime
from pathlib import Path


def _capture_root() -> Path:
    configured = os.environ.get("SCHOOK_HOOK_CAPTURE_ROOT")
    if configured:
        root = Path(configured)
    else:
        root = Path(__file__).resolve().parents[1] / "captures" / "raw"
    root.mkdir(parents=True, exist_ok=True)
    return root


def _timestamp() -> str:
    return datetime.now(UTC).strftime("%Y%m%dT%H%M%S.%fZ")


def _hook_name() -> str:
    override = os.environ.get("HOOK_NAME_OVERRIDE")
    if override:
        return override
    return Path(__file__).stem.replace("_", "-")


def _write_json(path: Path, payload: dict[str, object]) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    text = sys.stdin.read().strip()
    payload = json.loads(text) if text else {}
    hook_name = _hook_name()
    root = _capture_root()
    prefix = f"{_timestamp()}-{hook_name}"

    _write_json(root / f"{prefix}.json", payload if isinstance(payload, dict) else {"payload": payload})
    _write_json(
        root / f"{prefix}.env.json",
        {
            "hook_name": hook_name,
            "cursor_env": {
                key: value
                for key, value in os.environ.items()
                if key.startswith("CURSOR_")
            },
            "process_env": {
                "HOME": os.environ.get("HOME"),
                "PATH": os.environ.get("PATH"),
                "PWD": os.environ.get("PWD"),
                "USER": os.environ.get("USER"),
            },
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
