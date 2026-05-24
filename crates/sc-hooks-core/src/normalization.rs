#![allow(
    dead_code,
    reason = "Phase O lands the normalization foundation in O.3 and adopts it across runtime parity sprints O.4 and O.5."
)]

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use sc_lint_attributes::sc_lint;
use serde_json::{Value, json};
use thiserror::Error;

use crate::context::HookContext;
use crate::errors::HookError;
use crate::events::HookType;

#[sc_lint(boundary.internal_only)]
mod private {
    pub trait Sealed {}
}

/// Internal runtime provider source for the Phase O normalization seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum ProviderHookSource {
    #[error("codex")]
    Codex,
    #[error("gemini")]
    Gemini,
}

/// Raw provider payload and metadata passed into the normalization seam.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ProviderHookInput<'a> {
    pub(crate) provider: ProviderHookSource,
    pub(crate) raw: &'a Value,
    pub(crate) event: Option<&'a str>,
    pub(crate) metadata_path: Option<&'a Path>,
}

/// Canonical provider-normalized input that feeds the existing `HookContext` path.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NormalizedHookContext<'a> {
    pub(crate) hook: CanonicalHook,
    pub(crate) event: Option<HookEventName<'a>>,
    pub(crate) session_id: Option<SessionId<'a>>,
    pub(crate) project_root: Option<&'a Path>,
    pub(crate) current_dir: Option<&'a Path>,
    pub(crate) tool_name: Option<ToolName<'a>>,
    pub(crate) payload: CanonicalPayload<'a>,
}

/// Approved canonical hook variants for Phase O provider normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanonicalHook {
    Codex(CodexHook),
    Gemini(GeminiHook),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CodexHook {
    SessionStart,
    PreToolUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GeminiHook {
    SessionStart,
    SessionEnd,
    BeforeAgent,
    BeforeTool,
    AfterTool,
}

/// Canonical payload families accepted by the runtime normalization seam.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CanonicalPayload<'a> {
    Empty,
    ToolUse {
        tool_name: ToolName<'a>,
        body: &'a Value,
    },
    SessionLifecycle {
        body: &'a Value,
    },
    AgentLifecycle {
        body: &'a Value,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionId<'a>(pub(crate) Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolName<'a>(pub(crate) Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HookEventName<'a>(pub(crate) Cow<'a, str>);

/// Named provider-normalization failures attached to `HookError`.
// NormalizationError is pub to satisfy HookError's pub surface; O3-003 tracks
// the planned pub(crate) tightening once the boundary refactor lands.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NormalizationError {
    #[error("missing required field `{field}`")]
    MissingRequiredField { field: &'static str },
    #[error("invalid field `{field}`: {reason}")]
    InvalidFieldValue {
        field: &'static str,
        reason: &'static str,
    },
    #[error("invalid payload kind `{payload_kind}` for {hook}")]
    InvalidPayloadForHook {
        hook: &'static str,
        payload_kind: &'static str,
    },
    #[error("retryable gate input failure for `{field}`: {reason}. {recovery_hint}")]
    RetryableGateInput {
        field: &'static str,
        reason: &'static str,
        recovery_hint: &'static str,
    },
    #[error("unsupported approved surface `{hook}` for {provider}")]
    UnsupportedApprovedSurface {
        provider: &'static str,
        hook: &'static str,
    },
}

/// Compile-fail boundary proof: the trait is crate-private and cannot be
/// implemented by external crates.
#[sc_lint(boundary.forbid_external_impls)]
pub(crate) trait ProviderHookNormalizer: private::Sealed {
    fn normalize<'a>(
        &self,
        raw: ProviderHookInput<'a>,
    ) -> Result<NormalizedHookContext<'a>, HookError>;
}

#[derive(Debug, Default)]
struct CodexHookNormalizer;

#[derive(Debug, Default)]
struct GeminiHookNormalizer;

impl private::Sealed for CodexHookNormalizer {}
impl private::Sealed for GeminiHookNormalizer {}

impl ProviderHookNormalizer for CodexHookNormalizer {
    fn normalize<'a>(
        &self,
        raw: ProviderHookInput<'a>,
    ) -> Result<NormalizedHookContext<'a>, HookError> {
        let hook_name = raw_string(raw.raw, "hook_event_name")?;
        match hook_name {
            "SessionStart" => {
                require_string_field(raw.raw, "session_id")?;
                require_string_field(raw.raw, "cwd")?;
                require_string_field(raw.raw, "source")?;
                Ok(NormalizedHookContext {
                    hook: CanonicalHook::Codex(CodexHook::SessionStart),
                    event: None,
                    session_id: Some(SessionId(Cow::Borrowed(require_string_field(
                        raw.raw,
                        "session_id",
                    )?))),
                    project_root: Some(path_field(raw.raw, "cwd")?),
                    current_dir: Some(path_field(raw.raw, "cwd")?),
                    tool_name: None,
                    payload: CanonicalPayload::SessionLifecycle { body: raw.raw },
                })
            }
            "PreToolUse" => {
                require_retryable_string_field(
                    raw.raw,
                    "session_id",
                    "Codex PreToolUse must include session_id for gate state lookup.",
                )?;
                require_retryable_string_field(
                    raw.raw,
                    "cwd",
                    "Codex PreToolUse must include cwd so runtime gates can resolve the project root.",
                )?;
                let tool_name = require_retryable_string_field(
                    raw.raw,
                    "tool_name",
                    "Codex PreToolUse must include tool_name so runtime matchers can route the gate.",
                )?;
                if tool_name != "Bash" {
                    return Err(HookError::normalization(
                        NormalizationError::InvalidFieldValue {
                            field: "tool_name",
                            reason: "expected approved Codex Bash surface",
                        },
                    ));
                }
                if raw.raw.get("tool_input").is_none() {
                    return Err(HookError::normalization(
                        NormalizationError::RetryableGateInput {
                            field: "tool_input",
                            reason: "missing tool_input object",
                            recovery_hint: "Retry after Codex emits the complete PreToolUse payload for the Bash tool.",
                        },
                    ));
                }
                Ok(NormalizedHookContext {
                    hook: CanonicalHook::Codex(CodexHook::PreToolUse),
                    event: Some(HookEventName(Cow::Borrowed("Bash"))),
                    session_id: Some(SessionId(Cow::Borrowed(require_string_field(
                        raw.raw,
                        "session_id",
                    )?))),
                    project_root: Some(path_field(raw.raw, "cwd")?),
                    current_dir: Some(path_field(raw.raw, "cwd")?),
                    tool_name: Some(ToolName(Cow::Borrowed("Bash"))),
                    payload: CanonicalPayload::ToolUse {
                        tool_name: ToolName(Cow::Borrowed("Bash")),
                        body: raw.raw,
                    },
                })
            }
            other => Err(HookError::normalization(
                NormalizationError::UnsupportedApprovedSurface {
                    provider: raw.provider.as_str(),
                    hook: hook_label(other),
                },
            )),
        }
    }
}

impl ProviderHookNormalizer for GeminiHookNormalizer {
    fn normalize<'a>(
        &self,
        raw: ProviderHookInput<'a>,
    ) -> Result<NormalizedHookContext<'a>, HookError> {
        let hook_name = raw_string(raw.raw, "hook_event_name")?;
        match hook_name {
            "SessionStart" => {
                require_string_field(raw.raw, "session_id")?;
                require_string_field(raw.raw, "cwd")?;
                require_string_field(raw.raw, "source")?;
                Ok(NormalizedHookContext {
                    hook: CanonicalHook::Gemini(GeminiHook::SessionStart),
                    event: None,
                    session_id: Some(SessionId(Cow::Borrowed(require_string_field(
                        raw.raw,
                        "session_id",
                    )?))),
                    project_root: Some(path_field(raw.raw, "cwd")?),
                    current_dir: Some(path_field(raw.raw, "cwd")?),
                    tool_name: None,
                    payload: CanonicalPayload::SessionLifecycle { body: raw.raw },
                })
            }
            "SessionEnd" => {
                require_string_field(raw.raw, "session_id")?;
                require_string_field(raw.raw, "cwd")?;
                require_string_field(raw.raw, "reason")?;
                Ok(NormalizedHookContext {
                    hook: CanonicalHook::Gemini(GeminiHook::SessionEnd),
                    event: None,
                    session_id: Some(SessionId(Cow::Borrowed(require_string_field(
                        raw.raw,
                        "session_id",
                    )?))),
                    project_root: Some(path_field(raw.raw, "cwd")?),
                    current_dir: Some(path_field(raw.raw, "cwd")?),
                    tool_name: None,
                    payload: CanonicalPayload::SessionLifecycle { body: raw.raw },
                })
            }
            "BeforeAgent" => {
                require_retryable_string_field(
                    raw.raw,
                    "session_id",
                    "Gemini BeforeAgent must include session_id for agent-spawn policy state lookup.",
                )?;
                require_retryable_string_field(
                    raw.raw,
                    "cwd",
                    "Gemini BeforeAgent must include cwd so runtime gates can resolve the project root.",
                )?;
                require_retryable_string_field(
                    raw.raw,
                    "prompt",
                    "Gemini BeforeAgent must include prompt so Agent payload synthesis can proceed.",
                )?;
                Ok(NormalizedHookContext {
                    hook: CanonicalHook::Gemini(GeminiHook::BeforeAgent),
                    event: Some(HookEventName(Cow::Borrowed("Agent"))),
                    session_id: Some(SessionId(Cow::Borrowed(require_string_field(
                        raw.raw,
                        "session_id",
                    )?))),
                    project_root: Some(path_field(raw.raw, "cwd")?),
                    current_dir: Some(path_field(raw.raw, "cwd")?),
                    tool_name: Some(ToolName(Cow::Borrowed("Agent"))),
                    payload: CanonicalPayload::AgentLifecycle { body: raw.raw },
                })
            }
            "BeforeTool" => {
                require_retryable_string_field(
                    raw.raw,
                    "session_id",
                    "Gemini BeforeTool must include session_id for gate state lookup.",
                )?;
                require_retryable_string_field(
                    raw.raw,
                    "cwd",
                    "Gemini BeforeTool must include cwd so runtime gates can resolve the project root.",
                )?;
                require_retryable_string_field(
                    raw.raw,
                    "tool_name",
                    "Gemini BeforeTool must include tool_name so runtime matchers can route the gate.",
                )?;
                if raw.raw.get("tool_input").is_none() {
                    return Err(HookError::normalization(
                        NormalizationError::RetryableGateInput {
                            field: "tool_input",
                            reason: "missing tool_input object",
                            recovery_hint: "Retry after Gemini emits the complete BeforeTool payload for the approved shell surface.",
                        },
                    ));
                }
                Ok(NormalizedHookContext {
                    hook: CanonicalHook::Gemini(GeminiHook::BeforeTool),
                    event: Some(HookEventName(Cow::Borrowed("Bash"))),
                    session_id: Some(SessionId(Cow::Borrowed(require_string_field(
                        raw.raw,
                        "session_id",
                    )?))),
                    project_root: Some(path_field(raw.raw, "cwd")?),
                    current_dir: Some(path_field(raw.raw, "cwd")?),
                    tool_name: Some(ToolName(Cow::Borrowed("Bash"))),
                    payload: CanonicalPayload::ToolUse {
                        tool_name: ToolName(Cow::Borrowed("Bash")),
                        body: raw.raw,
                    },
                })
            }
            "AfterTool" => {
                require_retryable_string_field(
                    raw.raw,
                    "session_id",
                    "Gemini AfterTool must include session_id for gate state lookup.",
                )?;
                require_retryable_string_field(
                    raw.raw,
                    "cwd",
                    "Gemini AfterTool must include cwd so runtime gates can resolve the project root.",
                )?;
                require_retryable_string_field(
                    raw.raw,
                    "tool_name",
                    "Gemini AfterTool must include tool_name so runtime matchers can route the gate.",
                )?;
                if raw.raw.get("tool_input").is_none() {
                    return Err(HookError::normalization(
                        NormalizationError::RetryableGateInput {
                            field: "tool_input",
                            reason: "missing tool_input object",
                            recovery_hint: "Retry after Gemini emits the complete AfterTool payload for the approved shell surface.",
                        },
                    ));
                }
                if raw.raw.get("tool_response").is_none() {
                    return Err(HookError::normalization(
                        NormalizationError::RetryableGateInput {
                            field: "tool_response",
                            reason: "missing tool_response object",
                            recovery_hint: "Retry after Gemini emits the complete AfterTool response payload for the approved shell surface.",
                        },
                    ));
                }
                Ok(NormalizedHookContext {
                    hook: CanonicalHook::Gemini(GeminiHook::AfterTool),
                    event: Some(HookEventName(Cow::Borrowed("Bash"))),
                    session_id: Some(SessionId(Cow::Borrowed(require_string_field(
                        raw.raw,
                        "session_id",
                    )?))),
                    project_root: Some(path_field(raw.raw, "cwd")?),
                    current_dir: Some(path_field(raw.raw, "cwd")?),
                    tool_name: Some(ToolName(Cow::Borrowed("Bash"))),
                    payload: CanonicalPayload::ToolUse {
                        tool_name: ToolName(Cow::Borrowed("Bash")),
                        body: raw.raw,
                    },
                })
            }
            other => Err(HookError::normalization(
                NormalizationError::UnsupportedApprovedSurface {
                    provider: raw.provider.as_str(),
                    hook: hook_label(other),
                },
            )),
        }
    }
}

pub(crate) fn normalize_provider_hook<'a>(
    input: ProviderHookInput<'a>,
) -> Result<HookContext<'a>, HookError> {
    let metadata_path = input.metadata_path.map(Path::to_path_buf);
    let normalized = match input.provider {
        ProviderHookSource::Codex => CodexHookNormalizer.normalize(input)?,
        ProviderHookSource::Gemini => GeminiHookNormalizer.normalize(input)?,
    };
    normalized.into_hook_context(metadata_path)
}

impl<'a> NormalizedHookContext<'a> {
    fn into_hook_context(
        self,
        metadata_path: Option<PathBuf>,
    ) -> Result<HookContext<'a>, HookError> {
        ensure_payload_compatible(&self.hook, &self.payload)?;
        let hook = runtime_hook(&self.hook);
        let event = runtime_event(&self.hook, self.event)?;
        let payload = runtime_payload(&self.hook, self.payload)?;
        Ok(HookContext::new(
            hook,
            event,
            json!({ "hook": hook_json(hook), "payload": payload }),
            metadata_path,
        ))
    }
}

fn ensure_payload_compatible(
    hook: &CanonicalHook,
    payload: &CanonicalPayload<'_>,
) -> Result<(), HookError> {
    let valid = matches!(
        (hook, payload),
        (
            CanonicalHook::Codex(CodexHook::SessionStart),
            CanonicalPayload::SessionLifecycle { .. }
        ) | (
            CanonicalHook::Codex(CodexHook::PreToolUse),
            CanonicalPayload::ToolUse { .. }
        ) | (
            CanonicalHook::Gemini(GeminiHook::SessionStart),
            CanonicalPayload::SessionLifecycle { .. }
        ) | (
            CanonicalHook::Gemini(GeminiHook::SessionEnd),
            CanonicalPayload::SessionLifecycle { .. }
        ) | (
            CanonicalHook::Gemini(GeminiHook::BeforeAgent),
            CanonicalPayload::AgentLifecycle { .. }
        ) | (
            CanonicalHook::Gemini(GeminiHook::BeforeTool),
            CanonicalPayload::ToolUse { .. }
        ) | (
            CanonicalHook::Gemini(GeminiHook::AfterTool),
            CanonicalPayload::ToolUse { .. }
        )
    );
    if valid {
        Ok(())
    } else {
        Err(HookError::normalization(
            NormalizationError::InvalidPayloadForHook {
                hook: canonical_hook_label(hook),
                payload_kind: payload_kind(payload),
            },
        ))
    }
}

fn runtime_hook(hook: &CanonicalHook) -> HookType {
    match hook {
        CanonicalHook::Codex(CodexHook::SessionStart)
        | CanonicalHook::Gemini(GeminiHook::SessionStart) => HookType::SessionStart,
        CanonicalHook::Gemini(GeminiHook::SessionEnd) => HookType::SessionEnd,
        CanonicalHook::Codex(CodexHook::PreToolUse)
        | CanonicalHook::Gemini(GeminiHook::BeforeAgent)
        | CanonicalHook::Gemini(GeminiHook::BeforeTool) => HookType::PreToolUse,
        CanonicalHook::Gemini(GeminiHook::AfterTool) => HookType::PostToolUse,
    }
}

fn runtime_event<'a>(
    hook: &CanonicalHook,
    event: Option<HookEventName<'a>>,
) -> Result<Option<Cow<'a, str>>, HookError> {
    match hook {
        CanonicalHook::Codex(CodexHook::SessionStart)
        | CanonicalHook::Gemini(GeminiHook::SessionStart)
        | CanonicalHook::Gemini(GeminiHook::SessionEnd) => Ok(None),
        CanonicalHook::Codex(CodexHook::PreToolUse)
        | CanonicalHook::Gemini(GeminiHook::BeforeAgent)
        | CanonicalHook::Gemini(GeminiHook::BeforeTool)
        | CanonicalHook::Gemini(GeminiHook::AfterTool) => {
            event.map(|value| Some(value.0)).ok_or_else(|| {
                HookError::normalization(NormalizationError::MissingRequiredField {
                    field: "event",
                })
            })
        }
    }
}

fn runtime_payload(
    hook: &CanonicalHook,
    payload: CanonicalPayload<'_>,
) -> Result<Value, HookError> {
    match (hook, payload) {
        (
            CanonicalHook::Codex(CodexHook::SessionStart)
            | CanonicalHook::Gemini(GeminiHook::SessionStart)
            | CanonicalHook::Gemini(GeminiHook::SessionEnd),
            CanonicalPayload::SessionLifecycle { body },
        ) => Ok(body.clone()),
        (CanonicalHook::Codex(CodexHook::PreToolUse), CanonicalPayload::ToolUse { body, .. }) => {
            Ok(body.clone())
        }
        (CanonicalHook::Gemini(GeminiHook::BeforeTool), CanonicalPayload::ToolUse { body, .. }) => {
            Ok(json!({
                "session_id": require_string_field(body, "session_id")?,
                "transcript_path": optional_string(body, "transcript_path"),
                "cwd": require_string_field(body, "cwd")?,
                "tool_name": "Bash",
                "tool_input": body.get("tool_input").cloned().unwrap_or(Value::Null),
            }))
        }
        (CanonicalHook::Gemini(GeminiHook::AfterTool), CanonicalPayload::ToolUse { body, .. }) => {
            Ok(json!({
                "session_id": require_string_field(body, "session_id")?,
                "transcript_path": optional_string(body, "transcript_path"),
                "cwd": require_string_field(body, "cwd")?,
                "tool_name": "Bash",
                "tool_input": body.get("tool_input").cloned().unwrap_or(Value::Null),
                "tool_response": {
                    "stdout": llm_content(body),
                    "stderr": Value::Null,
                    "interrupted": false
                }
            }))
        }
        (
            CanonicalHook::Gemini(GeminiHook::BeforeAgent),
            CanonicalPayload::AgentLifecycle { body },
        ) => Ok(json!({
            "session_id": require_string_field(body, "session_id")?,
            "transcript_path": optional_string(body, "transcript_path"),
            "cwd": require_string_field(body, "cwd")?,
            "tool_name": "Agent",
            "tool_input": {
                "prompt": require_string_field(body, "prompt")?,
            }
        })),
        (bad_hook, bad_payload) => Err(HookError::normalization(
            NormalizationError::InvalidPayloadForHook {
                hook: canonical_hook_label(bad_hook),
                payload_kind: payload_kind(&bad_payload),
            },
        )),
    }
}

fn hook_json(hook: HookType) -> Value {
    json!({ "type": hook.as_str() })
}

fn llm_content(body: &Value) -> Option<&str> {
    body.get("tool_response")
        .and_then(Value::as_object)
        .and_then(|response| response.get("llmContent"))
        .and_then(Value::as_str)
}

fn payload_kind(payload: &CanonicalPayload<'_>) -> &'static str {
    match payload {
        CanonicalPayload::Empty => "empty",
        CanonicalPayload::ToolUse { .. } => "tool_use",
        CanonicalPayload::SessionLifecycle { .. } => "session_lifecycle",
        CanonicalPayload::AgentLifecycle { .. } => "agent_lifecycle",
    }
}

fn hook_label(value: &str) -> &'static str {
    match value {
        "SessionStart" => "SessionStart",
        "SessionEnd" => "SessionEnd",
        "BeforeAgent" => "BeforeAgent",
        "BeforeTool" => "BeforeTool",
        "AfterTool" => "AfterTool",
        "PreToolUse" => "PreToolUse",
        _ => "unknown",
    }
}

impl ProviderHookSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Gemini => "gemini",
        }
    }
}

fn canonical_hook_label(value: &CanonicalHook) -> &'static str {
    match value {
        CanonicalHook::Codex(CodexHook::SessionStart) => "Codex::SessionStart",
        CanonicalHook::Codex(CodexHook::PreToolUse) => "Codex::PreToolUse",
        CanonicalHook::Gemini(GeminiHook::SessionStart) => "Gemini::SessionStart",
        CanonicalHook::Gemini(GeminiHook::SessionEnd) => "Gemini::SessionEnd",
        CanonicalHook::Gemini(GeminiHook::BeforeAgent) => "Gemini::BeforeAgent",
        CanonicalHook::Gemini(GeminiHook::BeforeTool) => "Gemini::BeforeTool",
        CanonicalHook::Gemini(GeminiHook::AfterTool) => "Gemini::AfterTool",
    }
}

fn raw_string<'a>(payload: &'a Value, field: &'static str) -> Result<&'a str, HookError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| HookError::normalization(NormalizationError::MissingRequiredField { field }))
}

fn require_string_field<'a>(payload: &'a Value, field: &'static str) -> Result<&'a str, HookError> {
    raw_string(payload, field)
}

fn require_retryable_string_field<'a>(
    payload: &'a Value,
    field: &'static str,
    recovery_hint: &'static str,
) -> Result<&'a str, HookError> {
    payload.get(field).and_then(Value::as_str).ok_or_else(|| {
        HookError::normalization(NormalizationError::RetryableGateInput {
            field,
            reason: "missing required string field",
            recovery_hint,
        })
    })
}

fn path_field<'a>(payload: &'a Value, field: &'static str) -> Result<&'a Path, HookError> {
    Ok(Path::new(require_string_field(payload, field)?))
}

fn optional_string<'a>(payload: &'a Value, field: &'static str) -> Option<&'a str> {
    payload.get(field).and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODEX_PRE_TOOL_USE: &str =
        include_str!("../../../test-harness/hooks/codex/fixtures/approved/pretooluse-bash.json");
    const CODEX_SESSION_START: &str = include_str!(
        "../../../test-harness/hooks/codex/fixtures/approved/session-start-startup.json"
    );
    const GEMINI_BEFORE_AGENT: &str =
        include_str!("../../../test-harness/hooks/gemini/fixtures/approved/before-agent.json");
    const GEMINI_BEFORE_TOOL: &str =
        include_str!("../../../test-harness/hooks/gemini/fixtures/approved/before-tool.json");
    const GEMINI_AFTER_TOOL: &str =
        include_str!("../../../test-harness/hooks/gemini/fixtures/approved/after-tool.json");
    const GEMINI_SESSION_START: &str = include_str!(
        "../../../test-harness/hooks/gemini/fixtures/approved/session-start-startup.json"
    );
    const GEMINI_SESSION_END: &str =
        include_str!("../../../test-harness/hooks/gemini/fixtures/approved/session-end.json");

    fn fixture(body: &str) -> Value {
        serde_json::from_str(body).expect("fixture should parse")
    }

    #[test]
    fn codex_session_start_normalizes_into_runtime_context() {
        let raw = fixture(CODEX_SESSION_START);
        let context = normalize_provider_hook(ProviderHookInput {
            provider: ProviderHookSource::Codex,
            raw: &raw,
            event: None,
            metadata_path: None,
        })
        .expect("codex session start should normalize");

        assert_eq!(context.hook, HookType::SessionStart);
        assert!(context.event.is_none());
        let payload = context.payload_value().expect("payload");
        assert_eq!(payload["source"], "startup");
    }

    #[test]
    fn codex_pre_tool_use_normalizes_into_bash_gate_context() {
        let raw = fixture(CODEX_PRE_TOOL_USE);
        let context = normalize_provider_hook(ProviderHookInput {
            provider: ProviderHookSource::Codex,
            raw: &raw,
            event: None,
            metadata_path: None,
        })
        .expect("codex pre tool use should normalize");

        assert_eq!(context.hook, HookType::PreToolUse);
        assert_eq!(context.event.as_deref(), Some("Bash"));
        let payload = context.payload_value().expect("payload");
        assert_eq!(payload["tool_name"], "Bash");
        assert_eq!(payload["tool_input"]["command"], "pwd");
    }

    #[test]
    fn gemini_session_start_normalizes_into_runtime_context() {
        let raw = fixture(GEMINI_SESSION_START);
        let context = normalize_provider_hook(ProviderHookInput {
            provider: ProviderHookSource::Gemini,
            raw: &raw,
            event: None,
            metadata_path: None,
        })
        .expect("gemini session start should normalize");

        assert_eq!(context.hook, HookType::SessionStart);
        assert!(context.event.is_none());
        let payload = context.payload_value().expect("payload");
        assert_eq!(payload["source"], "startup");
    }

    #[test]
    fn gemini_session_end_normalizes_into_runtime_context() {
        let raw = fixture(GEMINI_SESSION_END);
        let context = normalize_provider_hook(ProviderHookInput {
            provider: ProviderHookSource::Gemini,
            raw: &raw,
            event: None,
            metadata_path: None,
        })
        .expect("gemini session end should normalize");

        assert_eq!(context.hook, HookType::SessionEnd);
        assert!(context.event.is_none());
        let payload = context.payload_value().expect("payload");
        assert_eq!(payload["reason"], "exit");
    }

    #[test]
    fn gemini_before_agent_normalizes_into_agent_gate_context() {
        let raw = fixture(GEMINI_BEFORE_AGENT);
        let context = normalize_provider_hook(ProviderHookInput {
            provider: ProviderHookSource::Gemini,
            raw: &raw,
            event: None,
            metadata_path: None,
        })
        .expect("gemini before agent should normalize");

        assert_eq!(context.hook, HookType::PreToolUse);
        assert_eq!(context.event.as_deref(), Some("Agent"));
        let payload = context.payload_value().expect("payload");
        assert_eq!(payload["tool_name"], "Agent");
        assert_eq!(payload["tool_input"]["prompt"], "Reply with exactly OK.");
    }

    #[test]
    fn gemini_before_tool_normalizes_into_bash_gate_context() {
        let raw = fixture(GEMINI_BEFORE_TOOL);
        let context = normalize_provider_hook(ProviderHookInput {
            provider: ProviderHookSource::Gemini,
            raw: &raw,
            event: None,
            metadata_path: None,
        })
        .expect("gemini before tool should normalize");

        assert_eq!(context.hook, HookType::PreToolUse);
        assert_eq!(context.event.as_deref(), Some("Bash"));
        let payload = context.payload_value().expect("payload");
        assert_eq!(payload["tool_name"], "Bash");
        assert_eq!(payload["tool_input"]["command"], "pwd");
    }

    #[test]
    fn gemini_after_tool_normalizes_into_post_tool_bash_context() {
        let raw = fixture(GEMINI_AFTER_TOOL);
        let context = normalize_provider_hook(ProviderHookInput {
            provider: ProviderHookSource::Gemini,
            raw: &raw,
            event: None,
            metadata_path: None,
        })
        .expect("gemini after tool should normalize");

        assert_eq!(context.hook, HookType::PostToolUse);
        assert_eq!(context.event.as_deref(), Some("Bash"));
        let payload = context.payload_value().expect("payload");
        assert_eq!(payload["tool_name"], "Bash");
        assert!(
            payload["tool_response"]["stdout"]
                .as_str()
                .expect("stdout")
                .contains("/synthetic/test/gemini-harness")
        );
    }

    #[test]
    fn compatibility_check_rejects_invalid_hook_payload_pairings() {
        let err = NormalizedHookContext {
            hook: CanonicalHook::Codex(CodexHook::SessionStart),
            event: None,
            session_id: None,
            project_root: None,
            current_dir: None,
            tool_name: None,
            payload: CanonicalPayload::ToolUse {
                tool_name: ToolName(Cow::Borrowed("Bash")),
                body: &fixture(CODEX_PRE_TOOL_USE),
            },
        }
        .into_hook_context(None)
        .expect_err("invalid pairing should fail");

        assert!(matches!(
            err,
            HookError::Normalization(NormalizationError::InvalidPayloadForHook { .. })
        ));
    }

    #[test]
    fn retryable_gate_input_errors_require_recovery_hints() {
        let raw = json!({
            "hook_event_name": "BeforeTool",
            "session_id": "sess",
            "tool_name": "run_shell_command"
        });
        let err = normalize_provider_hook(ProviderHookInput {
            provider: ProviderHookSource::Gemini,
            raw: &raw,
            event: None,
            metadata_path: None,
        })
        .expect_err("missing cwd should fail");

        assert!(matches!(
            err,
            HookError::Normalization(NormalizationError::RetryableGateInput {
                field: "cwd",
                recovery_hint: _,
                ..
            })
        ));
    }
}
