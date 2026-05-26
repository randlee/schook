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
        "HOOK_NAME_OVERRIDE": "session.idle",
        "HOME": "/synthetic/test/opencode/home",
        "PATH": os.environ.get("PATH", ""),
        "PWD": "/synthetic/test/opencode/workspace",
        "OPENCODE_CONFIG_DIR": "/synthetic/test/opencode/home/.config/opencode",
    }
    return subprocess.run(
        [sys.executable, str(script)],
        input=json.dumps(payload),
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )


@pytest.mark.provider_opencode
def test_manifest_matches_expected_opencode_surfaces(
    opencode_root: Path, expected_surfaces: list[str]
) -> None:
    manifest = json.loads(
        (opencode_root / "fixtures" / "approved" / "manifest.json").read_text(encoding="utf-8")
    )
    assert manifest["provider"] == "opencode"
    assert manifest["evidence_mode"] == "approved-reference"
    assert sorted(surface["surface"] for surface in manifest["hook_surfaces"]) == sorted(expected_surfaces)


@pytest.mark.provider_opencode
def test_manifest_is_non_empty_and_records_doc_backed_status(opencode_root: Path) -> None:
    manifest = json.loads(
        (opencode_root / "fixtures" / "approved" / "manifest.json").read_text(encoding="utf-8")
    )
    assert manifest["hook_surfaces"]
    retained_surface = manifest["hook_surfaces"][0]
    assert retained_surface["surface"] == "session.idle"
    assert retained_surface["status"] == "approved-reference"
    assert retained_surface["payload_fixture"] == "session-idle.json"
    assert retained_surface["env_fixture"] == "session-idle.env.json"


@pytest.mark.provider_opencode
def test_session_idle_capture_script_writes_payload_and_env_files(tmp_path: Path, opencode_root: Path) -> None:
    capture_root = tmp_path / "captures"
    payload = {"event": {"type": "session.idle"}}
    result = _run_hook(opencode_root / "hooks" / "session_idle.py", capture_root, payload)
    assert result.returncode == 0, result.stderr

    payload_files = sorted(
        path.name for path in capture_root.glob("*.json") if not path.name.endswith(".env.json")
    )
    env_files = sorted(path.name for path in capture_root.glob("*.env.json"))
    assert payload_files == ["session-idle.json"]
    assert env_files == ["session-idle.env.json"]


@pytest.mark.provider_opencode
def test_approved_env_snapshot_exists_for_retained_surface(
    opencode_root: Path, expected_surfaces: list[str]
) -> None:
    fixture_root = opencode_root / "fixtures" / "approved"
    for surface in expected_surfaces:
        env_fixture = fixture_root / f"{surface.replace('.', '-')}.env.json"
        assert env_fixture.is_file(), surface
        snapshot = json.loads(env_fixture.read_text(encoding="utf-8"))
        assert snapshot["hook_name"] == surface
        assert "opencode_env" in snapshot
        assert "process_env" in snapshot
