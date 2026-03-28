import json
from pathlib import Path

import pytest

from test_harness.hooks.claude.models.payloads import ClaudeHookPayload, validate_claude_hook_payload


@pytest.mark.provider_claude
def test_claude_hook_payload_imports() -> None:
    assert ClaudeHookPayload is not None


@pytest.mark.provider_claude
def test_all_approved_fixtures_validate_against_claude_hook_payload(claude_root: Path) -> None:
    fixture_dir = claude_root / "fixtures" / "approved"
    for fixture_path in sorted(fixture_dir.glob("*.json")):
        if fixture_path.name == "manifest.json":
            continue
        payload = json.loads(fixture_path.read_text(encoding="utf-8"))
        validated = validate_claude_hook_payload(payload)
        assert validated.payload is not None
