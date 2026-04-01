from __future__ import annotations

from enum import Enum
from typing import Annotated, Any, Literal, Optional, Union
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field, TypeAdapter, model_validator


class ProviderStatus(str, Enum):
    PASS = "PASS"
    DRIFT = "DRIFT"
    ERROR = "ERROR"
    NOT_SUPPORTED = "NOT_SUPPORTED"
    STALE = "STALE"


class DriftErrorCode(str, Enum):
    REQUIRED_FIELD_REMOVED = "REQUIRED_FIELD_REMOVED"
    FIELD_TYPE_CHANGED = "FIELD_TYPE_CHANGED"
    FIELD_ADDED = "FIELD_ADDED"
    OPTIONAL_FIELD_REMOVED = "OPTIONAL_FIELD_REMOVED"
    CAPTURE_FAILED = "CAPTURE_FAILED"
    PROVIDER_NOT_AVAILABLE = "PROVIDER_NOT_AVAILABLE"
    PROVIDER_NOT_SUPPORTED = "PROVIDER_NOT_SUPPORTED"


class BashToolInput(BaseModel):
    model_config = ConfigDict(extra="allow")

    command: str
    description: Optional[str] = None


class AgentToolInput(BaseModel):
    model_config = ConfigDict(extra="allow")

    prompt: str
    description: Optional[str] = None
    subagent_type: Optional[str] = None
    name: Optional[str] = None
    team_name: Optional[str] = None
    run_in_background: Optional[bool] = None


class BashToolResponse(BaseModel):
    model_config = ConfigDict(extra="allow")

    output: Optional[str] = None
    stdout: Optional[str] = None
    error: Optional[str] = None
    stderr: Optional[str] = None
    interrupted: bool = False
    isImage: Optional[bool] = None
    noOutputExpected: Optional[bool] = None


class HookPayloadBase(BaseModel):
    model_config = ConfigDict(extra="allow")

    session_id: UUID
    hook_event_name: str
    cwd: str
    transcript_path: Optional[str] = None


class SessionStartPayload(HookPayloadBase):
    hook_event_name: Literal["SessionStart"]
    source: str
    model: Optional[str] = None


class SessionEndPayload(HookPayloadBase):
    hook_event_name: Literal["SessionEnd"]
    reason: Optional[str] = None


class PreCompactPayload(HookPayloadBase):
    hook_event_name: Literal["PreCompact"]
    trigger: Optional[str] = None
    custom_instructions: Optional[str] = None


class PreToolUseBashPayload(HookPayloadBase):
    hook_event_name: Literal["PreToolUse"]
    tool_name: Literal["Bash"]
    tool_input: BashToolInput
    permission_mode: Optional[str] = None
    tool_use_id: Optional[str] = None


class PreToolUseAgentPayload(HookPayloadBase):
    hook_event_name: Literal["PreToolUse"]
    tool_name: Literal["Agent"]
    tool_input: AgentToolInput
    permission_mode: Optional[str] = None
    tool_use_id: Optional[str] = None


class PostToolUseBashPayload(HookPayloadBase):
    hook_event_name: Literal["PostToolUse"]
    tool_name: Literal["Bash"]
    tool_input: BashToolInput
    tool_response: BashToolResponse
    permission_mode: Optional[str] = None
    tool_use_id: Optional[str] = None


class PermissionSuggestionRule(BaseModel):
    model_config = ConfigDict(extra="allow")

    ruleContent: Optional[str] = None
    toolName: Optional[str] = None


class PermissionSuggestion(BaseModel):
    model_config = ConfigDict(extra="allow")

    type: str
    behavior: Optional[str] = None
    destination: Optional[str] = None
    mode: Optional[str] = None
    rules: Optional[list[PermissionSuggestionRule]] = None


class PermissionRequestPayload(HookPayloadBase):
    hook_event_name: Literal["PermissionRequest"]
    tool_name: str
    tool_input: dict[str, Any]
    permission_mode: Optional[str] = None
    permission_suggestions: Optional[list[PermissionSuggestion]] = None


class StopPayload(HookPayloadBase):
    hook_event_name: Literal["Stop"]
    stop_hook_active: bool = False
    permission_mode: Optional[str] = None
    last_assistant_message: Optional[str] = None


class NotificationPayload(HookPayloadBase):
    hook_event_name: Literal["Notification"]


PrimaryClaudeHookPayload = Annotated[
    Union[
        SessionStartPayload,
        SessionEndPayload,
        PreCompactPayload,
        PostToolUseBashPayload,
        PermissionRequestPayload,
        StopPayload,
        NotificationPayload,
    ],
    Field(discriminator="hook_event_name"),
]

PreToolUsePayload = Annotated[
    Union[PreToolUseBashPayload, PreToolUseAgentPayload],
    Field(discriminator="tool_name"),
]


class ClaudeHookPayload(BaseModel):
    payload: Union[PrimaryClaudeHookPayload, PreToolUsePayload]

    @model_validator(mode="before")
    @classmethod
    def dispatch_pre_tool_use(cls, value: Any) -> dict[str, Any]:
        if not isinstance(value, dict):
            raise TypeError("Claude hook payload must be a mapping")

        if value.get("hook_event_name") == "PreToolUse":
            tool_name = value.get("tool_name")
            if tool_name not in {"Bash", "Agent"}:
                raise ValueError(f"Unsupported PreToolUse tool_name: {tool_name!r}")

        return {"payload": value}


def validate_claude_hook_payload(payload: Any) -> ClaudeHookPayload:
    return ClaudeHookPayload.model_validate(payload)


class DriftEntry(BaseModel):
    model_config = ConfigDict(extra="forbid")

    hook_event_name: str
    field_name: Optional[str] = None
    error_code: DriftErrorCode
    old_value: Optional[str] = None
    new_value: Optional[str] = None
    source: Optional[str] = None
    action: Optional[str] = None
    recovery: Optional[str] = None
    message: str


class DriftReport(BaseModel):
    model_config = ConfigDict(extra="forbid")

    provider: str
    claude_version: Optional[str] = None
    run_timestamp: str
    status: ProviderStatus
    entries: list[DriftEntry]
    validated_fixtures: list[str] = []
    schema_paths: dict[str, str] = {}
    drift_history_path: Optional[str] = None
    report_path: Optional[str] = None
    section_paths: list[str] = []
