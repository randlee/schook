use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use atm_session_lifecycle::load_session_record;
use sc_hooks_core::dispatch::DispatchMode;
use sc_hooks_core::manifest::Manifest;
use sc_hooks_sdk::manifest::ManifestBuilder;
use sc_hooks_sdk::result::{HookResult, proceed};
use sc_hooks_sdk::traits::{ManifestProvider, SyncHandler};
use serde::Serialize;
use serde_json::Value;

const ENV_IDENTITY_DIR: &str = "SC_HOOK_ATM_IDENTITY_DIR";

#[derive(Debug, Clone, PartialEq, Eq)]
enum HookKind {
    PreToolUse,
    PostToolUse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HookInput {
    kind: HookKind,
    command: Option<String>,
    session_id: Option<String>,
    turn_key: Option<String>,
    agent_pid: Option<u32>,
}

#[derive(Debug, Serialize)]
struct IdentityFile {
    pid: Option<u32>,
    session_id: String,
    agent_name: Option<String>,
    team_name: Option<String>,
    created_at: f64,
}

pub struct AtmBashIdentity;

impl ManifestProvider for AtmBashIdentity {
    fn manifest(&self) -> Manifest {
        ManifestBuilder::new("atm-bash-identity", DispatchMode::Sync)
            .hooks(["PreToolUse", "PostToolUse"])
            .matchers(["Bash"])
            .build()
            .expect("manifest should be valid")
    }
}

impl SyncHandler for AtmBashIdentity {
    fn handle(&self, input: Value) -> Result<HookResult, String> {
        if let Err(err) = handle_input(&input) {
            eprintln!("[atm-hook] {err}");
        }
        Ok(proceed())
    }
}

pub fn handle_input(input: &Value) -> Result<(), String> {
    let parsed = parse_input(input);
    match parsed.kind {
        HookKind::PreToolUse => handle_pre_tool_use(&parsed),
        HookKind::PostToolUse => handle_post_tool_use(&parsed),
    }
}

fn handle_pre_tool_use(input: &HookInput) -> Result<(), String> {
    let Some(command) = input.command.as_deref() else {
        return Ok(());
    };
    if !is_atm_invocation(command) {
        return Ok(());
    }

    let Some(session_id) = input.session_id.as_deref() else {
        return Ok(());
    };
    let Some(record) = load_session_record(session_id)? else {
        return Ok(());
    };
    let Some(key) = input.turn_key.as_deref() else {
        return Ok(());
    };

    let identity = IdentityFile {
        pid: input.agent_pid.or(record.pid),
        session_id: record.session_id,
        agent_name: record.identity,
        team_name: record.team,
        created_at: unix_timestamp(),
    };
    let path = identity_file_path(key)?;
    write_identity_file(&path, &identity)
}

fn handle_post_tool_use(input: &HookInput) -> Result<(), String> {
    let Some(key) = input.turn_key.as_deref() else {
        return Ok(());
    };
    let path = identity_file_path(key)?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(format!(
            "failed to delete identity file {}: {err}",
            path.display()
        )),
    }
}

fn parse_input(input: &Value) -> HookInput {
    let hook_type = input
        .get("hook")
        .and_then(|hook| hook.get("type"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let kind = match hook_type {
        "PostToolUse" => HookKind::PostToolUse,
        _ => HookKind::PreToolUse,
    };
    let payload = input.get("payload").unwrap_or(&Value::Null);

    HookInput {
        kind,
        command: payload
            .get("tool_input")
            .and_then(|tool| tool.get("command"))
            .and_then(Value::as_str)
            .map(str::to_string),
        session_id: payload
            .get("session_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        turn_key: hook_key(payload),
        agent_pid: input
            .get("agent")
            .and_then(|agent| agent.get("pid"))
            .and_then(Value::as_u64)
            .and_then(|pid| u32::try_from(pid).ok()),
    }
}

fn hook_key(payload: &Value) -> Option<String> {
    first_non_empty([
        payload
            .get("turn-id")
            .and_then(Value::as_str)
            .map(str::to_string),
        payload
            .get("turn_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        payload
            .get("turnId")
            .and_then(Value::as_str)
            .map(str::to_string),
        payload
            .get("session_id")
            .and_then(Value::as_str)
            .map(str::to_string),
    ])
}

fn first_non_empty<const N: usize>(values: [Option<String>; N]) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
}

fn is_atm_invocation(command: &str) -> bool {
    command
        .split_whitespace()
        .map(|token| token.trim_matches(|ch| ch == '"' || ch == '\''))
        .any(|token| {
            token == "atm"
                || token.ends_with("/atm")
                || token.ends_with("\\atm")
                || token == "atm.exe"
                || token.ends_with("/atm.exe")
                || token.ends_with("\\atm.exe")
        })
}

fn identity_file_path(key: &str) -> Result<PathBuf, String> {
    let root = env::var_os(ENV_IDENTITY_DIR)
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    fs::create_dir_all(&root).map_err(|err| {
        format!(
            "failed to create identity directory {}: {err}",
            root.display()
        )
    })?;
    Ok(root.join(format!("atm-hook-{}.json", sanitize_key(key))))
}

fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn write_identity_file(path: &Path, identity: &IdentityFile) -> Result<(), String> {
    let content = serde_json::to_string(identity)
        .map_err(|err| format!("failed to serialize identity file: {err}"))?;
    fs::write(path, content)
        .map_err(|err| format!("failed to write identity file {}: {err}", path.display()))
}

fn unix_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_plain_atm_invocation() {
        assert!(is_atm_invocation("atm read --team atm-dev"));
        assert!(is_atm_invocation("/usr/local/bin/atm status"));
        assert!(!is_atm_invocation("echo atm-log.txt"));
    }

    #[test]
    fn turn_key_prefers_turn_fields() {
        let payload = serde_json::json!({
            "turn_id": "turn-7",
            "session_id": "session-1"
        });

        assert_eq!(hook_key(&payload).as_deref(), Some("turn-7"));
    }
}
