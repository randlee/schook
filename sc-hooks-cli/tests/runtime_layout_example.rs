use std::path::PathBuf;
use std::process::{Command, Stdio};

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

fn example_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root should resolve")
        .join("examples/runtime-layout")
}

#[test]
fn checked_runtime_layout_audits_and_runs_successfully() {
    let root = example_root();

    let audit = Command::new(cli_binary())
        .current_dir(&root)
        .arg("audit")
        .output()
        .expect("audit should execute");
    assert_eq!(
        audit.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );

    let mut run = Command::new(cli_binary())
        .current_dir(&root)
        .arg("run")
        .arg("PreToolUse")
        .arg("Write")
        .arg("--sync")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run should execute");
    if let Some(mut stdin) = run.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(br#"{"tool_input":{"command":"echo hi"}}"#)
            .expect("stdin payload should write");
    }
    let output = run
        .wait_with_output()
        .expect("run output should be collected");
    assert_eq!(
        output.status.code(),
        Some(sc_hooks_core::exit_codes::SUCCESS)
    );
}
