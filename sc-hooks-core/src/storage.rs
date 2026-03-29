use std::fs;
use std::path::{Path, PathBuf};

use sc_observability::LoggerConfig;
use sc_observability_types::{LevelFilter, ServiceName};
use tempfile::NamedTempFile;

use crate::context::HookContext;
use crate::errors::HookError;
use crate::session::{CanonicalSessionRecord, SessionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistOutcome {
    Created,
    Updated,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct SessionStore {
    root: PathBuf,
}

impl SessionStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn load_by_hook_context(
        &self,
        context: &HookContext,
    ) -> Result<Option<CanonicalSessionRecord>, HookError> {
        let payload = context.payload_value()?;
        let session_id = payload
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| HookError::validation("session_id", "missing from payload"))?;
        self.load(&SessionId::new(session_id.to_string())?)
    }

    pub fn load(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<CanonicalSessionRecord>, HookError> {
        let path = self.path_for(session_id);
        if !path.exists() {
            return Ok(None);
        }
        let body = fs::read_to_string(&path)
            .map_err(|source| HookError::state_io(path.clone(), source))?;
        let record = serde_json::from_str::<CanonicalSessionRecord>(&body).map_err(|source| {
            HookError::InvalidPayload {
                input_excerpt: body.chars().take(120).collect(),
                source: Some(source),
            }
        })?;
        Ok(Some(record))
    }

    pub fn persist(&self, record: &CanonicalSessionRecord) -> Result<PersistOutcome, HookError> {
        let path = self.path_for(&record.session_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|source| HookError::state_io(parent.to_path_buf(), source))?;
        }

        let rendered = serde_json::to_string_pretty(record).map_err(|err| {
            HookError::internal(format!("failed to serialize session record: {err}"))
        })?;
        if let Ok(existing) = fs::read_to_string(&path)
            && existing == rendered
        {
            return Ok(PersistOutcome::Unchanged);
        }

        let parent = path
            .parent()
            .ok_or_else(|| HookError::internal("session state file missing parent directory"))?;
        let mut temp = NamedTempFile::new_in(parent)
            .map_err(|source| HookError::state_io(parent.to_path_buf(), source))?;
        use std::io::Write;
        temp.write_all(rendered.as_bytes())
            .map_err(|source| HookError::state_io(temp.path().to_path_buf(), source))?;
        temp.flush()
            .map_err(|source| HookError::state_io(temp.path().to_path_buf(), source))?;
        let existed = path.exists();
        temp.persist(&path).map_err(|err| {
            let source = std::io::Error::new(err.error.kind(), err.error.to_string());
            HookError::state_io(path.clone(), source)
        })?;

        Ok(if existed {
            PersistOutcome::Updated
        } else {
            PersistOutcome::Created
        })
    }

    pub fn path_for(&self, session_id: &SessionId) -> PathBuf {
        self.root.join(format!("{session_id}.json"))
    }
}

pub fn resolve_state_root() -> Result<PathBuf, HookError> {
    let root = std::env::var_os("SC_HOOKS_STATE_DIR")
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .ok_or_else(|| {
            HookError::invalid_context("unable to resolve SC_HOOKS_STATE_DIR or home directory")
        })?;

    if std::env::var_os("SC_HOOKS_STATE_DIR").is_some() {
        Ok(root)
    } else {
        Ok(root.join(".sc-hooks").join("state").join("sessions"))
    }
}

pub fn observability_root_for(project_root: Option<&Path>) -> Result<PathBuf, HookError> {
    let base = match project_root {
        Some(root) => root.to_path_buf(),
        None => std::env::current_dir().map_err(|err| {
            HookError::invalid_context(format!("failed resolving current dir: {err}"))
        })?,
    };
    Ok(base.join(crate::OBSERVABILITY_ROOT))
}

pub fn default_logger_config(
    service: ServiceName,
    project_root: Option<&Path>,
) -> Result<LoggerConfig, HookError> {
    let root = observability_root_for(project_root)?;
    let mut config = LoggerConfig::default_for(service, root);
    config.level = LevelFilter::Info;
    config.enable_console_sink = env_flag("SC_HOOKS_ENABLE_CONSOLE_SINK").unwrap_or(false);
    config.enable_file_sink = env_flag("SC_HOOKS_ENABLE_FILE_SINK").unwrap_or(true);
    Ok(config)
}

fn env_flag(key: &str) -> Option<bool> {
    let value = std::env::var(key).ok()?;
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{ActivePid, AgentState, AiCurrentDir, AiRootDir, CanonicalSessionRecord};
    use std::path::{Path, PathBuf};

    #[test]
    fn unchanged_records_do_not_rewrite() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = SessionStore::new(temp.path().to_path_buf());
        let record = CanonicalSessionRecord::new(
            "claude",
            SessionId::new("session-1").expect("session"),
            ActivePid::new(11).expect("pid"),
            AiRootDir::new("/repo").expect("root"),
            AiCurrentDir::new("/repo/subdir").expect("current"),
            "startup",
            AgentState::Starting,
            "SessionStart",
            "session_started",
        );

        assert_eq!(
            store.persist(&record).expect("create"),
            PersistOutcome::Created
        );
        assert_eq!(
            store.persist(&record).expect("unchanged"),
            PersistOutcome::Unchanged
        );
    }

    #[test]
    fn observability_root_uses_current_dir_when_project_root_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let _cwd = scoped_current_dir(temp.path());
        let expected = std::env::current_dir()
            .expect("current dir after switch")
            .join(crate::OBSERVABILITY_ROOT);
        let path = observability_root_for(None).expect("root");
        assert_eq!(path, expected);
    }

    struct CurrentDirGuard {
        original: PathBuf,
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.original).expect("restore cwd");
        }
    }

    fn scoped_current_dir(path: &Path) -> CurrentDirGuard {
        let original = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(path).expect("switch cwd");
        CurrentDirGuard { original }
    }
}
