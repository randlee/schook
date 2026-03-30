use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use atm_extension::AtmExtensionHandler;
use sc_hooks_core::context::HookContext;
use sc_hooks_core::events::HookType;
use sc_hooks_core::session::{
    ActivePid, AgentState, AiCurrentDir, AiRootDir, CanonicalSessionRecord, Provider, SessionId,
    SessionStartSource, StateRoot, UtcTimestamp,
};
use sc_hooks_core::storage::SessionStore;
use sc_hooks_sdk::traits::SyncHandler;
use serde_json::Value;

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    entries: Vec<(String, Option<String>)>,
}

impl EnvGuard {
    fn set(pairs: &[(&str, &str)]) -> Self {
        let lock = test_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut entries = Vec::new();
        for (key, value) in pairs {
            entries.push(((*key).to_string(), std::env::var(key).ok()));
            // SAFETY: tests serialize env mutation through EnvGuard's mutex.
            unsafe { std::env::set_var(key, value) };
        }
        Self {
            _lock: lock,
            entries,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, original) in self.entries.drain(..) {
            match original {
                Some(value) => {
                    // SAFETY: tests serialize env mutation through EnvGuard's mutex.
                    unsafe { std::env::set_var(&key, value) };
                }
                None => {
                    // SAFETY: tests serialize env mutation through EnvGuard's mutex.
                    unsafe { std::env::remove_var(&key) };
                }
            }
        }
    }
}

fn hook_context(hook: HookType, event: Option<&str>, payload: Value) -> HookContext {
    HookContext::new(
        hook,
        event.map(str::to_string),
        serde_json::json!({
            "hook": { "type": hook.as_str(), "event": event },
            "payload": payload,
        }),
        None,
    )
}

fn write_session_record(state_root: &Path, ai_root_dir: &Path, session_id: &str, active_pid: u32) {
    let store = SessionStore::new(StateRoot::new(state_root).expect("state root"));
    let record = CanonicalSessionRecord::new(
        Provider::Claude,
        SessionId::new(session_id.to_string()).expect("session id"),
        ActivePid::new(active_pid).expect("pid"),
        AiRootDir::new(ai_root_dir).expect("ai root dir"),
        AiCurrentDir::new(ai_root_dir.join("subdir")).expect("ai current dir"),
        SessionStartSource::Startup,
        AgentState::Starting,
        "SessionStart",
        "session_started",
    )
    .expect("session record should construct");
    store
        .persist(&record)
        .expect("session record should persist");
}

fn write_ended_session_record(
    state_root: &Path,
    ai_root_dir: &Path,
    session_id: &str,
    active_pid: u32,
) {
    let store = SessionStore::new(StateRoot::new(state_root).expect("state root"));
    let mut record = CanonicalSessionRecord::new(
        Provider::Claude,
        SessionId::new(session_id.to_string()).expect("session id"),
        ActivePid::new(active_pid).expect("pid"),
        AiRootDir::new(ai_root_dir).expect("ai root dir"),
        AiCurrentDir::new(ai_root_dir.join("subdir")).expect("ai current dir"),
        SessionStartSource::Startup,
        AgentState::Starting,
        "SessionStart",
        "session_started",
    )
    .expect("session record should construct");
    record
        .apply_hook_update(
            record.active_pid(),
            record.ai_current_dir().clone(),
            record.session_start_source(),
            AgentState::Ended,
            UtcTimestamp::from_field("updated_at", "2026-03-30T00:00:00Z").expect("timestamp"),
            "SessionEnd",
            "session_ended",
            Some(UtcTimestamp::from_field("ended_at", "2026-03-30T00:00:00Z").expect("timestamp")),
        )
        .expect("ended session update should validate");
    store
        .persist(&record)
        .expect("ended session record should persist");
}

fn load_record(state_root: &Path, session_id: &str) -> serde_json::Value {
    let rendered = fs::read_to_string(state_root.join(format!("{session_id}.json")))
        .expect("state file should exist");
    serde_json::from_str(&rendered).expect("session record should parse")
}

fn write_atm_toml(root: &Path, team: &str, identity: &str) {
    fs::write(
        root.join(".atm.toml"),
        format!("[core]\ndefault_team = \"{team}\"\nidentity = \"{identity}\"\n"),
    )
    .expect(".atm.toml should write");
}

#[test]
fn pre_tool_use_writes_identity_file_and_updates_extensions() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let state_root = temp.path().join("state");
    let tmp_root = temp.path().join("tmp");
    fs::create_dir_all(&repo_root).expect("repo root");
    fs::create_dir_all(&tmp_root).expect("tmp root");
    write_atm_toml(&repo_root, "atm-dev", "arch-hook");
    write_session_record(&state_root, &repo_root, "sess-1", 9001);

    let _env = EnvGuard::set(&[
        ("SC_HOOKS_STATE_DIR", state_root.to_str().expect("utf8")),
        ("ATM_HOOK_TMP_DIR", tmp_root.to_str().expect("utf8")),
        ("ATM_TEAM", ""),
        ("ATM_IDENTITY", ""),
    ]);

    let result = AtmExtensionHandler
        .handle(hook_context(
            HookType::PreToolUse,
            Some("Bash"),
            serde_json::json!({
                "session_id": "sess-1",
                "hook_event_name": "PreToolUse",
                "cwd": repo_root,
                "tool_name": "Bash",
                "tool_input": {"command": "atm read --team atm-dev"},
            }),
        ))
        .expect("pre tool use should succeed");
    assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);

    let identity_file = tmp_root.join("atm-hook-9001.json");
    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(identity_file).expect("identity file"))
            .expect("json");
    assert_eq!(payload["session_id"], "sess-1");
    assert_eq!(payload["team_name"], "atm-dev");
    assert_eq!(payload["agent_name"], "arch-hook");

    let record = load_record(&state_root, "sess-1");
    assert_eq!(record["extensions"]["atm"]["atm_team"], "atm-dev");
    assert_eq!(record["extensions"]["atm"]["atm_identity"], "arch-hook");
}

#[test]
fn post_tool_use_deletes_identity_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let state_root = temp.path().join("state");
    let tmp_root = temp.path().join("tmp");
    fs::create_dir_all(&repo_root).expect("repo root");
    fs::create_dir_all(&tmp_root).expect("tmp root");
    write_atm_toml(&repo_root, "atm-dev", "arch-hook");
    write_session_record(&state_root, &repo_root, "sess-1", 9002);
    let identity_file = tmp_root.join("atm-hook-9002.json");
    fs::write(&identity_file, "{}").expect("identity file should pre-exist");

    let _env = EnvGuard::set(&[
        ("SC_HOOKS_STATE_DIR", state_root.to_str().expect("utf8")),
        ("ATM_HOOK_TMP_DIR", tmp_root.to_str().expect("utf8")),
        ("ATM_TEAM", ""),
        ("ATM_IDENTITY", ""),
    ]);

    AtmExtensionHandler
        .handle(hook_context(
            HookType::PostToolUse,
            Some("Bash"),
            serde_json::json!({
                "session_id": "sess-1",
                "hook_event_name": "PostToolUse",
                "cwd": repo_root,
                "tool_name": "Bash",
                "tool_input": {"command": "atm read --team atm-dev"},
            }),
        ))
        .expect("post tool use should succeed");

    assert!(!identity_file.exists());
}

#[test]
fn non_atm_bash_command_is_a_noop_for_identity_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let state_root = temp.path().join("state");
    let tmp_root = temp.path().join("tmp");
    fs::create_dir_all(&repo_root).expect("repo root");
    fs::create_dir_all(&tmp_root).expect("tmp root");
    write_atm_toml(&repo_root, "atm-dev", "arch-hook");
    write_session_record(&state_root, &repo_root, "sess-1", 9003);

    let _env = EnvGuard::set(&[
        ("SC_HOOKS_STATE_DIR", state_root.to_str().expect("utf8")),
        ("ATM_HOOK_TMP_DIR", tmp_root.to_str().expect("utf8")),
        ("ATM_TEAM", ""),
        ("ATM_IDENTITY", ""),
    ]);

    AtmExtensionHandler
        .handle(hook_context(
            HookType::PreToolUse,
            Some("Bash"),
            serde_json::json!({
                "session_id": "sess-1",
                "hook_event_name": "PreToolUse",
                "cwd": repo_root,
                "tool_name": "Bash",
                "tool_input": {"command": "echo hello"},
            }),
        ))
        .expect("pre tool use should succeed");

    assert!(!tmp_root.join("atm-hook-9003.json").exists());
}

#[test]
fn permission_request_updates_state_and_appends_event() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let state_root = temp.path().join("state");
    let atm_home = temp.path().join("atm-home");
    fs::create_dir_all(&repo_root).expect("repo root");
    fs::create_dir_all(&atm_home).expect("atm home");
    write_atm_toml(&repo_root, "atm-dev", "arch-hook");
    write_session_record(&state_root, &repo_root, "sess-2", 9004);

    let _env = EnvGuard::set(&[
        ("SC_HOOKS_STATE_DIR", state_root.to_str().expect("utf8")),
        ("ATM_HOME", atm_home.to_str().expect("utf8")),
        ("ATM_TEAM", ""),
        ("ATM_IDENTITY", ""),
    ]);

    AtmExtensionHandler
        .handle(hook_context(
            HookType::PermissionRequest,
            None,
            serde_json::json!({
                "session_id": "sess-2",
                "hook_event_name": "PermissionRequest",
                "cwd": repo_root,
                "tool_name": "Bash",
                "tool_input": {"command": "atm read --team atm-dev"},
                "permission_suggestions": [
                    {
                        "type": "addRules",
                        "rules": [{"toolName": "Bash", "ruleContent": "atm read:*"}]
                    }
                ],
            }),
        ))
        .expect("permission request should succeed");

    let record = load_record(&state_root, "sess-2");
    assert_eq!(record["agent_state"], "awaiting_permission");
    assert_eq!(record["extensions"]["atm"]["atm_team"], "atm-dev");

    let events = fs::read_to_string(atm_home.join(".atm/daemon/hooks/events.jsonl"))
        .expect("events file should exist");
    let event: serde_json::Value =
        serde_json::from_str(events.lines().next().expect("line")).expect("event should parse");
    assert_eq!(event["event"], "permission_request");
    assert_eq!(event["tool_name"], "Bash");
}

#[test]
fn stop_and_teammate_idle_map_to_idle_and_append_relay_events() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let state_root = temp.path().join("state");
    let atm_home = temp.path().join("atm-home");
    let tmp_root = temp.path().join("tmp");
    fs::create_dir_all(&repo_root).expect("repo root");
    fs::create_dir_all(&atm_home).expect("atm home");
    fs::create_dir_all(&tmp_root).expect("tmp root");
    write_atm_toml(&repo_root, "atm-dev", "arch-hook");
    write_session_record(&state_root, &repo_root, "sess-3", 9005);
    let identity_file = tmp_root.join("atm-hook-9005.json");
    fs::write(&identity_file, "{}").expect("identity file should pre-exist");

    let _env = EnvGuard::set(&[
        ("SC_HOOKS_STATE_DIR", state_root.to_str().expect("utf8")),
        ("ATM_HOME", atm_home.to_str().expect("utf8")),
        ("ATM_HOOK_TMP_DIR", tmp_root.to_str().expect("utf8")),
        ("ATM_TEAM", ""),
        ("ATM_IDENTITY", ""),
    ]);

    AtmExtensionHandler
        .handle(hook_context(
            HookType::Stop,
            None,
            serde_json::json!({
                "session_id": "sess-3",
                "hook_event_name": "Stop",
                "cwd": repo_root,
                "stop_hook_active": false,
            }),
        ))
        .expect("stop should succeed");
    AtmExtensionHandler
        .handle(hook_context(
            HookType::TeammateIdle,
            None,
            serde_json::json!({
                "session_id": "sess-3",
                "hook_event_name": "TeammateIdle",
                "name": "arch-hook",
                "team_name": "atm-dev",
            }),
        ))
        .expect("teammate idle should succeed");

    let record = load_record(&state_root, "sess-3");
    assert_eq!(record["agent_state"], "idle");
    assert_eq!(record["last_hook_event"], "TeammateIdle");

    let events = fs::read_to_string(atm_home.join(".atm/daemon/hooks/events.jsonl"))
        .expect("events file should exist");
    let lines: Vec<_> = events.lines().collect();
    assert_eq!(lines.len(), 2);
    let stop_event: serde_json::Value = serde_json::from_str(lines[0]).expect("stop event");
    let teammate_event: serde_json::Value =
        serde_json::from_str(lines[1]).expect("teammate idle event");
    assert_eq!(stop_event["event"], "stop");
    assert_eq!(teammate_event["event"], "teammate_idle");
    assert!(teammate_event["received_at"].is_string());
    assert!(!identity_file.exists());
}

#[test]
fn stop_does_not_revive_ended_record() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let state_root = temp.path().join("state");
    fs::create_dir_all(&repo_root).expect("repo root");
    write_atm_toml(&repo_root, "atm-dev", "arch-hook");
    write_ended_session_record(&state_root, &repo_root, "sess-ended", 9007);

    let _env = EnvGuard::set(&[
        ("SC_HOOKS_STATE_DIR", state_root.to_str().expect("utf8")),
        ("ATM_TEAM", "atm-dev"),
        ("ATM_IDENTITY", "arch-hook"),
    ]);

    let result = AtmExtensionHandler
        .handle(hook_context(
            HookType::Stop,
            None,
            serde_json::json!({
                "session_id": "sess-ended",
                "cwd": repo_root,
            }),
        ))
        .expect("ended sessions should be ignored");
    assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);

    let record = load_record(&state_root, "sess-ended");
    assert_eq!(record["agent_state"], "ended");
    assert_eq!(record["last_hook_event"], "SessionEnd");
}

#[test]
fn malformed_permission_suggestions_report_index_and_field() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let state_root = temp.path().join("state");
    fs::create_dir_all(&repo_root).expect("repo root");
    write_atm_toml(&repo_root, "atm-dev", "arch-hook");
    write_session_record(&state_root, &repo_root, "sess-4", 9006);

    let _env = EnvGuard::set(&[
        ("SC_HOOKS_STATE_DIR", state_root.to_str().expect("utf8")),
        ("ATM_TEAM", ""),
        ("ATM_IDENTITY", ""),
    ]);

    let err = AtmExtensionHandler
        .handle(hook_context(
            HookType::PermissionRequest,
            None,
            serde_json::json!({
                "session_id": "sess-4",
                "hook_event_name": "PermissionRequest",
                "cwd": repo_root,
                "tool_name": "Bash",
                "tool_input": {"command": "atm read --team atm-dev"},
                "permission_suggestions": [
                    {
                        "type": "addRules",
                        "rules": [{"toolName": 7, "ruleContent": "atm read:*"}]
                    }
                ],
            }),
        ))
        .expect_err("malformed permission_suggestions should fail");

    match err {
        sc_hooks_core::errors::HookError::Validation { field, message, .. } => {
            assert_eq!(field, "permission_suggestions[0].rules[0].toolName");
            assert_eq!(message, "must be a string");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn no_atm_context_is_fail_open() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let state_root = temp.path().join("state");
    let atm_home = temp.path().join("atm-home");
    fs::create_dir_all(&repo_root).expect("repo root");
    fs::create_dir_all(&atm_home).expect("atm home");
    write_session_record(&state_root, &repo_root, "sess-5", 9007);

    let _env = EnvGuard::set(&[
        ("SC_HOOKS_STATE_DIR", state_root.to_str().expect("utf8")),
        ("ATM_HOME", atm_home.to_str().expect("utf8")),
        ("ATM_TEAM", ""),
        ("ATM_IDENTITY", ""),
    ]);

    let result = AtmExtensionHandler
        .handle(hook_context(
            HookType::Stop,
            None,
            serde_json::json!({
                "session_id": "sess-5",
                "hook_event_name": "Stop",
                "cwd": repo_root,
                "stop_hook_active": false,
            }),
        ))
        .expect("missing ATM routing should still proceed");

    assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);
    assert!(!atm_home.join(".atm/daemon/hooks/events.jsonl").exists());
    let record = load_record(&state_root, "sess-5");
    assert!(record["extensions"].get("atm").is_none());
}
