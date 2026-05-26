from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest


def _run_hook(script: Path, capture_root: Path, payload: dict[str, object]) -> subprocess.CompletedProcess[str]:
    env = {
        **os.environ,
        "SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root),
        "HOOK_NAME_OVERRIDE": script.stem.replace("_", "-"),
        "CURSOR_PROJECT_DIR": "/synthetic/test/cursor-agent/workspace",
        "CURSOR_AGENT_HOME": "/synthetic/test/cursor-agent/home",
    }
    return subprocess.run(
        [sys.executable, str(script)],
        input=json.dumps(payload),
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )


@pytest.mark.provider_cursor_agent
def test_manifest_matches_expected_cursor_surfaces(
    cursor_agent_root: Path, expected_surfaces: list[str]
) -> None:
    manifest = json.loads(
        (cursor_agent_root / "fixtures" / "approved" / "manifest.json").read_text(encoding="utf-8")
    )
    assert manifest["provider"] == "cursor-agent"
    assert manifest["evidence_mode"] == "approved-reference"
    assert sorted(surface["surface"] for surface in manifest["hook_surfaces"]) == sorted(expected_surfaces)


@pytest.mark.provider_cursor_agent
def test_manifest_is_non_empty_and_records_doc_backed_status(cursor_agent_root: Path) -> None:
    manifest = json.loads(
        (cursor_agent_root / "fixtures" / "approved" / "manifest.json").read_text(encoding="utf-8")
    )
    assert manifest["hook_surfaces"]
    stop_surface = manifest["hook_surfaces"][0]
    assert stop_surface["surface"] == "stop"
    assert stop_surface["status"] == "approved-reference"
    assert stop_surface["payload_fixture"] == "stop.json"
    assert stop_surface["env_fixture"] == "stop.env.json"


@pytest.mark.provider_cursor_agent
def test_stop_capture_script_writes_payload_and_env_files(tmp_path: Path, cursor_agent_root: Path) -> None:
    capture_root = tmp_path / "captures"
    payload = {
        "hook_event_name": "stop",
        "transcript_path": "/synthetic/test/cursor-agent/transcript.jsonl",
        "tool_call_count": 1,
    }
    result = _run_hook(cursor_agent_root / "hooks" / "stop.py", capture_root, payload)
    assert result.returncode == 0, result.stderr

    payload_files = sorted(
        path.name for path in capture_root.glob("*.json") if not path.name.endswith(".env.json")
    )
    env_files = sorted(path.name for path in capture_root.glob("*.env.json"))
    assert len(payload_files) == 1
    assert len(env_files) == 1


@pytest.mark.provider_cursor_agent
def test_approved_env_snapshot_exists_for_retained_surface(
    cursor_agent_root: Path, expected_surfaces: list[str]
) -> None:
    fixture_root = cursor_agent_root / "fixtures" / "approved"
    for surface in expected_surfaces:
        env_fixture = fixture_root / f"{surface}.env.json"
        assert env_fixture.is_file(), surface
        snapshot = json.loads(env_fixture.read_text(encoding="utf-8"))
        assert snapshot["hook_name"] == surface
        assert "cursor_env" in snapshot
        assert "process_env" in snapshot
