#!/usr/bin/env python3
"""Shared capture helpers for the Claude hook harness."""

from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path


_SENSITIVE_ENV_MARKERS = (
    "AUTH",
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "PRIVATE_KEY",
    "ACCESS_KEY",
    "API_KEY",
)


def capture_root() -> Path:
    """Return the raw capture directory used by harness hooks."""
    root = os.environ.get("SCHOOK_HOOK_CAPTURE_ROOT", "").strip()
    if root:
        path = Path(root).expanduser().resolve()
    else:
        path = (Path(__file__).resolve().parent.parent / "captures" / "raw").resolve()
    path.mkdir(parents=True, exist_ok=True)
    return path


def _normalize_payload(raw_text: str) -> str:
    raw_text = raw_text.strip()
    if not raw_text:
        return "{}\n"
    try:
        parsed = json.loads(raw_text)
    except Exception:
        parsed = {"_invalid_json": raw_text}
    return json.dumps(parsed, indent=2, sort_keys=True) + "\n"


def _normalize_env_snapshot(hook_name: str, timestamp: str) -> str:
    def sanitize_env_value(key: str, value: str) -> str:
        if any(marker in key for marker in _SENSITIVE_ENV_MARKERS):
            return "<redacted>"
        return value

    def filtered_env(prefix: str) -> dict[str, str]:
        return {
            key: sanitize_env_value(key, value)
            for key, value in sorted(os.environ.items())
            if key.startswith(prefix)
        }

    grouped_env = {
        "claude_env": filtered_env("CLAUDE"),
        "atm_env": filtered_env("ATM"),
        "sc_hook_env": filtered_env("SC_HOOK"),
    }
    snapshot = {
        "captured_at": timestamp,
        "hook_name": hook_name,
        "cwd_from_getcwd": os.getcwd(),
        "pwd_env": os.environ.get("PWD"),
        **grouped_env,
    }
    return json.dumps(snapshot, indent=2, sort_keys=True) + "\n"


def write_capture(hook_name: str, raw_text: str) -> Path:
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    filename = f"{timestamp}-{hook_name}.json"
    output_path = capture_root() / filename
    output_path.write_text(_normalize_payload(raw_text), encoding="utf-8")
    env_output_path = capture_root() / f"{timestamp}-{hook_name}.env.json"
    env_output_path.write_text(
        _normalize_env_snapshot(hook_name, timestamp),
        encoding="utf-8",
    )
    return output_path


def run_capture_hook(hook_name: str) -> int:
    raw_text = sys.stdin.read()
    write_capture(hook_name, raw_text)
    return 0
