use std::fs;
use std::path::PathBuf;

use tempfile::NamedTempFile;

use crate::context::HookContext;
use crate::errors::HookError;
use crate::session::{AiRootDir, CanonicalSessionRecord, SessionId, StateRoot};
#[derive(Debug, Clone, PartialEq, Eq)]
/// Derived root directory used for observability file output.
pub struct ObservabilityRoot(PathBuf);

impl ObservabilityRoot {
    /// Wraps an already-resolved observability root path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// Borrows the wrapped path.
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }

    /// Unwraps the owned path buffer.
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Result of a persist attempt against canonical session storage.
pub enum PersistOutcome {
    /// A new state file was created.
    Created,
    /// An existing state file was rewritten.
    Updated,
    /// The rendered state was unchanged, so no write occurred.
    Unchanged,
}

#[derive(Debug, Clone)]
/// Atomic filesystem-backed store for canonical session records.
pub struct SessionStore {
    root: StateRoot,
}

impl SessionStore {
    /// Creates a session store rooted at the provided state directory.
    pub fn new(root: StateRoot) -> Self {
        Self { root }
    }

    /// Loads a canonical record using the `session_id` present in hook payload JSON.
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

    /// Loads a canonical record by session identifier.
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
        record.validate()?;
        Ok(Some(record))
    }

    /// Persists a canonical session record atomically.
    pub fn persist(&self, record: &CanonicalSessionRecord) -> Result<PersistOutcome, HookError> {
        let path = self.path_for(record.session_id());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|source| HookError::state_io(parent.to_path_buf(), source))?;
        }

        let rendered = serde_json::to_string_pretty(record).map_err(|source| {
            HookError::internal_with_source("failed to serialize session record", source)
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
        temp.persist(&path)
            .map_err(|err| HookError::state_io(path.clone(), err.error))?;

        Ok(if existed {
            PersistOutcome::Updated
        } else {
            PersistOutcome::Created
        })
    }

    /// Returns the state-file path for the provided session identifier.
    pub fn path_for(&self, session_id: &SessionId) -> PathBuf {
        self.root.join(format!("{session_id}.json"))
    }
}

/// Resolves the canonical session-state root from `SC_HOOKS_STATE_DIR` or the home directory.
pub fn resolve_state_root() -> Result<StateRoot, HookError> {
    match std::env::var_os("SC_HOOKS_STATE_DIR") {
        Some(dir) => StateRoot::new(PathBuf::from(dir)),
        None => dirs::home_dir()
            .map(|home| StateRoot::new(home.join(".sc-hooks").join("state").join("sessions")))
            .ok_or_else(|| {
                HookError::invalid_context("unable to resolve SC_HOOKS_STATE_DIR or home directory")
            })?,
    }
}

/// Resolves the observability root from an immutable project root or current directory.
pub fn observability_root_for(
    project_root: Option<&AiRootDir>,
) -> Result<ObservabilityRoot, HookError> {
    let base = match project_root {
        Some(root) => root.as_path().to_path_buf(),
        None => std::env::current_dir().map_err(|source| {
            HookError::internal_with_source("failed resolving current dir", source)
        })?,
    };
    Ok(ObservabilityRoot::new(base.join(crate::OBSERVABILITY_ROOT)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{
        ActivePid, AgentState, AiCurrentDir, AiRootDir, CanonicalSessionRecord, SessionStartSource,
    };
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn unchanged_records_do_not_rewrite() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = SessionStore::new(StateRoot::new(temp.path()).expect("state root"));
        let repo_root = temp.path().join("repo");
        let repo_subdir = repo_root.join("subdir");
        let record = CanonicalSessionRecord::new(
            crate::session::Provider::Claude,
            SessionId::new("session-1").expect("session"),
            ActivePid::new(11).expect("pid"),
            AiRootDir::new(&repo_root).expect("root"),
            AiCurrentDir::new(&repo_subdir).expect("current"),
            SessionStartSource::Startup,
            AgentState::Starting,
            "SessionStart",
            "session_started",
        )
        .expect("record");

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
        assert_eq!(path.as_path(), expected);
    }

    #[test]
    fn observability_root_uses_project_root_when_present() {
        let temp = tempfile::tempdir().expect("tempdir");
        let project_root = AiRootDir::new(temp.path().join("repo")).expect("project root");
        let path = observability_root_for(Some(&project_root)).expect("root");
        assert_eq!(
            path.as_path(),
            project_root.as_path().join(crate::OBSERVABILITY_ROOT)
        );
    }

    #[test]
    fn load_rejects_zero_state_revision() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = SessionStore::new(StateRoot::new(temp.path()).expect("state root"));
        let session_id = SessionId::new("session-invalid").expect("session id");
        let state_path = store.path_for(&session_id);
        fs::write(
            &state_path,
            serde_json::json!({
                "schema_version": "v1",
                "provider": "claude",
                "session_id": "session-invalid",
                "active_pid": 11,
                "ai_root_dir": temp.path().join("repo"),
                "ai_current_dir": temp.path().join("repo"),
                "session_start_source": "startup",
                "agent_state": "starting",
                "state_revision": 0,
                "created_at": "2026-03-30T00:00:00Z",
                "updated_at": "2026-03-30T00:00:00Z",
                "last_hook_event": "SessionStart",
                "last_hook_event_at": "2026-03-30T00:00:00Z",
                "state_reason": "session_started",
                "extensions": {}
            })
            .to_string(),
        )
        .expect("invalid state file");

        let err = store
            .load(&session_id)
            .expect_err("invalid revision should fail");
        match err {
            HookError::InvalidPayload {
                source: Some(source),
                ..
            } => assert!(source.to_string().contains("state_revision")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn load_rejects_empty_created_at() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = SessionStore::new(StateRoot::new(temp.path()).expect("state root"));
        let session_id = SessionId::new("session-invalid-created").expect("session id");
        let state_path = store.path_for(&session_id);
        fs::write(
            &state_path,
            serde_json::json!({
                "schema_version": "v1",
                "provider": "claude",
                "session_id": "session-invalid-created",
                "active_pid": 11,
                "ai_root_dir": temp.path().join("repo"),
                "ai_current_dir": temp.path().join("repo"),
                "session_start_source": "startup",
                "agent_state": "starting",
                "state_revision": 1,
                "created_at": " ",
                "updated_at": "2026-03-30T00:00:00Z",
                "last_hook_event": "SessionStart",
                "last_hook_event_at": "2026-03-30T00:00:00Z",
                "state_reason": "session_started",
                "extensions": {}
            })
            .to_string(),
        )
        .expect("invalid state file");

        let err = store
            .load(&session_id)
            .expect_err("blank created_at should fail");
        match err {
            HookError::InvalidPayload {
                source: Some(source),
                ..
            } => assert!(source.to_string().contains("created_at")),
            other => panic!("unexpected error: {other}"),
        }
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
