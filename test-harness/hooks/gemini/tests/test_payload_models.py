import json
from pathlib import Path

import pytest

from test_harness.hooks.gemini.models import validate_gemini_hook_payload


@pytest.mark.provider_gemini
def test_all_approved_payload_fixtures_validate(gemini_root: Path) -> None:
    fixture_root = gemini_root / "fixtures" / "approved"
    for path in sorted(fixture_root.glob("*.json")):
        if path.name.endswith(".env.json") or path.name == "manifest.json":
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        validate_gemini_hook_payload(payload)


@pytest.mark.provider_gemini
def test_session_start_fixtures_cover_startup_and_resume(gemini_root: Path) -> None:
    fixture_root = gemini_root / "fixtures" / "approved"
    startup = json.loads((fixture_root / "session-start-startup.json").read_text(encoding="utf-8"))
    resume = json.loads((fixture_root / "session-start-resume.json").read_text(encoding="utf-8"))

    assert startup["hook_event_name"] == "SessionStart"
    assert startup["source"] == "startup"
    assert resume["hook_event_name"] == "SessionStart"
    assert resume["source"] == "resume"


@pytest.mark.provider_gemini
def test_tool_fixtures_preserve_run_shell_command_shape(gemini_root: Path) -> None:
    fixture_root = gemini_root / "fixtures" / "approved"
    before = json.loads((fixture_root / "before-tool.json").read_text(encoding="utf-8"))
    after = json.loads((fixture_root / "after-tool.json").read_text(encoding="utf-8"))

    assert before["tool_name"] == "run_shell_command"
    assert before["tool_input"]["command"] == "pwd"
    assert after["tool_name"] == "run_shell_command"
    assert after["tool_input"]["command"] == "pwd"
    assert "tool_response" in after


@pytest.mark.provider_gemini
def test_after_agent_fixture_preserves_retained_surface_fields(gemini_root: Path) -> None:
    fixture_root = gemini_root / "fixtures" / "approved"
    after_agent = json.loads((fixture_root / "after-agent.json").read_text(encoding="utf-8"))

    assert after_agent["hook_event_name"] == "AfterAgent"
    assert after_agent["prompt"] == "Reply with exactly OK."
    assert isinstance(after_agent["prompt_response"], str)
    assert after_agent["stop_hook_active"] is False
