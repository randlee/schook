from __future__ import annotations

import json
import os
import stat
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

import pytest

from test_harness.hooks.codex.models.payloads import (
    validate_codex_env_snapshot,
    validate_codex_fixture_manifest,
    validate_codex_hook_payload,
)


def _write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


def _run_hook(
    script: Path,
    payload: dict[str, object],
    env: dict[str, str],
) -> subprocess.CompletedProcess[str]:
    pythonpath = str(Path.cwd())
    if existing := os.environ.get("PYTHONPATH"):
        pythonpath = f"{pythonpath}{os.pathsep}{existing}"
    command = [sys.executable, str(script)]
    text_payload = json.dumps(payload)
    hook_name = script.stem.replace("_", "-")
    if script.name == "notify.py":
        command.append(text_payload)
        input_text = ""
    else:
        input_text = text_payload
    return subprocess.run(
        command,
        input=input_text,
        text=True,
        capture_output=True,
        env={**os.environ, **env, "PYTHONPATH": pythonpath, "HOOK_NAME_OVERRIDE": hook_name},
        check=False,
    )


def _marker_dir(repo_root: Path) -> Path:
    return repo_root / ".sc" / "sessions" / "codex"


def _marker_name(state: str, identity: str = "tester") -> str:
    return f"{state}-{identity}.json"


def _write_session_record(repo_root: Path, session_id: str, cwd: Path) -> Path:
    sessions_dir = _marker_dir(repo_root)
    sessions_dir.mkdir(parents=True, exist_ok=True)
    record_path = sessions_dir / f"20260522000000000-{session_id}.json"
    record_path.write_text(
        json.dumps(
            {
                "cwd": str(cwd),
                "native_session_id": session_id,
                "project_dir": str(repo_root),
                "tool": "codex",
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    return record_path


@pytest.mark.provider_codex
def test_codex_harness_layout_exists(codex_root: Path) -> None:
    for name in ["hooks", "scripts", "tests"]:
        assert (codex_root / name).is_dir(), name


@pytest.mark.provider_codex
def test_notify_hook_schedules_pending_record(tmp_path: Path, codex_root: Path) -> None:
    """Notify debounce accepts the minimal fields it actually consumes: thread-id and cwd."""
    capture_root = tmp_path / "captures"
    state_root = tmp_path / "state"
    repo_root = tmp_path / "repo"
    repo_root.mkdir()

    result = _run_hook(
        codex_root / "hooks" / "notify.py",
        {"type": "agent-turn-complete", "thread-id": "thread-123", "state": "idle", "cwd": str(repo_root)},
        {
            "SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root),
            "SCHOOK_CODEX_HOOK_STATE_ROOT": str(state_root),
            "SCHOOK_CODEX_HOOK_PROJECT_ROOT": str(repo_root),
            "SCHOOK_CODEX_DEBOUNCE_SECONDS": "60",
            "SCHOOK_CODEX_DEBOUNCE_AUTOSTART": "0",
            "CODEX_PROJECT_DIR": str(repo_root),
            "ATM_IDENTITY": "tester",
        },
    )

    assert result.returncode == 0, result.stderr
    pending_files = sorted((state_root / "pending").glob("*.json"))
    assert len(pending_files) == 1
    pending = json.loads(pending_files[0].read_text(encoding="utf-8"))
    assert pending["key"] == "thread-123"
    assert pending["project_dir"] == str(repo_root)
    assert pending["payload"]["type"] == "agent-turn-complete"
    assert not list(_marker_dir(repo_root).glob("active-*.json"))
    assert not list(_marker_dir(repo_root).glob("idle-*.json"))

    payload_captures = sorted(capture_root.glob("*.json"))
    env_captures = sorted(capture_root.glob("*.env.json"))
    assert any(path.name.endswith("-notify.json") for path in payload_captures)
    assert any(path.name.endswith("-notify.env.json") for path in env_captures)


@pytest.mark.provider_codex
def test_pretooluse_cancels_pending_debounce(tmp_path: Path, codex_root: Path) -> None:
    """PreToolUse cancellation is intentionally tolerant of partial payloads by design."""
    capture_root = tmp_path / "captures"
    state_root = tmp_path / "state"
    repo_root = tmp_path / "repo"
    repo_root.mkdir()
    output_path = tmp_path / "invocations.log"
    tool_path = tmp_path / "emit.py"
    _write_executable(
        tool_path,
        "#!/usr/bin/env python3\n"
        "from pathlib import Path\n"
        "import sys\n"
        "Path(sys.argv[1]).write_text('fired\\n', encoding='utf-8')\n",
    )
    command = json.dumps([sys.executable, str(tool_path), str(output_path)])

    common_env = {
        "SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root),
        "SCHOOK_CODEX_HOOK_STATE_ROOT": str(state_root),
        "SCHOOK_CODEX_HOOK_PROJECT_ROOT": str(repo_root),
        "SCHOOK_CODEX_DEBOUNCE_SECONDS": "0.05",
        "SCHOOK_CODEX_DEBOUNCE_AUTOSTART": "0",
        "SCHOOK_CODEX_DEBOUNCE_COMMAND": command,
        "CODEX_PROJECT_DIR": str(repo_root),
        "ATM_IDENTITY": "tester",
    }

    notify_result = _run_hook(
        codex_root / "hooks" / "notify.py",
        {"type": "agent-turn-complete", "thread-id": "thread-123", "cwd": str(repo_root)},
        common_env,
    )
    assert notify_result.returncode == 0, notify_result.stderr

    cancel_result = _run_hook(
        codex_root / "hooks" / "pre_tool_use.py",
        {"hook_event_name": "PreToolUse", "thread-id": "thread-123", "cwd": str(repo_root)},
        common_env,
    )
    assert cancel_result.returncode == 0, cancel_result.stderr
    assert not list((state_root / "pending").glob("*.json"))
    active_marker = _marker_dir(repo_root) / _marker_name("active")
    idle_marker = _marker_dir(repo_root) / _marker_name("idle")
    assert active_marker.is_file()
    assert not idle_marker.exists()
    assert json.loads(active_marker.read_text(encoding="utf-8"))["state"] == "active"

    time.sleep(0.08)
    fire_result = _run_hook(codex_root / "scripts" / "fire_pending.py", {}, common_env)
    assert fire_result.returncode == 0, fire_result.stderr
    assert not output_path.exists()


@pytest.mark.provider_codex
def test_fire_pending_runs_command_once_after_due_time(tmp_path: Path, codex_root: Path) -> None:
    state_root = tmp_path / "state"
    repo_root = tmp_path / "repo"
    repo_root.mkdir()
    output_path = tmp_path / "invocations.log"
    tool_path = tmp_path / "emit.py"
    _write_executable(
        tool_path,
        "#!/usr/bin/env python3\n"
        "from pathlib import Path\n"
        "import sys\n"
        "path = Path(sys.argv[1])\n"
        "existing = path.read_text(encoding='utf-8') if path.exists() else ''\n"
        "path.write_text(existing + 'fired\\n', encoding='utf-8')\n",
    )
    command = json.dumps([sys.executable, str(tool_path), str(output_path)])
    env = {
        "SCHOOK_CODEX_HOOK_STATE_ROOT": str(state_root),
        "SCHOOK_CODEX_HOOK_PROJECT_ROOT": str(repo_root),
        "SCHOOK_CODEX_DEBOUNCE_SECONDS": "0.05",
        "SCHOOK_CODEX_DEBOUNCE_AUTOSTART": "0",
        "SCHOOK_CODEX_DEBOUNCE_COMMAND": command,
        "CODEX_PROJECT_DIR": str(repo_root),
        "ATM_IDENTITY": "tester",
    }

    notify_result = _run_hook(
        codex_root / "hooks" / "notify.py",
        {"type": "agent-turn-complete", "thread-id": "thread-456", "cwd": str(repo_root)},
        env,
    )
    assert notify_result.returncode == 0, notify_result.stderr

    time.sleep(0.08)
    fire_result = _run_hook(codex_root / "scripts" / "fire_pending.py", {}, env)
    assert fire_result.returncode == 0, fire_result.stderr
    second_fire_result = _run_hook(codex_root / "scripts" / "fire_pending.py", {}, env)
    assert second_fire_result.returncode == 0, second_fire_result.stderr

    assert output_path.read_text(encoding="utf-8") == "fired\n"
    assert not list((state_root / "pending").glob("*.json"))
    idle_marker = _marker_dir(repo_root) / _marker_name("idle")
    active_marker = _marker_dir(repo_root) / _marker_name("active")
    assert idle_marker.is_file()
    assert not active_marker.exists()
    marker_payload = json.loads(idle_marker.read_text(encoding="utf-8"))
    assert marker_payload["state"] == "idle"
    assert marker_payload["thread_id"] == "thread-456"
    assert marker_payload["idle_since"]


@pytest.mark.provider_codex
def test_notify_prefers_session_record_and_sends_atm_idle_notice(tmp_path: Path, codex_root: Path) -> None:
    state_root = tmp_path / "state"
    capture_root = tmp_path / "captures"
    repo_root = tmp_path / "repo"
    nested_cwd = repo_root / "nested" / "work"
    nested_cwd.mkdir(parents=True)
    session_id = "019e4d6c-6fe6-77e0-92e0-dfad6e86ec58"
    _write_session_record(repo_root, session_id, repo_root)
    (repo_root / ".atm.toml").write_text(
        "\n".join(
            [
                "[atm]",
                'default_team = "schook"',
                'identity = "team-lead"',
                "",
                "[atm.idle_notify]",
                'recipient = "team-lead"',
                "",
                "[atm.idle_notify.agent.tester]",
                "seconds = 0.05",
                "",
            ]
        ),
        encoding="utf-8",
    )

    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    atm_log = tmp_path / "atm-log.jsonl"
    _write_executable(
        bin_dir / "atm",
        "#!/usr/bin/env python3\n"
        "import json\n"
        "import os\n"
        "import sys\n"
        "from pathlib import Path\n"
        "Path(os.environ['ATM_LOG_PATH']).write_text(json.dumps(sys.argv[1:]) + '\\n', encoding='utf-8')\n",
    )

    env = {
        "SCHOOK_CODEX_HOOK_STATE_ROOT": str(state_root),
        "SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root),
        "SCHOOK_CODEX_DEBOUNCE_AUTOSTART": "0",
        "ATM_IDENTITY": "tester",
        "ATM_TEAM": "schook",
        "ATM_LOG_PATH": str(atm_log),
        "PATH": f"{bin_dir}{os.pathsep}{os.environ.get('PATH', '')}",
    }

    notify_result = _run_hook(
        codex_root / "hooks" / "notify.py",
        {"type": "agent-turn-complete", "thread-id": session_id, "cwd": str(nested_cwd)},
        env,
    )
    assert notify_result.returncode == 0, notify_result.stderr

    pending_files = sorted((state_root / "pending").glob("*.json"))
    assert len(pending_files) == 1
    pending = json.loads(pending_files[0].read_text(encoding="utf-8"))
    assert pending["project_dir"] == str(repo_root)
    assert pending["idle_notify"]["recipient"] == "team-lead"
    assert pending["idle_notify"]["team"] == "schook"
    assert pending["idle_notify"]["sender"] == "tester"
    assert pending["idle_notify"]["seconds"] == 0.05

    time.sleep(0.08)
    fire_result = _run_hook(codex_root / "scripts" / "fire_pending.py", {}, env)
    assert fire_result.returncode == 0, fire_result.stderr

    idle_marker = _marker_dir(repo_root) / _marker_name("idle")
    assert idle_marker.is_file()
    marker_payload = json.loads(idle_marker.read_text(encoding="utf-8"))
    assert marker_payload["cwd"] == str(repo_root)
    assert marker_payload["session_id"] == session_id
    assert marker_payload["idle_since"]

    atm_args = json.loads(atm_log.read_text(encoding="utf-8").strip())
    assert atm_args[:2] == ["send", "team-lead"]
    assert atm_args[3:] == ["--team", "schook", "--from", "tester"]
    assert "tester idle for 0.05 seconds @" in atm_args[2]


@pytest.mark.provider_codex
def test_project_scope_blocks_outside_directories(tmp_path: Path, codex_root: Path) -> None:
    capture_root = tmp_path / "captures"
    state_root = tmp_path / "state"
    project_root = tmp_path / "allowed"
    project_root.mkdir()
    outside_root = tmp_path / "outside"
    outside_root.mkdir()

    result = _run_hook(
        codex_root / "hooks" / "notify.py",
        {"type": "agent-turn-complete", "thread-id": "thread-789", "cwd": str(outside_root)},
        {
            "SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root),
            "SCHOOK_CODEX_HOOK_STATE_ROOT": str(state_root),
            "SCHOOK_CODEX_HOOK_PROJECT_ROOT": str(project_root),
            "SCHOOK_CODEX_DEBOUNCE_AUTOSTART": "0",
        },
    )

    assert result.returncode == 0, result.stderr
    assert not list((state_root / "pending").glob("*.json"))
    assert not list(capture_root.glob("*.json"))
    assert not list(_marker_dir(project_root).glob("*.json"))


@pytest.mark.provider_codex
def test_manifest_has_required_top_level_keys(codex_root: Path) -> None:
    manifest_path = codex_root / "fixtures" / "approved" / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    validated = validate_codex_fixture_manifest(manifest)

    assert validated.provider == "codex"
    assert validated.codex_version == "codex-cli 0.133.0"
    assert {surface.surface for surface in validated.hook_surfaces} >= {
        "SessionStart",
        "PreToolUse",
        "notify",
        "Stop",
        "resume",
        "fork",
    }


@pytest.mark.provider_codex
def test_approved_payload_fixtures_validate_against_models(
    codex_root: Path, expected_hooks: dict[str, str]
) -> None:
    fixture_dir = codex_root / "fixtures" / "approved"
    expected_payloads = {f"{stem}.json" for stem in expected_hooks.values()}

    for fixture_path in sorted(fixture_dir.glob("*.json")):
        fixture_text = fixture_path.read_text(encoding="utf-8")
        assert "$REPO_ROOT" not in fixture_text, fixture_path.name
        if fixture_path.name == "manifest.json" or fixture_path.name.endswith(".env.json"):
            continue
        payload = json.loads(fixture_text)
        validate_codex_hook_payload(payload)
        assert payload["cwd"].startswith("/synthetic/test/codex-harness"), fixture_path.name

    assert expected_payloads <= {path.name for path in fixture_dir.glob("*.json")}


@pytest.mark.provider_codex
def test_approved_env_fixtures_validate_and_redact_sensitive_values(codex_root: Path) -> None:
    fixture_dir = codex_root / "fixtures" / "approved"
    for fixture_path in sorted(fixture_dir.glob("*.env.json")):
        fixture_text = fixture_path.read_text(encoding="utf-8")
        assert "$REPO_ROOT" not in fixture_text, fixture_path.name
        validated = validate_codex_env_snapshot(json.loads(fixture_text))
        assert validated.atm_env["ATM_IDENTITY"].strip()
        assert validated.atm_env["ATM_TEAM"].strip()
        for key, value in validated.atm_env.items():
            if any(token in key for token in ("TOKEN", "AUTH")):
                assert value == "<redacted>"
        assert validated.atm_env["ATM_LOKI_INSTANCE_ID"] == "12345678"
        assert validated.atm_env["ATM_MIMIR_INSTANCE_ID"] == "12345678"
        assert validated.atm_env["ATM_TEMPO_INSTANCE_ID"] == "12345678"
        assert validated.atm_env["ATM_LOKI_URL"] == "https://synthetic-endpoint.example.com"
        assert validated.atm_env["ATM_MIMIR_QUERY_ENDPOINT"] == "https://synthetic-endpoint.example.com/api/prom"
        assert validated.atm_env["ATM_OTEL_ENDPOINT"] == "https://synthetic-endpoint.example.com/otlp"
        assert validated.atm_env["ATM_TEMPO_SEARCH_ENDPOINT"] == "https://synthetic-endpoint.example.com/tempo"
        assert validated.codex_env["CODEX_THREAD_ID"] == "00000000-0000-4000-8000-000000000000"
        assert validated.codex_env["CODEX_MANAGED_PACKAGE_ROOT"].strip()
        assert validated.cwd_from_getcwd.startswith("/synthetic/test/codex-harness")
        assert validated.pwd_env is None or validated.pwd_env.startswith("/synthetic/test/codex-harness")


@pytest.mark.provider_codex
def test_session_start_and_stop_capture_scripts_write_raw_files(tmp_path: Path, codex_root: Path) -> None:
    capture_root = tmp_path / "captures"
    project_root = tmp_path / "project"
    project_root.mkdir()
    transcript_path = tmp_path / "transcript.jsonl"

    session_start_result = _run_hook(
        codex_root / "hooks" / "session_start.py",
        {
            "cwd": str(project_root),
            "hook_event_name": "SessionStart",
            "model": "gpt-5.4",
            "permission_mode": "bypassPermissions",
            "session_id": "019e5106-079f-7121-9fef-f605ee1c0527",
            "source": "startup",
            "transcript_path": str(transcript_path),
        },
        {"SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root)},
    )
    assert session_start_result.returncode == 0, session_start_result.stderr

    stop_result = _run_hook(
        codex_root / "hooks" / "stop.py",
        {
            "cwd": str(project_root),
            "hook_event_name": "Stop",
            "session_id": "019e5106-079f-7121-9fef-f605ee1c0527",
        },
        {"SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root)},
    )
    assert stop_result.returncode == 0, stop_result.stderr

    created = sorted(path.name for path in capture_root.glob("*.json"))
    assert any(name.endswith("-session-start.json") for name in created)
    assert any(name.endswith("-session-start.env.json") for name in created)
    assert any(name.endswith("-stop.json") for name in created)
    assert any(name.endswith("-stop.env.json") for name in created)


@pytest.mark.provider_codex
def test_manifest_records_cd_scenario_through_approved_fixtures(codex_root: Path) -> None:
    fixture_dir = codex_root / "fixtures" / "approved"
    manifest = json.loads((fixture_dir / "manifest.json").read_text(encoding="utf-8"))
    startup = json.loads((fixture_dir / "session-start-startup.json").read_text(encoding="utf-8"))
    drift = json.loads((fixture_dir / "session-start-cwd-drift.json").read_text(encoding="utf-8"))

    assert startup["cwd"] != drift["cwd"]
    assert Path(drift["cwd"]).parts[-3:] == ("test-harness", "hooks", "codex")
    manifest_surfaces = {surface["surface"] for surface in manifest["hook_surfaces"]}
    assert {
        "SessionStart (--cd)",
        "PreToolUse (--cd)",
        "notify (--cd)",
    } <= manifest_surfaces
