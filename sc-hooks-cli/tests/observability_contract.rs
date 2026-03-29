use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use sc_hooks_test::fixtures;
use serde_json::Value;

struct DispatchHarness {
    binary: String,
}

impl DispatchHarness {
    fn new() -> Self {
        let current = std::env::current_exe().expect("test binary path should resolve");
        let binary = current
            .parent()
            .and_then(|deps| deps.parent())
            .map(|debug_dir| {
                debug_dir.join(if cfg!(windows) {
                    "sc-hooks-cli.exe"
                } else {
                    "sc-hooks-cli"
                })
            })
            .expect("target/debug directory should resolve");
        Self {
            binary: binary.display().to_string(),
        }
    }

    fn run_sync(
        &self,
        root: &Path,
        hook: &str,
        event: Option<&str>,
        payload: Option<Value>,
        session_id: Option<&str>,
    ) -> Output {
        let mut command = Command::new(&self.binary);
        command.current_dir(root).arg("run").arg(hook).arg("--sync");
        if let Some(event) = event {
            command.arg(event);
        }
        if let Some(session_id) = session_id {
            command.env("SC_HOOK_SESSION_ID", session_id);
        }

        if let Some(payload) = payload {
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("dispatch child should spawn");
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let body = serde_json::to_vec(&payload).expect("payload should serialize");
                stdin
                    .write_all(&body)
                    .expect("payload should write to stdin");
            }
            child
                .wait_with_output()
                .expect("dispatch output should be readable")
        } else {
            command.output().expect("dispatch command should execute")
        }
    }
}

fn read_last_log(root: &Path) -> Value {
    let rendered = fs::read_to_string(root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH))
        .expect("observability log should be readable");
    let line = rendered.lines().last().expect("log line should exist");
    serde_json::from_str(line).expect("log line should parse")
}

fn session_disables_plugin_for_reason(root: &Path, plugin: &str, reason: &str) -> bool {
    let state_path = root.join(".sc-hooks/state/session.json");
    let rendered = fs::read_to_string(state_path).expect("session state should exist");
    let parsed: Value = serde_json::from_str(&rendered).expect("session state should parse");
    parsed["sessions"].as_object().is_some_and(|sessions| {
        sessions
            .values()
            .any(|session| session["disabled_plugins"][plugin]["reason"].as_str() == Some(reason))
    })
}

#[test]
fn success_dispatch_emits_file_sink_log_event() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"proceed"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
    let log = read_last_log(root);
    assert_eq!(log["service"], "sc-hooks");
    assert_eq!(log["target"], "hook");
    assert_eq!(log["action"], "dispatch.complete");
    assert_eq!(log["outcome"], "proceed");
    assert_eq!(log["level"], "Info");
    assert_eq!(log["fields"]["hook"], "PreToolUse");
    assert_eq!(log["fields"]["event"], "Write");
    assert_eq!(log["fields"]["matcher"], "Write");
    assert_eq!(log["fields"]["mode"], "sync");
    assert_eq!(
        log["fields"]["handlers"],
        serde_json::json!(["probe-plugin"])
    );
    assert!(log["fields"]["total_ms"].as_u64().is_some());
    assert_eq!(log["fields"]["exit"], sc_hooks_core::exit_codes::SUCCESS);
    assert_eq!(log["fields"]["results"][0]["action"], "proceed");
    assert!(root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH).exists());
}

#[test]
fn blocked_dispatch_emits_warn_log_event() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        r#"{"action":"block","reason":"blocked by policy"}"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::BLOCKED)
    );
    let log = read_last_log(root);
    assert_eq!(log["outcome"], "block");
    assert_eq!(log["level"], "Warn");
    assert_eq!(log["fields"]["mode"], "sync");
    assert_eq!(
        log["fields"]["handlers"],
        serde_json::json!(["probe-plugin"])
    );
    assert!(log["fields"]["total_ms"].as_u64().is_some());
    assert_eq!(log["fields"]["exit"], sc_hooks_core::exit_codes::BLOCKED);
    let fields = log["fields"]
        .as_object()
        .expect("fields should be an object");
    let result = log["fields"]["results"][0]
        .as_object()
        .expect("result should be an object");
    assert_eq!(result["action"], "block");
    assert!(!result.contains_key("disabled"));
    assert!(!fields.contains_key("ai_notification"));
}

#[test]
fn invalid_json_dispatch_emits_error_log_and_disables_plugin() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin_script(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        "cat >/dev/null\nprintf '%s' 'not-json'\n",
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        Some("session-invalid-json"),
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::PLUGIN_ERROR)
    );
    let log = read_last_log(root);
    assert_eq!(log["outcome"], "error");
    assert_eq!(log["level"], "Error");
    assert_eq!(
        log["fields"]["exit"],
        sc_hooks_core::exit_codes::PLUGIN_ERROR
    );
    assert_eq!(log["fields"]["results"][0]["action"], "error");
    assert_eq!(log["fields"]["results"][0]["error_type"], "invalid_json");
    assert_eq!(log["fields"]["results"][0]["disabled"], true);
    assert!(
        log["fields"]["ai_notification"]
            .as_str()
            .is_some_and(|message| message.contains("invalid JSON"))
    );
    assert!(session_disables_plugin_for_reason(
        root,
        "probe-plugin",
        "runtime-error"
    ));
}

#[test]
fn timeout_dispatch_emits_error_log_and_disables_plugin() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    fixtures::write_minimal_config(root, "PreToolUse", "probe-plugin");
    fixtures::create_shell_plugin_script(
        &fixtures::plugin_path(root, "probe-plugin"),
        r#"{"contract_version":1,"name":"probe-plugin","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"timeout_ms":50,"requires":{}}"#,
        r#"cat >/dev/null
sleep 1
printf '%s\n' '{"action":"proceed"}'
"#,
    );

    let output = DispatchHarness::new().run_sync(
        root,
        "PreToolUse",
        Some("Write"),
        Some(serde_json::json!({"tool_input": {"command": "echo hi"}})),
        Some("session-timeout"),
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::TIMEOUT)
    );
    let log = read_last_log(root);
    assert_eq!(log["outcome"], "error");
    assert_eq!(log["level"], "Error");
    assert_eq!(log["fields"]["exit"], sc_hooks_core::exit_codes::TIMEOUT);
    assert_eq!(log["fields"]["results"][0]["action"], "error");
    assert_eq!(log["fields"]["results"][0]["error_type"], "timeout");
    assert_eq!(log["fields"]["results"][0]["disabled"], true);
    assert!(
        log["fields"]["ai_notification"]
            .as_str()
            .is_some_and(|message| message.contains("timed out after 50ms"))
    );
    assert!(session_disables_plugin_for_reason(
        root,
        "probe-plugin",
        "runtime-error"
    ));
}
