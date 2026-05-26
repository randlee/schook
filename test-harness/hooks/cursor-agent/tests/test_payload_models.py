import json
from pathlib import Path

import pytest

from test_harness.hooks.cursor_agent.models import StopPayload, validate_cursor_agent_hook_payload
from test_harness.hooks.cursor_agent.models.registry import CURSOR_AGENT_PAYLOAD_MODELS


@pytest.mark.provider_cursor_agent
def test_all_approved_cursor_payload_fixtures_validate(cursor_agent_root: Path) -> None:
    fixture_root = cursor_agent_root / "fixtures" / "approved"
    for path in sorted(fixture_root.glob("*.json")):
        if path.name.endswith(".env.json") or path.name == "manifest.json":
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        validate_cursor_agent_hook_payload(payload)


@pytest.mark.provider_cursor_agent
def test_registry_matches_retained_manifest_surfaces(cursor_agent_root: Path) -> None:
    manifest = json.loads(
        (cursor_agent_root / "fixtures" / "approved" / "manifest.json").read_text(encoding="utf-8")
    )
    retained_surfaces = [surface["surface"] for surface in manifest["hook_surfaces"]]
    assert sorted(CURSOR_AGENT_PAYLOAD_MODELS) == sorted(retained_surfaces)
    assert len(CURSOR_AGENT_PAYLOAD_MODELS) == len(retained_surfaces)


@pytest.mark.provider_cursor_agent
def test_stop_fixture_preserves_retained_reference_fields(cursor_agent_root: Path) -> None:
    fixture = json.loads(
        (cursor_agent_root / "fixtures" / "approved" / "stop.json").read_text(encoding="utf-8")
    )
    parsed = validate_cursor_agent_hook_payload(fixture)
    assert isinstance(parsed, StopPayload)
    assert parsed.hook_event_name == "stop"
    assert parsed.transcript_path == "/synthetic/test/cursor-agent/transcript.jsonl"
    assert parsed.message_count == 2
    assert parsed.tool_call_count == 1
