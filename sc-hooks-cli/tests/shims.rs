use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write_mock_sc_hooks(path: &Path) {
    let script = r#"#!/usr/bin/env bash
set -eu
printf "%s|%s|%s|%s\n" \
  "${SC_HOOK_AGENT_TYPE:-}" \
  "${SC_HOOK_SESSION_ID:-}" \
  "${SC_HOOK_AGENT_PID:-}" \
  "$*" > "${SHIM_TEST_OUTPUT}"
"#;
    fs::write(path, script).expect("mock sc-hooks should be writable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .expect("mock script metadata should be readable")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("mock script should be executable");
    }
}

fn shim_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("shims")
        .join(name)
}

fn prepend_path(dir: &Path) -> std::ffi::OsString {
    let mut paths = vec![dir.to_path_buf()];
    if let Some(original) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&original));
    }
    std::env::join_paths(paths).expect("joined PATH should be valid")
}

#[cfg(unix)]
#[test]
fn codex_shim_exports_contract_and_maps_pre_edit() {
    let temp = tempfile::tempdir().expect("tempdir should be creatable");
    let mock = temp.path().join("sc-hooks");
    write_mock_sc_hooks(&mock);
    let output = temp.path().join("out.txt");

    let test_path = prepend_path(temp.path());
    let status = Command::new(shim_path("codex-shim.sh"))
        .arg("pre-edit")
        .env("CODEX_SESSION_ID", "session-123")
        .env("SHIM_TEST_OUTPUT", &output)
        .env("PATH", test_path)
        .status()
        .expect("shim should execute");
    assert!(status.success());

    let captured = fs::read_to_string(output).expect("shim output should be captured");
    assert!(captured.contains("codex|session-123|"));
    assert!(captured.contains("run PreToolUse Write"));
}

#[cfg(unix)]
#[test]
fn gemini_shim_exports_contract_and_maps_pre_tool() {
    let temp = tempfile::tempdir().expect("tempdir should be creatable");
    let mock = temp.path().join("sc-hooks");
    write_mock_sc_hooks(&mock);
    let output = temp.path().join("out.txt");

    let test_path = prepend_path(temp.path());
    let status = Command::new(shim_path("gemini-shim.sh"))
        .arg("pre-tool")
        .env("GEMINI_SESSION_ID", "g-session-9")
        .env("SHIM_TEST_OUTPUT", &output)
        .env("PATH", test_path)
        .status()
        .expect("shim should execute");
    assert!(status.success());

    let captured = fs::read_to_string(output).expect("shim output should be captured");
    assert!(captured.contains("gemini|g-session-9|"));
    assert!(captured.contains("run PreToolUse Write"));
}
