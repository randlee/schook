//! Error types and constructor helpers for the `sc-hooks` CLI.

use std::sync::Arc;

use thiserror::Error;

use crate::config::ConfigError;
use crate::observability::ObservabilityInitError;
use sc_hooks_sdk::manifest::ManifestLoadError;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// Resolution-time failures encountered before a handler is executed.
#[derive(Debug, Error)]
pub enum ResolutionError {
    /// A configured handler could not be mapped to a runtime plugin executable.
    #[error("handler `{handler}` could not be resolved")]
    UnresolvedHandler { handler: String },

    /// A plugin manifest could not be loaded or validated during resolution.
    #[error(
        "plugin `{plugin}` manifest load failed{source_chain}",
        source_chain = format_source_chain(source)
    )]
    ManifestLoadFailed {
        /// Name of the plugin whose manifest load failed.
        plugin: String,
        #[source]
        /// Underlying manifest load or validation failure.
        source: ManifestLoadError,
    },

    /// A handler was rejected for dispatch after resolution completed.
    #[error(
        "handler `{plugin}` rejected for dispatch: {reason}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    HandlerRejected {
        /// Name of the rejected plugin.
        plugin: String,
        /// Human-readable rejection reason.
        reason: String,
        #[source]
        /// Optional lower-level cause for the rejection.
        source: Option<BoxedError>,
    },
}

/// Validation failures for required handler metadata.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// A handler-specific required field was not present in the prepared metadata.
    #[error("handler `{handler}` is missing required metadata field `{field}`")]
    MissingField { handler: String, field: String },

    /// A handler-specific required field was present but invalid.
    #[error("handler `{handler}` has invalid metadata field `{field}`: {reason}")]
    InvalidField {
        /// Handler that failed validation.
        handler: String,
        /// Metadata field that failed validation.
        field: String,
        /// Validation failure detail.
        reason: String,
    },
}

/// Top-level CLI error taxonomy and exit-code mapping surface.
#[derive(Debug, Error)]
pub enum CliError {
    /// Configuration loading or parsing failed.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Handler resolution failed before execution.
    #[error(transparent)]
    Resolution(#[from] ResolutionError),

    /// Required metadata validation failed.
    #[error(transparent)]
    Validation(#[from] ValidationError),

    /// Dispatch was intentionally blocked.
    #[error(
        "action blocked: {reason}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    Blocked {
        /// Human-readable block reason.
        reason: String,
        #[source]
        /// Optional lower-level cause for the block.
        source: Option<BoxedError>,
    },

    /// A plugin runtime or protocol failure occurred.
    #[error(
        "plugin error: {message}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    PluginError {
        /// Human-readable plugin failure detail.
        message: String,
        #[source]
        /// Optional lower-level cause for the plugin failure.
        source: Option<BoxedError>,
    },

    /// Dispatch exceeded its timeout budget.
    #[error(
        "operation timed out: {message}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    Timeout {
        /// Human-readable timeout detail.
        message: String,
        #[source]
        /// Optional lower-level cause for the timeout.
        source: Option<BoxedError>,
    },

    /// Static audit execution failed.
    #[error(
        "audit failed: {message}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    AuditFailure {
        /// Human-readable audit failure detail.
        message: String,
        #[source]
        /// Optional lower-level cause for the audit failure.
        source: Option<BoxedError>,
    },

    /// Observability initialization failed before logging could begin.
    #[error("observability initialization failed: {source}")]
    ObservabilityInit {
        #[source]
        /// Cached underlying observability initialization error.
        source: Arc<ObservabilityInitError>,
    },

    /// An internal host error occurred outside the more specific variants above.
    #[error(
        "{message}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    Internal {
        /// Human-readable internal error detail.
        message: String,
        #[source]
        /// Optional lower-level cause for the internal error.
        source: Option<BoxedError>,
    },
}

impl CliError {
    /// Creates an internal error without a source.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Creates an internal error with a source.
    pub fn internal_with_source(message: impl Into<String>, source: impl Into<BoxedError>) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Creates a blocked error without a source.
    pub fn blocked(reason: impl Into<String>) -> Self {
        Self::Blocked {
            reason: reason.into(),
            source: None,
        }
    }

    /// Creates a plugin error without a source.
    pub fn plugin_error(message: impl Into<String>) -> Self {
        Self::PluginError {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a plugin error with a source.
    pub fn plugin_error_with_source(
        message: impl Into<String>,
        source: impl Into<BoxedError>,
    ) -> Self {
        Self::PluginError {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Creates a timeout error without a source.
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
            source: None,
        }
    }

    /// Creates an audit failure without a source.
    pub fn audit_failure(message: impl Into<String>) -> Self {
        Self::AuditFailure {
            message: message.into(),
            source: None,
        }
    }

    /// Wraps a cached observability initialization error.
    pub fn observability_init(source: impl Into<Arc<ObservabilityInitError>>) -> Self {
        Self::ObservabilityInit {
            source: source.into(),
        }
    }

    /// Returns the exit code associated with this CLI error variant.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) => sc_hooks_core::exit_codes::CONFIG_ERROR,
            Self::Resolution(_) => sc_hooks_core::exit_codes::RESOLUTION_ERROR,
            Self::Validation(_) => sc_hooks_core::exit_codes::VALIDATION_ERROR,
            Self::Blocked { .. } => sc_hooks_core::exit_codes::BLOCKED,
            Self::PluginError { .. } => sc_hooks_core::exit_codes::PLUGIN_ERROR,
            Self::Timeout { .. } => sc_hooks_core::exit_codes::TIMEOUT,
            Self::AuditFailure { .. } => sc_hooks_core::exit_codes::AUDIT_FAILURE,
            Self::ObservabilityInit { .. } => sc_hooks_core::exit_codes::INTERNAL_ERROR,
            Self::Internal { .. } => sc_hooks_core::exit_codes::INTERNAL_ERROR,
        }
    }
}

fn format_optional_source(
    source: Option<&(dyn std::error::Error + Send + Sync + 'static)>,
) -> String {
    source
        .map(|err| format_source_chain(err))
        .unwrap_or_default()
}

fn format_source_chain(source: &(dyn std::error::Error + 'static)) -> String {
    let mut rendered = String::new();
    let mut current = Some(source);
    while let Some(err) = current {
        use std::fmt::Write as _;
        let _ = write!(&mut rendered, ": {err}");
        current = err.source();
    }
    rendered
}
