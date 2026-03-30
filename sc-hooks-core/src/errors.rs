use std::path::PathBuf;

use thiserror::Error;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

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

    #[error("divergence in CLAUDE_PROJECT_DIR from {immutable_root} to {observed} on {hook_event}")]
    RootDivergence {
        immutable_root: PathBuf,
        observed: PathBuf,
        hook_event: String,
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
        immutable_root: impl Into<PathBuf>,
        observed: impl Into<PathBuf>,
        hook_event: impl Into<String>,
    ) -> Self {
        Self::RootDivergence {
            immutable_root: immutable_root.into(),
            observed: observed.into(),
            hook_event: hook_event.into(),
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
