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
    let mut store = read_store(&path)?;
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
    let mut store = read_store(&path)?;
    store.sessions.remove(session_id);
    write_store(&path, &store)
}

pub fn clear_all_sessions() -> Result<(), CliError> {
    let path = state_path();
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(&path).map_err(|err| {
        CliError::internal(format!(
            "failed removing session state {}: {err}",
            path.display()
        ))
    })
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
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        mark_plugin_disabled(Some("session-a"), "guard-paths", "invalid-json")
            .expect("disable state should persist");
        mark_plugin_disabled(Some("session-a"), "notify", "timeout")
            .expect("second plugin should persist");

        let loaded = load_disabled_plugins(Some("session-a"));
        assert!(loaded.contains("guard-paths"));
        assert!(loaded.contains("notify"));
    }

    #[test]
    fn missing_state_file_is_fail_open() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        let loaded = load_disabled_plugins(Some("session-a"));
        assert!(loaded.is_empty());
    }

    #[test]
    fn clear_session_removes_record() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        mark_plugin_disabled(Some("session-a"), "guard-paths", "invalid-json")
            .expect("disable state should persist");
        clear_session(Some("session-a")).expect("session clear should succeed");

        let loaded = load_disabled_plugins(Some("session-a"));
        assert!(loaded.is_empty());
    }

    #[test]
    fn disabled_at_is_iso8601_like_timestamp() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        mark_plugin_disabled(Some("session-a"), "guard-paths", "invalid-json")
            .expect("disable state should persist");
        let content =
            fs::read_to_string(".sc-hooks/state/session.json").expect("state file should exist");
        assert!(content.contains('T'));
        assert!(content.contains('Z'));
    }

    #[test]
    fn clear_all_sessions_removes_state_file() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        mark_plugin_disabled(Some("session-a"), "guard-paths", "invalid-json")
            .expect("disable state should persist");
        assert!(Path::new(".sc-hooks/state/session.json").exists());

        clear_all_sessions().expect("clear_all_sessions should succeed");
        assert!(!Path::new(".sc-hooks/state/session.json").exists());
    }

    #[test]
    fn mark_plugin_disabled_fails_on_corrupt_state_file() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        fs::create_dir_all(".sc-hooks/state").expect("state dir should be creatable");
        fs::write(".sc-hooks/state/session.json", "{not-json")
            .expect("state file should be writable");

        let err = mark_plugin_disabled(Some("session-a"), "guard-paths", "invalid-json")
            .expect_err("corrupt session state should not be silently reset on write");
        assert!(err.to_string().contains("failed parsing session state"));
    }

    #[test]
    fn clear_session_fails_on_corrupt_state_file() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        fs::create_dir_all(".sc-hooks/state").expect("state dir should be creatable");
        fs::write(".sc-hooks/state/session.json", "{not-json")
            .expect("state file should be writable");

        let err = clear_session(Some("session-a"))
            .expect_err("corrupt session state should not be silently reset on clear");
        assert!(err.to_string().contains("failed parsing session state"));
    }
}
