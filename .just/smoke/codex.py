from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path


FIXTURE_PATH = Path(".just/smoke/fixtures/codex/expected.json")
CONFIG_PATH = Path.home() / ".codex" / "config.toml"
HOOKS_PATH = Path.home() / ".codex" / "hooks.json"
PROMPT = "Run pwd using Bash exactly once, then reply with OK only."
REQUIRED_LOG_HOOKS = ("SessionStart", "PreToolUse")
IDLE_IDENTITY = "codex-smoke"


def _env_path(name: str, default: Path) -> Path:
    return Path(os.environ.get(name, str(default))).expanduser()


def _observability_log() -> Path:
    path = os.environ.get("SC_HOOKS_AUDIT_PATH")
    if not path:
        raise SystemExit("SC_HOOKS_AUDIT_PATH must be set for Codex smoke validation")
    return Path(path).expanduser()


def _marker_dir(repo_root: Path) -> Path:
    return _env_path("SC_HOOKS_STATE_DIR", repo_root / ".sc" / "sessions" / "codex")


def _load_fixture(repo_root: Path) -> dict:
    fixture = repo_root / FIXTURE_PATH
    return json.loads(fixture.read_text(encoding="utf-8"))


def _load_codex_config() -> str:
    if not CONFIG_PATH.exists():
        raise SystemExit(f"missing Codex config file at {CONFIG_PATH}")
    return CONFIG_PATH.read_text(encoding="utf-8")


def _load_codex_hooks() -> dict:
    if not HOOKS_PATH.exists():
        raise SystemExit(f"missing Codex hooks file at {HOOKS_PATH}")
    return json.loads(HOOKS_PATH.read_text(encoding="utf-8"))


def _assert_install_paths(config_text: str, hooks_data: dict) -> None:
    notify_script = Path.home() / ".codex" / "scripts" / "schook-delay-notify.py"
    if f'notify = ["python3", "{notify_script}"]' not in config_text:
        raise SystemExit("~/.codex/config.toml is missing the production notify hook wiring")

    hooks = hooks_data.get("hooks")
    if not isinstance(hooks, dict):
        raise SystemExit("~/.codex/hooks.json is missing the hooks object")

    session_start = hooks.get("SessionStart")
    if not isinstance(session_start, list) or not session_start:
        raise SystemExit("~/.codex/hooks.json is missing SessionStart hook wiring")
    pre_tool_use = hooks.get("PreToolUse")
    if not isinstance(pre_tool_use, list) or not pre_tool_use:
        raise SystemExit("~/.codex/hooks.json is missing PreToolUse hook wiring")

    session_start_text = json.dumps(session_start)
    if str(Path.home() / ".local" / "bin" / "sc-hooks") not in session_start_text:
        raise SystemExit("Codex SessionStart hook is not wired to the installed sc-hooks binary")
    pre_tool_use_text = json.dumps(pre_tool_use)
    if str(Path.home() / ".codex" / "scripts" / "schook-delay-pretooluse.sh") not in pre_tool_use_text:
        raise SystemExit("Codex PreToolUse hook is not wired to the stable global pretooluse wrapper")


def _log_lines_since(previous_count: int) -> list[dict]:
    observability_log = _observability_log()
    if not observability_log.exists():
        raise SystemExit(f"observability log missing at {observability_log}")
    lines = observability_log.read_text(encoding="utf-8").splitlines()[previous_count:]
    return [json.loads(line) for line in lines if line.strip()]


def _recent_dispatch_complete_exists(hook: str, window_seconds: int) -> bool:
    observability_log = _observability_log()
    if not observability_log.exists():
        return False
    threshold = datetime.now(timezone.utc) - timedelta(seconds=window_seconds)
    for line in reversed(observability_log.read_text(encoding="utf-8").splitlines()):
        if not line.strip():
            continue
        record = json.loads(line)
        timestamp = record.get("timestamp")
        if not isinstance(timestamp, str):
            continue
        try:
            when = datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
        except ValueError:
            continue
        if when < threshold:
            return False
        fields = record.get("fields", {})
        if record.get("action") == "dispatch.complete" and fields.get("hook") == hook:
            return True
    return False


def _wait_for_idle_marker(marker_path: Path, timeout_seconds: float) -> dict:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        if marker_path.exists():
            return json.loads(marker_path.read_text(encoding="utf-8"))
        time.sleep(0.1)
    raise SystemExit(f"Codex smoke did not produce the idle marker at {marker_path}")


def _wait_for_no_pending(pending_root: Path, timeout_seconds: float) -> None:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        entries = list(pending_root.glob("*.json")) if pending_root.exists() else []
        if not entries:
            return
        time.sleep(0.1)
    raise SystemExit("Codex live smoke left pending idle records behind after debounce expiry")


def run(mode: str, repo_root: Path) -> int:
    fixture = _load_fixture(repo_root)
    if mode == "ci":
        if fixture.get("recommendation") != "accepted_baseline_live_record":
            raise SystemExit("Codex smoke fixture recommendation drifted")
        if fixture.get("required_surfaces") != ["SessionStart", "PreToolUse", "notify"]:
            raise SystemExit("Codex smoke fixture required_surfaces drifted")
        if fixture.get("required_post_tool_event") != "Bash":
            raise SystemExit("Codex smoke fixture required_post_tool_event drifted")
        if fixture.get("required_log_hooks") != list(REQUIRED_LOG_HOOKS):
            raise SystemExit("Codex smoke fixture required_log_hooks drifted")
        if fixture.get("required_state_record") != "idle-codex-smoke.json":
            raise SystemExit("Codex smoke fixture required_state_record drifted")
        if fixture.get("offline_assertion") != "fixture_contract_only":
            raise SystemExit("Codex smoke fixture offline_assertion drifted")
        print("codex smoke ci: fixture contract verified")
        return 0

    if shutil.which("codex") is None:
        raise SystemExit("Codex CLI is not installed or not on PATH")

    config_text = _load_codex_config()
    hooks_data = _load_codex_hooks()
    _assert_install_paths(config_text, hooks_data)

    marker_dir = _marker_dir(repo_root)
    idle_marker = marker_dir / f"idle-{IDLE_IDENTITY}.json"
    active_marker = marker_dir / f"active-{IDLE_IDENTITY}.json"
    idle_marker.unlink(missing_ok=True)
    active_marker.unlink(missing_ok=True)

    previous_log_count = 0
    observability_log = _observability_log()
    if observability_log.exists():
        previous_log_count = len(observability_log.read_text(encoding="utf-8").splitlines())

    with tempfile.TemporaryDirectory(prefix="codex-smoke-idle-") as state_root, tempfile.NamedTemporaryFile(
        prefix="codex-smoke-last-message-",
        suffix=".txt",
        delete=False,
    ) as last_message_file:
        last_message_path = Path(last_message_file.name)
        env = os.environ.copy()
        env.update(
            {
                "ATM_IDENTITY": IDLE_IDENTITY,
                "ATM_TEAM": "schook",
                "SCHOOK_CODEX_IDLE_SECONDS": "1",
                "SCHOOK_CODEX_IDLE_STATE_ROOT": state_root,
            }
        )

        completed = subprocess.run(
            [
                "codex",
                "exec",
                "-C",
                str(repo_root),
                "-o",
                last_message_file.name,
                PROMPT,
            ],
            cwd=repo_root,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
        if completed.returncode != 0:
            raise SystemExit(
                f"Codex live smoke failed: returncode={completed.returncode} stderr={completed.stderr.strip()}"
            )

        last_message = last_message_path.read_text(encoding="utf-8").strip()
        last_message_path.unlink(missing_ok=True)
        if last_message != "OK":
            raise SystemExit(f"Codex live smoke returned unexpected final message: {last_message!r}")

        marker = _wait_for_idle_marker(idle_marker, timeout_seconds=10.0)
        if marker.get("state") != "idle":
            raise SystemExit("Codex live smoke did not end in the idle marker state")
        if marker.get("project_dir") != str(repo_root):
            raise SystemExit("Codex live smoke marker project_dir drifted")
        if marker.get("cwd") != str(repo_root):
            raise SystemExit("Codex live smoke marker cwd drifted")
        if marker.get("atm_identity") != IDLE_IDENTITY:
            raise SystemExit("Codex live smoke marker ATM identity drifted")
        if not marker.get("session_id"):
            raise SystemExit("Codex live smoke marker is missing session_id")
        if active_marker.exists():
            raise SystemExit("Codex live smoke left an active marker behind after idle confirmation")

        pending_root = Path(state_root) / "pending"
        _wait_for_no_pending(pending_root, timeout_seconds=5.0)

    time.sleep(1.0)
    new_log_lines = _log_lines_since(previous_log_count)
    seen_hooks = {
        line.get("fields", {}).get("hook")
        for line in new_log_lines
        if line.get("action") == "dispatch.complete"
    }
    missing = [
        hook
        for hook in REQUIRED_LOG_HOOKS
        if hook not in seen_hooks and not _recent_dispatch_complete_exists(hook, window_seconds=300)
    ]
    if missing:
        raise SystemExit(f"Codex live smoke missing log hooks: {', '.join(missing)}")
    print("codex smoke live: install, runtime dispatch, idle lifecycle, and observability verified")
    return 0
