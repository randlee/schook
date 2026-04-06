use std::sync::Arc;

use thiserror::Error;

use crate::config::ConfigError;
use crate::observability::ObservabilityInitError;
use sc_hooks_sdk::manifest::ManifestLoadError;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("handler `{handler}` could not be resolved")]
    UnresolvedHandler { handler: String },

    #[error(
        "plugin `{plugin}` manifest load failed{source_chain}",
        source_chain = format_source_chain(source)
    )]
    ManifestLoadFailed {
        plugin: String,
        #[source]
        source: ManifestLoadError,
    },

    #[error(
        "handler `{plugin}` rejected for dispatch: {reason}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    HandlerRejected {
        plugin: String,
        reason: String,
        #[source]
        source: Option<BoxedError>,
    },
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

    #[error(
        "action blocked: {reason}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    Blocked {
        reason: String,
        #[source]
        source: Option<BoxedError>,
    },

    #[error(
        "plugin error: {message}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    PluginError {
        message: String,
        #[source]
        source: Option<BoxedError>,
    },

    #[error(
        "operation timed out: {message}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    Timeout {
        message: String,
        #[source]
        source: Option<BoxedError>,
    },

    #[error(
        "audit failed: {message}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    AuditFailure {
        message: String,
        #[source]
        source: Option<BoxedError>,
    },

    #[error("observability initialization failed: {source}")]
    ObservabilityInit {
        // Intentionally not marked as #[source] to avoid a circular error-source graph:
        // ObservabilityInitError already owns lower-level causes, and feeding it back through
        // CliError as a source would reintroduce the CLI/application boundary into that chain.
        source: Arc<ObservabilityInitError>,
    },

    #[error(
        "{message}{source_suffix}",
        source_suffix = format_optional_source(source.as_deref())
    )]
    Internal {
        message: String,
        #[source]
        source: Option<BoxedError>,
    },
}

impl CliError {
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

    pub fn blocked(reason: impl Into<String>) -> Self {
        Self::Blocked {
            reason: reason.into(),
            source: None,
        }
    }

    pub fn plugin_error(message: impl Into<String>) -> Self {
        Self::PluginError {
            message: message.into(),
            source: None,
        }
    }

    pub fn plugin_error_with_source(
        message: impl Into<String>,
        source: impl Into<BoxedError>,
    ) -> Self {
        Self::PluginError {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
            source: None,
        }
    }

    pub fn audit_failure(message: impl Into<String>) -> Self {
        Self::AuditFailure {
            message: message.into(),
            source: None,
        }
    }

    pub fn observability_init(source: impl Into<Arc<ObservabilityInitError>>) -> Self {
        Self::ObservabilityInit {
            source: source.into(),
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
