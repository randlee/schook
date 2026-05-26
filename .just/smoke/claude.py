from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path


FIXTURE_PATH = Path(".just/smoke/fixtures/claude/expected.json")
OBSERVABILITY_LOG = Path.home() / ".local/share/sc-hooks/runtime-layout/.sc-hooks/observability/logs/sc-hooks.log.jsonl"
STATE_ROOT = Path.home() / ".sc-hooks/state"
SETTINGS_PATH = Path.home() / ".claude/settings.json"
PROMPT = "Run pwd using Bash exactly once, then reply with OK only."
REQUIRED_HOOKS = ("SessionStart", "PreToolUse", "PostToolUse", "SessionEnd")


def _load_fixture(repo_root: Path) -> dict:
    fixture = repo_root / FIXTURE_PATH
    return json.loads(fixture.read_text(encoding="utf-8"))


def _assert_claude_install(settings: dict) -> None:
    hooks = settings.get("hooks")
    if not isinstance(hooks, dict):
        raise SystemExit("~/.claude/settings.json is missing hook configuration")
    for hook_name in ("SessionStart", "PreToolUse", "PostToolUse", "SessionEnd"):
        if hook_name not in hooks:
            raise SystemExit(f"~/.claude/settings.json is missing {hook_name} hook wiring")


def _recent_session_before(snapshot_cutoff: float) -> set[str]:
    seen: set[str] = set()
    for path in STATE_ROOT.glob("*.json"):
        if path.name == "session.json" or path.stat().st_mtime <= snapshot_cutoff:
            continue
        seen.add(path.name)
    return seen


def _new_session_record(previous_cutoff: float) -> dict:
    candidates = [
        path
        for path in STATE_ROOT.glob("*.json")
        if path.name != "session.json" and path.stat().st_mtime > previous_cutoff
    ]
    if not candidates:
        raise SystemExit("Claude live smoke did not create a new session record")
    latest = max(candidates, key=lambda path: path.stat().st_mtime)
    return json.loads(latest.read_text(encoding="utf-8"))


def _log_lines_since(previous_count: int) -> list[dict]:
    if not OBSERVABILITY_LOG.exists():
        raise SystemExit(f"observability log missing at {OBSERVABILITY_LOG}")
    lines = OBSERVABILITY_LOG.read_text(encoding="utf-8").splitlines()[previous_count:]
    return [json.loads(line) for line in lines if line.strip()]


def run(mode: str, repo_root: Path) -> int:
    fixture = _load_fixture(repo_root)
    if mode == "ci":
        if fixture.get("recommendation") != "accepted_baseline_live_record":
            raise SystemExit("Claude smoke fixture recommendation drifted")
        required_hooks = fixture.get("required_hooks")
        if required_hooks != list(REQUIRED_HOOKS):
            raise SystemExit("Claude smoke fixture required_hooks drifted")
        if fixture.get("offline_assertion") != "fixture_contract_only":
            raise SystemExit("Claude smoke fixture offline_assertion drifted")
        print("claude smoke ci: fixture contract verified")
        return 0

    if shutil.which("claude") is None:
        raise SystemExit("Claude CLI is not installed or not on PATH")
    if not SETTINGS_PATH.exists():
        raise SystemExit(f"missing Claude settings file at {SETTINGS_PATH}")

    settings = json.loads(SETTINGS_PATH.read_text(encoding="utf-8"))
    _assert_claude_install(settings)

    previous_log_count = 0
    if OBSERVABILITY_LOG.exists():
        previous_log_count = len(OBSERVABILITY_LOG.read_text(encoding="utf-8").splitlines())
    previous_cutoff = STATE_ROOT.stat().st_mtime if STATE_ROOT.exists() else 0.0

    env = os.environ.copy()
    env["SC_HOOKS_OBSERVABILITY_MODE"] = "standard"
    completed = subprocess.run(
        ["claude", "--print", PROMPT],
        cwd=repo_root,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise SystemExit(
            f"Claude live smoke failed: returncode={completed.returncode} stderr={completed.stderr.strip()}"
        )
    if completed.stdout.strip() != "OK":
        raise SystemExit(f"Claude live smoke returned unexpected stdout: {completed.stdout.strip()!r}")

    session_record = _new_session_record(previous_cutoff)
    if session_record.get("provider") != "claude":
        raise SystemExit("Claude live smoke session record provider drifted")
    if session_record.get("ai_root_dir") != str(repo_root):
        raise SystemExit("Claude live smoke session root drifted")
    if session_record.get("agent_state") != "ended":
        raise SystemExit("Claude live smoke did not end cleanly")
    if session_record.get("last_hook_event") != "SessionEnd":
        raise SystemExit("Claude live smoke did not record SessionEnd as the terminal event")

    new_log_lines = _log_lines_since(previous_log_count)
    seen_hooks = {
        line.get("fields", {}).get("hook")
        for line in new_log_lines
        if line.get("action") == "dispatch.complete"
    }
    missing = [hook for hook in REQUIRED_HOOKS if hook not in seen_hooks]
    if missing:
        raise SystemExit(f"Claude live smoke missing log hooks: {', '.join(missing)}")
    if not any(
        line.get("fields", {}).get("hook") == "PostToolUse"
        and line.get("fields", {}).get("event") == "Bash"
        for line in new_log_lines
    ):
        raise SystemExit("Claude live smoke missing PostToolUse(Bash) observability proof")

    print("claude smoke live: install, dispatch, plugin-chain, and observability verified")
    return 0
