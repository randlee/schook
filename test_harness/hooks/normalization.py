from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class NormalizationFixtureSpec:
    provider: str
    fixture_relpath: str
    required_fields: tuple[str, ...]


@dataclass(frozen=True)
class NormalizationObservation:
    provider: str
    fixture_path: str
    fields: dict[str, Any]


class NormalizationFixtureError(ValueError):
    def __init__(self, provider: str, fixture_path: str, field: str, reason: str) -> None:
        self.provider = provider
        self.fixture_path = fixture_path
        self.field = field
        self.reason = reason
        super().__init__(
            f"Normalization fixture error for provider={provider} path={fixture_path} "
            f"field={field}: {reason}"
        )


CORE_NORMALIZATION_SPECS: dict[str, NormalizationFixtureSpec] = {
    "claude_session_start": NormalizationFixtureSpec(
        provider="claude",
        fixture_relpath="test-harness/hooks/claude/fixtures/approved/session-start-startup.json",
        required_fields=("session_id", "cwd", "transcript_path", "hook_event_name", "source"),
    ),
    "claude_pretooluse_bash": NormalizationFixtureSpec(
        provider="claude",
        fixture_relpath="test-harness/hooks/claude/fixtures/approved/pretooluse-bash.json",
        required_fields=("session_id", "cwd", "transcript_path", "hook_event_name", "tool_name", "tool_input.command"),
    ),
    "codex_session_start": NormalizationFixtureSpec(
        provider="codex",
        fixture_relpath="test-harness/hooks/codex/fixtures/approved/session-start-startup.json",
        required_fields=("session_id", "cwd", "transcript_path", "hook_event_name", "source", "model"),
    ),
    "codex_pretooluse": NormalizationFixtureSpec(
        provider="codex",
        fixture_relpath="test-harness/hooks/codex/fixtures/approved/pretooluse-bash.json",
        required_fields=("session_id", "cwd", "transcript_path", "hook_event_name", "tool_name", "tool_input.command"),
    ),
    "codex_notify": NormalizationFixtureSpec(
        provider="codex",
        fixture_relpath="test-harness/hooks/codex/fixtures/approved/notify-agent-turn-complete.json",
        required_fields=("cwd", "thread-id", "turn-id", "type", "last-assistant-message"),
    ),
    "gemini_session_start": NormalizationFixtureSpec(
        provider="gemini",
        fixture_relpath="test-harness/hooks/gemini/fixtures/approved/session-start-startup.json",
        required_fields=("session_id", "cwd", "transcript_path", "hook_event_name", "source", "timestamp"),
    ),
    "gemini_before_tool": NormalizationFixtureSpec(
        provider="gemini",
        fixture_relpath="test-harness/hooks/gemini/fixtures/approved/before-tool.json",
        required_fields=("session_id", "cwd", "transcript_path", "hook_event_name", "tool_name", "tool_input.command"),
    ),
    "gemini_after_agent": NormalizationFixtureSpec(
        provider="gemini",
        fixture_relpath="test-harness/hooks/gemini/fixtures/approved/after-agent.json",
        # These fields stay provider-local in the checklist, but they are still
        # required here so the harness proves the evidence exists before the
        # normalization review rejects them from canonical mapping.
        required_fields=("session_id", "cwd", "transcript_path", "hook_event_name", "prompt_response", "stop_hook_active"),
    ),
}


def _get_field(payload: Any, field_path: str) -> Any:
    current = payload
    for segment in field_path.split("."):
        if not isinstance(current, dict) or segment not in current:
            raise KeyError(field_path)
        current = current[segment]
    return current


def load_normalization_fixture(
    repo_root: Path,
    provider: str,
    fixture_relpath: str,
    required_fields: tuple[str, ...],
) -> NormalizationObservation:
    fixture_path = repo_root / fixture_relpath
    if not fixture_path.exists():
        raise NormalizationFixtureError(
            provider=provider,
            fixture_path=fixture_relpath,
            field="fixture",
            reason="fixture file does not exist",
        )

    try:
        payload = json.loads(fixture_path.read_text())
    except json.JSONDecodeError as exc:
        raise NormalizationFixtureError(
            provider=provider,
            fixture_path=fixture_relpath,
            field="fixture",
            reason=f"fixture is not valid JSON: {exc.msg}",
        ) from exc

    observed_fields: dict[str, Any] = {}
    for field_path in required_fields:
        try:
            observed_fields[field_path] = _get_field(payload, field_path)
        except KeyError as exc:
            raise NormalizationFixtureError(
                provider=provider,
                fixture_path=fixture_relpath,
                field=field_path,
                reason="required field missing from approved fixture",
            ) from exc

    return NormalizationObservation(
        provider=provider,
        fixture_path=fixture_relpath,
        fields=observed_fields,
    )


def collect_core_normalization_observations(repo_root: Path) -> dict[str, NormalizationObservation]:
    return {
        name: load_normalization_fixture(
            repo_root=repo_root,
            provider=spec.provider,
            fixture_relpath=spec.fixture_relpath,
            required_fields=spec.required_fields,
        )
        for name, spec in CORE_NORMALIZATION_SPECS.items()
    }
