use std::path::PathBuf;

use crate::events::HookType;
use crate::session::{AiRootDir, SessionId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;
const ROOT_DIVERGENCE_NOTICE_PREFIX: &str = "sc-hooks.root_divergence=";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootDivergenceNotice {
    pub immutable_root: AiRootDir,
    pub observed: PathBuf,
    pub session_id: SessionId,
    pub hook_event: HookType,
}

impl RootDivergenceNotice {
    pub fn new(
        immutable_root: AiRootDir,
        observed: impl Into<PathBuf>,
        session_id: SessionId,
        hook_event: HookType,
    ) -> Self {
        Self {
            immutable_root,
            observed: observed.into(),
            session_id,
            hook_event,
        }
    }

    pub fn encode(&self) -> Result<String, HookError> {
        let encoded = serde_json::to_string(self).map_err(|source| {
            HookError::internal_with_source("failed to serialize root divergence notice", source)
        })?;
        Ok(format!("{ROOT_DIVERGENCE_NOTICE_PREFIX}{encoded}"))
    }

    pub fn decode(value: &str) -> Option<Self> {
        let payload = value.strip_prefix(ROOT_DIVERGENCE_NOTICE_PREFIX)?;
        serde_json::from_str(payload).ok()
    }

    pub fn warning_message(&self) -> String {
        format!(
            "divergence in CLAUDE_PROJECT_DIR from {} to {} on {}",
            self.immutable_root,
            self.observed.display(),
            self.hook_event
        )
    }
}

#[derive(Debug, Error)]
pub enum HookError {
    #[error("invalid payload near {input_excerpt}")]
    InvalidPayload {
        input_excerpt: String,
        #[source]
        source: Option<serde_json::Error>,
    },

    #[error("invalid context: {message}")]
    InvalidContext {
        message: String,
        #[source]
        source: Option<BoxedError>,
    },

    #[error("state I/O failed for {path}")]
    StateIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("validation failed for {field}: {message}")]
    Validation {
        field: String,
        message: String,
        #[source]
        source: Option<BoxedError>,
    },

    /// Added in S10-R2 to represent a mismatch between immutable
    /// `ai_root_dir` and inbound `CLAUDE_PROJECT_DIR`. The runtime continues
    /// with the immutable root, but dispatch must emit a prominent structured
    /// observability event for investigation.
    #[error("divergence in CLAUDE_PROJECT_DIR from {immutable_root} to {observed} on {hook_event}")]
    RootDivergence {
        immutable_root: AiRootDir,
        observed: PathBuf,
        hook_event: HookType,
    },

    #[error("internal hook error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<BoxedError>,
    },
}

impl HookError {
    pub fn invalid_context(message: impl Into<String>) -> Self {
        Self::InvalidContext {
            message: message.into(),
            source: None,
        }
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn invalid_context_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::InvalidContext {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn validation_with_source(
        field: impl Into<String>,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    pub fn root_divergence(
        immutable_root: AiRootDir,
        observed: impl Into<PathBuf>,
        hook_event: HookType,
    ) -> Self {
        Self::RootDivergence {
            immutable_root,
            observed: observed.into(),
            hook_event,
        }
    }

    pub fn internal_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn state_io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::StateIo {
            path: path.into(),
            source,
        }
    }
}
