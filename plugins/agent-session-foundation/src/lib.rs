mod payloads;

use std::collections::BTreeMap;

use payloads::{
    PreCompactPayload, SessionEndPayload, SessionStartPayload, SessionStartSource, StopPayload,
};
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
    session_start_source: Option<SessionStartSource>,
    state_reason: String,
    ended_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedRuntime {
    session_id: SessionId,
    active_pid: ActivePid,
    ai_root_dir: RootBinding,
    ai_current_dir: AiCurrentDir,
    transition: SessionTransition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EstablishedRoot(AiRootDir);

#[derive(Debug, Clone, PartialEq, Eq)]
struct PersistedRoot(AiRootDir);

#[derive(Debug, Clone, PartialEq, Eq)]
enum RootBinding {
    Established(EstablishedRoot),
    Persisted(PersistedRoot),
}

impl EstablishedRoot {
    fn from_root_establishing_session_start(context: &HookContext) -> Result<Self, HookError> {
        let cwd = context
            .payload_value()?
            .get("cwd")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| HookError::validation("cwd", "missing from payload"))?;
        let root = AiRootDir::new(cwd)?;
        verify_project_root_env_matches(context.hook.as_str(), root.as_path())?;
        Ok(Self(root))
    }

    fn as_ai_root_dir(&self) -> &AiRootDir {
        &self.0
    }
}

impl PersistedRoot {
    fn from_record(record: &CanonicalSessionRecord) -> Self {
        Self(record.ai_root_dir.clone())
    }

    fn as_ai_root_dir(&self) -> &AiRootDir {
        &self.0
    }
}

impl RootBinding {
    fn as_ai_root_dir(&self) -> &AiRootDir {
        match self {
            Self::Established(root) => root.as_ai_root_dir(),
            Self::Persisted(root) => root.as_ai_root_dir(),
        }
    }

    fn replaces_existing_root(&self) -> bool {
        matches!(self, Self::Established(_))
    }

    fn into_new_record_root(self) -> Result<AiRootDir, HookError> {
        match self {
            Self::Established(root) => Ok(root.0),
            Self::Persisted(_) => Err(HookError::invalid_context(
                "ai_root_dir unavailable before a root-establishing SessionStart",
            )),
        }
    }
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
            resolved.transition.session_start_source,
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
    let transition = resolve_transition(context, lifecycle_event)?;
    let session_id = transition.session_id.clone();
    let active_pid = resolve_active_pid(lifecycle_event, existing)?;
    let ai_root_dir = resolve_ai_root_dir(context, lifecycle_event, &transition, existing)?;
    let ai_current_dir = resolve_ai_current_dir(context)?;

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
    session_start_source: Option<SessionStartSource>,
) -> Result<CanonicalSessionRecord, HookError> {
    let event_name = lifecycle_event.as_str().to_string();
    let now = utc_timestamp_now();

    let mut record = match existing {
        Some(mut record) => {
            let next_source = session_start_source
                .map(SessionStartSource::as_str)
                .unwrap_or(record.session_start_source.as_str());
            let next_root = resolved.ai_root_dir.as_ai_root_dir();
            let root_changed =
                resolved.ai_root_dir.replaces_existing_root() && &record.ai_root_dir != next_root;
            let material_changed = record.active_pid != resolved.active_pid
                || root_changed
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
            if resolved.ai_root_dir.replaces_existing_root() {
                record.ai_root_dir = next_root.clone();
            }
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
            resolved.ai_root_dir.clone().into_new_record_root()?,
            resolved.ai_current_dir.clone(),
            session_start_source
                .map(SessionStartSource::as_str)
                .unwrap_or("startup"),
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

    record.last_hook_event = event_name;
    record.last_hook_event_at = now.clone();
    record.updated_at = now;
    record.state_reason = resolved.transition.state_reason.clone();
    record.ended_at = resolved.transition.ended_at.clone();

    Ok(record)
}

fn resolve_transition(
    context: &HookContext,
    lifecycle_event: LifecycleEvent,
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
}

fn resolve_ai_root_dir(
    context: &HookContext,
    lifecycle_event: LifecycleEvent,
    transition: &SessionTransition,
    existing: Option<&CanonicalSessionRecord>,
) -> Result<RootBinding, HookError> {
    if lifecycle_event == LifecycleEvent::SessionStart
        && transition
            .session_start_source
            .is_some_and(SessionStartSource::establishes_root)
    {
        return Ok(RootBinding::Established(
            EstablishedRoot::from_root_establishing_session_start(context)?,
        ));
    }

    let existing = existing.ok_or_else(|| {
        HookError::invalid_context(
            "ai_root_dir unavailable before a root-establishing SessionStart established canonical state",
        )
    })?;
    verify_project_root_env_matches(context.hook.as_str(), existing.ai_root_dir.as_path())?;
    Ok(RootBinding::Persisted(PersistedRoot::from_record(existing)))
}

fn resolve_ai_current_dir(context: &HookContext) -> Result<AiCurrentDir, HookError> {
    let cwd = context
        .payload_value()?
        .get("cwd")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| HookError::validation("cwd", "missing from payload"))?;
    AiCurrentDir::new(cwd)
}

fn verify_project_root_env_matches(
    hook_name: &str,
    expected_root: &std::path::Path,
) -> Result<(), HookError> {
    let Some(observed) = std::env::var_os("CLAUDE_PROJECT_DIR") else {
        return Ok(());
    };
    let observed = std::path::PathBuf::from(observed);
    if observed != expected_root {
        return Err(HookError::validation(
            "CLAUDE_PROJECT_DIR",
            format!(
                "must match immutable root on {hook_name}: expected {} but observed {}",
                expected_root.display(),
                observed.display()
            ),
        ));
    }
    Ok(())
}

fn resolve_active_pid(
    lifecycle_event: LifecycleEvent,
    existing: Option<&CanonicalSessionRecord>,
) -> Result<ActivePid, HookError> {
    if let Ok(raw) = std::env::var("SC_HOOK_AGENT_PID") {
        let parsed = raw.parse::<u32>().map_err(|parse_err| {
            HookError::validation(
                "SC_HOOK_AGENT_PID",
                format!("must parse as positive integer: {parse_err}"),
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
            )
            .expect("end transition")
            .agent_state,
            AgentState::Ended
        );
    }
}
