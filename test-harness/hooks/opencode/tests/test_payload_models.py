import json
from pathlib import Path

import pytest

from test_harness.hooks.opencode.models import SessionIdlePayload, validate_opencode_hook_payload
from test_harness.hooks.opencode.models.registry import OPENCODE_PAYLOAD_MODELS


@pytest.mark.provider_opencode
def test_all_approved_opencode_payload_fixtures_validate(opencode_root: Path) -> None:
    fixture_root = opencode_root / "fixtures" / "approved"
    for path in sorted(fixture_root.glob("*.json")):
        if path.name.endswith(".env.json") or path.name == "manifest.json":
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        validate_opencode_hook_payload(payload)


@pytest.mark.provider_opencode
def test_registry_matches_retained_manifest_surfaces(opencode_root: Path) -> None:
    manifest = json.loads(
        (opencode_root / "fixtures" / "approved" / "manifest.json").read_text(encoding="utf-8")
    )
    retained_surfaces = [surface["surface"] for surface in manifest["hook_surfaces"]]
    assert sorted(OPENCODE_PAYLOAD_MODELS) == sorted(retained_surfaces)
    assert len(OPENCODE_PAYLOAD_MODELS) == len(retained_surfaces)


@pytest.mark.provider_opencode
def test_session_idle_fixture_preserves_retained_reference_fields(opencode_root: Path) -> None:
    fixture = json.loads(
        (opencode_root / "fixtures" / "approved" / "session-idle.json").read_text(encoding="utf-8")
    )
    parsed = validate_opencode_hook_payload(fixture)
    assert isinstance(parsed, SessionIdlePayload)
    assert parsed.event.type == "session.idle"
