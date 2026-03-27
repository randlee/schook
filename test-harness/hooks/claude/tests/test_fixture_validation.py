import json
import os
import subprocess
import sys
from pathlib import Path

import pytest


def _run_hook(script: Path, capture_root: Path, payload: dict) -> subprocess.CompletedProcess[str]:
    env = {
        **os.environ,
        "SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root),
    }
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
        "pretooluse-task": "pre_tool_use_task.py",
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
    assert len(created_files) == len(expected_hooks)
    for hook_file_token in expected_hooks.values():
        assert any(name.endswith(f"{hook_file_token}.json") for name in created_files)


@pytest.mark.provider_claude
def test_runner_script_exists_and_is_executable(claude_root: Path) -> None:
    runner = claude_root / "scripts" / "run-capture.sh"
    assert runner.exists()
    assert runner.stat().st_mode & 0o111
