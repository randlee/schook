use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::Command;
use std::thread;

use serde_json::Value;

fn plugin_binary() -> &'static str {
    env!("CARGO_BIN_EXE_atm-state-relay")
}

#[test]
fn relays_notification_idle_prompt_using_session_record_context() {
    let request = run_and_capture(
        serde_json::json!({
            "hook": {"type": "Notification", "event": "idle_prompt"},
            "payload": {"session_id": "session-a"}
        }),
        "session-a",
        Some("atm-dev"),
        Some("arch-hook"),
    );

    assert_eq!(request["command"], "hook-event");
    assert_eq!(request["payload"]["event"], "notification_idle_prompt");
    assert_eq!(request["payload"]["team"], "atm-dev");
    assert_eq!(request["payload"]["agent"], "arch-hook");
    assert_eq!(request["payload"]["session_id"], "session-a");
}

#[test]
fn relays_permission_request_with_tool_name() {
    let request = run_and_capture(
        serde_json::json!({
            "hook": {"type": "PermissionRequest"},
            "payload": {
                "session_id": "session-b",
                "tool_input": {"name": "Bash"}
            }
        }),
        "session-b",
        Some("atm-dev"),
        Some("arch-hook"),
    );

    assert_eq!(request["payload"]["event"], "permission_request");
    assert_eq!(request["payload"]["tool_name"], "Bash");
}

#[test]
fn relays_stop_event() {
    let request = run_and_capture(
        serde_json::json!({
            "hook": {"type": "Stop"},
            "payload": {"session_id": "session-c"}
        }),
        "session-c",
        Some("atm-dev"),
        Some("arch-hook"),
    );

    assert_eq!(request["payload"]["event"], "stop");
    assert_eq!(request["payload"]["team"], "atm-dev");
    assert_eq!(request["payload"]["agent"], "arch-hook");
}

fn run_and_capture(
    payload: Value,
    session_id: &str,
    team: Option<&str>,
    identity: Option<&str>,
) -> Value {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let home = temp.path().join("home");
    let daemon_dir = home.join(".atm").join("daemon");
    let store_root = temp.path().join("store");
    fs::create_dir_all(&daemon_dir).expect("daemon dir should create");
    fs::create_dir_all(&store_root).expect("store dir should create");

    let socket_path = daemon_dir.join("atm-daemon.sock");
    let listener = UnixListener::bind(&socket_path).expect("unix socket should bind");
    write_session_record(&store_root, session_id, team, identity);

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("daemon should accept");
        let mut reader = BufReader::new(stream.try_clone().expect("stream should clone"));
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("daemon should read request line");
        let _ = stream.write_all(b"{\"ok\":true}\n");
        let text = line;
        serde_json::from_str::<Value>(&text).expect("request should parse")
    });

    run_plugin(temp.path(), &home, &store_root, payload);
    handle.join().expect("relay thread should join")
}

fn write_session_record(
    store_root: &Path,
    session_id: &str,
    team: Option<&str>,
    identity: Option<&str>,
) {
    let record = serde_json::json!({
        "session_id": session_id,
        "team": team,
        "identity": identity,
        "created_at": 1.0,
        "updated_at": 1.0
    });
    let path = store_root.join(format!("{session_id}.json"));
    fs::write(
        path,
        serde_json::to_vec(&record).expect("record should serialize"),
    )
    .expect("record should write");
}

fn run_plugin(cwd: &Path, home: &Path, store_root: &Path, payload: Value) {
    let mut child = Command::new(plugin_binary())
        .current_dir(cwd)
        .env("ATM_HOME", home)
        .env("SC_HOOK_SESSION_STORE_DIR", store_root)
        .env_remove("ATM_TEAM")
        .env_remove("ATM_IDENTITY")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("plugin should spawn");

    let body = serde_json::to_vec(&payload).expect("payload should serialize");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(&body)
        .expect("stdin should accept body");

    let output = child.wait_with_output().expect("plugin should finish");
    assert!(
        output.status.success(),
        "plugin failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let parsed: Value = serde_json::from_str(&rendered).expect("plugin should emit json");
    assert_eq!(parsed["action"], "proceed");
}
