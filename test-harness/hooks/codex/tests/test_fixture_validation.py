from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

from test_harness.hooks.codex.models.payloads import (
    validate_codex_env_snapshot,
    validate_codex_fixture_manifest,
    validate_codex_hook_payload,
)


def _run_hook(
    script: Path,
    payload: dict[str, object],
    capture_root: Path,
) -> subprocess.CompletedProcess[str]:
    pythonpath = str(Path.cwd())
    if existing := os.environ.get("PYTHONPATH"):
        pythonpath = f"{pythonpath}{os.pathsep}{existing}"
    command = [sys.executable, str(script)]
    text_payload = json.dumps(payload)
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
        env={
            **os.environ,
            "PYTHONPATH": pythonpath,
            "SCHOOK_HOOK_CAPTURE_ROOT": str(capture_root),
            "HOOK_NAME_OVERRIDE": script.stem.replace("_", "-"),
            "ATM_IDENTITY": "fixture-tester",
            "ATM_TEAM": "schook",
            "ATM_READ_TOKEN": "secret-token",
        },
        check=False,
    )


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
def test_approved_payload_fixtures_validate_against_models(codex_root: Path) -> None:
    fixture_dir = codex_root / "fixtures" / "approved"
    for fixture_path in sorted(fixture_dir.glob("*.json")):
        if fixture_path.name == "manifest.json" or fixture_path.name.endswith(".env.json"):
            continue
        validate_codex_hook_payload(json.loads(fixture_path.read_text(encoding="utf-8")))


@pytest.mark.provider_codex
def test_approved_env_fixtures_validate_and_redact_sensitive_values(codex_root: Path) -> None:
    fixture_dir = codex_root / "fixtures" / "approved"
    for fixture_path in sorted(fixture_dir.glob("*.env.json")):
        validated = validate_codex_env_snapshot(json.loads(fixture_path.read_text(encoding="utf-8")))
        assert validated.atm_env["ATM_IDENTITY"] == "chook"
        assert validated.atm_env["ATM_TEAM"] == "schook"
        assert validated.atm_env["ATM_GRAFANA_READ_TOKEN"] == "<redacted>"
        for key, value in validated.atm_env.items():
            if any(token in key for token in ("TOKEN", "AUTH")):
                assert value == "<redacted>"


@pytest.mark.provider_codex
def test_session_start_and_stop_capture_scripts_write_raw_files(tmp_path: Path, codex_root: Path) -> None:
    capture_root = tmp_path / "captures"

    session_start_result = _run_hook(
        codex_root / "hooks" / "session_start.py",
        {
            "cwd": "/tmp/project",
            "hook_event_name": "SessionStart",
            "model": "gpt-5.4",
            "permission_mode": "bypassPermissions",
            "session_id": "019e5106-079f-7121-9fef-f605ee1c0527",
            "source": "startup",
            "transcript_path": "/tmp/transcript.jsonl",
        },
        capture_root,
    )
    assert session_start_result.returncode == 0, session_start_result.stderr

    stop_result = _run_hook(
        codex_root / "hooks" / "stop.py",
        {
            "cwd": "/tmp/project",
            "hook_event_name": "Stop",
            "session_id": "019e5106-079f-7121-9fef-f605ee1c0527",
        },
        capture_root,
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
    startup = json.loads((fixture_dir / "session-start-startup.json").read_text(encoding="utf-8"))
    drift = json.loads((fixture_dir / "session-start-cwd-drift.json").read_text(encoding="utf-8"))

    assert startup["cwd"] != drift["cwd"]
    assert drift["cwd"].endswith("test-harness/hooks/codex")
