from __future__ import annotations

from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, TypeAdapter


class OpencodeEvent(BaseModel):
    model_config = ConfigDict(extra="allow")

    type: Literal["session.idle"]


class SessionIdlePayload(BaseModel):
    model_config = ConfigDict(extra="allow")

    event: OpencodeEvent


OpencodeHookPayload = SessionIdlePayload

OPENCODE_HOOK_PAYLOAD_ADAPTER = TypeAdapter(OpencodeHookPayload)


def validate_opencode_hook_payload(payload: Any) -> OpencodeHookPayload:
    return OPENCODE_HOOK_PAYLOAD_ADAPTER.validate_python(payload)
