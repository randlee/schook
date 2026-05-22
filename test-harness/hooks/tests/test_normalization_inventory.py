from __future__ import annotations

import json
from pathlib import Path

import pytest

from test_harness.hooks.normalization import (
    CORE_NORMALIZATION_SPECS,
    NormalizationFixtureError,
    collect_core_normalization_observations,
    load_normalization_fixture,
)


REPO_ROOT = Path(__file__).resolve().parents[3]


def test_collect_core_normalization_observations_reads_expected_provider_surfaces() -> None:
    observations = collect_core_normalization_observations(REPO_ROOT)

    assert observations["claude_session_start"].fields["hook_event_name"] == "SessionStart"
    assert observations["codex_notify"].fields["type"] == "agent-turn-complete"
    assert observations["codex_pretooluse"].fields["tool_input.command"] == "pwd"
    assert observations["gemini_before_tool"].fields["tool_name"] == "run_shell_command"
    assert observations["gemini_after_agent"].fields["stop_hook_active"] is False


def test_load_normalization_fixture_reports_missing_field_structurally(tmp_path: Path) -> None:
    fixture_path = tmp_path / "fixture.json"
    fixture_path.write_text(json.dumps({"session_id": "1234"}))

    with pytest.raises(NormalizationFixtureError) as exc_info:
        load_normalization_fixture(
            repo_root=tmp_path,
            provider="gemini",
            fixture_relpath="fixture.json",
            required_fields=("session_id", "cwd"),
        )

    error = exc_info.value
    assert error.provider == "gemini"
    assert error.fixture_path == "fixture.json"
    assert error.field == "cwd"
    assert error.reason == "required field missing from approved fixture"


def test_load_normalization_fixture_reports_invalid_json_structurally(tmp_path: Path) -> None:
    fixture_path = tmp_path / "broken.json"
    fixture_path.write_text("{invalid json")

    with pytest.raises(NormalizationFixtureError) as exc_info:
        load_normalization_fixture(
            repo_root=tmp_path,
            provider="codex",
            fixture_relpath="broken.json",
            required_fields=CORE_NORMALIZATION_SPECS["codex_session_start"].required_fields,
        )

    error = exc_info.value
    assert error.provider == "codex"
    assert error.fixture_path == "broken.json"
    assert error.field == "fixture"
    assert error.reason.startswith("fixture is not valid JSON:")
