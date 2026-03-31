use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::errors::HookError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
/// Stable identifier for a canonical Claude session record.
pub struct SessionId(String);

impl SessionId {
    /// Creates a validated session identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, HookError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(HookError::validation("session_id", "must be non-empty"));
        }
        Ok(Self(value))
    }

    /// Returns the session identifier as a borrowed string slice.
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
/// Positive process identifier for the currently active Claude process.
pub struct ActivePid(u32);

impl ActivePid {
    /// Creates a validated non-zero PID wrapper.
    pub fn new(value: u32) -> Result<Self, HookError> {
        if value == 0 {
            return Err(HookError::validation("active_pid", "must be > 0"));
        }
        Ok(Self(value))
    }

    /// Returns the wrapped PID value.
    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Supported hook payload providers.
pub enum Provider {
    /// Anthropic Claude Code.
    Claude,
}

impl Provider {
    /// Returns the serialized provider name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Version tag for the canonical session record schema.
pub enum SchemaVersion {
    /// First canonical schema version.
    V1,
}

impl SchemaVersion {
    /// Returns the serialized schema version.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V1 => "v1",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
/// Immutable project root established for a runtime instance.
pub struct AiRootDir(PathBuf);

impl AiRootDir {
    /// Creates a validated absolute root path.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, HookError> {
        let path = path.into();
        validate_nonblank_path("ai_root_dir", &path)?;
        Ok(Self(path))
    }

    /// Borrows the wrapped filesystem path.
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
/// Current working directory observed for the latest hook fire.
pub struct AiCurrentDir(PathBuf);

impl AiCurrentDir {
    /// Creates a validated absolute current-directory path.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, HookError> {
        let path = path.into();
        validate_nonblank_path("ai_current_dir", &path)?;
        Ok(Self(path))
    }

    /// Borrows the wrapped filesystem path.
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
/// Base directory that owns canonical session-state files.
pub struct StateRoot(PathBuf);

impl StateRoot {
    /// Creates a validated absolute state-root path.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, HookError> {
        let path = path.into();
        validate_nonblank_path("state_root", &path)?;
        Ok(Self(path))
    }

    /// Borrows the wrapped filesystem path.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Joins a child path under the state root.
    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }

    /// Returns the parent directory of the state root, when one exists.
    pub fn parent(&self) -> Option<&Path> {
        self.0.parent()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
/// RFC 3339 timestamp used throughout canonical session state.
pub struct UtcTimestamp(String);

impl UtcTimestamp {
    /// Creates a validated timestamp from a named serialized field.
    pub fn from_field(field: &str, value: impl Into<String>) -> Result<Self, HookError> {
        let value = value.into();
        validate_rfc3339_timestamp(field, &value)?;
        Ok(Self(value))
    }

    /// Returns the timestamp as a borrowed string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UtcTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for UtcTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        UtcTimestamp::from_field("utc_timestamp", value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Normalized agent state tracked across hook events.
#[non_exhaustive]
pub enum AgentState {
    /// The session has started but is not yet ready for general dispatch.
    Starting,
    /// The agent is currently executing work.
    Busy,
    /// The agent is blocked on a permission request.
    AwaitingPermission,
    /// The agent is compacting its context.
    Compacting,
    /// The agent is idle and ready for more input.
    Idle,
    /// The runtime instance has ended.
    Ended,
    /// Future or unknown state value preserved as a safe fallback.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
/// Source value reported by a `SessionStart` hook payload.
pub enum SessionStartSource {
    /// A fresh session startup.
    Startup,
    /// A resumed Claude session.
    Resume,
    /// A compact-triggered replacement session.
    Compact,
    /// A clear-triggered replacement session.
    Clear,
    /// Future or unknown source value preserved as a safe fallback.
    #[serde(other)]
    Unknown,
}

impl SessionStartSource {
    /// Returns whether this source establishes a fresh immutable root.
    pub fn establishes_root(self) -> bool {
        matches!(self, Self::Startup | Self::Resume | Self::Clear)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
/// Monotonic revision number for canonical session state.
pub struct StateRevision(u64);

impl StateRevision {
    /// Creates a validated state revision wrapper.
    pub fn new(value: u64) -> Result<Self, HookError> {
        if value == 0 {
            return Err(HookError::validation("state_revision", "must be >= 1"));
        }
        Ok(Self(value))
    }

    /// Returns the initial state revision for a newly created record.
    pub const fn initial() -> Self {
        Self(1)
    }

    /// Returns the wrapped revision number.
    pub fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for StateRevision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        StateRevision::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
/// Canonical hook-event name persisted in session state.
pub struct HookEventName(String);

impl HookEventName {
    /// Creates a validated hook-event name.
    pub fn new(value: impl Into<String>) -> Result<Self, HookError> {
        let value = value.into();
        validate_nonblank_text("last_hook_event", &value)?;
        Ok(Self(value))
    }

    /// Borrows the normalized hook-event name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for HookEventName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        HookEventName::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
/// Canonical human-readable reason stored with the latest session transition.
pub struct StateReason(String);

impl StateReason {
    /// Creates a validated state-reason wrapper.
    pub fn new(value: impl Into<String>) -> Result<Self, HookError> {
        let value = value.into();
        validate_nonblank_text("state_reason", &value)?;
        Ok(Self(value))
    }

    /// Borrows the stored state reason.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for StateReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        StateReason::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Persisted canonical session record used by hook utilities and extensions.
pub struct CanonicalSessionRecord {
    schema_version: SchemaVersion,
    provider: Provider,
    session_id: SessionId,
    #[serde(default)]
    active_pid: ActivePid,
    #[serde(default)]
    parent_session_id: Option<SessionId>,
    #[serde(default)]
    parent_active_pid: Option<ActivePid>,
    #[serde(alias = "project_root_dir")]
    ai_root_dir: AiRootDir,
    ai_current_dir: AiCurrentDir,
    session_start_source: SessionStartSource,
    agent_state: AgentState,
    state_revision: StateRevision,
    created_at: UtcTimestamp,
    updated_at: UtcTimestamp,
    #[serde(default)]
    ended_at: Option<UtcTimestamp>,
    last_hook_event: HookEventName,
    last_hook_event_at: UtcTimestamp,
    state_reason: StateReason,
    #[serde(default)]
    extensions: BTreeMap<String, Value>,
}

/// Active-session wrapper that exclusively owns record mutation APIs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActiveSessionRecord(CanonicalSessionRecord);

/// Ended-session wrapper used when the canonical record has reached terminal state.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EndedSessionRecord(CanonicalSessionRecord);

/// Result of a validated canonical-session transition.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SessionTransitionResult {
    /// The transition produced another active record.
    Active(ActiveSessionRecord),
    /// The transition produced a terminal ended record.
    Ended(EndedSessionRecord),
}

impl From<ActiveSessionRecord> for CanonicalSessionRecord {
    fn from(record: ActiveSessionRecord) -> Self {
        record.into_inner()
    }
}

impl AsRef<CanonicalSessionRecord> for ActiveSessionRecord {
    fn as_ref(&self) -> &CanonicalSessionRecord {
        &self.0
    }
}

impl AsRef<CanonicalSessionRecord> for EndedSessionRecord {
    fn as_ref(&self) -> &CanonicalSessionRecord {
        &self.0
    }
}

impl std::ops::Deref for ActiveSessionRecord {
    type Target = CanonicalSessionRecord;

    /// Exposes read-only access to the underlying canonical record while
    /// keeping mutation APIs on `ActiveSessionRecord`.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for EndedSessionRecord {
    type Target = CanonicalSessionRecord;

    /// Exposes read-only access to the terminal record. `DerefMut` is
    /// intentionally absent so ended records cannot be mutated back into an
    /// active lifecycle state.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SessionTransitionResult {
    /// Consumes the transition result and returns the underlying canonical record.
    pub fn into_record(self) -> CanonicalSessionRecord {
        match self {
            Self::Active(record) => record.into_inner(),
            Self::Ended(record) => record.0,
        }
    }
}

impl CanonicalSessionRecord {
    #[expect(
        clippy::too_many_arguments,
        reason = "canonical session construction keeps the persisted identity tuple explicit"
    )]
    /// Creates a new canonical session record at revision `1`.
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
            state_revision: StateRevision::initial(),
            created_at: now.clone(),
            updated_at: now.clone(),
            ended_at: None,
            last_hook_event: HookEventName::new(last_hook_event)?,
            last_hook_event_at: now,
            state_reason: StateReason::new(state_reason)?,
            extensions: BTreeMap::new(),
        };
        record.validate()?;
        Ok(record)
    }

    /// Returns the immutable root directory for the runtime instance.
    pub fn ai_root_dir(&self) -> &AiRootDir {
        &self.ai_root_dir
    }

    /// Returns the canonical session identifier.
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns the current active PID.
    pub fn active_pid(&self) -> ActivePid {
        self.active_pid
    }

    /// Returns the parent session identifier for subagent sessions, when present.
    pub fn parent_session_id(&self) -> Option<&SessionId> {
        self.parent_session_id.as_ref()
    }

    /// Returns the parent active PID for subagent sessions, when present.
    pub fn parent_active_pid(&self) -> Option<ActivePid> {
        self.parent_active_pid
    }

    /// Returns the latest observed current working directory.
    pub fn ai_current_dir(&self) -> &AiCurrentDir {
        &self.ai_current_dir
    }

    /// Returns the `SessionStart.source` that established the current record shape.
    pub fn session_start_source(&self) -> SessionStartSource {
        self.session_start_source
    }

    /// Returns the normalized agent state.
    pub fn agent_state(&self) -> AgentState {
        self.agent_state
    }

    /// Returns the schema version for the serialized record.
    pub fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    /// Returns the upstream provider name.
    pub fn provider(&self) -> Provider {
        self.provider
    }

    /// Returns the monotonic state revision.
    pub fn state_revision(&self) -> StateRevision {
        self.state_revision
    }

    /// Returns the original creation timestamp.
    pub fn created_at(&self) -> &UtcTimestamp {
        &self.created_at
    }

    /// Returns the timestamp of the latest state mutation.
    pub fn updated_at(&self) -> &UtcTimestamp {
        &self.updated_at
    }

    /// Returns the ended timestamp when the session has terminated.
    pub fn ended_at(&self) -> Option<&UtcTimestamp> {
        self.ended_at.as_ref()
    }

    /// Returns the hook event name that produced the latest mutation.
    pub fn last_hook_event(&self) -> &str {
        self.last_hook_event.as_str()
    }

    /// Returns the timestamp of the latest hook event recorded in state.
    pub fn last_hook_event_at(&self) -> &UtcTimestamp {
        &self.last_hook_event_at
    }

    /// Returns the human-readable reason associated with the latest state mutation.
    pub fn state_reason(&self) -> &str {
        self.state_reason.as_str()
    }

    /// Returns the extension map attached to the record.
    pub fn extensions(&self) -> &BTreeMap<String, Value> {
        &self.extensions
    }

    /// Returns a single extension value by key.
    pub fn extension(&self, key: &str) -> Option<&Value> {
        self.extensions.get(key)
    }

    /// Returns whether the record is already in terminal ended state.
    pub fn is_ended(&self) -> bool {
        self.agent_state == AgentState::Ended
    }

    /// Converts the record into the active-session mutation wrapper.
    #[allow(clippy::result_large_err)]
    pub fn try_into_active(self) -> Result<ActiveSessionRecord, EndedSessionRecord> {
        if self.is_ended() {
            Err(EndedSessionRecord::from_validated(self).expect("validated ended record"))
        } else {
            Ok(ActiveSessionRecord::from_validated(self).expect("validated active record"))
        }
    }

    /// Validates record invariants prior to persistence.
    pub fn validate(&self) -> Result<(), HookError> {
        validate_timestamp("created_at", &self.created_at)?;
        validate_timestamp("updated_at", &self.updated_at)?;
        StateRevision::new(self.state_revision.get())?;
        HookEventName::new(self.last_hook_event.as_str())?;
        validate_timestamp("last_hook_event_at", &self.last_hook_event_at)?;
        StateReason::new(self.state_reason.as_str())?;
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

impl ActiveSessionRecord {
    fn from_validated(record: CanonicalSessionRecord) -> Result<Self, HookError> {
        record.validate()?;
        if matches!(record.agent_state, AgentState::Ended) {
            return Err(HookError::validation(
                "agent_state",
                "active session record cannot wrap AgentState::Ended",
            ));
        }
        Ok(Self(record))
    }

    /// Consumes the active wrapper and returns the canonical record.
    pub fn into_inner(self) -> CanonicalSessionRecord {
        self.0
    }

    /// Sets or replaces an extension value and reports whether the record changed.
    pub fn set_extension(
        &mut self,
        key: impl Into<String>,
        value: Value,
    ) -> Result<bool, HookError> {
        let key = key.into();
        if self.0.extensions.get(&key) == Some(&value) {
            return Ok(false);
        }
        let mut next = self.0.clone();
        next.extensions.insert(key, value);
        next.validate()?;
        self.0 = next;
        Ok(true)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "root-change rebuild preserves canonical identity and timestamps under one validated constructor path"
    )]
    /// Rebuilds the record after a root-establishing transition such as resume or clear.
    pub fn rebuild_with_root_change(
        self,
        active_pid: ActivePid,
        ai_root_dir: AiRootDir,
        ai_current_dir: AiCurrentDir,
        session_start_source: SessionStartSource,
        agent_state: AgentState,
        last_hook_event: impl Into<String>,
        state_reason: impl Into<String>,
        ended_at: Option<UtcTimestamp>,
        updated_at: UtcTimestamp,
    ) -> Result<SessionTransitionResult, HookError> {
        let last_hook_event = HookEventName::new(last_hook_event.into())?;
        let state_reason = StateReason::new(state_reason.into())?;
        let record = CanonicalSessionRecord {
            schema_version: self.0.schema_version,
            provider: self.0.provider,
            session_id: self.0.session_id.clone(),
            active_pid,
            parent_session_id: self.0.parent_session_id.clone(),
            parent_active_pid: self.0.parent_active_pid,
            ai_root_dir,
            ai_current_dir,
            session_start_source,
            agent_state,
            state_revision: StateRevision::new(self.0.state_revision.get() + 1)?,
            created_at: self.0.created_at.clone(),
            updated_at: updated_at.clone(),
            ended_at,
            last_hook_event,
            last_hook_event_at: updated_at,
            state_reason,
            extensions: self.0.extensions.clone(),
        };
        record.validate()?;
        if record.is_ended() {
            Ok(SessionTransitionResult::Ended(
                EndedSessionRecord::from_validated(record)?,
            ))
        } else {
            Ok(SessionTransitionResult::Active(
                ActiveSessionRecord::from_validated(record)?,
            ))
        }
    }

    /// Bumps the record revision and updates the mutation timestamp without changing semantic fields.
    pub fn mark_material_change(&mut self, updated_at: UtcTimestamp) -> Result<(), HookError> {
        let mut next = self.0.clone();
        next.state_revision = StateRevision::new(next.state_revision.get() + 1)?;
        next.updated_at = updated_at;
        next.validate()?;
        self.0 = next;
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "hook-state updates need to carry the full validated canonical mutation set"
    )]
    /// Applies a validated hook-driven mutation to the canonical record.
    pub fn apply_hook_update(
        self,
        active_pid: ActivePid,
        ai_current_dir: AiCurrentDir,
        session_start_source: SessionStartSource,
        agent_state: AgentState,
        updated_at: UtcTimestamp,
        last_hook_event: impl Into<String>,
        state_reason: impl Into<String>,
        ended_at: Option<UtcTimestamp>,
    ) -> Result<SessionTransitionResult, HookError> {
        let mut next = self.0.clone();
        next.state_revision = StateRevision::new(next.state_revision.get() + 1)?;
        next.active_pid = active_pid;
        next.ai_current_dir = ai_current_dir;
        next.session_start_source = session_start_source;
        next.agent_state = agent_state;
        next.updated_at = updated_at.clone();
        next.last_hook_event = HookEventName::new(last_hook_event.into())?;
        next.last_hook_event_at = updated_at;
        next.state_reason = StateReason::new(state_reason.into())?;
        next.ended_at = ended_at;
        next.validate()?;
        if next.is_ended() {
            Ok(SessionTransitionResult::Ended(
                EndedSessionRecord::from_validated(next)?,
            ))
        } else {
            Ok(SessionTransitionResult::Active(
                ActiveSessionRecord::from_validated(next)?,
            ))
        }
    }
}

impl EndedSessionRecord {
    fn from_validated(record: CanonicalSessionRecord) -> Result<Self, HookError> {
        record.validate()?;
        if !matches!(record.agent_state, AgentState::Ended) {
            return Err(HookError::validation(
                "agent_state",
                "ended session record requires AgentState::Ended",
            ));
        }
        Ok(Self(record))
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
    validate_rfc3339_timestamp(field, value.as_str())
}

fn validate_rfc3339_timestamp(field: &str, value: &str) -> Result<(), HookError> {
    validate_nonblank_text(field, value)?;
    OffsetDateTime::parse(value, &Rfc3339).map_err(|source| {
        HookError::validation_with_source(field, "must be a valid RFC 3339 timestamp", source)
    })?;
    Ok(())
}

/// Returns the current UTC timestamp in RFC 3339 format.
pub fn utc_timestamp_now() -> UtcTimestamp {
    let now = OffsetDateTime::now_utc();
    let rendered = now
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    UtcTimestamp::from_field("utc_timestamp", rendered).unwrap_or_else(|_| {
        UtcTimestamp::from_field("utc_timestamp", "1970-01-01T00:00:00Z")
            .expect("fallback timestamp must be valid")
    })
}

#[derive(Debug, Deserialize)]
struct CanonicalSessionRecordWire {
    schema_version: SchemaVersion,
    provider: Provider,
    session_id: SessionId,
    active_pid: ActivePid,
    #[serde(default)]
    parent_session_id: Option<SessionId>,
    #[serde(default)]
    parent_active_pid: Option<ActivePid>,
    #[serde(alias = "project_root_dir")]
    ai_root_dir: AiRootDir,
    ai_current_dir: AiCurrentDir,
    session_start_source: SessionStartSource,
    agent_state: AgentState,
    state_revision: StateRevision,
    #[serde(deserialize_with = "deserialize_created_at")]
    created_at: UtcTimestamp,
    #[serde(deserialize_with = "deserialize_updated_at")]
    updated_at: UtcTimestamp,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_ended_at")]
    ended_at: Option<UtcTimestamp>,
    last_hook_event: HookEventName,
    #[serde(deserialize_with = "deserialize_last_hook_event_at")]
    last_hook_event_at: UtcTimestamp,
    state_reason: StateReason,
    #[serde(default)]
    extensions: BTreeMap<String, Value>,
}

impl<'de> Deserialize<'de> for CanonicalSessionRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = CanonicalSessionRecordWire::deserialize(deserializer)?;
        let record = Self {
            schema_version: wire.schema_version,
            provider: wire.provider,
            session_id: wire.session_id,
            active_pid: wire.active_pid,
            parent_session_id: wire.parent_session_id,
            parent_active_pid: wire.parent_active_pid,
            ai_root_dir: wire.ai_root_dir,
            ai_current_dir: wire.ai_current_dir,
            session_start_source: wire.session_start_source,
            agent_state: wire.agent_state,
            state_revision: wire.state_revision,
            created_at: wire.created_at,
            updated_at: wire.updated_at,
            ended_at: wire.ended_at,
            last_hook_event: wire.last_hook_event,
            last_hook_event_at: wire.last_hook_event_at,
            state_reason: wire.state_reason,
            extensions: wire.extensions,
        };
        record.validate().map_err(serde::de::Error::custom)?;
        Ok(record)
    }
}

fn deserialize_created_at<'de, D>(deserializer: D) -> Result<UtcTimestamp, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_timestamp_field(deserializer, "created_at")
}

fn deserialize_updated_at<'de, D>(deserializer: D) -> Result<UtcTimestamp, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_timestamp_field(deserializer, "updated_at")
}

fn deserialize_last_hook_event_at<'de, D>(deserializer: D) -> Result<UtcTimestamp, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_timestamp_field(deserializer, "last_hook_event_at")
}

fn deserialize_optional_ended_at<'de, D>(deserializer: D) -> Result<Option<UtcTimestamp>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|value| UtcTimestamp::from_field("ended_at", value))
        .transpose()
        .map_err(serde::de::Error::custom)
}

fn deserialize_timestamp_field<'de, D>(
    deserializer: D,
    field: &'static str,
) -> Result<UtcTimestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    UtcTimestamp::from_field(field, value).map_err(serde::de::Error::custom)
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

    #[test]
    fn session_start_source_deserializes_unknown_values_to_unknown() {
        let reparsed: SessionStartSource =
            serde_json::from_str("\"future_source\"").expect("source should deserialize");
        assert_eq!(reparsed, SessionStartSource::Unknown);
    }

    #[test]
    fn agent_state_deserializes_unknown_values_to_unknown() {
        let reparsed: AgentState =
            serde_json::from_str("\"future_state\"").expect("state should deserialize");
        assert_eq!(reparsed, AgentState::Unknown);
    }

    #[test]
    fn validate_rejects_ended_at_when_agent_state_is_not_ended() {
        let temp = tempfile::tempdir().expect("tempdir");
        let err = serde_json::from_value::<CanonicalSessionRecord>(serde_json::json!({
            "schema_version": "v1",
            "provider": "claude",
            "session_id": "session-ended-at-mismatch",
            "active_pid": 10,
            "ai_root_dir": temp.path().join("repo"),
            "ai_current_dir": temp.path().join("repo"),
            "session_start_source": "startup",
            "agent_state": "starting",
            "state_revision": 1,
            "created_at": "2026-03-30T00:00:00Z",
            "updated_at": "2026-03-30T00:00:00Z",
            "ended_at": "2026-03-30T00:00:01Z",
            "last_hook_event": "SessionStart",
            "last_hook_event_at": "2026-03-30T00:00:00Z",
            "state_reason": "session_started",
            "extensions": {}
        }))
        .expect_err("mismatched ended_at should fail validation");

        assert!(
            err.to_string()
                .contains("must be absent unless agent_state is ended"),
            "unexpected validation error: {err}"
        );
    }
}
