from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


SENSITIVE_ENV_TOKENS = ("KEY", "TOKEN", "SECRET", "PASSWORD", "CRED")
SYNTHETIC_PROJECT_ROOT = "/synthetic/test/gemini-harness"
SYNTHETIC_HOME = "/synthetic/test/gemini-home"
SYNTHETIC_PLANS_DIR = "/synthetic/test/gemini-harness/.gemini/tmp/plans"
REDACTED_PATH = "<machine-path-redacted>"
REDACTED_OPERATOR = "<operator>"


def capture_root() -> Path:
    override = os.environ.get("SCHOOK_HOOK_CAPTURE_ROOT")
    if override:
        return Path(override).resolve()
    return (Path(__file__).resolve().parents[3] / "test-harness" / "hooks" / "gemini" / "captures" / "raw").resolve()


def _timestamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")


def _redact_env(name: str, value: str) -> str:
    if any(token in name for token in SENSITIVE_ENV_TOKENS):
        return "<redacted>"
    if name == "PATH":
        return REDACTED_PATH
    if name == "USER":
        return REDACTED_OPERATOR
    if name == "HOME":
        return SYNTHETIC_HOME
    if name == "GEMINI_PLANS_DIR":
        return SYNTHETIC_PLANS_DIR
    if name in {"PWD", "GEMINI_CWD", "GEMINI_PROJECT_DIR"}:
        return SYNTHETIC_PROJECT_ROOT
    return value


def _env_snapshot() -> dict[str, Any]:
    return {
        "gemini_env": {
            key: _redact_env(key, value)
            for key, value in sorted(os.environ.items())
            if key.startswith("GEMINI_")
        },
        "process_env": {
            key: _redact_env(key, os.environ[key])
            for key in ("HOME", "PATH", "PWD", "USER")
            if key in os.environ
        },
    }


def write_capture(surface: str, raw_text: str) -> None:
    root = capture_root()
    root.mkdir(parents=True, exist_ok=True)
    stem = f"{_timestamp()}-{surface}"
    (root / f"{stem}.json").write_text(raw_text, encoding="utf-8")
    (root / f"{stem}.env.json").write_text(
        json.dumps({"hook_name": surface, **_env_snapshot()}, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def capture_from_stdin(surface: str) -> int:
    write_capture(surface, sys.stdin.read())
    return 0
