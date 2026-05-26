from __future__ import annotations

from typing import Annotated, Any, Literal

from pydantic import BaseModel, ConfigDict, Field, TypeAdapter


class StopPayload(BaseModel):
    model_config = ConfigDict(extra="allow")

    hook_event_name: Literal["stop"]
    transcript_path: str
    git_branch: str
    duration_ms: int
    message_count: int
    tool_call_count: int
    loop_count: int
    modified_files: list[str]


CursorAgentHookPayload = Annotated[StopPayload, Field(discriminator="hook_event_name")]

CURSOR_AGENT_HOOK_PAYLOAD_ADAPTER = TypeAdapter(CursorAgentHookPayload)


def validate_cursor_agent_hook_payload(payload: Any) -> CursorAgentHookPayload:
    return CURSOR_AGENT_HOOK_PAYLOAD_ADAPTER.validate_python(payload)
