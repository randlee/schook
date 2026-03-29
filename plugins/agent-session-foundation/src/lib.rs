mod payloads;

use std::collections::BTreeMap;

use payloads::{PreCompactPayload, SessionEndPayload, SessionStartPayload, StopPayload};
use sc_hooks_core::context::HookContext;
use sc_hooks_core::dispatch::DispatchMode;
use sc_hooks_core::errors::HookError;
use sc_hooks_core::events::HookType;
use sc_hooks_core::manifest::Manifest;
use sc_hooks_core::results::HookResult;
use sc_hooks_core::session::{
    ActivePid, AgentState, AiCurrentDir, AiRootDir, CanonicalSessionRecord, SessionId,
    utc_timestamp_now,
};
use sc_hooks_core::storage::{SessionStore, resolve_state_root};
use sc_hooks_sdk::result::proceed;
use sc_hooks_sdk::traits::{ManifestProvider, SyncHandler};

/// Sync lifecycle handler that owns canonical session-state persistence for the
/// verified Claude hook lifecycle surfaces.
#[derive(Debug, Default)]
pub struct SessionFoundationHandler;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LifecycleEvent {
    SessionStart,
    SessionEnd,
    PreCompact,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SessionTransition {
    session_id: SessionId,
    agent_state: AgentState,
    session_start_source: Option<String>,
    state_reason: String,
    ended_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedRuntime {
    session_id: SessionId,
    active_pid: ActivePid,
    ai_root_dir: AiRootDir,
    ai_current_dir: AiCurrentDir,
    transition: SessionTransition,
}

impl ManifestProvider for SessionFoundationHandler {
    fn manifest(&self) -> Manifest {
        Manifest {
            contract_version: 1,
            name: "agent-session-foundation".to_string(),
            mode: DispatchMode::Sync,
            hooks: vec![
                "SessionStart".to_string(),
                "SessionEnd".to_string(),
                "PreCompact".to_string(),
                "Stop".to_string(),
            ],
            matchers: vec!["*".to_string()],
            payload_conditions: Vec::new(),
            timeout_ms: Some(2_000),
            long_running: false,
            response_time: None,
            requires: BTreeMap::new(),
            optional: BTreeMap::new(),
            sandbox: None,
            description: Some(
                "Persists canonical hook session state from verified lifecycle inputs.".to_string(),
            ),
        }
    }
}

impl SyncHandler for SessionFoundationHandler {
    fn handle(&self, context: HookContext) -> Result<HookResult, HookError> {
        let lifecycle_event = LifecycleEvent::try_from(context.hook)?;
        let state_root = resolve_state_root()?;
        let store = SessionStore::new(state_root);
        let existing = store.load_by_hook_context(&context)?;
        let resolved = resolve_runtime(&context, lifecycle_event, existing.as_ref())?;

        let next_record = build_next_record(
            lifecycle_event,
            &context,
            existing,
            &resolved,
            resolved.transition.session_start_source.as_deref(),
        )?;
        let _persist = store.persist(&next_record)?;

        Ok(proceed())
    }
}

impl TryFrom<HookType> for LifecycleEvent {
    type Error = HookError;

    fn try_from(value: HookType) -> Result<Self, Self::Error> {
        match value {
            HookType::SessionStart => Ok(Self::SessionStart),
            HookType::SessionEnd => Ok(Self::SessionEnd),
            HookType::PreCompact => Ok(Self::PreCompact),
            HookType::Stop => Ok(Self::Stop),
            _ => Err(HookError::invalid_context(format!(
                "unsupported hook for session foundation: {}",
                value.as_str()
            ))),
        }
    }
}

impl LifecycleEvent {
    fn as_str(self) -> &'static str {
        match self {
            Self::SessionStart => "SessionStart",
            Self::SessionEnd => "SessionEnd",
            Self::PreCompact => "PreCompact",
            Self::Stop => "Stop",
        }
    }
}

fn resolve_runtime(
    context: &HookContext,
    lifecycle_event: LifecycleEvent,
    existing: Option<&CanonicalSessionRecord>,
) -> Result<ResolvedRuntime, HookError> {
    let session_id = session_id_from_context(context, lifecycle_event)?;
    let active_pid = resolve_active_pid(lifecycle_event, existing)?;
    let ai_root_dir = resolve_ai_root_dir(lifecycle_event, existing)?;
    let ai_current_dir = resolve_ai_current_dir(context)?;
    let transition = resolve_transition(context, lifecycle_event, &session_id)?;

    Ok(ResolvedRuntime {
        session_id,
        active_pid,
        ai_root_dir,
        ai_current_dir,
        transition,
    })
}

fn build_next_record(
    lifecycle_event: LifecycleEvent,
    _context: &HookContext,
    existing: Option<CanonicalSessionRecord>,
    resolved: &ResolvedRuntime,
    session_start_source: Option<&str>,
) -> Result<CanonicalSessionRecord, HookError> {
    let event_name = lifecycle_event.as_str().to_string();
    let now = utc_timestamp_now();

    let mut record = match existing {
        Some(mut record) => {
            let next_source = session_start_source.unwrap_or(record.session_start_source.as_str());
            let material_changed = record.active_pid != resolved.active_pid
                || record.ai_root_dir != resolved.ai_root_dir
                || record.ai_current_dir != resolved.ai_current_dir
                || record.agent_state != resolved.transition.agent_state
                || record.session_start_source != next_source
                || record.last_hook_event != event_name
                || record.state_reason != resolved.transition.state_reason
                || record.ended_at != resolved.transition.ended_at;
            if !material_changed {
                return Ok(record);
            }

            record.active_pid = resolved.active_pid;
            record.ai_root_dir = resolved.ai_root_dir.clone();
            record.ai_current_dir = resolved.ai_current_dir.clone();
            record.agent_state = resolved.transition.agent_state;
            record.session_start_source = next_source.to_string();
            record.state_revision += 1;
            record
        }
        None => CanonicalSessionRecord::new(
            "claude",
            resolved.session_id.clone(),
            resolved.active_pid,
            resolved.ai_root_dir.clone(),
            resolved.ai_current_dir.clone(),
            session_start_source.unwrap_or("startup"),
            resolved.transition.agent_state,
            event_name.clone(),
            resolved.transition.state_reason.clone(),
        ),
    };

    if record.session_id != resolved.session_id {
        return Err(HookError::validation(
            "session_id",
            "existing record does not match resolved session id",
        ));
    }

    record.agent_state = resolved.transition.agent_state;
    record.last_hook_event = event_name;
    record.last_hook_event_at = now.clone();
    record.updated_at = now;
    record.state_reason = resolved.transition.state_reason.clone();
    record.ended_at = resolved.transition.ended_at.clone();

    if lifecycle_event == LifecycleEvent::SessionStart && record.created_at.is_empty() {
        record.created_at = utc_timestamp_now();
    }

    Ok(record)
}

fn resolve_transition(
    context: &HookContext,
    lifecycle_event: LifecycleEvent,
    session_id: &SessionId,
) -> Result<SessionTransition, HookError> {
    match lifecycle_event {
        LifecycleEvent::SessionStart => {
            let payload: SessionStartPayload = context.payload()?;
            Ok(SessionTransition {
                session_id: SessionId::new(payload.session_id)?,
                agent_state: AgentState::Starting,
                session_start_source: Some(payload.source),
                state_reason: "session_started".to_string(),
                ended_at: None,
            })
        }
        LifecycleEvent::SessionEnd => {
            let payload: SessionEndPayload = context.payload()?;
            Ok(SessionTransition {
                session_id: SessionId::new(payload.session_id)?,
                agent_state: AgentState::Ended,
                session_start_source: None,
                state_reason: payload
                    .reason
                    .unwrap_or_else(|| "session_ended".to_string()),
                ended_at: Some(utc_timestamp_now()),
            })
        }
        LifecycleEvent::PreCompact => {
            let payload: PreCompactPayload = context.payload()?;
            Ok(SessionTransition {
                session_id: SessionId::new(payload.session_id)?,
                agent_state: AgentState::Compacting,
                session_start_source: None,
                state_reason: "compaction_started".to_string(),
                ended_at: None,
            })
        }
        LifecycleEvent::Stop => {
            let payload: StopPayload = context.payload()?;
            Ok(SessionTransition {
                session_id: SessionId::new(payload.session_id)?,
                agent_state: AgentState::Idle,
                session_start_source: None,
                state_reason: "turn_completed".to_string(),
                ended_at: None,
            })
        }
    }
    .and_then(|transition| {
        if &transition.session_id == session_id {
            Ok(transition)
        } else {
            Err(HookError::validation(
                "session_id",
                "resolved session id does not match payload session id",
            ))
        }
    })
}

fn session_id_from_context(
    context: &HookContext,
    lifecycle_event: LifecycleEvent,
) -> Result<SessionId, HookError> {
    match lifecycle_event {
        LifecycleEvent::SessionStart => {
            let payload: SessionStartPayload = context.payload()?;
            SessionId::new(payload.session_id)
        }
        LifecycleEvent::SessionEnd => {
            let payload: SessionEndPayload = context.payload()?;
            SessionId::new(payload.session_id)
        }
        LifecycleEvent::PreCompact => {
            let payload: PreCompactPayload = context.payload()?;
            SessionId::new(payload.session_id)
        }
        LifecycleEvent::Stop => {
            let payload: StopPayload = context.payload()?;
            SessionId::new(payload.session_id)
        }
    }
}

fn resolve_ai_root_dir(
    lifecycle_event: LifecycleEvent,
    existing: Option<&CanonicalSessionRecord>,
) -> Result<AiRootDir, HookError> {
    if lifecycle_event == LifecycleEvent::SessionStart {
        let env_root = std::env::var("CLAUDE_PROJECT_DIR").map_err(|err| {
            HookError::invalid_context(format!(
                "CLAUDE_PROJECT_DIR is required on SessionStart: {err}"
            ))
        })?;
        return AiRootDir::new(env_root);
    }

    existing
        .map(|record| record.ai_root_dir.clone())
        .ok_or_else(|| {
            HookError::invalid_context(
                "ai_root_dir unavailable before SessionStart established canonical state",
            )
        })
}

fn resolve_ai_current_dir(context: &HookContext) -> Result<AiCurrentDir, HookError> {
    let cwd = context
        .payload_value()?
        .get("cwd")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| HookError::validation("cwd", "missing from payload"))?;
    AiCurrentDir::new(cwd)
}

fn resolve_active_pid(
    lifecycle_event: LifecycleEvent,
    existing: Option<&CanonicalSessionRecord>,
) -> Result<ActivePid, HookError> {
    if let Ok(raw) = std::env::var("SC_HOOK_AGENT_PID") {
        let parsed = raw.parse::<u32>().map_err(|err| {
            HookError::validation(
                "SC_HOOK_AGENT_PID",
                format!("must parse as positive integer: {err}"),
            )
        })?;
        return ActivePid::new(parsed);
    }

    if lifecycle_event == LifecycleEvent::SessionStart {
        return Err(HookError::invalid_context(
            "SC_HOOK_AGENT_PID is required on SessionStart",
        ));
    }

    existing
        .map(|record| record.active_pid)
        .ok_or_else(|| HookError::invalid_context("active_pid unavailable before SessionStart"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_all_normalized_state_transitions() {
        assert_eq!(
            resolve_transition(
                &HookContext::new(
                    HookType::SessionStart,
                    None,
                    serde_json::json!({"payload":{"session_id":"s1","cwd":"/projects/agent","source":"startup"}}),
                    None,
                ),
                LifecycleEvent::SessionStart,
                &SessionId::new("s1").expect("session id"),
            )
            .expect("start transition")
            .agent_state,
            AgentState::Starting
        );
        assert_eq!(
            resolve_transition(
                &HookContext::new(
                    HookType::PreCompact,
                    None,
                    serde_json::json!({"payload":{"session_id":"s1","cwd":"/projects/agent"}}),
                    None,
                ),
                LifecycleEvent::PreCompact,
                &SessionId::new("s1").expect("session id"),
            )
            .expect("compact transition")
            .agent_state,
            AgentState::Compacting
        );
        assert_eq!(
            resolve_transition(
                &HookContext::new(
                    HookType::Stop,
                    None,
                    serde_json::json!({"payload":{"session_id":"s1","cwd":"/projects/agent","stop_hook_active":false}}),
                    None,
                ),
                LifecycleEvent::Stop,
                &SessionId::new("s1").expect("session id"),
            )
            .expect("stop transition")
            .agent_state,
            AgentState::Idle
        );
        assert_eq!(
            resolve_transition(
                &HookContext::new(
                    HookType::SessionEnd,
                    None,
                    serde_json::json!({"payload":{"session_id":"s1","cwd":"/projects/agent"}}),
                    None,
                ),
                LifecycleEvent::SessionEnd,
                &SessionId::new("s1").expect("session id"),
            )
            .expect("end transition")
            .agent_state,
            AgentState::Ended
        );
    }
}
