use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use agent_session_foundation::SessionFoundationHandler;
use sc_hooks_core::context::HookContext;
use sc_hooks_core::errors::RootDivergenceNotice;
use sc_hooks_core::events::HookType;
use sc_hooks_sdk::traits::SyncHandler;

fn hook_context_with_payload(
    hook: HookType,
    event: Option<&str>,
    payload: serde_json::Value,
) -> HookContext {
    HookContext::new(
        hook,
        event.map(|value| std::borrow::Cow::Owned(value.to_string())),
        serde_json::json!({
            "hook": { "type": hook.as_str(), "event": event },
            "payload": payload
        }),
        None,
    )
}

fn session_start_payload(session_id: &str, source: &str, cwd: &Path) -> serde_json::Value {
    serde_json::json!({
        "session_id": session_id,
        "cwd": cwd.to_str().expect("cwd utf8"),
        "source": source,
    })
}

fn stop_payload(session_id: &str, cwd: &Path) -> serde_json::Value {
    serde_json::json!({
        "session_id": session_id,
        "cwd": cwd.to_str().expect("cwd utf8"),
        "stop_hook_active": false,
    })
}

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
            // SAFETY: tests serialize environment mutation through EnvGuard's mutex.
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
                    // SAFETY: tests serialize environment mutation through EnvGuard's mutex.
                    unsafe { std::env::set_var(&key, value) };
                }
                None => {
                    // SAFETY: tests serialize environment mutation through EnvGuard's mutex.
                    unsafe { std::env::remove_var(&key) };
                }
            }
        }
    }
}

#[test]
fn startup_establishes_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
    fs::create_dir_all(&project_root).expect("project root");
    let session_id = "a760f75c-055a-46f9-bcbb-447c47a22f3c";
    let _env = EnvGuard::set(&[
        (
            "SC_HOOKS_STATE_DIR",
            temp.path().join("state").to_str().expect("state root utf8"),
        ),
        (
            "CLAUDE_PROJECT_DIR",
            project_root.to_str().expect("project root utf8"),
        ),
        ("SC_HOOK_AGENT_PID", "777"),
    ]);
    let handler = SessionFoundationHandler;

    handler
        .handle(hook_context_with_payload(
            HookType::SessionStart,
            None,
            session_start_payload(session_id, "startup", &project_root),
        ))
        .expect("session start should persist");

    let state_file = temp.path().join(format!("state/{session_id}.json"));
    let rendered = fs::read_to_string(state_file).expect("state file should exist");
    let parsed: serde_json::Value =
        serde_json::from_str(&rendered).expect("session state should parse");
    assert_eq!(parsed["session_id"], session_id);
    assert_eq!(parsed["active_pid"], 777);
    assert_eq!(parsed["ai_root_dir"], project_root.to_str().expect("utf8"));
    assert_eq!(
        parsed["ai_current_dir"],
        project_root.to_str().expect("utf8")
    );
    assert_eq!(parsed["agent_state"], "starting");
}

#[test]
fn bash_cd_drift_updates_current_dir_only() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
    let other_dir = temp.path().join("repo-b");
    let session_id = "session-drift";
    fs::create_dir_all(&project_root).expect("project root");
    fs::create_dir_all(&other_dir).expect("other dir");
    let handler = SessionFoundationHandler;

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                project_root.to_str().expect("project root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "900"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::SessionStart,
                None,
                session_start_payload(session_id, "startup", &project_root),
            ))
            .expect("session start should persist");
    }

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "900"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::Stop,
                None,
                stop_payload(session_id, &other_dir),
            ))
            .expect("stop should update existing session");
    }

    let state_file = temp.path().join(format!("state/{session_id}.json"));
    let rendered = fs::read_to_string(state_file).expect("state file should exist");
    let parsed: serde_json::Value =
        serde_json::from_str(&rendered).expect("session state should parse");
    assert_eq!(parsed["ai_root_dir"], project_root.to_str().expect("utf8"));
    assert_eq!(parsed["ai_current_dir"], other_dir.to_str().expect("utf8"));
    assert_eq!(parsed["agent_state"], "idle");
}

#[test]
fn noop_session_start_keeps_single_persisted_record() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
    let session_id = "session-noop";
    fs::create_dir_all(&project_root).expect("project root");
    let _env = EnvGuard::set(&[
        (
            "SC_HOOKS_STATE_DIR",
            temp.path().join("state").to_str().expect("state root utf8"),
        ),
        (
            "CLAUDE_PROJECT_DIR",
            project_root.to_str().expect("project root utf8"),
        ),
        ("SC_HOOK_AGENT_PID", "42"),
    ]);
    let handler = SessionFoundationHandler;

    handler
        .handle(hook_context_with_payload(
            HookType::SessionStart,
            None,
            session_start_payload(session_id, "startup", &project_root),
        ))
        .expect("session start should persist");
    handler
        .handle(hook_context_with_payload(
            HookType::SessionStart,
            None,
            session_start_payload(session_id, "startup", &project_root),
        ))
        .expect("second identical invocation should be allowed");

    let state_file = temp.path().join(format!("state/{session_id}.json"));
    let rendered = fs::read_to_string(state_file).expect("state file should exist");
    let parsed: serde_json::Value =
        serde_json::from_str(&rendered).expect("session state should parse");
    assert_eq!(parsed["state_revision"], 1);
    assert_eq!(parsed["last_hook_event"], "SessionStart");
}

#[test]
fn session_start_requires_injected_agent_pid() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
    let session_id = "session-missing-pid";
    fs::create_dir_all(&project_root).expect("project root");
    let _env = EnvGuard::set(&[
        (
            "SC_HOOKS_STATE_DIR",
            temp.path().join("state").to_str().expect("state root utf8"),
        ),
        (
            "CLAUDE_PROJECT_DIR",
            project_root.to_str().expect("project root utf8"),
        ),
    ]);
    let handler = SessionFoundationHandler;

    let err = handler
        .handle(hook_context_with_payload(
            HookType::SessionStart,
            None,
            session_start_payload(session_id, "startup", &project_root),
        ))
        .expect_err("session start should fail without injected agent pid");
    assert!(err.to_string().contains("SC_HOOK_AGENT_PID"));
}

#[test]
fn resume_establishes_new_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let startup_root = temp.path().join("repo-start");
    let resumed_root = temp.path().join("repo-resumed");
    let session_id = "session-resume";
    fs::create_dir_all(&startup_root).expect("startup root");
    fs::create_dir_all(&resumed_root).expect("resumed root");
    let handler = SessionFoundationHandler;

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                startup_root.to_str().expect("startup root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "501"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::SessionStart,
                None,
                session_start_payload(session_id, "startup", &startup_root),
            ))
            .expect("startup should persist");
    }

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                resumed_root.to_str().expect("resumed root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "777"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::SessionStart,
                None,
                session_start_payload(session_id, "resume", &resumed_root),
            ))
            .expect("resume should reestablish root");
    }

    let state_file = temp.path().join(format!("state/{session_id}.json"));
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(state_file).expect("state file should exist"))
            .expect("session state should parse");
    assert_eq!(parsed["ai_root_dir"], resumed_root.to_str().expect("utf8"));
    assert_eq!(
        parsed["ai_current_dir"],
        resumed_root.to_str().expect("utf8")
    );
    assert_eq!(parsed["active_pid"], 777);
    assert_eq!(parsed["session_start_source"], "resume");
}

#[test]
fn compact_preserves_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
    let drift_dir = temp.path().join("repo-a/subdir");
    let session_id = "session-compact";
    fs::create_dir_all(&drift_dir).expect("drift dir");
    let handler = SessionFoundationHandler;

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                project_root.to_str().expect("project root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "900"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::SessionStart,
                None,
                session_start_payload(session_id, "startup", &project_root),
            ))
            .expect("startup should persist");
    }

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                project_root.to_str().expect("project root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "900"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::Stop,
                None,
                stop_payload(session_id, &drift_dir),
            ))
            .expect("stop should preserve root");
        handler
            .handle(hook_context_with_payload(
                HookType::SessionStart,
                None,
                session_start_payload(session_id, "compact", &drift_dir),
            ))
            .expect("compact should preserve immutable root");
    }

    let state_file = temp.path().join(format!("state/{session_id}.json"));
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(state_file).expect("state file should exist"))
            .expect("session state should parse");
    assert_eq!(parsed["ai_root_dir"], project_root.to_str().expect("utf8"));
    assert_eq!(parsed["ai_current_dir"], drift_dir.to_str().expect("utf8"));
    assert_eq!(parsed["session_start_source"], "compact");
}

#[test]
fn clear_establishes_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let old_root = temp.path().join("repo-old");
    let new_root = temp.path().join("repo-new");
    let old_session = "session-before-clear";
    let new_session = "session-after-clear";
    fs::create_dir_all(&old_root).expect("old root");
    fs::create_dir_all(&new_root).expect("new root");
    let handler = SessionFoundationHandler;

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                old_root.to_str().expect("old root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "71"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::SessionStart,
                None,
                session_start_payload(old_session, "startup", &old_root),
            ))
            .expect("startup should persist");
    }

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                new_root.to_str().expect("new root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "88"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::SessionStart,
                None,
                session_start_payload(new_session, "clear", &new_root),
            ))
            .expect("clear should establish new root");
    }

    let old_state = temp.path().join(format!("state/{old_session}.json"));
    let new_state = temp.path().join(format!("state/{new_session}.json"));
    let old_parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(old_state).expect("old state"))
            .expect("old state should parse");
    let new_parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(new_state).expect("new state"))
            .expect("new state should parse");
    assert_eq!(old_parsed["ai_root_dir"], old_root.to_str().expect("utf8"));
    assert_eq!(new_parsed["ai_root_dir"], new_root.to_str().expect("utf8"));
    assert_eq!(new_parsed["session_start_source"], "clear");
}

#[test]
fn startup_project_dir_divergence_logs_and_uses_payload_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
    let mismatched_root = temp.path().join("repo-b");
    let session_id = "session-start-divergence";
    fs::create_dir_all(&project_root).expect("project root");
    fs::create_dir_all(&mismatched_root).expect("mismatched root");
    let handler = SessionFoundationHandler;

    let _env = EnvGuard::set(&[
        (
            "SC_HOOKS_STATE_DIR",
            temp.path().join("state").to_str().expect("state root utf8"),
        ),
        (
            "CLAUDE_PROJECT_DIR",
            mismatched_root.to_str().expect("mismatched root utf8"),
        ),
        ("SC_HOOK_AGENT_PID", "900"),
    ]);
    let result = handler
        .handle(hook_context_with_payload(
            HookType::SessionStart,
            None,
            session_start_payload(session_id, "startup", &project_root),
        ))
        .expect("startup divergence should surface structured notice");

    let state_file = temp.path().join(format!("state/{session_id}.json"));
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(state_file).expect("state file should exist"))
            .expect("session state should parse");
    assert_eq!(parsed["ai_root_dir"], project_root.to_str().expect("utf8"));
    assert_eq!(
        parsed["ai_current_dir"],
        project_root.to_str().expect("utf8")
    );

    let notice = RootDivergenceNotice::decode(
        result
            .additional_context
            .as_deref()
            .expect("root divergence should produce additional context"),
    )
    .expect("root divergence context should decode");
    assert_eq!(notice.immutable_root.as_path(), project_root.as_path());
    assert_eq!(notice.observed, mismatched_root);
    assert_eq!(notice.session_id.as_str(), session_id);
    assert_eq!(notice.hook_event, HookType::SessionStart);
}

#[test]
fn mismatched_project_dir_logs_and_preserves_immutable_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
    let drift_dir = temp.path().join("repo-a/subdir");
    let mismatched_root = temp.path().join("repo-b");
    let session_id = "session-mismatch";
    fs::create_dir_all(&drift_dir).expect("drift dir");
    fs::create_dir_all(&mismatched_root).expect("mismatched root");
    let handler = SessionFoundationHandler;

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                project_root.to_str().expect("project root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "900"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::SessionStart,
                None,
                session_start_payload(session_id, "startup", &project_root),
            ))
            .expect("startup should persist");
    }

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                mismatched_root.to_str().expect("mismatched root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "900"),
        ]);
        let result = handler
            .handle(hook_context_with_payload(
                HookType::Stop,
                None,
                stop_payload(session_id, &drift_dir),
            ))
            .expect("mismatched CLAUDE_PROJECT_DIR should log and continue");
        let notice = RootDivergenceNotice::decode(
            result
                .additional_context
                .as_deref()
                .expect("root divergence should produce additional context"),
        )
        .expect("root divergence context should decode");
        assert_eq!(notice.immutable_root.as_path(), project_root.as_path());
        assert_eq!(notice.observed, mismatched_root);
        assert_eq!(notice.session_id.as_str(), session_id);
        assert_eq!(notice.hook_event, HookType::Stop);
    }

    let state_file = temp.path().join(format!("state/{session_id}.json"));
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(state_file).expect("state file should exist"))
            .expect("session state should parse");
    assert_eq!(parsed["ai_root_dir"], project_root.to_str().expect("utf8"));
    assert_eq!(parsed["ai_current_dir"], drift_dir.to_str().expect("utf8"));
    assert_eq!(parsed["agent_state"], "idle");
}

#[test]
fn missing_project_dir_preserves_root_without_divergence_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
    let drift_dir = project_root.join("subdir");
    let session_id = "session-missing-project-dir";
    fs::create_dir_all(&drift_dir).expect("drift dir");
    let handler = SessionFoundationHandler;

    {
        let _env = EnvGuard::set(&[
            (
                "SC_HOOKS_STATE_DIR",
                temp.path().join("state").to_str().expect("state root utf8"),
            ),
            (
                "CLAUDE_PROJECT_DIR",
                project_root.to_str().expect("project root utf8"),
            ),
            ("SC_HOOK_AGENT_PID", "900"),
        ]);
        handler
            .handle(hook_context_with_payload(
                HookType::SessionStart,
                None,
                session_start_payload(session_id, "startup", &project_root),
            ))
            .expect("startup should persist");
    }

    let _env = EnvGuard::set(&[
        (
            "SC_HOOKS_STATE_DIR",
            temp.path().join("state").to_str().expect("state root utf8"),
        ),
        ("SC_HOOK_AGENT_PID", "900"),
    ]);
    let result = handler
        .handle(hook_context_with_payload(
            HookType::Stop,
            None,
            stop_payload(session_id, &drift_dir),
        ))
        .expect("missing project dir should preserve root without error");
    assert!(result.additional_context.is_none());

    let state_file = temp.path().join(format!("state/{session_id}.json"));
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(state_file).expect("state file should exist"))
            .expect("session state should parse");
    assert_eq!(parsed["ai_root_dir"], project_root.to_str().expect("utf8"));
    assert_eq!(parsed["ai_current_dir"], drift_dir.to_str().expect("utf8"));
    assert_eq!(parsed["agent_state"], "idle");
}
