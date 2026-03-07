use thiserror::Error;

use crate::config::ConfigError;
use crate::exit_codes;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("handler `{handler}` could not be resolved")]
    UnresolvedHandler { handler: String },
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("handler `{handler}` is missing required metadata field `{field}`")]
    MissingField { handler: String, field: String },

    #[error("handler `{handler}` has invalid metadata field `{field}`: {reason}")]
    InvalidField {
        handler: String,
        field: String,
        reason: String,
    },
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Resolution(#[from] ResolutionError),

    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error("{message}")]
    Internal { message: String },
}

#[allow(dead_code)]
impl CliError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) => exit_codes::CONFIG_ERROR,
            Self::Resolution(_) => exit_codes::RESOLUTION_ERROR,
            Self::Validation(_) => exit_codes::VALIDATION_ERROR,
            Self::Internal { .. } => exit_codes::INTERNAL_ERROR,
        }
    }
}
