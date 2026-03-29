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
    InvalidContext { message: String },

    #[error("state I/O failed for {path}")]
    StateIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("validation failed for {field}: {message}")]
    Validation { field: String, message: String },

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
        }
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    pub fn internal_with_source(message: impl Into<String>, source: impl Into<BoxedError>) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    pub fn state_io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::StateIo {
            path: path.into(),
            source,
        }
    }
}
