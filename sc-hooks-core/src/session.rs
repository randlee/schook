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
        validate_nonblank_path("ai_root_dir", &path)?;
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
        validate_nonblank_path("ai_current_dir", &path)?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStartSource {
    Startup,
    Resume,
    Compact,
    Clear,
}

impl SessionStartSource {
    pub fn establishes_root(self) -> bool {
        matches!(self, Self::Startup | Self::Resume | Self::Clear)
    }
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
    pub session_start_source: SessionStartSource,
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
        session_start_source: SessionStartSource,
        agent_state: AgentState,
        last_hook_event: impl Into<String>,
        state_reason: impl Into<String>,
    ) -> Result<Self, HookError> {
        if agent_state == AgentState::Ended {
            return Err(HookError::validation(
                "agent_state",
                "AgentState::Ended requires ended_at and must be created through an ended-state transition",
            ));
        }
        let now = utc_timestamp_now();
        let record = Self {
            schema_version: "v1".to_string(),
            provider: provider.into(),
            session_id,
            active_pid,
            parent_session_id: None,
            parent_active_pid: None,
            ai_root_dir,
            ai_current_dir,
            session_start_source,
            agent_state,
            state_revision: 1,
            created_at: now.clone(),
            updated_at: now.clone(),
            ended_at: None,
            last_hook_event: last_hook_event.into(),
            last_hook_event_at: now,
            state_reason: state_reason.into(),
            extensions: BTreeMap::new(),
        };
        record.validate()?;
        Ok(record)
    }

    pub fn ai_root_dir(&self) -> &AiRootDir {
        &self.ai_root_dir
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "root-change rebuild preserves canonical identity and timestamps under one validated constructor path"
    )]
    pub fn rebuild_with_root_change(
        &self,
        active_pid: ActivePid,
        ai_root_dir: AiRootDir,
        ai_current_dir: AiCurrentDir,
        session_start_source: SessionStartSource,
        agent_state: AgentState,
        last_hook_event: impl Into<String>,
        state_reason: impl Into<String>,
        ended_at: Option<String>,
        updated_at: String,
    ) -> Result<Self, HookError> {
        let last_hook_event = last_hook_event.into();
        let state_reason = state_reason.into();
        let record = Self {
            schema_version: self.schema_version.clone(),
            provider: self.provider.clone(),
            session_id: self.session_id.clone(),
            active_pid,
            parent_session_id: self.parent_session_id.clone(),
            parent_active_pid: self.parent_active_pid,
            ai_root_dir,
            ai_current_dir,
            session_start_source,
            agent_state,
            state_revision: self.state_revision + 1,
            created_at: self.created_at.clone(),
            updated_at: updated_at.clone(),
            ended_at,
            last_hook_event,
            last_hook_event_at: updated_at,
            state_reason,
            extensions: self.extensions.clone(),
        };
        record.validate()?;
        Ok(record)
    }

    pub fn validate(&self) -> Result<(), HookError> {
        if self.schema_version.trim().is_empty() {
            return Err(HookError::validation("schema_version", "must be non-empty"));
        }
        if self.provider.trim().is_empty() {
            return Err(HookError::validation("provider", "must be non-empty"));
        }
        if self.state_revision == 0 {
            return Err(HookError::validation("state_revision", "must be >= 1"));
        }
        validate_nonblank_text("created_at", &self.created_at)?;
        validate_nonblank_text("updated_at", &self.updated_at)?;
        validate_nonblank_text("last_hook_event", &self.last_hook_event)?;
        validate_nonblank_text("last_hook_event_at", &self.last_hook_event_at)?;
        validate_nonblank_text("state_reason", &self.state_reason)?;
        match self.agent_state {
            AgentState::Ended => {
                let ended_at = self.ended_at.as_deref().ok_or_else(|| {
                    HookError::validation("ended_at", "must be present when agent_state is ended")
                })?;
                validate_nonblank_text("ended_at", ended_at)?;
            }
            _ if self.ended_at.is_some() => {
                return Err(HookError::validation(
                    "ended_at",
                    "must be absent unless agent_state is ended",
                ));
            }
            _ => {}
        }
        Ok(())
    }
}

fn validate_nonblank_path(field: &str, path: &Path) -> Result<(), HookError> {
    let rendered = path.to_string_lossy();
    if rendered.trim().is_empty() {
        return Err(HookError::validation(field, "must be non-empty"));
    }
    Ok(())
}

fn validate_nonblank_text(field: &str, value: &str) -> Result<(), HookError> {
    if value.trim().is_empty() {
        return Err(HookError::validation(field, "must be non-empty"));
    }
    Ok(())
}

pub fn utc_timestamp_now() -> String {
    let now = OffsetDateTime::now_utc();
    match now.format(&Rfc3339) {
        Ok(rendered) => rendered,
        Err(_) => "1970-01-01T00:00:00Z".to_string(),
    }
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
            SessionStartSource::Startup,
            AgentState::AwaitingPermission,
            "PermissionRequest",
            "permission_requested",
        )
        .expect("record should construct");

        let rendered = serde_json::to_value(record).expect("record should serialize");
        assert_eq!(rendered["agent_state"], "awaiting_permission");
    }

    #[test]
    fn ai_root_dir_rejects_empty_and_whitespace_only_paths() {
        assert!(AiRootDir::new("").is_err());
        assert!(AiRootDir::new("   ").is_err());
    }

    #[test]
    fn ai_current_dir_rejects_empty_and_whitespace_only_paths() {
        assert!(AiCurrentDir::new("").is_err());
        assert!(AiCurrentDir::new("   ").is_err());
    }

    #[test]
    fn ai_root_dir_getter_returns_private_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        let repo_subdir = repo_root.join("subdir");
        let record = CanonicalSessionRecord::new(
            "claude",
            SessionId::new("session-2").expect("session id"),
            ActivePid::new(7).expect("pid"),
            AiRootDir::new(&repo_root).expect("root"),
            AiCurrentDir::new(&repo_subdir).expect("current"),
            SessionStartSource::Startup,
            AgentState::Starting,
            "SessionStart",
            "session_started",
        )
        .expect("record should construct");

        assert_eq!(record.ai_root_dir().as_path(), repo_root.as_path());
    }

    #[test]
    fn canonical_record_new_rejects_ended_state_without_ended_at() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        let repo_subdir = repo_root.join("subdir");
        let err = CanonicalSessionRecord::new(
            "claude",
            SessionId::new("session-3").expect("session id"),
            ActivePid::new(9).expect("pid"),
            AiRootDir::new(&repo_root).expect("root"),
            AiCurrentDir::new(&repo_subdir).expect("current"),
            SessionStartSource::Startup,
            AgentState::Ended,
            "SessionEnd",
            "session_ended",
        )
        .expect_err("ended state should require ended_at");
        assert!(err.to_string().contains("AgentState::Ended"));
    }
}
