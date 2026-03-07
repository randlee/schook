use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::errors::CliError;

const STATE_PATH: &str = ".sc-hooks/state/session.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct DisabledPluginInfo {
    reason: String,
    disabled_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
struct SessionRecord {
    #[serde(default)]
    disabled_plugins: BTreeMap<String, DisabledPluginInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
struct SessionStore {
    #[serde(default)]
    sessions: BTreeMap<String, SessionRecord>,
}

pub fn load_disabled_plugins(session_id: Option<&str>) -> BTreeSet<String> {
    let Some(session_id) = normalize_session_id(session_id) else {
        return BTreeSet::new();
    };

    let store = read_store(&state_path()).unwrap_or_default();
    store
        .sessions
        .get(session_id)
        .map(|record| record.disabled_plugins.keys().cloned().collect())
        .unwrap_or_default()
}

pub fn mark_plugin_disabled(
    session_id: Option<&str>,
    plugin: &str,
    reason: &str,
) -> Result<(), CliError> {
    let Some(session_id) = normalize_session_id(session_id) else {
        return Ok(());
    };

    let path = state_path();
    let mut store = read_store(&path).unwrap_or_default();
    let record = store.sessions.entry(session_id.to_string()).or_default();
    record.disabled_plugins.insert(
        plugin.to_string(),
        DisabledPluginInfo {
            reason: reason.to_string(),
            disabled_at: now_timestamp(),
        },
    );

    write_store(&path, &store)
}

pub fn clear_session(session_id: Option<&str>) -> Result<(), CliError> {
    let Some(session_id) = normalize_session_id(session_id) else {
        return Ok(());
    };

    let path = state_path();
    let mut store = read_store(&path).unwrap_or_default();
    store.sessions.remove(session_id);
    write_store(&path, &store)
}

fn now_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    #[cfg(unix)]
    {
        let raw = seconds as nix::libc::time_t;
        // SAFETY: `gmtime_r` writes to the provided `tm` struct for a valid `time_t` pointer.
        unsafe {
            let mut tm: nix::libc::tm = std::mem::zeroed();
            if nix::libc::gmtime_r(&raw, &mut tm).is_null() {
                return seconds.to_string();
            }
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                tm.tm_year + 1900,
                tm.tm_mon + 1,
                tm.tm_mday,
                tm.tm_hour,
                tm.tm_min,
                tm.tm_sec
            )
        }
    }
    #[cfg(not(unix))]
    {
        seconds.to_string()
    }
}

fn state_path() -> PathBuf {
    PathBuf::from(STATE_PATH)
}

fn normalize_session_id(session_id: Option<&str>) -> Option<&str> {
    let id = session_id?;
    if id.trim().is_empty() { None } else { Some(id) }
}

fn read_store(path: &Path) -> Result<SessionStore, CliError> {
    if !path.exists() {
        return Ok(SessionStore::default());
    }

    let content = fs::read_to_string(path).map_err(|err| {
        CliError::internal(format!(
            "failed reading session state {}: {err}",
            path.display()
        ))
    })?;

    serde_json::from_str::<SessionStore>(&content).map_err(|err| {
        CliError::internal(format!(
            "failed parsing session state {}: {err}",
            path.display()
        ))
    })
}

fn write_store(path: &Path, store: &SessionStore) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            CliError::internal(format!(
                "failed creating session state directory {}: {err}",
                parent.display()
            ))
        })?;
    }

    let content = serde_json::to_string_pretty(store)
        .map_err(|err| CliError::internal(format!("failed serializing session state: {err}")))?;
    fs::write(path, content).map_err(|err| {
        CliError::internal(format!(
            "failed writing session state {}: {err}",
            path.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    #[test]
    fn persists_and_loads_disabled_plugins() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch");

        mark_plugin_disabled(Some("session-a"), "guard-paths", "invalid-json")
            .expect("disable state should persist");
        mark_plugin_disabled(Some("session-a"), "notify", "timeout")
            .expect("second plugin should persist");

        let loaded = load_disabled_plugins(Some("session-a"));
        assert!(loaded.contains("guard-paths"));
        assert!(loaded.contains("notify"));

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn missing_state_file_is_fail_open() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch");

        let loaded = load_disabled_plugins(Some("session-a"));
        assert!(loaded.is_empty());

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn clear_session_removes_record() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch");

        mark_plugin_disabled(Some("session-a"), "guard-paths", "invalid-json")
            .expect("disable state should persist");
        clear_session(Some("session-a")).expect("session clear should succeed");

        let loaded = load_disabled_plugins(Some("session-a"));
        assert!(loaded.is_empty());

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn disabled_at_is_iso8601_like_timestamp() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch");

        mark_plugin_disabled(Some("session-a"), "guard-paths", "invalid-json")
            .expect("disable state should persist");
        let content =
            fs::read_to_string(".sc-hooks/state/session.json").expect("state file should exist");
        assert!(content.contains('T'));
        assert!(content.contains('Z'));

        std::env::set_current_dir(original).expect("cwd should restore");
    }
}
