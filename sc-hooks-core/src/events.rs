use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
/// Canonical hook names supported by the runtime.
pub enum HookType {
    /// Fires before a tool invocation.
    PreToolUse,
    /// Fires after a tool invocation.
    PostToolUse,
    /// Fires before Claude compacts context.
    PreCompact,
    /// Fires after Claude compacts context.
    PostCompact,
    /// Fires when Claude starts a session.
    SessionStart,
    /// Fires when Claude ends a session.
    SessionEnd,
    /// Fires for notification surfaces when payload support exists.
    Notification,
    /// Fires when a teammate agent becomes idle.
    TeammateIdle,
    /// Fires when Claude asks for user permission.
    PermissionRequest,
    /// Fires when Claude stops a turn or session.
    Stop,
}

impl HookType {
    /// Returns the provider-facing hook name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreToolUse => "PreToolUse",
            Self::PostToolUse => "PostToolUse",
            Self::PreCompact => "PreCompact",
            Self::PostCompact => "PostCompact",
            Self::SessionStart => "SessionStart",
            Self::SessionEnd => "SessionEnd",
            Self::Notification => "Notification",
            Self::TeammateIdle => "TeammateIdle",
            Self::PermissionRequest => "PermissionRequest",
            Self::Stop => "Stop",
        }
    }
}

impl fmt::Display for HookType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HookType {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "PreToolUse" => Ok(Self::PreToolUse),
            "PostToolUse" => Ok(Self::PostToolUse),
            "PreCompact" => Ok(Self::PreCompact),
            "PostCompact" => Ok(Self::PostCompact),
            "SessionStart" => Ok(Self::SessionStart),
            "SessionEnd" => Ok(Self::SessionEnd),
            "Notification" => Ok(Self::Notification),
            "TeammateIdle" => Ok(Self::TeammateIdle),
            "PermissionRequest" => Ok(Self::PermissionRequest),
            "Stop" => Ok(Self::Stop),
            _ => Err("unknown hook type"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
/// Canonical matcher/event taxonomy used for hook routing.
pub enum EventTaxonomy {
    /// Shell command execution.
    Bash,
    /// File read operation.
    Read,
    /// File write operation.
    Write,
    /// File edit operation.
    Edit,
    /// Glob pattern lookup.
    Glob,
    /// Grep or ripgrep search.
    Grep,
    /// Web fetch request.
    WebFetch,
    /// Web search request.
    WebSearch,
    /// Agent or subagent spawn.
    Agent,
    /// Notebook cell edit.
    NotebookEdit,
    /// Todo list write.
    TodoWrite,
    /// Follow-up question request.
    AskFollowup,
    /// Message send action.
    SendMessage,
    /// Historical task-tool surface.
    Task,
    /// Idle prompt event.
    IdlePrompt,
    /// Wildcard matcher.
    Wildcard,
}

impl EventTaxonomy {
    /// Returns the serialized matcher/event name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bash => "Bash",
            Self::Read => "Read",
            Self::Write => "Write",
            Self::Edit => "Edit",
            Self::Glob => "Glob",
            Self::Grep => "Grep",
            Self::WebFetch => "WebFetch",
            Self::WebSearch => "WebSearch",
            Self::Agent => "Agent",
            Self::NotebookEdit => "NotebookEdit",
            Self::TodoWrite => "TodoWrite",
            Self::AskFollowup => "AskFollowup",
            Self::SendMessage => "SendMessage",
            Self::Task => "Task",
            Self::IdlePrompt => "idle_prompt",
            Self::Wildcard => "*",
        }
    }
}
