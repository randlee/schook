use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::errors::HookError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(value: impl Into<String>) -> Result<Self, HookError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(HookError::validation("session_id", "must be non-empty"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActivePid(u32);

impl ActivePid {
    pub fn new(value: u32) -> Result<Self, HookError> {
        if value == 0 {
            return Err(HookError::validation("active_pid", "must be > 0"));
        }
        Ok(Self(value))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AiRootDir(PathBuf);

impl AiRootDir {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, HookError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(HookError::validation("ai_root_dir", "must be non-empty"));
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for AiRootDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AiCurrentDir(PathBuf);

impl AiCurrentDir {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, HookError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(HookError::validation("ai_current_dir", "must be non-empty"));
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for AiCurrentDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display().fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Starting,
    Busy,
    AwaitingPermission,
    Compacting,
    Idle,
    Ended,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalSessionRecord {
    pub schema_version: String,
    pub provider: String,
    pub session_id: SessionId,
    pub active_pid: ActivePid,
    #[serde(default)]
    pub parent_session_id: Option<SessionId>,
    #[serde(default)]
    pub parent_active_pid: Option<ActivePid>,
    #[serde(alias = "project_root_dir")]
    ai_root_dir: AiRootDir,
    pub ai_current_dir: AiCurrentDir,
    pub session_start_source: String,
    pub agent_state: AgentState,
    pub state_revision: u64,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub ended_at: Option<String>,
    pub last_hook_event: String,
    pub last_hook_event_at: String,
    pub state_reason: String,
    #[serde(default)]
    pub extensions: BTreeMap<String, Value>,
}

impl CanonicalSessionRecord {
    #[expect(
        clippy::too_many_arguments,
        reason = "canonical session construction keeps the persisted identity tuple explicit"
    )]
    pub fn new(
        provider: impl Into<String>,
        session_id: SessionId,
        active_pid: ActivePid,
        ai_root_dir: AiRootDir,
        ai_current_dir: AiCurrentDir,
        session_start_source: impl Into<String>,
        agent_state: AgentState,
        last_hook_event: impl Into<String>,
        state_reason: impl Into<String>,
    ) -> Self {
        let now = utc_timestamp_now();
        Self {
            schema_version: "v1".to_string(),
            provider: provider.into(),
            session_id,
            active_pid,
            parent_session_id: None,
            parent_active_pid: None,
            ai_root_dir,
            ai_current_dir,
            session_start_source: session_start_source.into(),
            agent_state,
            state_revision: 1,
            created_at: now.clone(),
            updated_at: now.clone(),
            ended_at: None,
            last_hook_event: last_hook_event.into(),
            last_hook_event_at: now,
            state_reason: state_reason.into(),
            extensions: BTreeMap::new(),
        }
    }

    pub fn ai_root_dir(&self) -> &AiRootDir {
        &self.ai_root_dir
    }
}

pub fn utc_timestamp_now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("Rfc3339 formatting should succeed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_rejects_empty_values() {
        let err = SessionId::new("   ").expect_err("empty session id should fail");
        assert!(err.to_string().contains("session_id"));
    }

    #[test]
    fn active_pid_rejects_zero() {
        let err = ActivePid::new(0).expect_err("zero pid should fail");
        assert!(err.to_string().contains("active_pid"));
    }

    #[test]
    fn canonical_record_uses_lowercase_agent_state_strings() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        let repo_subdir = repo_root.join("subdir");
        let record = CanonicalSessionRecord::new(
            "claude",
            SessionId::new("session-1").expect("session id"),
            ActivePid::new(42).expect("pid"),
            AiRootDir::new(&repo_root).expect("root"),
            AiCurrentDir::new(&repo_subdir).expect("current"),
            "startup",
            AgentState::AwaitingPermission,
            "PermissionRequest",
            "permission_requested",
        );

        let rendered = serde_json::to_value(record).expect("record should serialize");
        assert_eq!(rendered["agent_state"], "awaiting_permission");
    }
}
