from __future__ import annotations

from typing import Annotated, Any, Literal, Union
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field, TypeAdapter


class HookPayloadBase(BaseModel):
    model_config = ConfigDict(extra="allow")

    session_id: UUID
    transcript_path: str
    cwd: str
    hook_event_name: str
    timestamp: str


class SessionStartPayload(HookPayloadBase):
    hook_event_name: Literal["SessionStart"]
    source: Literal["startup", "resume"]


class SessionEndPayload(HookPayloadBase):
    hook_event_name: Literal["SessionEnd"]
    reason: str


class BeforeAgentPayload(HookPayloadBase):
    hook_event_name: Literal["BeforeAgent"]
    prompt: str


class ToolInput(BaseModel):
    model_config = ConfigDict(extra="allow")

    command: str
    description: str | None = None


class BeforeToolPayload(HookPayloadBase):
    hook_event_name: Literal["BeforeTool"]
    tool_name: str
    tool_input: ToolInput


class ToolResponse(BaseModel):
    model_config = ConfigDict(extra="allow")

    llmContent: str | None = None
    returnDisplay: list[Any] | None = None


class AfterToolPayload(HookPayloadBase):
    hook_event_name: Literal["AfterTool"]
    tool_name: str
    tool_input: ToolInput
    tool_response: ToolResponse


class AfterAgentPayload(HookPayloadBase):
    hook_event_name: Literal["AfterAgent"]
    prompt: str
    prompt_response: str
    stop_hook_active: bool


GeminiHookPayload = Annotated[
    Union[
        SessionStartPayload,
        SessionEndPayload,
        BeforeAgentPayload,
        BeforeToolPayload,
        AfterToolPayload,
        AfterAgentPayload,
    ],
    Field(discriminator="hook_event_name"),
]

GEMINI_HOOK_PAYLOAD_ADAPTER = TypeAdapter(GeminiHookPayload)


def validate_gemini_hook_payload(payload: Any) -> GeminiHookPayload:
    return GEMINI_HOOK_PAYLOAD_ADAPTER.validate_python(payload)
