use thiserror::Error;

use crate::config::ConfigError;
use sc_hooks_sdk::manifest::ManifestLoadError;

#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("handler `{handler}` could not be resolved")]
    UnresolvedHandler { handler: String },

    #[error("plugin `{plugin}` manifest load failed")]
    ManifestLoad {
        plugin: String,
        #[source]
        source: ManifestLoadError,
    },

    #[error("plugin `{plugin}` was rejected during resolution: {reason}")]
    HandlerRejected { plugin: String, reason: String },
}

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

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Resolution(#[from] ResolutionError),

    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error("action blocked: {reason}")]
    Blocked { reason: String },

    #[error("plugin error: {message}")]
    PluginError { message: String },

    #[error("operation timed out: {message}")]
    Timeout { message: String },

    #[error("audit failed: {message}")]
    AuditFailure { message: String },

    #[error("{message}")]
    Internal { message: String },
}

impl CliError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) => sc_hooks_core::exit_codes::CONFIG_ERROR,
            Self::Resolution(_) => sc_hooks_core::exit_codes::RESOLUTION_ERROR,
            Self::Validation(_) => sc_hooks_core::exit_codes::VALIDATION_ERROR,
            Self::Blocked { .. } => sc_hooks_core::exit_codes::BLOCKED,
            Self::PluginError { .. } => sc_hooks_core::exit_codes::PLUGIN_ERROR,
            Self::Timeout { .. } => sc_hooks_core::exit_codes::TIMEOUT,
            Self::AuditFailure { .. } => sc_hooks_core::exit_codes::AUDIT_FAILURE,
            Self::Internal { .. } => sc_hooks_core::exit_codes::INTERNAL_ERROR,
        }
    }
}
