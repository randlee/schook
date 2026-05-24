#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use sc_hooks_test::fixtures;
use serde_json::Value;

fn cli_binary() -> String {
    let current = std::env::current_exe().expect("test binary path should resolve");
    current
        .parent()
        .and_then(|deps| deps.parent())
        .map(|debug_dir| {
            debug_dir.join(if cfg!(windows) {
                "sc-hooks.exe"
            } else {
                "sc-hooks"
            })
        })
        .expect("target/debug directory should resolve")
        .display()
        .to_string()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root should resolve")
        .to_path_buf()
}

fn gemini_fixture(name: &str) -> Value {
    let path = repo_root()
        .join("test-harness/hooks/gemini/fixtures/approved")
        .join(name);
    serde_json::from_str(&fs::read_to_string(path).expect("fixture should read"))
        .expect("fixture should parse")
}

fn write_runtime_config(root: &Path) {
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"[meta]
version = 1

[hooks]
SessionStart = ["agent-session-foundation"]
SessionEnd = ["agent-session-foundation"]
PreToolUse = ["agent-spawn-gates", "atm-extension"]
PostToolUse = ["tool-output-gates"]
"#,
    )
    .expect("config should write");
}

fn write_full_runtime_config(root: &Path) {
    fs::create_dir_all(root.join(".sc-hooks")).expect(".sc-hooks should create");
    fs::write(
        root.join(".sc-hooks/config.toml"),
        r#"[meta]
version = 1

[hooks]
SessionStart = ["agent-session-foundation"]
SessionEnd = ["agent-session-foundation"]
PreToolUse = ["agent-spawn-gates", "atm-extension"]
PostToolUse = ["tool-output-gates"]

[observability]
mode = "full"
"#,
    )
    .expect("config should write");
}

fn write_atm_config(root: &Path) {
    fs::write(
        root.join(".atm.toml"),
        r#"[core]
default_team = "schook"
identity = "chook"
"#,
    )
    .expect(".atm.toml should write");
}

fn plugin_wrapper_body(package: &str) -> String {
    let manifest = repo_root().join("Cargo.toml");
    format!(
        "#!/bin/sh\nexec cargo run --quiet --manifest-path '{}' -p {} -- \"$@\"\n",
        manifest.display(),
        package
    )
}

fn install_plugin_wrapper(root: &Path, plugin_name: &str, package: &str) {
    fixtures::create_executable_script(
        fixtures::plugin_path(root, plugin_name),
        &plugin_wrapper_body(package),
    );
}

fn install_runtime_plugin_set(root: &Path) {
    install_plugin_wrapper(root, "agent-session-foundation", "agent-session-foundation");
    install_plugin_wrapper(root, "agent-spawn-gates", "agent-spawn-gates");
    install_plugin_wrapper(root, "atm-extension", "atm-extension");
    install_plugin_wrapper(root, "tool-output-gates", "tool-output-gates");
}

fn run_gemini_hook(root: &Path, hook: &str, event: Option<&str>, payload: Value) -> Output {
    let mut command = Command::new(cli_binary());
    command
        .current_dir(root)
        .arg("run")
        .arg(hook)
        .arg("--sync")
        .env("SC_HOOKS_STATE_DIR", root.join(".sc-hooks/state"))
        .env("SC_HOOK_AGENT_TYPE", "gemini")
        .env("SC_HOOK_AGENT_PID", "52")
        .env("ATM_TEAM", "schook")
        .env("ATM_IDENTITY", "chook");
    if let Some(event) = event {
        command.arg(event);
    }

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
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn read_full_audit_events(root: &Path) -> Vec<Value> {
    let runs_root = root.join(".sc-hooks/audit/runs");
    let mut entries = fs::read_dir(&runs_root)
        .expect("full audit runs dir should be readable")
        .map(|entry| entry.expect("run entry should be readable").path())
        .collect::<Vec<_>>();
    entries.sort();
    let run_dir = entries.pop().expect("full audit run should exist");
    fs::read_to_string(run_dir.join("events.jsonl"))
        .expect("events should read")
        .lines()
        .map(|line| serde_json::from_str(line).expect("event should parse"))
        .collect()
}

fn write_disabled_session_state(root: &Path, session_id: &str) {
    let state_root = root.join(".sc-hooks/state");
    fs::create_dir_all(&state_root).expect("state root should create");
    fs::write(
        state_root.join("session.json"),
        serde_json::json!({
            "sessions": {
                session_id: {
                    "disabled_plugins": {
                        "notify": {
                            "reason": "runtime-error",
                            "disabled_at": "2026-05-24T00:00:00Z"
                        }
                    }
                }
            }
        })
        .to_string(),
    )
    .expect("session state should write");
}

#[test]
fn gemini_runtime_path_updates_session_state_gate_and_cleanup() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    write_runtime_config(root);
    write_atm_config(root);
    install_runtime_plugin_set(root);

    let session_start_payload = gemini_fixture("session-start-startup.json");
    let session_id = session_start_payload["session_id"]
        .as_str()
        .expect("session id should be present")
        .to_string();
    let project_root = session_start_payload["cwd"]
        .as_str()
        .expect("cwd should be present")
        .to_string();
    let session_start = run_gemini_hook(root, "SessionStart", None, session_start_payload.clone());
    assert_eq!(
        session_start.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );

    let mut before_agent_payload = gemini_fixture("before-agent.json");
    before_agent_payload["session_id"] = Value::String(session_id.clone());
    before_agent_payload["cwd"] = Value::String(project_root.clone());
    let before_agent = run_gemini_hook(root, "PreToolUse", Some("Agent"), before_agent_payload);
    assert_eq!(
        before_agent.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );

    let mut before_tool_payload = gemini_fixture("before-tool.json");
    before_tool_payload["session_id"] = Value::String(session_id.clone());
    before_tool_payload["cwd"] = Value::String(project_root.clone());
    before_tool_payload["tool_input"]["command"] =
        Value::String("atm read --team schook".to_string());
    let before_tool = run_gemini_hook(root, "PreToolUse", Some("Bash"), before_tool_payload);
    assert_eq!(
        before_tool.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );

    let rendered = fs::read_to_string(
        root.join(".sc-hooks/state")
            .join(format!("{session_id}.json")),
    )
    .expect("state file should read");
    let record: Value = serde_json::from_str(&rendered).expect("state should parse");
    assert_eq!(record["provider"], "gemini");
    assert_eq!(record["extensions"]["atm"]["atm_team"], "schook");
    assert_eq!(record["extensions"]["atm"]["atm_identity"], "chook");
    assert_eq!(
        record["extensions"]["spawn_gate"]["last_requested_spawn"]["spawn_kind"],
        "named_agent"
    );

    write_disabled_session_state(root, &session_id);
    let mut session_end_payload = gemini_fixture("session-end.json");
    session_end_payload["session_id"] = Value::String(session_id.clone());
    session_end_payload["cwd"] = Value::String(project_root);
    let session_end = run_gemini_hook(root, "SessionEnd", None, session_end_payload);
    assert_eq!(
        session_end.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );

    let rendered = fs::read_to_string(
        root.join(".sc-hooks/state")
            .join(format!("{session_id}.json")),
    )
    .expect("state file should read after end");
    let ended_record: Value = serde_json::from_str(&rendered).expect("ended state should parse");
    assert_eq!(ended_record["agent_state"], "ended");

    let disabled_state = fs::read_to_string(root.join(".sc-hooks/state/session.json"))
        .expect("disabled session state should exist");
    let disabled_state: Value =
        serde_json::from_str(&disabled_state).expect("disabled state should parse");
    assert!(disabled_state["sessions"].get(&session_id).is_none());
}

#[test]
fn gemini_after_tool_uses_generic_post_tool_path() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    write_runtime_config(root);
    install_runtime_plugin_set(root);

    let mut payload = gemini_fixture("after-tool.json");
    payload["tool_input"]["json_schema"] = serde_json::json!({
        "type": "object",
        "properties": {
            "status": { "type": "string" }
        },
        "required": ["status"],
        "additionalProperties": false
    });
    payload["tool_response"]["llmContent"] =
        Value::String("```json\n{\"status\":\"ok\"}\n```".to_string());
    let output = run_gemini_hook(root, "PostToolUse", Some("Bash"), payload);
    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
}

#[test]
fn gemini_retryable_normalization_failure_surfaces_recovery_hint() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    write_full_runtime_config(root);
    install_runtime_plugin_set(root);

    let output = run_gemini_hook(
        root,
        "PostToolUse",
        Some("Bash"),
        serde_json::json!({
            "hook_event_name": "AfterTool",
            "session_id": "gemini-session-001",
            "cwd": "/synthetic/test/gemini-harness",
            "tool_name": "run_shell_command",
            "tool_input": { "command": "pwd" }
        }),
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::BLOCKED)
    );
    let stderr = stderr_text(&output);
    assert!(stderr.contains("provider runtime normalization failed"));
    assert!(
        stderr.contains(
            "Retry after Gemini emits the complete AfterTool response payload for the approved shell surface."
        )
    );

    let events = read_full_audit_events(root);
    let error = events[0]["error"]
        .as_str()
        .expect("error should be present");
    assert!(error.contains("provider runtime normalization failed"));
    assert_eq!(
        events[0]["recovery_hint"],
        "Retry after Gemini emits the complete AfterTool response payload for the approved shell surface."
    );
}
