from __future__ import annotations

from typing import Any, Literal, Union
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field, model_validator


class CodexBashToolInput(BaseModel):
    model_config = ConfigDict(extra="allow")

    command: str


class CodexHookPayloadBase(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)

    cwd: str


class CodexNotifyPayload(CodexHookPayloadBase):
    client: str
    input_messages: list[str] = Field(alias="input-messages")
    last_assistant_message: str = Field(alias="last-assistant-message")
    thread_id: UUID = Field(alias="thread-id")
    turn_id: UUID = Field(alias="turn-id")
    type: Literal["agent-turn-complete"]


class CodexPreToolUsePayload(CodexHookPayloadBase):
    hook_event_name: Literal["PreToolUse"]
    model: str
    permission_mode: str
    session_id: UUID
    tool_input: CodexBashToolInput
    tool_name: Literal["Bash"]
    tool_use_id: str
    transcript_path: str
    turn_id: UUID


class CodexSessionStartPayload(CodexHookPayloadBase):
    hook_event_name: Literal["SessionStart"]
    model: str
    permission_mode: str
    session_id: UUID
    source: str
    transcript_path: str


class CodexHookPayload(BaseModel):
    payload: Union[CodexNotifyPayload, CodexPreToolUsePayload, CodexSessionStartPayload]

    @model_validator(mode="before")
    @classmethod
    def dispatch_payload(cls, value: Any) -> dict[str, Any]:
        if not isinstance(value, dict):
            raise TypeError("Codex hook payload must be a mapping")
        if value.get("hook_event_name") == "PreToolUse":
            return {"payload": value}
        if value.get("hook_event_name") == "SessionStart":
            return {"payload": value}
        if value.get("type") == "agent-turn-complete":
            return {"payload": value}
        raise ValueError("Unsupported Codex hook payload shape")


class CodexEnvSnapshot(BaseModel):
    model_config = ConfigDict(extra="forbid")

    atm_env: dict[str, str]
    captured_at: str
    codex_env: dict[str, str]
    cwd_from_getcwd: str
    hook_name: str
    pwd_env: str | None = None
    sc_hook_env: dict[str, str]


class CodexControlSemantics(BaseModel):
    model_config = ConfigDict(extra="forbid")

    blocking: bool
    exit_code_contract: str
    stdio_contract: str


class CodexHookSurfaceDisposition(BaseModel):
    model_config = ConfigDict(extra="forbid")

    env_fixture: str | None = None
    payload_fixture: str | None = None
    reason: str | None = None
    status: Literal["captured", "confirmed-not-exercisable"]
    surface: str
    control_semantics: CodexControlSemantics | None = None

    @model_validator(mode="after")
    def validate_status_shape(self) -> "CodexHookSurfaceDisposition":
        if self.status == "captured":
            if not self.payload_fixture or not self.env_fixture or self.control_semantics is None:
                raise ValueError("captured surfaces require payload_fixture, env_fixture, and control_semantics")
        if self.status == "confirmed-not-exercisable" and not self.reason:
            raise ValueError("confirmed-not-exercisable surfaces require a reason")
        return self


class CodexFixtureManifest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    capture_date: str
    capture_root: str
    codex_version: str
    hook_registration: dict[str, str]
    hook_surfaces: list[CodexHookSurfaceDisposition]
    provider: Literal["codex"]
    prototype_baseline: dict[str, Any]


def validate_codex_hook_payload(payload: Any) -> CodexHookPayload:
    return CodexHookPayload.model_validate(payload)


def validate_codex_env_snapshot(payload: Any) -> CodexEnvSnapshot:
    return CodexEnvSnapshot.model_validate(payload)


def validate_codex_fixture_manifest(payload: Any) -> CodexFixtureManifest:
    return CodexFixtureManifest.model_validate(payload)
