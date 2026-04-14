use std::backtrace::Backtrace;
use std::path::PathBuf;

use crate::events::HookType;
use crate::session::{AiCurrentDir, AiRootDir, SessionId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;
const ROOT_DIVERGENCE_NOTICE_PREFIX: &str = "sc-hooks.root_divergence=";

/// Structured notice emitted when inbound `CLAUDE_PROJECT_DIR` diverges from immutable root state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootDivergenceNotice {
    /// Canonical immutable runtime root.
    pub immutable_root: AiRootDir,
    /// Divergent project directory reported by the provider.
    pub observed: AiCurrentDir,
    /// Session identifier associated with the divergence.
    pub session_id: SessionId,
    /// Hook event that surfaced the divergence.
    pub hook_event: HookType,
}

impl RootDivergenceNotice {
    /// Builds a structured divergence notice from canonical runtime values.
    ///
    /// # Errors
    ///
    /// Returns an error when the observed project directory cannot be validated
    /// as an `AiCurrentDir`.
    pub fn new(
        immutable_root: AiRootDir,
        observed: impl Into<PathBuf>,
        session_id: SessionId,
        hook_event: HookType,
    ) -> Result<Self, HookError> {
        Ok(Self {
            immutable_root,
            observed: AiCurrentDir::new(observed.into())?,
            session_id,
            hook_event,
        })
    }

    /// Serializes the notice into the prefixed string format used in logs and stderr.
    ///
    /// # Errors
    ///
    /// Returns an internal hook error when the notice cannot be serialized to
    /// JSON.
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
            self.observed.as_path().display(),
            self.hook_event
        )
    }
}

/// Payload and validation failures surfaced across the hook runtime.
#[derive(Debug, Error)]
pub enum PayloadError {
    /// Hook payload JSON could not be parsed or validated.
    #[error("invalid payload near {input_excerpt}")]
    InvalidPayload {
        /// Short excerpt of the offending input body.
        input_excerpt: String,
        /// Underlying serde parser error when one is available.
        #[source]
        source: Option<serde_json::Error>,
    },

    /// Hook context construction failed before runtime dispatch.
    #[error("invalid context: {message}")]
    InvalidContext {
        /// Human-readable validation message.
        message: String,
        /// Underlying source error when one is available.
        #[source]
        source: Option<BoxedError>,
    },

    /// A named field failed runtime validation.
    #[error("validation failed for {field}: {message}")]
    Validation {
        /// Field name or logical field path.
        field: String,
        /// Human-readable validation message.
        message: String,
        /// Underlying source error when one is available.
        #[source]
        source: Option<BoxedError>,
    },
}

impl PayloadError {
    /// Creates an `InvalidPayload` error without a source.
    pub fn invalid_payload(input_excerpt: impl Into<String>) -> Self {
        Self::InvalidPayload {
            input_excerpt: input_excerpt.into(),
            source: None,
        }
    }

    /// Creates an `InvalidPayload` error that preserves an underlying source.
    pub fn invalid_payload_with_source(
        input_excerpt: impl Into<String>,
        source: serde_json::Error,
    ) -> Self {
        Self::InvalidPayload {
            input_excerpt: input_excerpt.into(),
            source: Some(source),
        }
    }

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
}

/// Runtime and persistence failures surfaced across the hook runtime.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Session-state I/O failed for a specific path.
    #[error("state I/O failed for {path}")]
    StateIo {
        /// State path involved in the failed operation.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
        /// Captured backtrace for diagnostics when backtraces are enabled.
        backtrace: Box<Backtrace>,
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
        /// Underlying source error when one is available.
        #[source]
        source: Option<BoxedError>,
        /// Captured backtrace for diagnostics when backtraces are enabled.
        backtrace: Box<Backtrace>,
    },
}

impl RuntimeError {
    /// Creates an `Internal` error without a source.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
            backtrace: Box::new(Backtrace::capture()),
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
            backtrace: Box::new(Backtrace::capture()),
        }
    }

    /// Creates a `StateIo` error for a concrete filesystem path.
    pub fn state_io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::StateIo {
            path: path.into(),
            source,
            backtrace: Box::new(Backtrace::capture()),
        }
    }

    /// Returns the captured backtrace when one exists on this runtime error.
    ///
    /// `StateIo` and `Internal` always carry a captured backtrace;
    /// `RootDivergence` never does.
    pub fn backtrace(&self) -> Option<&Backtrace> {
        match self {
            Self::StateIo { backtrace, .. } | Self::Internal { backtrace, .. } => {
                Some(backtrace.as_ref())
            }
            Self::RootDivergence { .. } => None,
        }
    }
}

/// Compatibility wrapper that preserves the historical cross-crate hook error surface.
#[derive(Debug, Error)]
pub enum HookError {
    /// Payload or validation failure.
    #[error(transparent)]
    Payload(#[from] PayloadError),
    /// Runtime or persistence failure.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

/// SDK handler-facing error alias retained while runtime crates migrate to the
/// split taxonomy explicitly.
pub type HandlerError = HookError;

impl HookError {
    /// Creates an `InvalidPayload` error without a source.
    pub fn invalid_payload(input_excerpt: impl Into<String>) -> Self {
        PayloadError::invalid_payload(input_excerpt).into()
    }

    /// Creates an `InvalidPayload` error that preserves an underlying source.
    pub fn invalid_payload_with_source(
        input_excerpt: impl Into<String>,
        source: serde_json::Error,
    ) -> Self {
        PayloadError::invalid_payload_with_source(input_excerpt, source).into()
    }

    /// Creates an `InvalidContext` error without a source.
    pub fn invalid_context(message: impl Into<String>) -> Self {
        PayloadError::invalid_context(message).into()
    }

    /// Creates a `Validation` error without a source.
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        PayloadError::validation(field, message).into()
    }

    /// Creates an `InvalidContext` error that preserves an underlying source.
    pub fn invalid_context_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        PayloadError::invalid_context_with_source(message, source).into()
    }

    /// Creates a `Validation` error that preserves an underlying source.
    pub fn validation_with_source(
        field: impl Into<String>,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        PayloadError::validation_with_source(field, message, source).into()
    }

    /// Creates an `Internal` error without a source.
    pub fn internal(message: impl Into<String>) -> Self {
        RuntimeError::internal(message).into()
    }

    /// Creates a `RootDivergence` error from canonical root values.
    pub fn root_divergence(
        immutable_root: AiRootDir,
        observed: impl Into<PathBuf>,
        hook_event: HookType,
    ) -> Self {
        RuntimeError::root_divergence(immutable_root, observed, hook_event).into()
    }

    /// Creates an `Internal` error that preserves an underlying source.
    pub fn internal_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        RuntimeError::internal_with_source(message, source).into()
    }

    /// Creates a `StateIo` error for a concrete filesystem path.
    pub fn state_io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        RuntimeError::state_io(path, source).into()
    }

    /// Returns the payload/validation half of the split taxonomy when present.
    pub fn as_payload(&self) -> Option<&PayloadError> {
        match self {
            Self::Payload(error) => Some(error),
            Self::Runtime(_) => None,
        }
    }

    /// Returns the runtime/persistence half of the split taxonomy when present.
    pub fn as_runtime(&self) -> Option<&RuntimeError> {
        match self {
            Self::Payload(_) => None,
            Self::Runtime(error) => Some(error),
        }
    }

    /// Returns the captured backtrace when the wrapped runtime error carries one.
    ///
    /// Payload errors never carry a captured backtrace; only runtime errors do.
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.as_runtime().and_then(RuntimeError::backtrace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::HookType;

    #[test]
    fn root_divergence_notice_round_trips_through_encode_and_decode() {
        let notice = RootDivergenceNotice::new(
            AiRootDir::new("/repo").expect("root"),
            "/repo/subdir",
            SessionId::new("session-1").expect("session"),
            HookType::SessionStart,
        )
        .expect("notice should construct");

        let encoded = notice.encode().expect("notice should encode");
        let decoded = RootDivergenceNotice::decode(&encoded).expect("notice should decode");
        assert_eq!(decoded, notice);
    }

    #[test]
    fn root_divergence_notice_decode_rejects_unprefixed_payloads() {
        assert!(RootDivergenceNotice::decode("{\"hook_event\":\"SessionStart\"}").is_none());
    }

    #[test]
    fn root_divergence_notice_warning_message_mentions_all_key_fields() {
        let notice = RootDivergenceNotice::new(
            AiRootDir::new("/repo").expect("root"),
            "/other",
            SessionId::new("session-2").expect("session"),
            HookType::PostToolUse,
        )
        .expect("notice should construct");

        let warning = notice.warning_message();
        assert!(warning.contains("/repo"));
        assert!(warning.contains("/other"));
        assert!(warning.contains("PostToolUse"));
    }

    #[test]
    fn hook_error_constructors_cover_all_variants() {
        let invalid_context = HookError::invalid_context("bad");
        assert!(matches!(
            invalid_context,
            HookError::Payload(PayloadError::InvalidContext { .. })
        ));

        let invalid_context_with_source =
            HookError::invalid_context_with_source("bad", std::io::Error::other("source"));
        assert!(matches!(
            invalid_context_with_source,
            HookError::Payload(PayloadError::InvalidContext {
                source: Some(_),
                ..
            })
        ));

        let validation = HookError::validation("field", "invalid");
        assert!(matches!(
            validation,
            HookError::Payload(PayloadError::Validation { .. })
        ));

        let validation_with_source =
            HookError::validation_with_source("field", "invalid", std::io::Error::other("source"));
        assert!(matches!(
            validation_with_source,
            HookError::Payload(PayloadError::Validation {
                source: Some(_),
                ..
            })
        ));

        let invalid_payload = HookError::invalid_payload("{oops");
        assert!(matches!(
            invalid_payload,
            HookError::Payload(PayloadError::InvalidPayload { source: None, .. })
        ));

        let internal = HookError::internal("boom");
        assert!(matches!(
            internal,
            HookError::Runtime(RuntimeError::Internal { .. })
        ));
        assert!(internal.backtrace().is_some());

        let internal_with_source =
            HookError::internal_with_source("boom", std::io::Error::other("source"));
        assert!(matches!(
            internal_with_source,
            HookError::Runtime(RuntimeError::Internal {
                source: Some(_),
                ..
            })
        ));
        assert!(internal_with_source.backtrace().is_some());

        let state_path = std::env::temp_dir().join("state.json");
        let state_io = HookError::state_io(state_path, std::io::Error::other("disk"));
        assert!(matches!(
            state_io,
            HookError::Runtime(RuntimeError::StateIo { .. })
        ));
        assert!(state_io.backtrace().is_some());

        let divergence = HookError::root_divergence(
            AiRootDir::new("/repo").expect("root"),
            "/repo/subdir",
            HookType::SessionStart,
        );
        assert!(matches!(
            divergence,
            HookError::Runtime(RuntimeError::RootDivergence { .. })
        ));
        assert!(divergence.backtrace().is_none());
    }
}
