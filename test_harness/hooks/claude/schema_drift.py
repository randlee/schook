from __future__ import annotations

import html
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from jinja2 import Environment, select_autoescape
from markupsafe import Markup

from test_harness.hooks.claude.models.payloads import (
    ClaudeHookPayload,
    DriftEntry,
    DriftErrorCode,
    DriftReport,
    ProviderStatus,
    validate_claude_hook_payload,
)
from test_harness.hooks.paths import (
    CLAUDE_DRIFT_HISTORY_ROOT,
    CLAUDE_FIXTURES_ROOT,
    CLAUDE_REPORTS_ROOT,
    CLAUDE_ROOT,
    CLAUDE_SCHEMA_ROOT,
    GLOBAL_HTML_SKILL_ROOT,
)


def _utc_timestamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")


def _atomic_write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_name(f"{path.name}.tmp")
    try:
        tmp_path.write_text(content, encoding="utf-8")
        tmp_path.replace(path)
    except Exception:
        if tmp_path.exists():
            tmp_path.unlink()
        raise


def _atomic_write_json(path: Path, payload: Any) -> None:
    _atomic_write_text(path, json.dumps(payload, indent=2, sort_keys=True) + "\n")


def _load_manifest() -> dict[str, Any]:
    return json.loads((CLAUDE_FIXTURES_ROOT / "manifest.json").read_text(encoding="utf-8"))


def _load_fixture(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _load_template_source(name: str) -> str:
    template_path = GLOBAL_HTML_SKILL_ROOT / name
    raw = template_path.read_text(encoding="utf-8")
    if raw.startswith("---\n"):
        parts = raw.split("\n---\n", 1)
        if len(parts) == 2:
            raw = parts[1]
    if name.endswith(".html.j2"):
        raw = raw.replace(' type="text/javascript"', "")
    return raw


def _render_section_card(
    *,
    hook_name: str,
    summary: str,
    json_payload: dict[str, Any],
    context_text: str,
    fragment_relpath: str | None,
) -> str:
    copy_json = html.escape(json.dumps(json_payload, separators=(",", ":")), quote=True)
    copy_context = html.escape(context_text, quote=True)
    buttons = (
        "<div class=\"section-actions\">"
        f"<button type=\"button\" class=\"icon-button\" title=\"Copy JSON\" data-copy-text=\"{copy_json}\">{{}}</button>"
        f"<button type=\"button\" class=\"icon-button\" title=\"Copy Context\" data-copy-text=\"{copy_context}\">⎘</button>"
        "</div>"
    )
    fragment_line = ""
    if fragment_relpath:
        fragment_line = (
            f"<p class=\"fragments\">XHTML fragment: <code>{fragment_relpath}</code> "
            "(agent-generated commentary optional)</p>"
        )
    return (
        "<section class=\"section\">"
        "<div class=\"section-head\">"
        f"<div><h2>{hook_name}</h2><p class=\"subtitle\">Status: PASS</p></div>"
        f"{buttons}"
        "</div>"
        f"<p>{summary}</p>"
        f"{fragment_line}"
        "</section>"
    )


def _html_validate(path: Path) -> None:
    result = subprocess.run(
        ["npx", "--yes", "html-validate", str(path)],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or result.stdout.strip() or "html-validate failed")


def _xmllint(path: Path) -> None:
    result = subprocess.run(
        ["xmllint", "--noout", str(path)],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or result.stdout.strip() or "xmllint failed")


def _build_environment() -> Environment:
    return Environment(
        autoescape=select_autoescape(enabled_extensions=("html", "xhtml", "xml")),
    )


def _generate_report(run_timestamp: str, validated_by_hook: dict[str, list[str]]) -> tuple[Path, list[Path]]:
    report_dir = CLAUDE_REPORTS_ROOT / run_timestamp
    report_dir.mkdir(parents=True, exist_ok=True)
    sections_dir = report_dir / "sections"
    sections_dir.mkdir(parents=True, exist_ok=True)

    env = _build_environment()
    report_template = env.from_string(_load_template_source("report-template.html.j2"))
    section_template = env.from_string(_load_template_source("section-template.xhtml.j2"))

    section_paths: list[Path] = []
    section_cards: list[str] = []
    for hook_name, fixtures in sorted(validated_by_hook.items()):
        summary = f"{len(fixtures)} approved fixture(s) validated successfully for {hook_name}."
        json_payload = {
            "hook_event_name": hook_name,
            "validated_fixtures": fixtures,
            "status": "PASS",
        }
        context_text = (
            f"{hook_name}: PASS. {len(fixtures)} approved fixture(s) validated successfully. "
            "No schema drift was detected for this run."
        )

        fragment_relpath: str | None = None
        if fixtures:
            slug = hook_name.lower().replace("(", "-").replace(")", "").replace(" ", "-")
            fragment_path = sections_dir / f"{slug}.xhtml"
            fragment_relpath = str(fragment_path.relative_to(report_dir))
            fragment_html = section_template.render(
                title=hook_name,
                header_color="#1e40af",
                accent_color="#3b82f6",
                fragment_source="auto-generated",
                copy_json=json.dumps(json_payload, separators=(",", ":")),
                copy_context=context_text,
                body_html=Markup(f"<p>{html.escape(summary)}</p>"),
            )
            _atomic_write_text(fragment_path, fragment_html)
            _xmllint(fragment_path)
            section_paths.append(fragment_path)

        section_cards.append(
            _render_section_card(
                hook_name=hook_name,
                summary=summary,
                json_payload=json_payload,
                context_text=context_text,
                fragment_relpath=fragment_relpath,
            )
        )

    report_html = report_template.render(
        output_path=str(report_dir / "schema-drift-report.html"),
        json_output_path=str(report_dir / "schema-drift-report.json"),
        title="Claude Hook Schema Drift Report",
        subtitle="Approved fixture validation",
        generated_at=run_timestamp,
        status="PASS",
        status_class="status-pass",
        summary_html=Markup("<h2>Summary</h2><p>All approved Claude fixtures validated against the Phase 3 Pydantic model.</p>"),
        sections_html=Markup("\n".join(section_cards)),
        recommendations_html=Markup("<ul><li>No drift detected. Continue using the approved fixtures as the current Claude baseline.</li></ul>"),
        footer_html=Markup("<p>Generated from the Sprint 9 Phase 3 schema drift tool.</p>"),
    )
    report_path = report_dir / "schema-drift-report.html"
    _atomic_write_text(report_path, report_html)
    _html_validate(report_path)
    return report_path, section_paths


def _write_schema_artifacts() -> dict[str, str]:
    CLAUDE_SCHEMA_ROOT.mkdir(parents=True, exist_ok=True)
    payload_schema_path = CLAUDE_SCHEMA_ROOT / "claude-hook-payload.schema.json"
    drift_schema_path = CLAUDE_SCHEMA_ROOT / "drift-report.schema.json"
    _atomic_write_json(payload_schema_path, ClaudeHookPayload.model_json_schema())
    _atomic_write_json(drift_schema_path, DriftReport.model_json_schema())
    return {
        "claude_hook_payload": str(payload_schema_path),
        "drift_report": str(drift_schema_path),
    }


def run_drift(output_dir: Path) -> DriftReport:
    manifest = _load_manifest()
    validated_fixtures: list[str] = []
    validated_by_hook: dict[str, list[str]] = {}

    for hook_name, filenames in manifest["hooks"].items():
        validated_by_hook[hook_name] = []
        for filename in filenames:
            fixture_path = CLAUDE_FIXTURES_ROOT / filename
            payload = _load_fixture(fixture_path)
            validate_claude_hook_payload(payload)
            validated_fixtures.append(filename)
            validated_by_hook[hook_name].append(filename)

    run_timestamp = _utc_timestamp()
    schema_paths = _write_schema_artifacts()
    report_path, section_paths = _generate_report(run_timestamp, validated_by_hook)

    drift_history_path = output_dir / "drift-history" / f"{run_timestamp}-drift.json"
    report = DriftReport(
        provider="claude",
        run_timestamp=run_timestamp,
        status=ProviderStatus.PASS,
        entries=[],
        validated_fixtures=validated_fixtures,
        schema_paths=schema_paths,
        drift_history_path=str(drift_history_path),
        report_path=str(report_path),
        section_paths=[str(path) for path in section_paths],
    )
    _atomic_write_json(drift_history_path, report.model_dump(mode="json"))
    _atomic_write_json(report_path.with_suffix(".json"), report.model_dump(mode="json"))
    return report


def build_failure_report(error_code: DriftErrorCode, message: str, source: str) -> DriftReport:
    run_timestamp = _utc_timestamp()
    return DriftReport(
        provider="claude",
        run_timestamp=run_timestamp,
        status=ProviderStatus.ERROR,
        entries=[
            DriftEntry(
                hook_event_name="schema-drift",
                error_code=error_code,
                source=source,
                action="inspect stderr and rerun after fixing the reported problem",
                message=message,
            )
        ],
    )
