use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn plugin_binary() -> &'static str {
    env!("CARGO_BIN_EXE_atm-session-lifecycle")
}

#[test]
fn session_start_and_end_survive_directory_change() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let dir_a = temp.path().join("dir-a");
    let dir_b = temp.path().join("dir-b");
    let store_root = temp.path().join("store");
    fs::create_dir_all(&dir_a).expect("dir-a should be creatable");
    fs::create_dir_all(&dir_b).expect("dir-b should be creatable");

    run_plugin(
        &dir_a,
        &store_root,
        Some("atm-dev"),
        Some("arch-hook"),
        serde_json::json!({
            "hook": {"type": "SessionStart"},
            "agent": {"pid": 4242},
            "payload": {"session_id": "session-a", "source": "init"}
        }),
    );

    let record_path = store_root.join("session-a.json");
    assert!(
        record_path.exists(),
        "SessionStart should persist the record"
    );

    let stored: Value =
        serde_json::from_str(&fs::read_to_string(&record_path).expect("record should be readable"))
            .expect("record should parse");
    assert_eq!(stored["session_id"], "session-a");
    assert_eq!(stored["team"], "atm-dev");
    assert_eq!(stored["identity"], "arch-hook");
    assert_eq!(stored["pid"], 4242);

    run_plugin(
        &dir_b,
        &store_root,
        None,
        None,
        serde_json::json!({
            "hook": {"type": "SessionEnd"},
            "payload": {"session_id": "session-a"}
        }),
    );

    assert!(
        !record_path.exists(),
        "SessionEnd should delete the persisted record even from a different cwd"
    );
}

#[test]
fn session_start_preserves_created_at_when_re_fired() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let store_root = temp.path().join("store");

    run_plugin(
        temp.path(),
        &store_root,
        Some("atm-dev"),
        Some("arch-hook"),
        serde_json::json!({
            "hook": {"type": "SessionStart"},
            "agent": {"pid": 1111},
            "payload": {"session_id": "session-b", "source": "init"}
        }),
    );

    let record_path = store_root.join("session-b.json");
    let first: Value =
        serde_json::from_str(&fs::read_to_string(&record_path).expect("first record should exist"))
            .expect("first record should parse");

    run_plugin(
        temp.path(),
        &store_root,
        Some("atm-dev"),
        Some("arch-hook"),
        serde_json::json!({
            "hook": {"type": "SessionStart"},
            "agent": {"pid": 2222},
            "payload": {"session_id": "session-b", "source": "compact"}
        }),
    );

    let second: Value = serde_json::from_str(
        &fs::read_to_string(&record_path).expect("second record should exist"),
    )
    .expect("second record should parse");

    assert_eq!(first["created_at"], second["created_at"]);
    assert_eq!(second["pid"], 2222);
}

fn run_plugin(
    cwd: &Path,
    store_root: &Path,
    atm_team: Option<&str>,
    atm_identity: Option<&str>,
    payload: Value,
) {
    let mut command = Command::new(plugin_binary());
    command
        .current_dir(cwd)
        .env("SC_HOOK_SESSION_STORE_DIR", store_root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if let Some(team) = atm_team {
        command.env("ATM_TEAM", team);
    } else {
        command.env_remove("ATM_TEAM");
    }
    if let Some(identity) = atm_identity {
        command.env("ATM_IDENTITY", identity);
    } else {
        command.env_remove("ATM_IDENTITY");
    }

    let mut child = command.spawn().expect("plugin process should spawn");
    {
        use std::io::Write;
        let body = serde_json::to_vec(&payload).expect("payload should serialize");
        child
            .stdin
            .take()
            .expect("stdin should be piped")
            .write_all(&body)
            .expect("stdin should accept payload");
    }

    let output = child.wait_with_output().expect("plugin should exit");
    if !output.status.success() {
        panic!(
            "plugin failed: status={:?} stdout={} stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let rendered = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let parsed: Value = serde_json::from_str(&rendered).expect("plugin should emit json");
    assert_eq!(parsed["action"], "proceed");
}
