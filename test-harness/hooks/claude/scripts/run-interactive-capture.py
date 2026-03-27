#!/usr/bin/env python3
"""Run an interactive Claude harness session for surfaces that do not work in -p mode."""

from __future__ import annotations

import argparse
import json
import os
import signal
import tempfile
import time
from pathlib import Path

import pexpect


def build_settings(claude_dir: Path, capture_root: Path) -> Path:
    hook_dir = claude_dir / "hooks"
    settings = tempfile.NamedTemporaryFile("w", delete=False)
    settings.write(
        json.dumps(
            {
                "hooks": {
                    "SessionStart": [
                        {
                            "matcher": "*",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": f"SCHOOK_HOOK_CAPTURE_ROOT='{capture_root}' python3 '{hook_dir / 'session_start.py'}'",
                                }
                            ],
                        }
                    ],
                    "SessionEnd": [
                        {
                            "matcher": "*",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": f"SCHOOK_HOOK_CAPTURE_ROOT='{capture_root}' python3 '{hook_dir / 'session_end.py'}'",
                                }
                            ],
                        }
                    ],
                    "PreCompact": [
                        {
                            "matcher": "",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": f"SCHOOK_HOOK_CAPTURE_ROOT='{capture_root}' python3 '{hook_dir / 'pre_compact.py'}'",
                                }
                            ],
                        }
                    ],
                    "PermissionRequest": [
                        {
                            "matcher": "*",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": f"SCHOOK_HOOK_CAPTURE_ROOT='{capture_root}' python3 '{hook_dir / 'permission_request.py'}'",
                                }
                            ],
                        }
                    ],
                    "Notification": [
                        {
                            "matcher": "",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": f"SCHOOK_HOOK_CAPTURE_ROOT='{capture_root}' python3 '{hook_dir / 'notification.py'}'",
                                }
                            ],
                        }
                    ],
                    "Stop": [
                        {
                            "matcher": "*",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": f"SCHOOK_HOOK_CAPTURE_ROOT='{capture_root}' python3 '{hook_dir / 'stop.py'}'",
                                }
                            ],
                        }
                    ],
                }
            }
        )
    )
    settings.close()
    return Path(settings.name)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("surface", choices=["notification-idle-prompt", "permission-request"])
    parser.add_argument("--model", default=os.environ.get("CLAUDE_MODEL", "haiku"))
    parser.add_argument("--capture-root")
    args = parser.parse_args()

    claude_dir = Path(__file__).resolve().parent.parent
    capture_root = Path(
        args.capture_root or os.environ.get("SCHOOK_HOOK_CAPTURE_ROOT", claude_dir / "captures" / "raw")
    ).expanduser().resolve()
    capture_root.mkdir(parents=True, exist_ok=True)
    prompt_path = claude_dir / "prompts" / f"{args.surface}.md"
    prompt = prompt_path.read_text(encoding="utf-8").strip()

    settings_path = build_settings(claude_dir, capture_root)
    cmd = (
        f"claude --model {args.model} --setting-sources local "
        f"--permission-mode default --settings {settings_path}"
    )

    child = pexpect.spawn(cmd, cwd=os.getcwd(), encoding="utf-8", timeout=30)
    child.delaybeforesend = 0.2

    try:
        time.sleep(3)
        if args.surface == "notification-idle-prompt":
            # No prompt: let Claude sit fully idle long enough for Notification to fire.
            time.sleep(70)
        else:
            child.sendline(prompt)
            time.sleep(10)
        if child.isalive():
            child.kill(signal.SIGTERM)
            time.sleep(2)
            if child.isalive():
                child.kill(signal.SIGKILL)
    finally:
        try:
            settings_path.unlink(missing_ok=True)
        except Exception:
            pass

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
