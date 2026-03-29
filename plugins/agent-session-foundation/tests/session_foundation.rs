use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use agent_session_foundation::SessionFoundationHandler;
use sc_hooks_core::context::HookContext;
use sc_hooks_core::events::HookType;
use sc_hooks_sdk::traits::SyncHandler;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-harness/hooks/claude/fixtures/approved")
        .join(name)
}

fn load_fixture(name: &str) -> serde_json::Value {
    let path = fixture_path(name);
    let body = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("failed reading fixture {}: {err}", path.display());
    });
    serde_json::from_str(&body).expect("fixture json should parse")
}

fn hook_context(hook: HookType, event: Option<&str>, fixture: &str) -> HookContext {
    hook_context_with_payload(hook, event, load_fixture(fixture))
}

fn hook_context_with_payload(
    hook: HookType,
    event: Option<&str>,
    payload: serde_json::Value,
) -> HookContext {
    HookContext::new(
        hook,
        event.map(str::to_string),
        serde_json::json!({
            "hook": { "type": hook.as_str(), "event": event },
            "payload": payload
        }),
        None,
    )
}

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    entries: Vec<(String, Option<String>)>,
}

impl EnvGuard {
    fn set(pairs: &[(&str, &str)]) -> Self {
        let mut entries = Vec::new();
        for (key, value) in pairs {
            entries.push(((*key).to_string(), std::env::var(key).ok()));
            unsafe { std::env::set_var(key, value) };
        }
        Self { entries }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, original) in self.entries.drain(..) {
            match original {
                Some(value) => unsafe { std::env::set_var(&key, value) },
                None => unsafe { std::env::remove_var(&key) },
            }
        }
    }
}

struct CurrentDirGuard {
    original: PathBuf,
}

impl CurrentDirGuard {
    fn set(path: &Path) -> Self {
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(path).expect("cwd should change");
        Self { original }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original).expect("cwd should restore");
    }
}

#[test]
fn persists_session_record_by_session_id() {
    let _lock = test_lock().lock().expect("test lock");
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
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
        ("SC_HOOK_AGENT_PID", "777"),
    ]);
    let handler = SessionFoundationHandler;

    handler
        .handle(hook_context(
            HookType::SessionStart,
            None,
            "session-start-startup.json",
        ))
        .expect("session start should persist");

    let state_file = temp
        .path()
        .join("state/a760f75c-055a-46f9-bcbb-447c47a22f3c.json");
    let rendered = fs::read_to_string(state_file).expect("state file should exist");
    let parsed: serde_json::Value =
        serde_json::from_str(&rendered).expect("session state should parse");
    assert_eq!(parsed["session_id"], "a760f75c-055a-46f9-bcbb-447c47a22f3c");
    assert_eq!(parsed["active_pid"], 777);
    assert_eq!(parsed["ai_root_dir"], project_root.to_str().expect("utf8"));
    assert_eq!(parsed["ai_current_dir"], "/synthetic/test/session-start");
    assert_eq!(parsed["agent_state"], "starting");
}

#[test]
fn later_lifecycle_events_correlate_across_directory_changes() {
    let _lock = test_lock().lock().expect("test lock");
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
    let other_dir = temp.path().join("repo-b");
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
            .handle(hook_context(
                HookType::SessionStart,
                None,
                "session-start-startup.json",
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
        let _cwd = CurrentDirGuard::set(&other_dir);
        let mut payload = load_fixture("stop.json");
        payload["session_id"] =
            serde_json::Value::String("a760f75c-055a-46f9-bcbb-447c47a22f3c".to_string());
        payload["cwd"] = serde_json::Value::String(other_dir.to_str().expect("utf8").to_string());
        handler
            .handle(hook_context_with_payload(HookType::Stop, None, payload))
            .expect("stop should update existing session");
    }

    let state_file = temp
        .path()
        .join("state/a760f75c-055a-46f9-bcbb-447c47a22f3c.json");
    let rendered = fs::read_to_string(state_file).expect("state file should exist");
    let parsed: serde_json::Value =
        serde_json::from_str(&rendered).expect("session state should parse");
    assert_eq!(parsed["ai_root_dir"], project_root.to_str().expect("utf8"));
    assert_eq!(parsed["ai_current_dir"], other_dir.to_str().expect("utf8"));
    assert_eq!(parsed["agent_state"], "idle");
}

#[test]
fn emits_hook_log_for_state_changes_and_noop_writes() {
    let _lock = test_lock().lock().expect("test lock");
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
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
        .handle(hook_context(
            HookType::SessionStart,
            None,
            "session-start-startup.json",
        ))
        .expect("session start should persist");
    handler
        .handle(hook_context(
            HookType::SessionStart,
            None,
            "session-start-startup.json",
        ))
        .expect("second identical invocation should be allowed");

    let log_path = project_root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH);
    let rendered = fs::read_to_string(log_path).expect("hook log should exist");
    let lines: Vec<_> = rendered.lines().collect();
    assert_eq!(lines.len(), 2);
    let first: serde_json::Value =
        serde_json::from_str(lines[0]).expect("first log line should parse");
    let second: serde_json::Value =
        serde_json::from_str(lines[1]).expect("second log line should parse");
    assert_eq!(first["fields"]["persist_outcome"], "created");
    assert_eq!(second["fields"]["persist_outcome"], "unchanged");
}

#[test]
fn session_start_requires_injected_agent_pid() {
    let _lock = test_lock().lock().expect("test lock");
    let temp = tempfile::tempdir().expect("tempdir");
    let project_root = temp.path().join("repo-a");
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
        .handle(hook_context(
            HookType::SessionStart,
            None,
            "session-start-startup.json",
        ))
        .expect_err("session start should fail without injected agent pid");
    assert!(err.to_string().contains("SC_HOOK_AGENT_PID"));
}
