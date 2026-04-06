#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sc_hooks_core::errors::HookError;
use serde_json::json;

use crate::fixtures;

struct HookOutcome {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn run_command_hook(
    script: &Path,
    input: serde_json::Value,
    extra_env: &[(&str, &Path)],
) -> HookOutcome {
    let mut command = Command::new(script);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in extra_env {
        command.env(key, value);
    }

    let mut child = {
        let mut last_err = None;
        let mut result = None;
        for _ in 0..3 {
            match command.spawn() {
                Ok(child) => {
                    result = Some(child);
                    break;
                }
                Err(err) if err.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                    last_err = Some(err);
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(err) => panic!("hook script should spawn: {err}"),
            }
        }
        result.unwrap_or_else(|| panic!("hook script should spawn: {}", last_err.unwrap()))
    };
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let body = serde_json::to_vec(&input).expect("hook input should serialize");
        stdin
            .write_all(&body)
            .expect("hook stdin should be writable");
    }
    let output = child.wait_with_output().expect("hook should complete");

    HookOutcome {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    }
}

fn parse_worktree_path(stdout: &str) -> Result<PathBuf, HookError> {
    let path = PathBuf::from(stdout.trim());
    if stdout.trim().is_empty() {
        return Err(HookError::validation(
            "stdout",
            "missing worktree path on stdout",
        ));
    }
    if !path.is_absolute() {
        return Err(HookError::validation(
            "stdout",
            format!(
                "worktree hooks require an absolute stdout path, got `{}`",
                stdout.trim()
            ),
        ));
    }
    Ok(path)
}

#[test]
fn worktree_create_returns_absolute_path_for_authorized_folder() {
    let temp = tempfile::tempdir().expect("tempdir should exist");
    let root = temp.path();
    let script = root.join("worktree-create.sh");
    let allowed_root = root.join("allowed-root");
    let requested_cwd = allowed_root.join("repo");
    let transcript_path = root.join("transcript.jsonl");
    std::fs::create_dir_all(&requested_cwd).expect("authorized cwd should exist");

    fixtures::create_executable_script(
        &script,
        r#"#!/bin/sh
python3 -c '
import json
import os
import pathlib
import sys

payload = json.load(sys.stdin)
allowed_root = pathlib.Path(os.environ["ALLOWED_ROOT"]).resolve()
cwd = pathlib.Path(payload["cwd"]).resolve()
name = payload["name"]

if not str(cwd).startswith(str(allowed_root)):
    print("WorktreeCreate hook rejected: use /sc-git-worktree instead of EnterWorktree directly", file=sys.stderr)
    raise SystemExit(1)

target = allowed_root / "worktrees" / name
target.mkdir(parents=True, exist_ok=True)
print(target)
'
"#,
    );

    let outcome = run_command_hook(
        &script,
        json!({
            "session_id": "abc123",
            "transcript_path": transcript_path,
            "cwd": requested_cwd,
            "hook_event_name": "WorktreeCreate",
            "name": "feature-auth"
        }),
        &[("ALLOWED_ROOT", allowed_root.as_path())],
    );

    assert_eq!(outcome.exit_code, 0, "stderr: {}", outcome.stderr);
    assert!(outcome.stderr.is_empty());
    let path = parse_worktree_path(&outcome.stdout).expect("stdout should carry worktree path");
    let expected = allowed_root.join("worktrees/feature-auth");
    assert_eq!(
        std::fs::canonicalize(&path).expect("worktree path should canonicalize"),
        std::fs::canonicalize(&expected).expect("expected path should canonicalize")
    );
}

#[test]
fn worktree_create_blocks_unauthorized_folder_with_redirect_message() {
    let temp = tempfile::tempdir().expect("tempdir should exist");
    let root = temp.path();
    let script = root.join("worktree-create.sh");
    let allowed_root = root.join("allowed-root");
    let unauthorized_cwd = root.join("unauthorized-root/repo");
    let transcript_path = root.join("transcript.jsonl");
    std::fs::create_dir_all(&unauthorized_cwd).expect("unauthorized cwd should exist");

    fixtures::create_executable_script(
        &script,
        r#"#!/bin/sh
python3 -c '
import json
import os
import pathlib
import sys

payload = json.load(sys.stdin)
allowed_root = pathlib.Path(os.environ["ALLOWED_ROOT"]).resolve()
cwd = pathlib.Path(payload["cwd"]).resolve()

if not str(cwd).startswith(str(allowed_root)):
    print("WorktreeCreate hook rejected: use /sc-git-worktree instead of EnterWorktree directly", file=sys.stderr)
    raise SystemExit(1)

print(allowed_root / "worktrees" / payload["name"])
'
"#,
    );

    let outcome = run_command_hook(
        &script,
        json!({
            "session_id": "abc123",
            "transcript_path": transcript_path,
            "cwd": unauthorized_cwd,
            "hook_event_name": "WorktreeCreate",
            "name": "feature-auth"
        }),
        &[("ALLOWED_ROOT", allowed_root.as_path())],
    );

    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome
            .stderr
            .contains("use /sc-git-worktree instead of EnterWorktree directly")
    );
    assert!(
        outcome.stdout.is_empty(),
        "stdout should not carry a path on block"
    );
}

#[test]
fn worktree_create_does_not_accept_json_control_output_as_a_path() {
    let temp = tempfile::tempdir().expect("tempdir should exist");
    let root = temp.path();
    let script = root.join("worktree-create-json.sh");
    let transcript_path = root.join("transcript.jsonl");
    let repo_path = root.join("repo");

    fixtures::create_executable_script(
        &script,
        r#"#!/bin/sh
cat >/dev/null
printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny"}}'
"#,
    );

    let outcome = run_command_hook(
        &script,
        json!({
            "session_id": "abc123",
            "transcript_path": transcript_path,
            "cwd": repo_path,
            "hook_event_name": "WorktreeCreate",
            "name": "feature-auth"
        }),
        &[],
    );

    assert_eq!(outcome.exit_code, 0);
    let error = parse_worktree_path(&outcome.stdout)
        .expect_err("json control output must not be treated as a valid worktree path");
    assert!(error.to_string().contains("absolute stdout path"));
}

#[test]
fn worktree_remove_reads_worktree_path_from_input() {
    let temp = tempfile::tempdir().expect("tempdir should exist");
    let root = temp.path();
    let script = root.join("worktree-remove.sh");
    let worktree = root.join("allowed-root/worktrees/feature-auth");
    let transcript_path = root.join("transcript.jsonl");
    std::fs::create_dir_all(&worktree).expect("worktree should exist");

    fixtures::create_executable_script(
        &script,
        r#"#!/bin/sh
python3 -c '
import json
import pathlib
import shutil
import sys

payload = json.load(sys.stdin)
path = pathlib.Path(payload["worktree_path"])
shutil.rmtree(path)
'
"#,
    );

    let outcome = run_command_hook(
        &script,
        json!({
            "session_id": "abc123",
            "transcript_path": transcript_path,
            "cwd": root,
            "hook_event_name": "WorktreeRemove",
            "worktree_path": worktree
        }),
        &[],
    );

    assert_eq!(outcome.exit_code, 0, "stderr: {}", outcome.stderr);
    assert!(!worktree.exists(), "worktree path should be removed");
}
