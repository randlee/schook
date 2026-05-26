from __future__ import annotations

import json
from pathlib import Path

import pytest

from test_harness.hooks.normalization import (
    CORE_NORMALIZATION_SPECS,
    NormalizationFixtureError,
    load_normalization_fixture,
)


def test_missing_fixture_reports_structured_error(tmp_path: Path) -> None:
    with pytest.raises(NormalizationFixtureError) as exc_info:
        load_normalization_fixture(
            repo_root=tmp_path,
            provider="claude",
            fixture_relpath="missing.json",
            required_fields=("session_id",),
        )

    error = exc_info.value
    assert error.provider == "claude"
    assert error.fixture_path == "missing.json"
    assert error.field == "fixture"
    assert error.reason == "fixture file does not exist"


def test_unparseable_fixture_reports_structured_error(tmp_path: Path) -> None:
    fixture_path = tmp_path / "broken.json"
    fixture_path.write_text("{invalid json", encoding="utf-8")

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


def test_missing_required_field_reports_structured_error(tmp_path: Path) -> None:
    fixture_path = tmp_path / "fixture.json"
    fixture_path.write_text(json.dumps({"session_id": "1234"}), encoding="utf-8")

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
