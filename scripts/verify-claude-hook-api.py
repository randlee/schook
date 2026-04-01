#!/usr/bin/env python3
import json
import os
import subprocess
import sys
from pathlib import Path


def main() -> int:
    root = Path(os.environ.get("SC_HOOK_REPO_ROOT", Path(__file__).resolve().parent.parent))
    manifest = root / "test-harness" / "hooks" / "claude" / "fixtures" / "approved" / "manifest.json"
    payload = json.loads(manifest.read_text(encoding="utf-8"))
    recorded = payload.get("claude_version")
    if not isinstance(recorded, str) or not recorded.strip():
        print(f"manifest missing claude_version: {manifest}", file=sys.stderr)
        return 1
    result = subprocess.run(["claude", "--version"], text=True, capture_output=True, check=False)
    current = (result.stdout or result.stderr).strip()
    if result.returncode != 0 or not current:
        print("claude --version failed", file=sys.stderr)
        return 1
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
