use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::OnceLock;

use log::warn;
use sc_hooks_core::errors::RootDivergenceNotice;
use sc_observability::{Logger, LoggerConfig};
use sc_observability_types::{
    ActionName, Level, LevelFilter, LogEvent, ProcessIdentity, ServiceName, TargetCategory,
};
use serde::Serialize;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::errors::CliError;
use sc_hooks_core::session::AiRootDir;
const SERVICE_NAME: &str = "sc-hooks";
static LOGGER: OnceLock<Logger> = OnceLock::new();
static LOGGER_ROOT: OnceLock<AiRootDir> = OnceLock::new();

#[derive(Debug, Error)]
enum ObservabilityInitError {
    #[error("invalid service name: {source}")]
    InvalidServiceName {
        #[source]
        source: sc_observability_types::ValueValidationError,
    },
    #[error(
        "project_root mismatch for cached logger: initialized at {initialized}, requested {requested}"
    )]
    ProjectRootMismatch {
        initialized: PathBuf,
        requested: PathBuf,
    },
    #[error("failed resolving observability root")]
    ResolveRoot {
        #[source]
        source: CliError,
    },
    #[error("failed to initialize observability: {source}")]
    LoggerInit {
        #[source]
        source: sc_observability_types::InitError,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
/// Structured per-handler dispatch outcome captured in observability events.
pub struct HandlerResultRecord {
    pub handler: String,
    pub action: Cow<'static, str>,
    pub ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// Arguments required to emit one `dispatch.complete` observability event.
pub struct DispatchEventArgs<'a> {
    pub hook: &'a str,
    pub event: Option<&'a str>,
    pub matcher: &'a str,
    pub mode: sc_hooks_core::dispatch::DispatchMode,
    pub handler_chain: &'a [String],
    pub results: &'a [HandlerResultRecord],
    pub total_ms: u128,
    pub exit: i32,
    pub ai_notification: Option<&'a str>,
    pub project_root: &'a AiRootDir,
}

/// Arguments required to emit one `session.root_divergence` observability event.
pub struct RootDivergenceEventArgs<'a> {
    pub notice: &'a RootDivergenceNotice,
    pub project_root: &'a AiRootDir,
}

/// Emits the canonical `dispatch.complete` observability event for one host dispatch.
///
/// Callers must pass the real dispatch project root for every invocation.
/// The logger root is cached process-wide, so falling back to `current_dir()`
/// would reintroduce cwd-dependent nondeterminism after initialization.
///
/// # Errors
///
/// Returns an error when logger initialization fails, when structured event
/// fields cannot be serialized, or when the underlying observability sink
/// fails during emit or flush.
pub fn emit_dispatch_event(args: DispatchEventArgs<'_>) -> Result<(), CliError> {
    let service = ServiceName::new(SERVICE_NAME)
        .map_err(|source| CliError::internal_with_source("invalid service name", source))?;
    let logger = logger(args.project_root)?;
    let target = TargetCategory::new("hook")
        .map_err(|source| CliError::internal_with_source("invalid log target", source))?;
    let action = ActionName::new("dispatch.complete")
        .map_err(|source| CliError::internal_with_source("invalid log action", source))?;

    let mut fields = Map::new();
    fields.insert("hook".to_string(), Value::String(args.hook.to_string()));
    if let Some(event) = args.event {
        fields.insert("event".to_string(), Value::String(event.to_string()));
    }
    fields.insert(
        "matcher".to_string(),
        Value::String(args.matcher.to_string()),
    );
    fields.insert(
        "mode".to_string(),
        Value::String(args.mode.as_str().to_string()),
    );
    fields.insert(
        "handlers".to_string(),
        serde_json::to_value(args.handler_chain).map_err(|source| {
            CliError::internal_with_source("failed to serialize handlers", source)
        })?,
    );
    fields.insert(
        "results".to_string(),
        serde_json::to_value(args.results).map_err(|source| {
            CliError::internal_with_source("failed to serialize results", source)
        })?,
    );
    fields.insert(
        "total_ms".to_string(),
        Value::from(args.total_ms.min(u64::MAX as u128) as u64),
    );
    fields.insert("exit".to_string(), Value::from(args.exit));
    if let Some(ai_notification) = args.ai_notification {
        fields.insert(
            "ai_notification".to_string(),
            Value::String(ai_notification.to_string()),
        );
    }

    let event = LogEvent {
        version: sc_observability_types::constants::OBSERVATION_ENVELOPE_VERSION.to_string(),
        timestamp: sc_observability_types::Timestamp::now_utc(),
        level: dispatch_level(args.exit, args.results, args.ai_notification),
        service,
        target,
        action,
        message: Some(dispatch_message(
            args.hook,
            args.event,
            args.mode,
            args.handler_chain.len(),
            args.exit,
        )),
        identity: ProcessIdentity {
            hostname: None,
            pid: Some(std::process::id()),
        },
        trace: None,
        request_id: None,
        correlation_id: None,
        outcome: Some(dispatch_outcome(args.exit).to_string()),
        diagnostic: None,
        state_transition: None,
        fields,
    };

    logger.emit(event).map_err(|source| {
        CliError::internal_with_source("failed emitting observability event", source)
    })?;
    logger.flush().map_err(|source| {
        CliError::internal_with_source("failed flushing observability event", source)
    })?;
    Ok(())
}

/// Emits the canonical `session.root_divergence` observability event.
///
/// # Errors
///
/// Returns an error when logger initialization fails or when the underlying
/// observability sink fails during emit or flush.
pub fn emit_root_divergence_event(args: RootDivergenceEventArgs<'_>) -> Result<(), CliError> {
    let service = ServiceName::new(SERVICE_NAME)
        .map_err(|source| CliError::internal_with_source("invalid service name", source))?;
    let logger = logger(args.project_root)?;
    let target = TargetCategory::new("hook")
        .map_err(|source| CliError::internal_with_source("invalid log target", source))?;
    let action = ActionName::new("session.root_divergence")
        .map_err(|source| CliError::internal_with_source("invalid log action", source))?;

    let mut fields = Map::new();
    fields.insert(
        "immutable_root".to_string(),
        Value::String(args.notice.immutable_root.as_path().display().to_string()),
    );
    fields.insert(
        "observed".to_string(),
        Value::String(args.notice.observed.as_path().display().to_string()),
    );
    fields.insert(
        "session_id".to_string(),
        Value::String(args.notice.session_id.to_string()),
    );
    fields.insert(
        "hook_event".to_string(),
        Value::String(args.notice.hook_event.as_str().to_string()),
    );

    let event = LogEvent {
        version: sc_observability_types::constants::OBSERVATION_ENVELOPE_VERSION.to_string(),
        timestamp: sc_observability_types::Timestamp::now_utc(),
        level: Level::Error,
        service,
        target,
        action,
        message: Some(args.notice.warning_message()),
        identity: ProcessIdentity {
            hostname: None,
            pid: Some(std::process::id()),
        },
        trace: None,
        request_id: None,
        correlation_id: None,
        outcome: Some("error".to_string()),
        diagnostic: None,
        state_transition: None,
        fields,
    };

    logger.emit(event).map_err(|source| {
        CliError::internal_with_source("failed emitting observability event", source)
    })?;
    logger.flush().map_err(|source| {
        CliError::internal_with_source("failed flushing observability event", source)
    })?;
    Ok(())
}
fn logger(project_root: &AiRootDir) -> Result<&'static Logger, CliError> {
    #[cfg(test)]
    let _ = project_root;
    #[cfg(test)]
    let effective_root =
        AiRootDir::new(crate::test_support::shared_observability_root()).map_err(|source| {
            CliError::internal_with_source(
                "failed resolving shared test observability root",
                source,
            )
        })?;
    #[cfg(not(test))]
    let effective_root = project_root.clone();

    let initialized_root = LOGGER_ROOT.get_or_init(|| effective_root.clone());
    if initialized_root != &effective_root {
        return Err(CliError::internal_with_source(
            "observability logger project root mismatch",
            ObservabilityInitError::ProjectRootMismatch {
                initialized: initialized_root.as_path().to_path_buf(),
                requested: effective_root.as_path().to_path_buf(),
            },
        ));
    }
    if let Some(logger) = LOGGER.get() {
        return Ok(logger);
    }

    let service = ServiceName::new(SERVICE_NAME).map_err(|source| {
        CliError::internal_with_source(
            "failed to initialize observability logger",
            ObservabilityInitError::InvalidServiceName { source },
        )
    })?;
    let config = default_logger_config(service, initialized_root).map_err(|source| {
        CliError::internal_with_source(
            "failed to initialize observability logger",
            ObservabilityInitError::ResolveRoot { source },
        )
    })?;
    let logger = Logger::new(config).map_err(|source| {
        CliError::internal_with_source(
            "failed to initialize observability logger",
            ObservabilityInitError::LoggerInit { source },
        )
    })?;
    Ok(LOGGER.get_or_init(|| logger))
}

fn default_logger_config(
    service: ServiceName,
    project_root: &AiRootDir,
) -> Result<LoggerConfig, CliError> {
    let root =
        sc_hooks_core::storage::observability_root_for(Some(project_root)).map_err(|source| {
            CliError::internal_with_source("failed resolving observability root", source)
        })?;
    let mut config = LoggerConfig::default_for(service, root.into_path_buf());
    config.level = LevelFilter::Info;
    config.enable_console_sink = env_flag("SC_HOOKS_ENABLE_CONSOLE_SINK").unwrap_or(false);
    config.enable_file_sink = env_flag("SC_HOOKS_ENABLE_FILE_SINK").unwrap_or(true);
    Ok(config)
}

fn env_flag(key: &str) -> Option<bool> {
    let value = std::env::var(key).ok()?;
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => {
            warn!(
                "warning: unrecognized value for {key}: {value:?} (expected 1/true/yes/on or 0/false/no/off)"
            );
            None
        }
    }
}

fn dispatch_level(
    exit: i32,
    results: &[HandlerResultRecord],
    ai_notification: Option<&str>,
) -> Level {
    if exit == sc_hooks_core::exit_codes::SUCCESS
        && ai_notification.is_none()
        && results.iter().all(|result| {
            result.error_type.is_none() && result.warning.is_none() && result.disabled != Some(true)
        })
    {
        Level::Info
    } else if exit == sc_hooks_core::exit_codes::BLOCKED {
        Level::Warn
    } else {
        Level::Error
    }
}

fn dispatch_outcome(exit: i32) -> &'static str {
    match exit {
        sc_hooks_core::exit_codes::SUCCESS => "proceed",
        sc_hooks_core::exit_codes::BLOCKED => "block",
        _ => "error",
    }
}

fn dispatch_message(
    hook: &str,
    event: Option<&str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
    handler_count: usize,
    exit: i32,
) -> String {
    format!(
        "dispatch hook={hook} event={} mode={} handlers={} outcome={}",
        event.unwrap_or("*"),
        mode.as_str(),
        handler_count,
        dispatch_outcome(exit)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn emits_service_scoped_sc_observability_log_event() {
        let root = crate::test_support::shared_observability_root();
        let project_root = AiRootDir::new(root.clone()).expect("root should be absolute");
        let _cwd = crate::test_support::scoped_current_dir(&root);

        emit_dispatch_event(DispatchEventArgs {
            hook: "PreToolUse",
            event: Some("Write"),
            matcher: "Write",
            mode: sc_hooks_core::dispatch::DispatchMode::Sync,
            handler_chain: &["guard-paths".to_string()],
            results: &[HandlerResultRecord {
                handler: "guard-paths".to_string(),
                action: Cow::Borrowed("proceed"),
                ms: 2,
                error_type: None,
                stderr: None,
                warning: None,
                disabled: None,
            }],
            total_ms: 2,
            exit: sc_hooks_core::exit_codes::SUCCESS,
            ai_notification: None,
            project_root: &project_root,
        })
        .expect("observability event should emit");

        let path = root.join(".sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl");
        let rendered = fs::read_to_string(path).expect("log should be readable");
        let line = rendered.lines().last().expect("log line should exist");
        let parsed: serde_json::Value =
            serde_json::from_str(line).expect("log line should parse as json");
        assert_eq!(parsed["service"], "sc-hooks");
        assert_eq!(parsed["target"], "hook");
        assert_eq!(parsed["action"], "dispatch.complete");
        assert_eq!(parsed["outcome"], "proceed");
        assert_eq!(parsed["fields"]["hook"], "PreToolUse");
        assert_eq!(parsed["fields"]["matcher"], "Write");
        assert_eq!(parsed["fields"]["results"][0]["handler"], "guard-paths");
    }
}
