use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn plugin_binary() -> &'static str {
    env!("CARGO_BIN_EXE_atm-bash-identity")
}

#[test]
fn pre_and_post_tool_use_write_and_remove_identity_file() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let store_root = temp.path().join("sessions");
    let hook_root = temp.path().join("hook-files");
    let key = "turn-1";

    write_session_record(&store_root, "session-1", "atm-dev", "arch-hook");

    run_plugin(
        temp.path(),
        &store_root,
        &hook_root,
        serde_json::json!({
            "hook": {"type": "PreToolUse"},
            "payload": {
                "session_id": "session-1",
                "turn_id": key,
                "tool_input": {"command": "atm read --team atm-dev"}
            }
        }),
    );

    let hook_file = hook_root.join("atm-hook-turn-1.json");
    assert!(
        hook_file.exists(),
        "pre-hook should create the identity file"
    );
    let content: Value = serde_json::from_str(
        &fs::read_to_string(&hook_file).expect("hook file should be readable"),
    )
    .expect("hook file should parse");
    assert_eq!(content["session_id"], "session-1");
    assert_eq!(content["agent_name"], "arch-hook");
    assert_eq!(content["team_name"], "atm-dev");

    run_plugin(
        temp.path(),
        &store_root,
        &hook_root,
        serde_json::json!({
            "hook": {"type": "PostToolUse"},
            "payload": {
                "session_id": "session-1",
                "turn_id": key
            }
        }),
    );

    assert!(
        !hook_file.exists(),
        "post-hook should delete the paired identity file"
    );
}

#[test]
fn non_atm_command_is_ignored() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let store_root = temp.path().join("sessions");
    let hook_root = temp.path().join("hook-files");

    write_session_record(&store_root, "session-1", "atm-dev", "arch-hook");

    run_plugin(
        temp.path(),
        &store_root,
        &hook_root,
        serde_json::json!({
            "hook": {"type": "PreToolUse"},
            "payload": {
                "session_id": "session-1",
                "turn_id": "turn-2",
                "tool_input": {"command": "echo hello"}
            }
        }),
    );

    assert!(
        !hook_root.join("atm-hook-turn-2.json").exists(),
        "non-atm commands should not create identity files"
    );
}

fn run_plugin(cwd: &Path, store_root: &Path, hook_root: &Path, payload: Value) {
    let mut child = Command::new(plugin_binary())
        .current_dir(cwd)
        .env("SC_HOOK_SESSION_STORE_DIR", store_root)
        .env("SC_HOOK_ATM_IDENTITY_DIR", hook_root)
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
    let rendered = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let parsed: Value = serde_json::from_str(&rendered).expect("stdout should be json");
    assert_eq!(parsed["action"], "proceed");
}

fn write_session_record(store_root: &Path, session_id: &str, team: &str, identity: &str) {
    fs::create_dir_all(store_root).expect("store root should be creatable");
    fs::write(
        store_root.join(format!("{session_id}.json")),
        serde_json::json!({
            "session_id": session_id,
            "team": team,
            "identity": identity,
            "pid": 4242,
            "created_at": 1.0,
            "updated_at": 1.0
        })
        .to_string(),
    )
    .expect("session record should be writable");
}
