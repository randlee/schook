use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use sc_hooks_test::fixtures;

fn cli_binary() -> String {
    let current = std::env::current_exe().expect("test binary path should resolve");
    current
        .parent()
        .and_then(|deps| deps.parent())
        .map(|debug_dir| {
            debug_dir.join(if cfg!(windows) {
                "sc-hooks-cli.exe"
            } else {
                "sc-hooks-cli"
            })
        })
        .expect("target/debug directory should resolve")
        .display()
        .to_string()
}

#[cfg(unix)]
#[test]
fn sync_long_running_without_timeout_override_runs_past_default_timeout() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    let plugin_name = "linger";
    let plugin_path = fixtures::plugin_path(root, plugin_name);
    fixtures::write_minimal_config(root, "PreToolUse", plugin_name);
    fixtures::create_shell_plugin_script(
        &plugin_path,
        r#"{"contract_version":1,"name":"linger","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"long_running":true,"description":"allow long sync work","requires":{}}"#,
        r#"cat >/dev/null
sleep 6
printf '%s\n' '{"action":"proceed"}'
"#,
    );

    let start = Instant::now();
    let output = Command::new(cli_binary())
        .current_dir(root)
        .arg("run")
        .arg("PreToolUse")
        .arg("Write")
        .arg("--sync")
        .stdin(Stdio::null())
        .output()
        .expect("run command should execute");

    assert!(
        start.elapsed() >= Duration::from_millis(5_500),
        "sync long_running should not use the default 5000ms timeout"
    );
    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
}

#[cfg(unix)]
#[test]
fn async_long_running_manifest_is_rejected_by_host_runtime() {
    let temp = tempfile::tempdir().expect("tempdir should create");
    let root = temp.path();
    let plugin_name = "notify";
    let plugin_path = fixtures::plugin_path(root, plugin_name);
    fixtures::write_minimal_config(root, "PostToolUse", plugin_name);
    fixtures::create_shell_plugin_script(
        &plugin_path,
        r#"{"contract_version":1,"name":"notify","mode":"async","hooks":["PostToolUse"],"matchers":["*"],"long_running":true,"description":"wait for remote ack","requires":{}}"#,
        r#"cat >/dev/null
printf '%s\n' '{"action":"proceed"}'
"#,
    );

    let output = Command::new(cli_binary())
        .current_dir(root)
        .arg("run")
        .arg("PostToolUse")
        .arg("Write")
        .arg("--async")
        .output()
        .expect("run command should execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::RESOLUTION_ERROR)
    );
    assert!(stderr.contains("manifest load failed"));
    assert!(stderr.contains("notify"));

    let handlers = Command::new(cli_binary())
        .current_dir(root)
        .arg("handlers")
        .output()
        .expect("handlers command should execute");
    let handlers_stdout = String::from_utf8_lossy(&handlers.stdout);
    assert!(handlers_stdout.contains("manifest_error"));
}
