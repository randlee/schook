import json
from pathlib import Path

import pytest

from test_harness.hooks.claude.models.payloads import DriftErrorCode
from test_harness.hooks.claude.schema_drift import classify_payload_drift, run_drift


def _load_fixture(claude_root: Path, name: str) -> dict:
    return json.loads((claude_root / "fixtures" / "approved" / name).read_text(encoding="utf-8"))


@pytest.mark.provider_claude
def test_extra_field_generates_field_added(claude_root: Path) -> None:
    baseline = _load_fixture(claude_root, "pretooluse-agent.json")
    candidate = json.loads(json.dumps(baseline))
    candidate["tool_input"]["extra_flag"] = "new"

    entries = classify_payload_drift(baseline_payload=baseline, candidate_payload=candidate)

    assert any(
        entry.error_code == DriftErrorCode.FIELD_ADDED and entry.field_name == "tool_input.extra_flag"
        for entry in entries
    )


@pytest.mark.provider_claude
def test_required_field_removal_generates_required_field_removed(claude_root: Path) -> None:
    baseline = _load_fixture(claude_root, "pretooluse-agent.json")
    candidate = json.loads(json.dumps(baseline))
    del candidate["tool_input"]["prompt"]

    entries = classify_payload_drift(baseline_payload=baseline, candidate_payload=candidate)

    assert any(
        entry.error_code == DriftErrorCode.REQUIRED_FIELD_REMOVED and entry.field_name == "tool_input.prompt"
        for entry in entries
    )


@pytest.mark.provider_claude
def test_optional_field_removal_generates_optional_field_removed(claude_root: Path) -> None:
    baseline = _load_fixture(claude_root, "session-end-clear.json")
    candidate = json.loads(json.dumps(baseline))
    del candidate["reason"]

    entries = classify_payload_drift(baseline_payload=baseline, candidate_payload=candidate)

    assert any(
        entry.error_code == DriftErrorCode.OPTIONAL_FIELD_REMOVED and entry.field_name == "reason"
        for entry in entries
    )


@pytest.mark.provider_claude
def test_type_change_generates_field_type_changed(claude_root: Path) -> None:
    baseline = _load_fixture(claude_root, "stop.json")
    candidate = json.loads(json.dumps(baseline))
    candidate["stop_hook_active"] = {"unexpected": True}

    entries = classify_payload_drift(baseline_payload=baseline, candidate_payload=candidate)

    assert any(
        entry.error_code == DriftErrorCode.FIELD_TYPE_CHANGED and entry.field_name == "stop_hook_active"
        for entry in entries
    )


@pytest.mark.provider_claude
def test_html_report_structure_is_self_contained(tmp_path: Path) -> None:
    report = run_drift(tmp_path / "claude")
    html_path = Path(report.report_path)
    html_text = html_path.read_text(encoding="utf-8")

    assert html_text.startswith("<!DOCTYPE html>")
    assert '<meta charset="UTF-8">' in html_text or '<meta charset="UTF-8" />' in html_text
    assert "<link " not in html_text
    assert "<script src=" not in html_text

