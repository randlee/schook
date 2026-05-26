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
        "PYTHONPATH": str(Path.cwd()),
    }
    return subprocess.run(
        [sys.executable, str(script)],
        input=json.dumps(payload),
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )


@pytest.mark.provider_gemini
def test_fixture_manifest_matches_expected_hook_surfaces(gemini_root: Path, expected_surfaces: list[str]) -> None:
    manifest = json.loads((gemini_root / "fixtures" / "approved" / "manifest.json").read_text(encoding="utf-8"))

    assert manifest["provider"] == "gemini"
    assert sorted(surface["surface"] for surface in manifest["hook_surfaces"]) == sorted(expected_surfaces)


@pytest.mark.provider_gemini
def test_manifest_records_after_agent_as_retained_surface(gemini_root: Path) -> None:
    manifest = json.loads((gemini_root / "fixtures" / "approved" / "manifest.json").read_text(encoding="utf-8"))

    after_agent = next(surface for surface in manifest["hook_surfaces"] if surface["surface"] == "after-agent")
    assert after_agent["status"] == "captured"
    assert after_agent["payload_fixture"] == "after-agent.json"
    assert after_agent["env_fixture"] == "after-agent.env.json"
    assert after_agent["control_semantics"]["blocking"] is False
    assert "caveat" in after_agent


@pytest.mark.provider_gemini
def test_capture_scripts_write_raw_payload_and_env_files(tmp_path: Path, gemini_root: Path) -> None:
    hooks_dir = gemini_root / "hooks"
    capture_root = tmp_path / "captures"
    scripts = {
        "session-start": "session_start.py",
        "session-end": "session_end.py",
        "before-agent": "before_agent.py",
        "before-tool": "before_tool.py",
        "after-tool": "after_tool.py",
        "after-agent": "after_agent.py",
    }
    payload = {"hook_event_name": "TestHook", "session_id": "11111111-1111-1111-1111-111111111111"}

    for surface, filename in scripts.items():
        result = _run_hook(hooks_dir / filename, capture_root, {**payload, "surface": surface})
        assert result.returncode == 0, result.stderr

    payload_files = sorted(path.name for path in capture_root.glob("*.json") if not path.name.endswith(".env.json"))
    env_files = sorted(path.name for path in capture_root.glob("*.env.json"))
    assert len(payload_files) == len(scripts)
    assert len(env_files) == len(scripts)


@pytest.mark.provider_gemini
def test_approved_env_snapshots_exist_for_each_surface(gemini_root: Path, expected_surfaces: list[str]) -> None:
    fixture_root = gemini_root / "fixtures" / "approved"
    for surface in expected_surfaces:
        env_fixture = fixture_root / f"{surface}.env.json"
        assert env_fixture.is_file(), surface
        snapshot = json.loads(env_fixture.read_text(encoding="utf-8"))
        assert snapshot["hook_name"] == surface
        assert "gemini_env" in snapshot
        assert "process_env" in snapshot
        if "GEMINI_API_KEY" in snapshot["gemini_env"]:
            assert snapshot["gemini_env"]["GEMINI_API_KEY"] == "<redacted>"


@pytest.mark.provider_gemini
def test_approved_fixtures_redact_machine_local_paths(gemini_root: Path, expected_surfaces: list[str]) -> None:
    fixture_root = gemini_root / "fixtures" / "approved"
    machine_local_prefix = "/Users/" + "randlee/"
    synthetic_root = "/synthetic/test/gemini-harness"
    synthetic_home = "/synthetic/test/gemini-home"
    synthetic_plans = "/synthetic/test/gemini-plans"
    synthetic_transcript_prefixes = (
        "/synthetic/test/gemini-transcripts/",
        "/synthetic/test/gemini-harness/chats/",
    )

    for surface in expected_surfaces:
        payload_path = fixture_root / f"{surface}.json"
        env_path = fixture_root / f"{surface}.env.json"

        payload_text = payload_path.read_text(encoding="utf-8")
        env = json.loads(env_path.read_text(encoding="utf-8"))

        assert machine_local_prefix not in payload_text, payload_path.name
        assert "/tmp/schook-gemini-" not in payload_text, payload_path.name
        assert "/private/tmp/schook-gemini-" not in payload_text, payload_path.name
        assert "Process Group PGID: 31010" not in payload_text, payload_path.name
        assert machine_local_prefix not in json.dumps(env, sort_keys=True), env_path.name
        assert "/tmp/schook-gemini-" not in json.dumps(env, sort_keys=True), env_path.name
        assert "/private/tmp/schook-gemini-" not in json.dumps(env, sort_keys=True), env_path.name

        process_env = env["process_env"]
        gemini_env = env["gemini_env"]
        assert process_env["PATH"] == "<machine-path-redacted>"
        assert process_env["USER"] == "<operator>"
        if "PWD" in process_env:
            assert process_env["PWD"] == synthetic_root
        if "HOME" in process_env:
            assert process_env["HOME"] == synthetic_home
        if "GEMINI_CWD" in gemini_env:
            assert gemini_env["GEMINI_CWD"] == synthetic_root
        if "GEMINI_PROJECT_DIR" in gemini_env:
            assert gemini_env["GEMINI_PROJECT_DIR"] == synthetic_root
        if "GEMINI_PLANS_DIR" in gemini_env:
            assert gemini_env["GEMINI_PLANS_DIR"] == synthetic_plans
        payload = json.loads(payload_text)
        if "transcript_path" in payload:
            assert payload["transcript_path"].startswith(synthetic_transcript_prefixes)


@pytest.mark.provider_gemini
def test_gemini_hook_api_records_after_agent_as_retained_surface(gemini_root: Path) -> None:
    doc = (gemini_root.parents[2] / "docs" / "hook-api" / "gemini-hook-api.md").read_text(encoding="utf-8")

    assert "### `AfterAgent`" in doc
    assert "Retained Phase P harness disposition" in doc
    assert "`AfterAgent` is a maintained Gemini harness surface" in doc
