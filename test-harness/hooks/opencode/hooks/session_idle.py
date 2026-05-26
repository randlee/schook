from __future__ import annotations

import json
import os
from pathlib import Path


def _surface(payload: dict[str, object]) -> str:
    event = payload.get("event")
    if isinstance(event, dict):
        event_type = event.get("type")
        if isinstance(event_type, str) and event_type.strip():
            return event_type.strip()
    override = os.environ.get("HOOK_NAME_OVERRIDE")
    if override:
        return override
    return "session.idle"


if __name__ == "__main__":
    import sys

    payload = json.loads(sys.stdin.read() or "{}")
    surface = _surface(payload)
    stem = surface.replace(".", "-")
    capture_root = Path(os.environ["SCHOOK_HOOK_CAPTURE_ROOT"])
    capture_root.mkdir(parents=True, exist_ok=True)
    (capture_root / f"{stem}.json").write_text(json.dumps(payload, indent=2), encoding="utf-8")
    env_snapshot = {
        "hook_name": surface,
        "opencode_env": {
            key: value
            for key, value in os.environ.items()
            if key.startswith("OPENCODE_")
        },
        "process_env": {
            "HOME": os.environ.get("HOME", ""),
            "PATH": "<machine-path-redacted>",
            "PWD": os.environ.get("PWD", ""),
            "USER": os.environ.get("USER", "<operator>"),
        },
    }
    (capture_root / f"{stem}.env.json").write_text(
        json.dumps(env_snapshot, indent=2), encoding="utf-8"
    )
