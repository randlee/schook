//! PreToolUse(Agent) policy gate for named-agent vs background-agent launches.
//! Reads canonical session state, enforces `.atm.toml` project policy, and
//! records subagent linkage metadata for later relay/runtime steps.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sc_hooks_core::context::HookContext;
use sc_hooks_core::dispatch::DispatchMode;
use sc_hooks_core::errors::HookError;
use sc_hooks_core::events::HookType;
use sc_hooks_core::manifest::Manifest;
use sc_hooks_core::results::HookResult;
use sc_hooks_core::session::utc_timestamp_now;
use sc_hooks_core::storage::{SessionStore, resolve_state_root};
use sc_hooks_core::tools::{SpawnKind, ToolName};
use sc_hooks_sdk::result::{block, proceed};
use sc_hooks_sdk::traits::{ManifestProvider, SyncHandler};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Default)]
pub struct AgentSpawnGatesHandler;

#[derive(Debug, Deserialize)]
struct AgentToolInput {
    prompt: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    run_in_background: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PreToolUseAgentPayload {
    tool_name: String,
    #[serde(default)]
    tool_use_id: Option<String>,
    tool_input: AgentToolInput,
}

#[derive(Debug, Deserialize, Default)]
struct AtmTomlRoot {
    #[serde(default)]
    agent_spawn: AgentSpawnPolicy,
}

#[derive(Debug, Deserialize, Default)]
struct AgentSpawnPolicy {
    #[serde(default)]
    background_only: bool,
}

impl ManifestProvider for AgentSpawnGatesHandler {
    fn manifest(&self) -> Manifest {
        Manifest {
            contract_version: 1,
            name: "agent-spawn-gates".to_string(),
            mode: DispatchMode::Sync,
            hooks: vec!["PreToolUse".to_string()],
            matchers: vec!["Agent".to_string()],
            payload_conditions: Vec::new(),
            timeout_ms: Some(2_000),
            long_running: false,
            response_time: None,
            requires: BTreeMap::new(),
            optional: BTreeMap::new(),
            sandbox: None,
            description: Some(
                "Applies named-agent/background-agent spawn policy and writes subagent linkage state."
                    .to_string(),
            ),
        }
    }
}

impl SyncHandler for AgentSpawnGatesHandler {
    fn handle(&self, context: HookContext) -> Result<HookResult, HookError> {
        if context.hook != HookType::PreToolUse {
            return Ok(proceed());
        }

        let payload: PreToolUseAgentPayload = context.payload()?;
        let tool_name = ToolName::new(payload.tool_name.clone())?;
        if tool_name.as_str() != "Agent" {
            return Ok(proceed());
        }

        let store = SessionStore::new(resolve_state_root()?);
        let mut record = match store.load_by_hook_context(&context)? {
            Some(record) => record,
            None => {
                return Ok(block(
                    "Agent spawn blocked: canonical session state is unavailable. Retry after SessionStart establishes state for this session.",
                ));
            }
        };

        let spawn_kind = if payload.tool_input.run_in_background.unwrap_or(false) {
            SpawnKind::BackgroundAgent
        } else {
            SpawnKind::NamedAgent
        };

        let policy = load_agent_spawn_policy(record.ai_root_dir.as_path())?;
        if policy.background_only && spawn_kind == SpawnKind::NamedAgent {
            return Ok(block(
                "Agent spawn blocked: this project requires background agents. Retry with `tool_input.run_in_background=true`.",
            ));
        }

        let next_extension = spawn_extension(&record, &payload, spawn_kind);
        let changed = record.extensions.get("spawn_gate") != Some(&next_extension);
        if changed {
            record
                .extensions
                .insert("spawn_gate".to_string(), next_extension);
            record.state_revision += 1;
            record.updated_at = utc_timestamp_now();
        }
        let persist = store.persist(&record)?;
        debug_assert!(matches!(
            persist,
            sc_hooks_core::storage::PersistOutcome::Created
                | sc_hooks_core::storage::PersistOutcome::Updated
                | sc_hooks_core::storage::PersistOutcome::Unchanged
        ));

        Ok(proceed())
    }
}

fn load_agent_spawn_policy(project_root_dir: &Path) -> Result<AgentSpawnPolicy, HookError> {
    let config_path = project_root_dir.join(".atm.toml");
    if !config_path.exists() {
        return Ok(AgentSpawnPolicy::default());
    }
    let body = fs::read_to_string(&config_path)
        .map_err(|source| HookError::state_io(config_path.clone(), source))?;
    toml::from_str::<AtmTomlRoot>(&body)
        .map(|root| root.agent_spawn)
        .map_err(|err| {
            HookError::validation(
                ".atm.toml",
                format!("failed to parse {}: {err}", config_path.display()),
            )
        })
}

fn spawn_extension(
    record: &sc_hooks_core::session::CanonicalSessionRecord,
    payload: &PreToolUseAgentPayload,
    spawn_kind: SpawnKind,
) -> Value {
    json!({
        "last_requested_spawn": {
            "tool_use_id": payload.tool_use_id,
            "spawn_kind": spawn_kind.as_str(),
            "parent_session_id": record.session_id.as_str(),
            "parent_active_pid": record.active_pid.get(),
            "prompt_excerpt": excerpt(&payload.tool_input.prompt),
            "description": payload.tool_input.description,
            "run_in_background": payload.tool_input.run_in_background.unwrap_or(false),
        }
    })
}

fn excerpt(text: &str) -> String {
    text.chars().take(120).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_hooks_core::session::{
        ActivePid, AgentState, AiCurrentDir, AiRootDir, CanonicalSessionRecord, SessionId,
    };
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        key: &'static str,
        original: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &std::path::Path) -> Self {
            let original = std::env::var_os(key);
            // SAFETY: tests serialize env mutation with a process-wide mutex.
            unsafe { std::env::set_var(key, value) };
            Self { key, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.original {
                // SAFETY: tests serialize env mutation with a process-wide mutex.
                unsafe { std::env::set_var(self.key, value) };
            } else {
                // SAFETY: tests serialize env mutation with a process-wide mutex.
                unsafe { std::env::remove_var(self.key) };
            }
        }
    }

    fn write_record(state_root: &Path, project_root: &Path) -> SessionId {
        let store = SessionStore::new(state_root.to_path_buf());
        let session_id = SessionId::new("session-1").expect("session");
        let record = CanonicalSessionRecord::new(
            "claude",
            session_id.clone(),
            ActivePid::new(4242).expect("pid"),
            AiRootDir::new(project_root).expect("root"),
            AiCurrentDir::new(project_root.join("agents")).expect("current"),
            "startup",
            AgentState::Busy,
            "PreToolUse",
            "tool_invocation_started",
        );
        store.persist(&record).expect("persist");
        session_id
    }

    fn agent_context(run_in_background: Option<bool>, tool_name: &str) -> HookContext {
        HookContext::new(
            HookType::PreToolUse,
            Some(tool_name.to_string()),
            json!({
                "payload": {
                    "session_id": "session-1",
                    "tool_name": tool_name,
                    "tool_use_id": "toolu_123",
                    "tool_input": {
                        "prompt": "Reply READY only.",
                        "description": "health-check child",
                        "run_in_background": run_in_background
                    }
                }
            }),
            None,
        )
    }

    #[test]
    fn non_agent_payloads_are_ignored() {
        let handler = AgentSpawnGatesHandler;
        let result = handler
            .handle(agent_context(None, "Bash"))
            .expect("non-agent payload should not error");
        assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);
    }

    #[test]
    fn named_agent_is_blocked_when_project_requires_background_agents() {
        let _guard = test_lock().lock().expect("lock");
        let state_root = tempfile::tempdir().expect("state root");
        let project_root = tempfile::tempdir().expect("project root");
        fs::write(
            project_root.path().join(".atm.toml"),
            "[agent_spawn]\nbackground_only = true\n",
        )
        .expect("write .atm.toml");
        let _env = EnvGuard::set("SC_HOOKS_STATE_DIR", state_root.path());
        let _session_id = write_record(state_root.path(), project_root.path());

        let handler = AgentSpawnGatesHandler;
        let result = handler
            .handle(agent_context(Some(false), "Agent"))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Block);
        assert_eq!(
            result.reason.as_deref(),
            Some(
                "Agent spawn blocked: this project requires background agents. Retry with `tool_input.run_in_background=true`."
            )
        );
    }

    #[test]
    fn background_agent_writes_linkage_into_canonical_state() {
        let _guard = test_lock().lock().expect("lock");
        let state_root = tempfile::tempdir().expect("state root");
        let project_root = tempfile::tempdir().expect("project root");
        let _env = EnvGuard::set("SC_HOOKS_STATE_DIR", state_root.path());
        let session_id = write_record(state_root.path(), project_root.path());

        let handler = AgentSpawnGatesHandler;
        let result = handler
            .handle(agent_context(Some(true), "Agent"))
            .expect("handler result");
        assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);

        let store = SessionStore::new(state_root.path().to_path_buf());
        let updated = store
            .load(&session_id)
            .expect("load")
            .expect("record should exist");
        let linkage = &updated.extensions["spawn_gate"]["last_requested_spawn"];
        assert_eq!(linkage["spawn_kind"], "background_agent");
        assert_eq!(linkage["parent_session_id"], "session-1");
        assert_eq!(linkage["parent_active_pid"], 4242);
        assert_eq!(linkage["run_in_background"], true);
    }

    #[test]
    fn missing_atm_file_falls_back_to_generic_policy() {
        let _guard = test_lock().lock().expect("lock");
        let state_root = tempfile::tempdir().expect("state root");
        let project_root = tempfile::tempdir().expect("project root");
        let _env = EnvGuard::set("SC_HOOKS_STATE_DIR", state_root.path());
        let _session_id = write_record(state_root.path(), project_root.path());

        let handler = AgentSpawnGatesHandler;
        let result = handler
            .handle(agent_context(None, "Agent"))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);
    }

    #[test]
    fn missing_session_record_blocks_with_retryable_reason() {
        let _guard = test_lock().lock().expect("lock");
        let state_root = tempfile::tempdir().expect("state root");
        let _env = EnvGuard::set("SC_HOOKS_STATE_DIR", state_root.path());

        let handler = AgentSpawnGatesHandler;
        let result = handler
            .handle(agent_context(Some(true), "Agent"))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Block);
        assert_eq!(
            result.reason.as_deref(),
            Some(
                "Agent spawn blocked: canonical session state is unavailable. Retry after SessionStart establishes state for this session.",
            )
        );
    }
}
