#!/usr/bin/env python3
import json
import os
import subprocess
import sys
from pathlib import Path


def emit_error(code: str, message: str, **extra: object) -> int:
    payload = {"error": code, "message": message, **extra}
    print(json.dumps(payload, sort_keys=True), file=sys.stderr)
    return 1


def main() -> int:
    root = Path(os.environ.get("SC_HOOK_REPO_ROOT", Path(__file__).resolve().parent.parent))
    manifest = root / "test-harness" / "hooks" / "claude" / "fixtures" / "approved" / "manifest.json"
    try:
        manifest_text = manifest.read_text(encoding="utf-8")
    except (FileNotFoundError, OSError) as exc:
        return emit_error(
            "manifest_not_found",
            "approved manifest not found",
            path=str(manifest),
            detail=str(exc),
        )
    try:
        payload = json.loads(manifest_text)
    except json.JSONDecodeError as exc:
        return emit_error(
            "invalid_manifest_json",
            "approved manifest is not valid JSON",
            path=str(manifest),
            detail=str(exc),
        )
    recorded = payload.get("claude_version")
    if not isinstance(recorded, str) or not recorded.strip():
        return emit_error(
            "missing_claude_version",
            "manifest missing claude_version",
            path=str(manifest),
        )
    try:
        result = subprocess.run(["claude", "--version"], text=True, capture_output=True, check=False)
    except (FileNotFoundError, OSError) as exc:
        return emit_error(
            "claude_version_exec_failed",
            "claude --version could not be executed",
            detail=str(exc),
        )
    current = (result.stdout or result.stderr).strip()
    if result.returncode != 0 or not current:
        return emit_error(
            "claude_version_failed",
            "claude --version failed",
            returncode=result.returncode,
        )
    if current == recorded:
        print(f"Claude version matches approved manifest: {current}")
        return 0
    print(
        f"WARNING: Claude version changed from {recorded} to {current}; rerun live hook validation",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
