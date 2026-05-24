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

fn codex_fixture(name: &str) -> Value {
    let path = repo_root()
        .join("test-harness/hooks/codex/fixtures/approved")
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
PreToolUse = ["atm-extension"]
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
PreToolUse = ["atm-extension"]

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

fn run_codex_hook(root: &Path, hook: &str, event: Option<&str>, payload: Value) -> Output {
    let mut command = Command::new(cli_binary());
    command
        .current_dir(root)
        .arg("run")
        .arg(hook)
        .arg("--sync")
        .env("SC_HOOKS_STATE_DIR", root.join(".sc-hooks/state"))
        .env("SC_HOOK_AGENT_TYPE", "codex")
        .env("SC_HOOK_AGENT_PID", "42")
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

#[test]
fn codex_runtime_path_updates_session_state_and_atm_extension() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    write_runtime_config(root);
    write_atm_config(root);
    install_plugin_wrapper(root, "agent-session-foundation", "agent-session-foundation");
    install_plugin_wrapper(root, "atm-extension", "atm-extension");

    let session_start_payload = codex_fixture("session-start-startup.json");
    let session_id = session_start_payload["session_id"]
        .as_str()
        .expect("session id should be present")
        .to_string();
    let session_start = run_codex_hook(root, "SessionStart", None, session_start_payload);
    assert_eq!(
        session_start.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );

    let pre_tool_use = run_codex_hook(
        root,
        "PreToolUse",
        Some("Bash"),
        codex_fixture("pretooluse-bash.json"),
    );
    assert_eq!(
        pre_tool_use.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );

    let rendered = fs::read_to_string(
        root.join(".sc-hooks/state")
            .join(format!("{session_id}.json")),
    )
    .expect("state file should read");
    let record: Value = serde_json::from_str(&rendered).expect("state should parse");
    assert_eq!(record["provider"], "codex");
    assert_eq!(record["extensions"]["atm"]["atm_team"], "schook");
    assert_eq!(record["extensions"]["atm"]["atm_identity"], "chook");
}

#[test]
fn codex_retryable_normalization_failure_surfaces_recovery_hint() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    write_full_runtime_config(root);
    install_plugin_wrapper(root, "atm-extension", "atm-extension");

    let output = run_codex_hook(
        root,
        "PreToolUse",
        Some("Bash"),
        serde_json::json!({
            "hook_event_name": "PreToolUse",
            "session_id": "codex-session-001",
            "cwd": "/synthetic/test/codex-harness",
            "tool_name": "Bash"
        }),
    );

    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::BLOCKED)
    );
    let stderr = stderr_text(&output);
    assert!(stderr.contains("provider runtime normalization failed"));
    assert!(
        stderr
            .contains("Retry after Codex emits the complete PreToolUse payload for the Bash tool.")
    );

    let events = read_full_audit_events(root);
    let error = events[0]["error"]
        .as_str()
        .expect("error should be present");
    assert!(error.contains("provider runtime normalization failed"));
    assert!(
        error
            .contains("Retry after Codex emits the complete PreToolUse payload for the Bash tool.")
    );
}
