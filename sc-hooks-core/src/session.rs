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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Claude,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaVersion {
    V1,
}

impl SchemaVersion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V1 => "v1",
        }
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StateRoot(PathBuf);

impl StateRoot {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, HookError> {
        let path = path.into();
        validate_nonblank_path("state_root", &path)?;
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }

    pub fn parent(&self) -> Option<&Path> {
        self.0.parent()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UtcTimestamp(String);

impl UtcTimestamp {
    pub fn from_field(field: &str, value: impl Into<String>) -> Result<Self, HookError> {
        let value = value.into();
        validate_nonblank_text(field, &value)?;
        OffsetDateTime::parse(&value, &Rfc3339).map_err(|source| {
            HookError::validation_with_source(field, "must be a valid RFC 3339 timestamp", source)
        })?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UtcTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
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
    schema_version: SchemaVersion,
    provider: Provider,
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
    state_revision: u64,
    created_at: UtcTimestamp,
    updated_at: UtcTimestamp,
    #[serde(default)]
    ended_at: Option<UtcTimestamp>,
    pub last_hook_event: String,
    last_hook_event_at: UtcTimestamp,
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
        provider: Provider,
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
            schema_version: SchemaVersion::V1,
            provider,
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

    pub fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    pub fn provider(&self) -> Provider {
        self.provider
    }

    pub fn state_revision(&self) -> u64 {
        self.state_revision
    }

    pub fn created_at(&self) -> &UtcTimestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &UtcTimestamp {
        &self.updated_at
    }

    pub fn ended_at(&self) -> Option<&UtcTimestamp> {
        self.ended_at.as_ref()
    }

    pub fn last_hook_event_at(&self) -> &UtcTimestamp {
        &self.last_hook_event_at
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
        ended_at: Option<UtcTimestamp>,
        updated_at: UtcTimestamp,
    ) -> Result<Self, HookError> {
        let last_hook_event = last_hook_event.into();
        let state_reason = state_reason.into();
        let record = Self {
            schema_version: self.schema_version,
            provider: self.provider,
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

    pub fn mark_material_change(&mut self, updated_at: UtcTimestamp) -> Result<(), HookError> {
        self.state_revision += 1;
        self.updated_at = updated_at;
        self.validate()
    }

    pub fn apply_hook_update(
        &mut self,
        updated_at: UtcTimestamp,
        last_hook_event: impl Into<String>,
        state_reason: impl Into<String>,
        ended_at: Option<UtcTimestamp>,
    ) -> Result<(), HookError> {
        self.state_revision += 1;
        self.updated_at = updated_at.clone();
        self.last_hook_event = last_hook_event.into();
        self.last_hook_event_at = updated_at;
        self.state_reason = state_reason.into();
        self.ended_at = ended_at;
        self.validate()
    }

    pub fn validate(&self) -> Result<(), HookError> {
        if self.state_revision == 0 {
            return Err(HookError::validation("state_revision", "must be >= 1"));
        }
        validate_timestamp("created_at", &self.created_at)?;
        validate_timestamp("updated_at", &self.updated_at)?;
        validate_nonblank_text("last_hook_event", &self.last_hook_event)?;
        validate_timestamp("last_hook_event_at", &self.last_hook_event_at)?;
        validate_nonblank_text("state_reason", &self.state_reason)?;
        match self.agent_state {
            AgentState::Ended => {
                let ended_at = self.ended_at.as_ref().ok_or_else(|| {
                    HookError::validation("ended_at", "must be present when agent_state is ended")
                })?;
                validate_timestamp("ended_at", ended_at)?;
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
    if !path.is_absolute() {
        return Err(HookError::validation(field, "must be an absolute path"));
    }
    Ok(())
}

fn validate_nonblank_text(field: &str, value: &str) -> Result<(), HookError> {
    if value.trim().is_empty() {
        return Err(HookError::validation(field, "must be non-empty"));
    }
    Ok(())
}

fn validate_timestamp(field: &str, value: &UtcTimestamp) -> Result<(), HookError> {
    UtcTimestamp::from_field(field, value.as_str()).map(|_| ())
}

pub fn utc_timestamp_now() -> UtcTimestamp {
    let now = OffsetDateTime::now_utc();
    let rendered = now
        .format(&Rfc3339)
        .expect("RFC 3339 formatting for UTC timestamps should be infallible");
    UtcTimestamp(rendered)
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
            Provider::Claude,
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
        assert!(AiRootDir::new("relative/path").is_err());
    }

    #[test]
    fn ai_current_dir_rejects_empty_and_whitespace_only_paths() {
        assert!(AiCurrentDir::new("").is_err());
        assert!(AiCurrentDir::new("   ").is_err());
        assert!(AiCurrentDir::new("relative/path").is_err());
    }

    #[test]
    fn ai_root_dir_getter_returns_private_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        let repo_subdir = repo_root.join("subdir");
        let record = CanonicalSessionRecord::new(
            Provider::Claude,
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
            Provider::Claude,
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

    #[test]
    fn session_start_source_roundtrips_all_variants() {
        for source in [
            SessionStartSource::Startup,
            SessionStartSource::Resume,
            SessionStartSource::Compact,
            SessionStartSource::Clear,
        ] {
            let rendered = serde_json::to_string(&source).expect("source should serialize");
            let reparsed: SessionStartSource =
                serde_json::from_str(&rendered).expect("source should deserialize");
            assert_eq!(reparsed, source);
        }
    }
}
