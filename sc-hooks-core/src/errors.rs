use std::path::PathBuf;

use crate::events::HookType;
use crate::session::{AiRootDir, SessionId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;
const ROOT_DIVERGENCE_NOTICE_PREFIX: &str = "sc-hooks.root_divergence=";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Structured notice emitted when inbound `CLAUDE_PROJECT_DIR` diverges from immutable root state.
pub struct RootDivergenceNotice {
    /// Canonical immutable runtime root.
    pub immutable_root: AiRootDir,
    /// Divergent project directory reported by the provider.
    pub observed: PathBuf,
    /// Session identifier associated with the divergence.
    pub session_id: SessionId,
    /// Hook event that surfaced the divergence.
    pub hook_event: HookType,
}

impl RootDivergenceNotice {
    /// Builds a structured divergence notice from canonical runtime values.
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

    /// Serializes the notice into the prefixed string format used in logs and stderr.
    pub fn encode(&self) -> Result<String, HookError> {
        let encoded = serde_json::to_string(self).map_err(|source| {
            HookError::internal_with_source("failed to serialize root divergence notice", source)
        })?;
        Ok(format!("{ROOT_DIVERGENCE_NOTICE_PREFIX}{encoded}"))
    }

    /// Deserializes a prefixed divergence notice from a string payload.
    pub fn decode(value: &str) -> Option<Self> {
        let payload = value.strip_prefix(ROOT_DIVERGENCE_NOTICE_PREFIX)?;
        serde_json::from_str(payload).ok()
    }

    /// Formats the human-readable warning text associated with the notice.
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
/// Shared error type for hook parsing, validation, persistence, and runtime failures.
pub enum HookError {
    /// Hook payload JSON could not be parsed or validated.
    #[error("invalid payload near {input_excerpt}")]
    InvalidPayload {
        /// Short excerpt of the offending input body.
        input_excerpt: String,
        #[source]
        /// Underlying serde parser error when one is available.
        source: Option<serde_json::Error>,
    },

    /// Hook context construction failed before runtime dispatch.
    #[error("invalid context: {message}")]
    InvalidContext {
        /// Human-readable validation message.
        message: String,
        #[source]
        /// Underlying source error when one is available.
        source: Option<BoxedError>,
    },

    /// Session-state I/O failed for a specific path.
    #[error("state I/O failed for {path}")]
    StateIo {
        /// State path involved in the failed operation.
        path: PathBuf,
        #[source]
        /// Underlying filesystem error.
        source: std::io::Error,
    },

    /// A named field failed runtime validation.
    #[error("validation failed for {field}: {message}")]
    Validation {
        /// Field name or logical field path.
        field: String,
        /// Human-readable validation message.
        message: String,
        #[source]
        /// Underlying source error when one is available.
        source: Option<BoxedError>,
    },
    /// Added in S10-R2 to represent a mismatch between immutable
    /// `ai_root_dir` and inbound `CLAUDE_PROJECT_DIR`. The runtime continues
    /// with the immutable root, but dispatch must emit a prominent structured
    /// observability event for investigation.
    #[error("divergence in CLAUDE_PROJECT_DIR from {immutable_root} to {observed} on {hook_event}")]
    RootDivergence {
        /// Canonical immutable root recorded for the session.
        immutable_root: AiRootDir,
        /// Divergent project directory reported by the provider.
        observed: PathBuf,
        /// Hook event that surfaced the divergence.
        hook_event: HookType,
    },

    /// Internal host failure that does not map to a more specific variant.
    #[error("internal hook error: {message}")]
    Internal {
        /// Human-readable internal error message.
        message: String,
        #[source]
        /// Underlying source error when one is available.
        source: Option<BoxedError>,
    },
}

impl HookError {
    /// Creates an `InvalidContext` error without a source.
    pub fn invalid_context(message: impl Into<String>) -> Self {
        Self::InvalidContext {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a `Validation` error without a source.
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Creates an `InvalidContext` error that preserves an underlying source.
    pub fn invalid_context_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::InvalidContext {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates a `Validation` error that preserves an underlying source.
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

    /// Creates an `Internal` error without a source.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a `RootDivergence` error from canonical root values.
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

    /// Creates an `Internal` error that preserves an underlying source.
    pub fn internal_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Creates a `StateIo` error for a concrete filesystem path.
    pub fn state_io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::StateIo {
            path: path.into(),
            source,
        }
    }
}
