use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn plugin_binary() -> &'static str {
    env!("CARGO_BIN_EXE_gate-agent-spawns")
}

#[test]
fn blocks_background_spawn_when_agent_requires_named_teammate() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    write_session_record(temp.path(), "session-1", "atm-dev", "team-lead");
    write_atm_toml(
        temp.path(),
        r#"[core]
default_team = "atm-dev"
identity = "team-lead"
"#,
    );
    write_agent_file(
        temp.path(),
        "planner",
        r#"---
metadata:
  spawn_policy: named_teammate_required
---
"#,
    );

    let result = run_plugin(
        temp.path(),
        temp.path(),
        serde_json::json!({
            "hook": {"type": "PreToolUse"},
            "payload": {
                "session_id": "session-1",
                "tool_input": {"subagent_type": "planner"}
            }
        }),
    );

    assert_eq!(result["action"], "block");
}

#[test]
fn blocks_team_name_mismatch() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    write_session_record(temp.path(), "session-1", "atm-dev", "team-lead");
    write_atm_toml(
        temp.path(),
        r#"[core]
default_team = "atm-dev"
identity = "team-lead"
"#,
    );

    let result = run_plugin(
        temp.path(),
        temp.path(),
        serde_json::json!({
            "hook": {"type": "PreToolUse"},
            "payload": {
                "session_id": "session-1",
                "tool_input": {"subagent_type": "planner", "name": "worker-1", "team_name": "other-team"}
            }
        }),
    );

    assert_eq!(result["action"], "block");
}

#[test]
fn blocks_named_spawn_for_non_lead_member_under_leaders_only_policy() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    write_session_record(temp.path(), "member-session", "atm-dev", "arch-dev");
    write_atm_toml(
        temp.path(),
        r#"[core]
default_team = "atm-dev"

[team."atm-dev"]
spawn_policy = "leaders-only"
co_leaders = ["arch-hook"]
"#,
    );
    write_team_config(
        temp.path(),
        "atm-dev",
        serde_json::json!({
            "leadSessionId": "lead-session",
            "members": [{"name": "arch-dev", "sessionId": "member-session"}]
        }),
    );

    let result = run_plugin(
        temp.path(),
        temp.path(),
        serde_json::json!({
            "hook": {"type": "PreToolUse"},
            "payload": {
                "session_id": "member-session",
                "tool_input": {"subagent_type": "planner", "name": "worker-1", "team_name": "atm-dev"}
            }
        }),
    );

    assert_eq!(result["action"], "block");
}

#[test]
fn allows_named_spawn_for_team_lead() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    write_session_record(temp.path(), "lead-session", "atm-dev", "team-lead");
    write_atm_toml(
        temp.path(),
        r#"[core]
default_team = "atm-dev"

[team."atm-dev"]
spawn_policy = "leaders-only"
"#,
    );
    write_team_config(
        temp.path(),
        "atm-dev",
        serde_json::json!({
            "leadSessionId": "lead-session",
            "members": []
        }),
    );

    let result = run_plugin(
        temp.path(),
        temp.path(),
        serde_json::json!({
            "hook": {"type": "PreToolUse"},
            "payload": {
                "session_id": "lead-session",
                "tool_input": {"subagent_type": "planner", "name": "worker-1", "team_name": "atm-dev"}
            }
        }),
    );

    assert_eq!(result["action"], "proceed");
}

fn run_plugin(cwd: &Path, store_root: &Path, payload: Value) -> Value {
    let mut child = Command::new(plugin_binary())
        .current_dir(cwd)
        .env("SC_HOOK_SESSION_STORE_DIR", store_root.join("sessions"))
        .env("ATM_HOME", store_root)
        .env_remove("ATM_IDENTITY")
        .env_remove("ATM_TEAM")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("plugin should spawn");

    {
        use std::io::Write;
        child
            .stdin
            .take()
            .expect("stdin should be piped")
            .write_all(&serde_json::to_vec(&payload).expect("payload should serialize"))
            .expect("stdin should accept payload");
    }

    let output = child.wait_with_output().expect("plugin should exit");
    assert!(output.status.success(), "plugin should succeed");
    serde_json::from_slice(&output.stdout).expect("plugin should emit JSON")
}

fn write_session_record(root: &Path, session_id: &str, team: &str, identity: &str) {
    let sessions_dir = root.join("sessions");
    fs::create_dir_all(&sessions_dir).expect("sessions dir should be creatable");
    fs::write(
        sessions_dir.join(format!("{session_id}.json")),
        serde_json::json!({
            "session_id": session_id,
            "team": team,
            "identity": identity,
            "pid": 1234,
            "created_at": 1.0,
            "updated_at": 1.0
        })
        .to_string(),
    )
    .expect("session record should be writable");
}

fn write_atm_toml(root: &Path, content: &str) {
    fs::write(root.join(".atm.toml"), content).expect(".atm.toml should be writable");
}

fn write_agent_file(root: &Path, agent_type: &str, content: &str) {
    let path = root
        .join(".claude")
        .join("agents")
        .join(format!("{agent_type}.md"));
    fs::create_dir_all(path.parent().expect("agent path should have a parent"))
        .expect("agents dir should be creatable");
    fs::write(path, content).expect("agent file should be writable");
}

fn write_team_config(root: &Path, team: &str, content: Value) {
    let path = root
        .join(".claude")
        .join("teams")
        .join(team)
        .join("config.json");
    fs::create_dir_all(path.parent().expect("team config should have a parent"))
        .expect("team config dir should be creatable");
    fs::write(path, content.to_string()).expect("team config should be writable");
}
