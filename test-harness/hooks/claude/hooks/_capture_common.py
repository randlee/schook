#!/usr/bin/env python3
"""Shared capture helpers for the Claude hook harness."""

from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path


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


def write_capture(hook_name: str, raw_text: str) -> Path:
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    filename = f"{timestamp}-{hook_name}.json"
    output_path = capture_root() / filename
    output_path.write_text(_normalize_payload(raw_text), encoding="utf-8")
    return output_path


def run_capture_hook(hook_name: str) -> int:
    raw_text = sys.stdin.read()
    write_capture(hook_name, raw_text)
    return 0
