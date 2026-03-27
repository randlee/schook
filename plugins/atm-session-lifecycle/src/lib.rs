use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sc_hooks_core::dispatch::DispatchMode;
use sc_hooks_core::manifest::Manifest;
use sc_hooks_sdk::manifest::ManifestBuilder;
use sc_hooks_sdk::result::{HookResult, proceed};
use sc_hooks_sdk::traits::{ManifestProvider, SyncHandler};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const ENV_STORE_DIR: &str = "SC_HOOK_SESSION_STORE_DIR";
const ENV_ATM_HOME: &str = "ATM_HOME";
const ENV_ATM_TEAM: &str = "ATM_TEAM";
const ENV_ATM_IDENTITY: &str = "ATM_IDENTITY";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RoutingContext {
    team: Option<String>,
    identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionRecord {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub created_at: f64,
    pub updated_at: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HookKind {
    SessionStart,
    SessionEnd,
}

#[derive(Debug, Clone, PartialEq)]
struct HookInput {
    kind: HookKind,
    session_id: Option<String>,
    source: Option<String>,
}

pub struct AtmSessionLifecycle;

impl ManifestProvider for AtmSessionLifecycle {
    fn manifest(&self) -> Manifest {
        ManifestBuilder::new("atm-session-lifecycle", DispatchMode::Sync)
            .hooks(["SessionStart", "SessionEnd"])
            .matchers(["*"])
            .build()
            .expect("manifest should be valid")
    }
}

impl SyncHandler for AtmSessionLifecycle {
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
        HookKind::SessionStart => handle_session_start(&parsed),
        HookKind::SessionEnd => handle_session_end(&parsed),
    }
}

fn handle_session_start(input: &HookInput) -> Result<(), String> {
    let Some(session_id) = non_empty(input.session_id.as_deref()) else {
        return Ok(());
    };

    let store = SessionStore::discover()?;
    let existing = store.read(&session_id)?;
    let routing = resolve_routing_context();
    maybe_warn_team_override();

    let now = unix_timestamp();
    let record = SessionRecord {
        session_id: session_id.clone(),
        team: routing.team.clone(),
        identity: routing.identity.clone(),
        pid: existing.as_ref().and_then(|record| record.pid),
        created_at: existing.map(|record| record.created_at).unwrap_or(now),
        updated_at: now,
    };

    store.write(&record)?;

    if let (Some(team), Some(identity), Some(process_id)) =
        (routing.team, routing.identity, record.pid)
    {
        send_hook_event(serde_json::json!({
            "event": "session_start",
            "session_id": record.session_id,
            "agent": identity,
            "team": team,
            "source": {"kind": "claude_hook"},
            "process_id": process_id,
        }));
    }

    let _ = &input.source;
    Ok(())
}

fn handle_session_end(input: &HookInput) -> Result<(), String> {
    let Some(session_id) = non_empty(input.session_id.as_deref()) else {
        return Ok(());
    };

    let store = SessionStore::discover()?;
    let existing = store.read(&session_id)?;
    let routing = resolve_routing_context().merge(existing.as_ref());
    let process_id = existing.as_ref().and_then(|record| record.pid);

    if let (Some(team), Some(identity), Some(process_id)) =
        (routing.team, routing.identity, process_id)
    {
        send_hook_event(serde_json::json!({
            "event": "session_end",
            "session_id": session_id,
            "process_id": process_id,
            "agent": identity,
            "team": team,
            "reason": "session_exit",
            "source": {"kind": "claude_hook"},
        }));
    }

    store.delete(&session_id)?;
    Ok(())
}

impl RoutingContext {
    fn merge(mut self, record: Option<&SessionRecord>) -> Self {
        if self.team.is_none() {
            self.team = record.and_then(|entry| entry.team.clone());
        }
        if self.identity.is_none() {
            self.identity = record.and_then(|entry| entry.identity.clone());
        }
        self
    }
}

fn parse_input(input: &Value) -> HookInput {
    let hook_type = input
        .get("hook")
        .and_then(|hook| hook.get("type"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let kind = match hook_type {
        "SessionEnd" => HookKind::SessionEnd,
        _ => HookKind::SessionStart,
    };

    let payload = input.get("payload").unwrap_or(&Value::Null);

    HookInput {
        kind,
        session_id: payload
            .get("session_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        source: payload
            .get("source")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn resolve_routing_context() -> RoutingContext {
    let toml = read_atm_toml();
    let core = toml
        .as_ref()
        .and_then(|value| value.get("core"))
        .and_then(toml::Value::as_table);

    let team = first_non_empty([
        env::var(ENV_ATM_TEAM).ok(),
        core.and_then(|table| table.get("default_team"))
            .and_then(toml::Value::as_str)
            .map(str::to_string),
    ]);
    let identity = first_non_empty([
        env::var(ENV_ATM_IDENTITY).ok(),
        core.and_then(|table| table.get("identity"))
            .and_then(toml::Value::as_str)
            .map(str::to_string),
    ]);

    RoutingContext { team, identity }
}

fn maybe_warn_team_override() {
    let Some(env_team) = non_empty(env::var(ENV_ATM_TEAM).ok().as_deref()) else {
        return;
    };
    let Some(toml) = read_atm_toml() else {
        return;
    };
    let Some(toml_team) = toml
        .get("core")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("default_team"))
        .and_then(toml::Value::as_str)
        .and_then(|value| non_empty(Some(value)))
    else {
        return;
    };

    if env_team != toml_team {
        eprintln!(
            "[atm-hook] WARNING: ATM_TEAM='{env_team}' overrides .atm.toml default_team='{toml_team}'"
        );
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

fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn unix_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or_default()
}

fn atm_home() -> Option<PathBuf> {
    if let Some(path) = env::var_os(ENV_ATM_HOME) {
        return Some(PathBuf::from(path));
    }
    home_dir()
}

fn session_store_root() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os(ENV_STORE_DIR) {
        return Ok(PathBuf::from(path));
    }

    if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(xdg_data_home)
            .join("sc-hooks")
            .join("sessions"));
    }

    let home = home_dir().ok_or_else(|| "unable to resolve home directory".to_string())?;
    Ok(home
        .join(".local")
        .join("share")
        .join("sc-hooks")
        .join("sessions"))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
}

struct SessionStore {
    root: PathBuf,
}

impl SessionStore {
    fn discover() -> Result<Self, String> {
        Ok(Self {
            root: session_store_root()?,
        })
    }

    fn record_path(&self, session_id: &str) -> PathBuf {
        self.root.join(format!("{session_id}.json"))
    }

    fn read(&self, session_id: &str) -> Result<Option<SessionRecord>, String> {
        let path = self.record_path(session_id);
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .map_err(|err| format!("failed to read session record {}: {err}", path.display()))?;
        let record = serde_json::from_str::<SessionRecord>(&content)
            .map_err(|err| format!("failed to parse session record {}: {err}", path.display()))?;
        Ok(Some(record))
    }

    fn write(&self, record: &SessionRecord) -> Result<(), String> {
        fs::create_dir_all(&self.root).map_err(|err| {
            format!(
                "failed to create session store directory {}: {err}",
                self.root.display()
            )
        })?;

        let content = serde_json::to_string(record)
            .map_err(|err| format!("failed to serialize session record: {err}"))?;
        let path = self.record_path(&record.session_id);
        fs::write(&path, content)
            .map_err(|err| format!("failed to write session record {}: {err}", path.display()))
    }

    fn delete(&self, session_id: &str) -> Result<(), String> {
        let path = self.record_path(session_id);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(format!(
                "failed to delete session record {}: {err}",
                path.display()
            )),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_store_root_prefers_explicit_override() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        unsafe {
            env::set_var(ENV_STORE_DIR, temp.path());
        }
        let root = session_store_root().expect("store root should resolve");
        assert_eq!(root, temp.path());
        unsafe {
            env::remove_var(ENV_STORE_DIR);
        }
    }

    #[test]
    fn parse_input_reads_hook_and_payload_fields() {
        let input = serde_json::json!({
            "hook": {"type": "SessionEnd"},
            "payload": {
                "session_id": "session-1",
                "source": "compact"
            }
        });

        let parsed = parse_input(&input);
        assert_eq!(parsed.kind, HookKind::SessionEnd);
        assert_eq!(parsed.session_id.as_deref(), Some("session-1"));
        assert_eq!(parsed.source.as_deref(), Some("compact"));
    }
}
