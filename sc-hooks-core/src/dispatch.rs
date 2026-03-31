use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Plugin dispatch mode.
pub enum DispatchMode {
    /// Synchronous hook execution.
    Sync,
    /// Asynchronous hook execution.
    Async,
}

impl DispatchMode {
    /// Returns the serialized dispatch-mode name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sync => "sync",
            Self::Async => "async",
        }
    }
}
