import json
import os
import subprocess
import sys
from pathlib import Path

import pytest


def _run_hook(
    script: Path,
    capture_root: Path,
    payload: dict,
    extra_env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    env = {
        **os.environ,
        "SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root),
    }
    if extra_env:
        env.update(extra_env)
    return subprocess.run(
        [sys.executable, str(script)],
        input=json.dumps(payload),
        text=True,
        capture_output=True,
        check=True,
        env=env,
    )


@pytest.mark.provider_claude
def test_fixture_manifest_matches_expected_hook_surfaces(claude_root: Path, expected_hooks: dict[str, str]) -> None:
    manifest_path = claude_root / "fixtures" / "approved" / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    assert manifest["provider"] == "claude"
    assert set(manifest["hooks"]) == set(expected_hooks)


@pytest.mark.provider_claude
def test_capture_scripts_write_raw_payload_files(tmp_path: Path, claude_root: Path, expected_hooks: dict[str, str]) -> None:
    hooks_dir = claude_root / "hooks"
    capture_root = tmp_path / "captures"

    script_names = {
        "session-start": "session_start.py",
        "session-end": "session_end.py",
        "pre-compact": "pre_compact.py",
        "pretooluse-bash": "pre_tool_use_bash.py",
        "posttooluse-bash": "post_tool_use_bash.py",
        "pretooluse-agent": "pre_tool_use_agent.py",
        "permission-request": "permission_request.py",
        "notification": "notification.py",
        "stop": "stop.py",
    }

    for hook_file_token in expected_hooks.values():
        payload = {"hook_name": hook_file_token, "session_id": "test-session"}
        result = _run_hook(hooks_dir / script_names[hook_file_token], capture_root, payload)
        assert result.stdout == ""
        assert result.stderr == ""

    created_files = sorted(path.name for path in capture_root.glob("*.json"))
    payload_files = [name for name in created_files if not name.endswith(".env.json")]
    env_files = [name for name in created_files if name.endswith(".env.json")]
    assert len(payload_files) == len(expected_hooks)
    assert len(env_files) == len(expected_hooks)
    for hook_file_token in expected_hooks.values():
        assert any(name.endswith(f"{hook_file_token}.json") for name in payload_files)
        env_matches = [name for name in env_files if name.endswith(f"{hook_file_token}.env.json")]
        assert env_matches, hook_file_token
        env_snapshot = json.loads((capture_root / env_matches[0]).read_text(encoding="utf-8"))
        assert env_snapshot["hook_name"] == hook_file_token
        assert "claude_env" in env_snapshot
        assert "atm_env" in env_snapshot
        assert "sc_hook_env" in env_snapshot


@pytest.mark.provider_claude
def test_capture_scripts_redact_sensitive_env_values(tmp_path: Path, claude_root: Path) -> None:
    hooks_dir = claude_root / "hooks"
    capture_root = tmp_path / "captures"
    payload = {"hook_name": "session-start", "session_id": "test-session"}
    result = _run_hook(
        hooks_dir / "session_start.py",
        capture_root,
        payload,
        extra_env={
            "ATM_IDENTITY": "test-chook",
            "ATM_TEAM": "atm-dev",
            "ATM_READ_TOKEN": "secret-token",
            "CLAUDE_PROJECT_DIR": "/tmp/project-root",
        },
    )
    assert result.stdout == ""
    assert result.stderr == ""

    env_snapshot_path = next(capture_root.glob("*.env.json"))
    env_snapshot = json.loads(env_snapshot_path.read_text(encoding="utf-8"))
    assert env_snapshot["atm_env"]["ATM_IDENTITY"] == "test-chook"
    assert env_snapshot["atm_env"]["ATM_TEAM"] == "atm-dev"
    assert env_snapshot["atm_env"]["ATM_READ_TOKEN"] == "<redacted>"
    assert env_snapshot["claude_env"]["CLAUDE_PROJECT_DIR"] == "/tmp/project-root"


@pytest.mark.provider_claude
def test_runner_script_exists_and_is_executable(claude_root: Path) -> None:
    runner = claude_root / "scripts" / "run-capture.sh"
    assert runner.exists()
    assert runner.stat().st_mode & 0o111


@pytest.mark.provider_claude
def test_pretooluse_agent_fixture_matches_captured_spawn_shape(claude_root: Path) -> None:
    fixture_path = claude_root / "fixtures" / "approved" / "pretooluse-agent.json"
    fixture = json.loads(fixture_path.read_text(encoding="utf-8"))

    assert fixture["hook_event_name"] == "PreToolUse"
    assert fixture["tool_name"] == "Agent"
    assert fixture["session_id"]
    assert fixture["tool_use_id"]

    tool_input = fixture["tool_input"]
    assert tool_input["name"] == "harness-child"
    assert tool_input["run_in_background"] is True
    assert isinstance(tool_input["prompt"], str) and tool_input["prompt"]
    assert isinstance(tool_input["description"], str) and tool_input["description"]


@pytest.mark.provider_claude
def test_session_start_fixtures_cover_startup_compact_and_resume(claude_root: Path) -> None:
    fixture_dir = claude_root / "fixtures" / "approved"
    observed_sources = {}
    for filename in [
        "session-start-startup.json",
        "session-start-compact.json",
        "session-start-resume.json",
        "session-start-clear.json",
    ]:
        fixture = json.loads((fixture_dir / filename).read_text(encoding="utf-8"))
        observed_sources[filename] = fixture["source"]

    assert observed_sources == {
        "session-start-startup.json": "startup",
        "session-start-compact.json": "compact",
        "session-start-resume.json": "resume",
        "session-start-clear.json": "clear",
    }


@pytest.mark.provider_claude
def test_clear_transition_produces_session_end_reason_clear_and_new_start(claude_root: Path) -> None:
    fixture_dir = claude_root / "fixtures" / "approved"
    session_end = json.loads((fixture_dir / "session-end-clear.json").read_text(encoding="utf-8"))
    session_start = json.loads((fixture_dir / "session-start-clear.json").read_text(encoding="utf-8"))

    assert session_end["hook_event_name"] == "SessionEnd"
    assert session_end["reason"] == "clear"
    assert session_start["hook_event_name"] == "SessionStart"
    assert session_start["source"] == "clear"
    assert session_end["session_id"] != session_start["session_id"]
