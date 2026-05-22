from __future__ import annotations

import json
import os
import stat
import subprocess
import sys
import time
from pathlib import Path

import pytest


def _write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


def _run_hook(script: Path, payload: dict[str, object], env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    pythonpath = str(Path.cwd())
    if existing := os.environ.get("PYTHONPATH"):
        pythonpath = f"{pythonpath}{os.pathsep}{existing}"
    return subprocess.run(
        [sys.executable, str(script)],
        input=json.dumps(payload),
        text=True,
        capture_output=True,
        env={**os.environ, **env, "PYTHONPATH": pythonpath},
        check=False,
    )


@pytest.mark.provider_codex
def test_codex_harness_layout_exists(codex_root: Path) -> None:
    for name in ["hooks", "scripts", "tests"]:
        assert (codex_root / name).is_dir(), name


@pytest.mark.provider_codex
def test_stop_hook_schedules_pending_record(tmp_path: Path, codex_root: Path) -> None:
    capture_root = tmp_path / "captures"
    state_root = tmp_path / "state"
    repo_root = tmp_path / "repo"
    repo_root.mkdir()

    result = _run_hook(
        codex_root / "hooks" / "stop.py",
        {"type": "agent-turn-complete", "thread-id": "thread-123", "state": "idle", "cwd": str(repo_root)},
        {
            "SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root),
            "SCHOOK_CODEX_HOOK_STATE_ROOT": str(state_root),
            "SCHOOK_CODEX_HOOK_PROJECT_ROOT": str(repo_root),
            "SCHOOK_CODEX_DEBOUNCE_SECONDS": "60",
            "SCHOOK_CODEX_DEBOUNCE_AUTOSTART": "0",
            "CODEX_PROJECT_DIR": str(repo_root),
        },
    )

    assert result.returncode == 0, result.stderr
    pending_files = sorted((state_root / "pending").glob("*.json"))
    assert len(pending_files) == 1
    pending = json.loads(pending_files[0].read_text(encoding="utf-8"))
    assert pending["key"] == "thread-123"
    assert pending["project_dir"] == str(repo_root)
    assert pending["payload"]["type"] == "agent-turn-complete"

    payload_captures = sorted(capture_root.glob("*.json"))
    env_captures = sorted(capture_root.glob("*.env.json"))
    assert any(path.name.endswith("-stop.json") for path in payload_captures)
    assert any(path.name.endswith("-stop.env.json") for path in env_captures)


@pytest.mark.provider_codex
def test_pretooluse_cancels_pending_debounce(tmp_path: Path, codex_root: Path) -> None:
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
    }

    stop_result = _run_hook(
        codex_root / "hooks" / "stop.py",
        {"type": "agent-turn-complete", "thread-id": "thread-123", "cwd": str(repo_root)},
        common_env,
    )
    assert stop_result.returncode == 0, stop_result.stderr

    cancel_result = _run_hook(
        codex_root / "hooks" / "pre_tool_use.py",
        {"hook_event_name": "PreToolUse", "thread-id": "thread-123", "cwd": str(repo_root)},
        common_env,
    )
    assert cancel_result.returncode == 0, cancel_result.stderr
    assert not list((state_root / "pending").glob("*.json"))

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
    }

    stop_result = _run_hook(
        codex_root / "hooks" / "stop.py",
        {"type": "agent-turn-complete", "thread-id": "thread-456", "cwd": str(repo_root)},
        env,
    )
    assert stop_result.returncode == 0, stop_result.stderr

    time.sleep(0.08)
    fire_result = _run_hook(codex_root / "scripts" / "fire_pending.py", {}, env)
    assert fire_result.returncode == 0, fire_result.stderr
    second_fire_result = _run_hook(codex_root / "scripts" / "fire_pending.py", {}, env)
    assert second_fire_result.returncode == 0, second_fire_result.stderr

    assert output_path.read_text(encoding="utf-8") == "fired\n"
    assert not list((state_root / "pending").glob("*.json"))


@pytest.mark.provider_codex
def test_project_scope_blocks_outside_directories(tmp_path: Path, codex_root: Path) -> None:
    capture_root = tmp_path / "captures"
    state_root = tmp_path / "state"
    project_root = tmp_path / "allowed"
    project_root.mkdir()
    outside_root = tmp_path / "outside"
    outside_root.mkdir()

    result = _run_hook(
        codex_root / "hooks" / "stop.py",
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
