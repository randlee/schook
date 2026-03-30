use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HookType {
    PreToolUse,
    PostToolUse,
    PreCompact,
    PostCompact,
    SessionStart,
    SessionEnd,
    Notification,
    TeammateIdle,
    PermissionRequest,
    Stop,
}

impl HookType {
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
pub enum EventTaxonomy {
    Bash,
    Read,
    Write,
    Edit,
    Glob,
    Grep,
    WebFetch,
    WebSearch,
    Agent,
    NotebookEdit,
    TodoWrite,
    AskFollowup,
    SendMessage,
    Task,
    IdlePrompt,
    Wildcard,
}

impl EventTaxonomy {
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
