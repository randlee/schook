use std::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::HookError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
/// Validated tool name captured from hook payloads.
pub struct ToolName(String);

impl ToolName {
    /// Creates a validated non-empty tool name.
    pub fn new(value: impl Into<String>) -> Result<Self, HookError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(HookError::validation("tool_name", "must be non-empty"));
        }
        Ok(Self(value))
    }

    /// Returns the tool name as a borrowed string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Spawn flavor used by the agent spawn gate and ATM relay.
pub enum SpawnKind {
    /// Named foreground agent spawn.
    NamedAgent,
    /// Background agent spawn.
    BackgroundAgent,
}

impl SpawnKind {
    /// Returns the serialized spawn-kind value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NamedAgent => "named_agent",
            Self::BackgroundAgent => "background_agent",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_name_rejects_empty_values() {
        let err = ToolName::new(" ").expect_err("empty tool name should fail");
        assert!(err.to_string().contains("tool_name"));
    }
}
