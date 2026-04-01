#!/usr/bin/env python3
"""Run Claude harness sessions for surfaces that need interactive or multi-step flow."""

from __future__ import annotations

import argparse
import json
import os
import signal
import subprocess
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


def _spawn_interactive(
    cmd: str,
    *,
    prompt: str | None = None,
    followup_command: str | None = None,
    idle_seconds: int = 0,
) -> None:
    child = pexpect.spawn(cmd, cwd=os.getcwd(), encoding="utf-8", timeout=30)
    child.delaybeforesend = 0.2

    try:
        time.sleep(3)
        if prompt:
            child.sendline(prompt)
            time.sleep(10)
        if followup_command:
            child.sendline(followup_command)
            time.sleep(10)
        if idle_seconds:
            time.sleep(idle_seconds)
        if child.isalive():
            child.kill(signal.SIGTERM)
            time.sleep(2)
            if child.isalive():
                child.kill(signal.SIGKILL)
    finally:
        if child.isalive():
            child.kill(signal.SIGKILL)


def _run_resume_capture(settings_path: Path, capture_root: Path, model: str) -> None:
    before = {path.name for path in capture_root.glob("*-session-start.json")}
    subprocess.run(
        [
            "claude",
            "--model",
            model,
            "--setting-sources",
            "local",
            "--permission-mode",
            "default",
            "--settings",
            str(settings_path),
            "-p",
            "Reply DONE and then allow the session to exit normally.",
        ],
        cwd=os.getcwd(),
        text=True,
        capture_output=True,
        check=True,
    )
    new_starts = sorted(
        path for path in capture_root.glob("*-session-start.json") if path.name not in before
    )
    if not new_starts:
        raise RuntimeError("resume capture could not find the initial session-start artifact")
    initial_session = json.loads(new_starts[-1].read_text(encoding="utf-8"))
    session_id = initial_session["session_id"]
    subprocess.run(
        [
            "claude",
            "--resume",
            session_id,
            "-p",
            "Reply RESUMED and then allow the session to exit normally.",
            "--model",
            model,
            "--setting-sources",
            "local",
            "--permission-mode",
            "default",
            "--settings",
            str(settings_path),
        ],
        cwd=os.getcwd(),
        text=True,
        capture_output=True,
        check=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "surface",
        choices=["notification", "permission-request", "compact", "clear", "resume"],
    )
    parser.add_argument("--model", default=os.environ.get("CLAUDE_MODEL", "haiku"))
    parser.add_argument("--capture-root")
    args = parser.parse_args()

    claude_dir = Path(__file__).resolve().parent.parent
    capture_root = Path(
        args.capture_root or os.environ.get("SCHOOK_HOOK_CAPTURE_ROOT", claude_dir / "captures" / "raw")
    ).expanduser().resolve()
    capture_root.mkdir(parents=True, exist_ok=True)
    prompt = None
    if args.surface in {"permission-request", "compact"}:
        prompt_name = "permission-request" if args.surface == "permission-request" else "pre-compact"
        prompt_path = claude_dir / "prompts" / f"{prompt_name}.md"
        prompt = prompt_path.read_text(encoding="utf-8").strip()

    settings_path = build_settings(claude_dir, capture_root)
    cmd = (
        f"claude --model {args.model} --setting-sources local "
        f"--permission-mode default --settings {settings_path}"
    )

    try:
        if args.surface == "notification":
            _spawn_interactive(cmd, idle_seconds=70)
        elif args.surface == "permission-request":
            _spawn_interactive(cmd, prompt=prompt)
        elif args.surface == "compact":
            _spawn_interactive(cmd, prompt=prompt, followup_command="/compact")
        elif args.surface == "clear":
            _spawn_interactive(cmd, followup_command="/clear")
        elif args.surface == "resume":
            _run_resume_capture(settings_path, capture_root, args.model)
        else:
            raise RuntimeError(f"unsupported surface: {args.surface}")
    finally:
        try:
            settings_path.unlink(missing_ok=True)
        except Exception:
            pass

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
