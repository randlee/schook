use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use atm_session_lifecycle::load_session_record;
use sc_hooks_core::dispatch::DispatchMode;
use sc_hooks_core::manifest::Manifest;
use sc_hooks_sdk::manifest::ManifestBuilder;
use sc_hooks_sdk::result::{HookResult, proceed};
use sc_hooks_sdk::traits::{ManifestProvider, SyncHandler};
use serde_json::Value;

const ENV_ATM_HOME: &str = "ATM_HOME";
const ENV_ATM_TEAM: &str = "ATM_TEAM";
const ENV_ATM_IDENTITY: &str = "ATM_IDENTITY";

#[derive(Debug, Clone, PartialEq, Eq)]
enum HookKind {
    NotificationIdlePrompt,
    PermissionRequest,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HookInput {
    kind: HookKind,
    session_id: Option<String>,
    team: Option<String>,
    agent: Option<String>,
    tool_name: Option<String>,
}

pub struct AtmStateRelay;

impl ManifestProvider for AtmStateRelay {
    fn manifest(&self) -> Manifest {
        ManifestBuilder::new("atm-state-relay", DispatchMode::Sync)
            .hooks(["Notification", "PermissionRequest", "Stop"])
            .matchers(["*", "idle_prompt"])
            .build()
            .expect("manifest should be valid")
    }
}

impl SyncHandler for AtmStateRelay {
    fn handle(&self, input: Value) -> Result<HookResult, String> {
        if let Err(err) = handle_input(&input) {
            eprintln!("[atm-hook] {err}");
        }
        Ok(proceed())
    }
}

pub fn handle_input(input: &Value) -> Result<(), String> {
    let parsed = parse_input(input);
    let Some(session_id) = parsed.session_id.clone() else {
        return Ok(());
    };

    let record = load_session_record(&session_id)?;
    let atm_config = read_atm_toml();
    let core = atm_config
        .as_ref()
        .and_then(|value| value.get("core"))
        .and_then(toml::Value::as_table);

    let team = first_non_empty([
        parsed.team,
        env::var(ENV_ATM_TEAM).ok(),
        core.and_then(|table| table.get("default_team"))
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        record.as_ref().and_then(|entry| entry.team.clone()),
    ]);
    let agent = first_non_empty([
        parsed.agent,
        env::var(ENV_ATM_IDENTITY).ok(),
        core.and_then(|table| table.get("identity"))
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        record.as_ref().and_then(|entry| entry.identity.clone()),
    ]);

    if atm_config.is_none() && team.is_none() && agent.is_none() {
        return Ok(());
    }
    let (Some(team), Some(agent)) = (team, agent) else {
        return Ok(());
    };

    let mut event = serde_json::json!({
        "event": event_name(&parsed.kind),
        "session_id": session_id,
        "process_id": parent_process_id(),
        "agent": agent,
        "team": team,
        "source": {"kind": "claude_hook"},
    });
    if let Some(tool_name) = parsed.tool_name {
        event["tool_name"] = Value::String(tool_name);
    }

    send_hook_event(event);
    Ok(())
}

fn parse_input(input: &Value) -> HookInput {
    let hook_type = input
        .get("hook")
        .and_then(|hook| hook.get("type"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let hook_event = input
        .get("hook")
        .and_then(|hook| hook.get("event"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let kind = match hook_type {
        "PermissionRequest" => HookKind::PermissionRequest,
        "Stop" => HookKind::Stop,
        _ if hook_event == "idle_prompt" || hook_type == "Notification" => {
            HookKind::NotificationIdlePrompt
        }
        _ => HookKind::NotificationIdlePrompt,
    };
    let payload = input.get("payload").unwrap_or(&Value::Null);
    let tool_input = payload
        .get("tool_input")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    HookInput {
        kind,
        session_id: payload
            .get("session_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        team: first_non_empty([
            payload
                .get("team_name")
                .and_then(Value::as_str)
                .map(str::to_string),
            payload
                .get("team")
                .and_then(Value::as_str)
                .map(str::to_string),
        ]),
        agent: first_non_empty([
            payload
                .get("teammate_name")
                .and_then(Value::as_str)
                .map(str::to_string),
            payload
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string),
            payload
                .get("agent")
                .and_then(Value::as_str)
                .map(str::to_string),
        ]),
        tool_name: first_non_empty([
            payload
                .get("tool_name")
                .and_then(Value::as_str)
                .map(str::to_string),
            tool_input
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string),
        ]),
    }
}

fn event_name(kind: &HookKind) -> &'static str {
    match kind {
        HookKind::NotificationIdlePrompt => "notification_idle_prompt",
        HookKind::PermissionRequest => "permission_request",
        HookKind::Stop => "stop",
    }
}

fn read_atm_toml() -> Option<toml::Value> {
    let path = Path::new(".atm.toml");
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

fn first_non_empty<const N: usize>(values: [Option<String>; N]) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
}

fn atm_home() -> Option<PathBuf> {
    if let Some(path) = env::var_os(ENV_ATM_HOME) {
        return Some(PathBuf::from(path));
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
}

fn send_hook_event(payload: Value) {
    let Some(home) = atm_home() else {
        return;
    };
    let daemon_dir = home.join(".atm").join("daemon");
    let request = serde_json::json!({
        "version": 1,
        "request_id": format!("{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or_default()),
        "command": "hook-event",
        "payload": payload,
    });
    let mut body = match serde_json::to_vec(&request) {
        Ok(body) => body,
        Err(err) => {
            eprintln!("[atm-hook] failed to serialize hook event: {err}");
            return;
        }
    };
    body.push(b'\n');

    #[cfg(windows)]
    {
        send_tcp(&daemon_dir, &body);
    }

    #[cfg(not(windows))]
    {
        send_unix(&daemon_dir, &body);
    }
}

#[cfg(not(windows))]
fn send_unix(daemon_dir: &Path, body: &[u8]) {
    use std::os::unix::net::UnixStream;

    let socket_path = daemon_dir.join("atm-daemon.sock");
    if !socket_path.exists() {
        return;
    }

    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(1)));
            let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(1)));
            if let Err(err) = stream.write_all(body) {
                eprintln!("[atm-hook] unix socket send failed: {err}");
                return;
            }
            let mut response = [0_u8; 4096];
            let _ = stream.read(&mut response);
        }
        Err(err) => eprintln!("[atm-hook] unix socket send failed: {err}"),
    }
}

#[cfg(windows)]
fn send_tcp(daemon_dir: &Path, body: &[u8]) {
    use std::net::TcpStream;

    let port_path = daemon_dir.join("atm-daemon.port");
    let Ok(content) = fs::read_to_string(&port_path) else {
        return;
    };
    let Ok(port) = content.trim().parse::<u16>() else {
        return;
    };

    match TcpStream::connect(("127.0.0.1", port)) {
        Ok(mut stream) => {
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(1)));
            let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(1)));
            if let Err(err) = stream.write_all(body) {
                eprintln!("[atm-hook] tcp socket send failed: {err}");
                return;
            }
            let mut response = [0_u8; 4096];
            let _ = stream.read(&mut response);
        }
        Err(err) => eprintln!("[atm-hook] tcp socket send failed: {err}"),
    }
}

#[cfg(unix)]
fn parent_process_id() -> u32 {
    unsafe { libc::getppid() as u32 }
}

#[cfg(not(unix))]
fn parent_process_id() -> u32 {
    std::process::id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_permission_request_prefers_payload_tool_name() {
        let input = serde_json::json!({
            "hook": {"type": "PermissionRequest"},
            "payload": {
                "session_id": "session-1",
                "tool_name": "Bash",
                "tool_input": {"name": "Task"},
                "team": "atm-dev",
                "agent": "arch-hook"
            }
        });

        let parsed = parse_input(&input);
        assert_eq!(parsed.kind, HookKind::PermissionRequest);
        assert_eq!(parsed.tool_name.as_deref(), Some("Bash"));
        assert_eq!(parsed.session_id.as_deref(), Some("session-1"));
    }
}
