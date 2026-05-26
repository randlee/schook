from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from pathlib import Path


FIXTURE_PATH = Path(".just/smoke/fixtures/gemini/expected.json")
SETTINGS_PATH = Path.home() / ".gemini" / "settings.json"
OBSERVABILITY_LOG = (
    Path.home()
    / ".local/share/sc-hooks/runtime-layout/.sc-hooks/observability/logs/sc-hooks.log.jsonl"
)
STATE_ROOT = Path.home() / ".sc-hooks" / "state"
PROMPT = "Reply with OK only."
REQUIRED_HOOKS = ("SessionStart", "SessionEnd")


def _load_fixture(repo_root: Path) -> dict:
    fixture = repo_root / FIXTURE_PATH
    return json.loads(fixture.read_text(encoding="utf-8"))


def _load_settings() -> dict:
    if not SETTINGS_PATH.exists():
        raise SystemExit(f"missing Gemini settings file at {SETTINGS_PATH}")
    return json.loads(SETTINGS_PATH.read_text(encoding="utf-8"))


def _assert_install_paths(settings: dict) -> None:
    hooks = settings.get("hooks")
    if not isinstance(hooks, dict):
        raise SystemExit("~/.gemini/settings.json is missing hook configuration")

    for hook_name in ("SessionStart", "BeforeAgent", "SessionEnd"):
        entries = hooks.get(hook_name)
        if not isinstance(entries, list) or not entries:
            raise SystemExit(f"~/.gemini/settings.json is missing {hook_name} hook wiring")
        entry_text = json.dumps(entries)
        if "/Users/randlee/.local/bin/sc-hooks" not in entry_text:
            raise SystemExit(f"Gemini {hook_name} hook is not wired to the installed sc-hooks binary")


def _log_lines_since(previous_count: int) -> list[dict]:
    if not OBSERVABILITY_LOG.exists():
        raise SystemExit(f"observability log missing at {OBSERVABILITY_LOG}")
    lines = OBSERVABILITY_LOG.read_text(encoding="utf-8").splitlines()[previous_count:]
    return [json.loads(line) for line in lines if line.strip()]


def _new_state_record(previous_files: set[str]) -> dict:
    candidates = [
        path
        for path in STATE_ROOT.glob("*.json")
        if path.name != "session.json" and path.name not in previous_files
    ]
    if not candidates:
        raise SystemExit("Gemini live smoke did not create a new state record")
    latest = max(candidates, key=lambda path: path.stat().st_mtime)
    return json.loads(latest.read_text(encoding="utf-8"))


def run(mode: str, repo_root: Path) -> int:
    fixture = _load_fixture(repo_root)
    if mode == "ci":
        if fixture.get("recommendation") != "accepted_baseline_live_record":
            raise SystemExit("Gemini smoke fixture recommendation drifted")
        if fixture.get("required_surfaces") != ["SessionStart", "BeforeAgent", "SessionEnd"]:
            raise SystemExit("Gemini smoke fixture required_surfaces drifted")
        if fixture.get("offline_assertion") != "fixture_contract_only":
            raise SystemExit("Gemini smoke fixture offline_assertion drifted")
        print("gemini smoke ci: fixture contract verified")
        return 0

    if shutil.which("gemini") is None:
        raise SystemExit("Gemini CLI is not installed or not on PATH")

    settings = _load_settings()
    _assert_install_paths(settings)

    previous_log_count = 0
    if OBSERVABILITY_LOG.exists():
        previous_log_count = len(OBSERVABILITY_LOG.read_text(encoding="utf-8").splitlines())
    previous_files = {
        path.name
        for path in STATE_ROOT.glob("*.json")
        if path.name != "session.json"
    }

    env = os.environ.copy()
    env.update({"ATM_IDENTITY": "gemini-smoke", "ATM_TEAM": "schook"})
    completed = subprocess.run(
        ["gemini", "-p", PROMPT, "--yolo", "--skip-trust"],
        cwd=repo_root,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise SystemExit(
            f"Gemini live smoke failed: returncode={completed.returncode} stderr={completed.stderr.strip()}"
        )
    if completed.stdout.strip() != "OK":
        raise SystemExit(f"Gemini live smoke returned unexpected stdout: {completed.stdout.strip()!r}")

    time.sleep(1.0)
    state_record = _new_state_record(previous_files)
    if state_record.get("provider") != "gemini":
        raise SystemExit("Gemini live smoke state record provider drifted")
    if state_record.get("ai_root_dir") != str(repo_root):
        raise SystemExit("Gemini live smoke state root drifted")
    if state_record.get("agent_state") != "ended":
        raise SystemExit("Gemini live smoke did not end cleanly")
    if state_record.get("last_hook_event") != "SessionEnd":
        raise SystemExit("Gemini live smoke did not record SessionEnd as the terminal event")

    new_log_lines = _log_lines_since(previous_log_count)
    seen_hooks = {
        line.get("fields", {}).get("hook")
        for line in new_log_lines
        if line.get("action") == "dispatch.complete"
    }
    missing = [hook for hook in REQUIRED_HOOKS if hook not in seen_hooks]
    if missing:
        raise SystemExit(f"Gemini live smoke missing log hooks: {', '.join(missing)}")
    if not any(
        line.get("fields", {}).get("hook") == "PreToolUse"
        and line.get("fields", {}).get("event") == "Agent"
        for line in new_log_lines
    ):
        raise SystemExit("Gemini live smoke missing BeforeAgent observability proof")

    print("gemini smoke live: install, runtime dispatch, lifecycle, and observability verified")
    return 0
