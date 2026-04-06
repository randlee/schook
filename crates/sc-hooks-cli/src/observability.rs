use std::borrow::Cow;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
#[cfg(test)]
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::warn;
use sc_hooks_core::errors::RootDivergenceNotice;
use sc_observability::{Logger, LoggerConfig};
use sc_observability_types::{
    ActionName, Level, LevelFilter, LogEvent, ProcessIdentity, ServiceName, TargetCategory,
};
use serde::Serialize;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::config::{
    CapturePayloads, CaptureStdio, FullAuditProfile, ObservabilityConfig, ObservabilityMode,
    RedactionMode, RetainDays, RetainRunCount,
};
use crate::errors::CliError;
use sc_hooks_core::session::{AiRootDir, UtcTimestamp, utc_timestamp_now};
use tempfile::NamedTempFile;
const SERVICE_NAME: &str = "sc-hooks";
const DEBUG_EXCERPT_LIMIT: usize = 160;
static LOGGER: OnceLock<Logger> = OnceLock::new();
static LOGGER_ROOT: OnceLock<AiRootDir> = OnceLock::new();
static FULL_AUDIT_RUN: OnceLock<FullAuditRunState> = OnceLock::new();
#[cfg(test)]
static TEST_LOGGER_ROOT_OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
const TEST_FORCE_OBSERVABILITY_FAILURE_ENV: &str = "SC_HOOKS_TEST_FORCE_OBSERVABILITY_FAILURE";
const TEST_MODE_ENV: &str = "SC_HOOKS_TEST_MODE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForcedObservabilityFailure {
    LoggerInit,
    Emit,
    AuditAppend,
    AuditPrune,
}

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
    #[error(
        "full audit root mismatch for cached run: initialized at {initialized}, requested {requested}"
    )]
    FullAuditRootMismatch {
        initialized: PathBuf,
        requested: PathBuf,
    },
}

#[derive(Debug)]
struct FullAuditRunState {
    run_id: String,
    invocation_id: String,
    root: PathBuf,
    events_path: PathBuf,
    meta_path: PathBuf,
    project_root: String,
    profile: FullAuditProfile,
    started_at: UtcTimestamp,
}

#[derive(Debug, Serialize)]
struct FullAuditMeta<'a> {
    schema_version: u32,
    service: &'static str,
    run_id: &'a str,
    invocation_id: &'a str,
    profile: &'static str,
    started_at: &'a UtcTimestamp,
    project_root: &'a str,
    pid: u32,
}

#[derive(Debug, Serialize)]
struct FullAuditRecord<'a> {
    schema_version: u32,
    timestamp: UtcTimestamp,
    service: &'static str,
    run_id: &'a str,
    invocation_id: &'a str,
    name: &'static str,
    hook: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    hook_event: Option<&'a str>,
    mode: &'static str,
    profile: &'static str,
    project_root: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_dir: Option<String>,
    pid: u32,
    outcome: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    handler_chain: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    handler_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ai_notification: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    degraded: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config_source_summary: Option<FullAuditConfigSourceSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config_layer_resolution: Option<FullAuditConfigLayerResolution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision_trace_summary: Option<DecisionTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    handler_stderr_excerpt: Option<Vec<FullAuditHandlerExcerpt>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    handler_stdout_excerpt: Option<Vec<FullAuditHandlerExcerpt>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redaction_actions: Option<Vec<RedactionAction>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_capture_state: Option<FullAuditPayloadCaptureState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_excerpt: Option<DebugExcerpt>,
}

struct FullAuditRecordArgs<'a> {
    name: &'static str,
    hook: &'a str,
    event: Option<&'a str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
    project_root: &'a AiRootDir,
    observability: &'a ObservabilityConfig,
    outcome: &'a str,
    stage: Option<&'a str>,
    handler_chain: Option<&'a [String]>,
    total_ms: Option<u128>,
    exit: Option<i32>,
    error: Option<String>,
    ai_notification: Option<&'a str>,
    degraded: Option<bool>,
    results: Option<&'a [HandlerResultRecord]>,
    payload: Option<&'a Value>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DebugExcerpt {
    pub excerpt: String,
    pub truncated: bool,
    pub redacted: bool,
}

#[derive(Debug, Serialize)]
struct FullAuditConfigSourceSummary {
    local_config_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_config_path: Option<String>,
    global_config_present: bool,
    env_overrides: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FullAuditConfigLayerResolution {
    mode_source: &'static str,
    full_profile_source: &'static str,
    path_source: &'static str,
    console_mirror_source: &'static str,
    retain_runs_source: &'static str,
    retain_days_source: &'static str,
    redaction_source: &'static str,
    capture_payloads_source: &'static str,
    capture_stdio_source: &'static str,
}

#[derive(Debug, Serialize)]
struct FullAuditHandlerExcerpt {
    handler: String,
    excerpt: String,
    truncated: bool,
    redacted: bool,
}

#[derive(Debug, Serialize)]
struct FullAuditPayloadCaptureState {
    redaction: &'static str,
    capture_payloads: CapturePayloads,
    capture_stdio: &'static str,
    payloads_included: bool,
}

#[derive(Debug, Serialize)]
struct DecisionTrace {
    record: &'static str,
    mode: &'static str,
    profile: &'static str,
    outcome: String,
    redaction: &'static str,
    capture_stdio: &'static str,
    capture_payloads: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    handler_chain_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    degraded: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum RedactionAction {
    RedactionMode { mode: &'static str },
    StdioCapture { capture: &'static str },
    PayloadCaptureDisabled,
    StrictModeSummarizesSensitiveText,
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
    #[serde(skip_serializing)]
    pub debug_stderr_excerpt: Option<DebugExcerpt>,
    #[serde(skip_serializing)]
    pub debug_stdout_excerpt: Option<DebugExcerpt>,
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
    pub observability: &'a ObservabilityConfig,
    pub payload: Option<&'a Value>,
}

/// Arguments required to emit one `session.root_divergence` observability event.
pub struct RootDivergenceEventArgs<'a> {
    pub notice: &'a RootDivergenceNotice,
    pub project_root: &'a AiRootDir,
    pub observability: &'a ObservabilityConfig,
}

pub struct FullAuditPreDispatchFailureArgs<'a> {
    pub observability: &'a ObservabilityConfig,
    pub hook: &'a str,
    pub event: Option<&'a str>,
    pub mode: sc_hooks_core::dispatch::DispatchMode,
    pub project_root: &'a AiRootDir,
    pub stage: &'a str,
    pub err: &'a CliError,
    pub payload: Option<&'a Value>,
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
    if matches!(args.observability.mode, ObservabilityMode::Off) {
        return Ok(());
    }

    let service = ServiceName::new(SERVICE_NAME)
        .map_err(|source| CliError::internal_with_source("invalid service name", source))?;
    let logger = logger(args.project_root, args.observability)?;
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

    if should_force_observability_failure(ForcedObservabilityFailure::Emit) {
        return Err(CliError::internal("forced observability emit failure"));
    }

    logger.emit(event).map_err(|source| {
        CliError::internal_with_source("failed emitting observability event", source)
    })?;
    logger.flush().map_err(|source| {
        CliError::internal_with_source("failed flushing observability event", source)
    })?;
    emit_full_audit_record(FullAuditRecordArgs {
        name: "hook.dispatch.completed",
        hook: args.hook,
        event: args.event,
        mode: args.mode,
        project_root: args.project_root,
        observability: args.observability,
        outcome: dispatch_outcome(args.exit),
        stage: None,
        handler_chain: Some(args.handler_chain),
        total_ms: Some(args.total_ms),
        exit: Some(args.exit),
        error: None,
        ai_notification: args.ai_notification,
        degraded: None,
        results: Some(args.results),
        payload: args.payload,
    })?;
    Ok(())
}

pub fn emit_full_audit_invocation_received(
    observability: &ObservabilityConfig,
    hook: &str,
    event: Option<&str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
    project_root: &AiRootDir,
    payload: Option<&Value>,
) {
    emit_full_audit_record_with_fallback(FullAuditRecordArgs {
        name: "hook.invocation.received",
        hook,
        event,
        mode,
        project_root,
        observability,
        outcome: "received",
        stage: None,
        handler_chain: None,
        total_ms: None,
        exit: None,
        error: None,
        ai_notification: None,
        degraded: None,
        results: None,
        payload,
    });
}

pub fn emit_full_audit_zero_match(
    observability: &ObservabilityConfig,
    hook: &str,
    event: Option<&str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
    project_root: &AiRootDir,
    payload: Option<&Value>,
) {
    emit_full_audit_record_with_fallback(FullAuditRecordArgs {
        name: "hook.invocation.zero_match",
        hook,
        event,
        mode,
        project_root,
        observability,
        outcome: "zero_match",
        stage: None,
        handler_chain: None,
        total_ms: None,
        exit: Some(sc_hooks_core::exit_codes::SUCCESS),
        error: None,
        ai_notification: None,
        degraded: None,
        results: None,
        payload,
    });
}

pub fn emit_full_audit_pre_dispatch_failure(args: FullAuditPreDispatchFailureArgs<'_>) {
    emit_full_audit_record_with_fallback(FullAuditRecordArgs {
        name: "hook.invocation.failed_pre_dispatch",
        hook: args.hook,
        event: args.event,
        mode: args.mode,
        project_root: args.project_root,
        observability: args.observability,
        outcome: "error",
        stage: Some(args.stage),
        handler_chain: None,
        total_ms: None,
        exit: Some(args.err.exit_code()),
        error: Some(args.err.to_string()),
        ai_notification: None,
        degraded: Some(true),
        results: None,
        payload: args.payload,
    });
}

pub fn emit_standard_degraded_signal(
    observability: &ObservabilityConfig,
    hook: &str,
    event: Option<&str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
    stage: &str,
    err: &CliError,
) {
    if !matches!(observability.mode, ObservabilityMode::Standard) {
        return;
    }

    let message = format!(
        "sc-hooks: standard observability degraded before dispatch.complete: stage={stage} hook={hook} event={} mode={} error={err}",
        event.unwrap_or("*"),
        mode.as_str(),
    );
    warn!("{message}");
    let _ = writeln!(std::io::stderr(), "{message}");
}

fn emit_full_audit_record_with_fallback(args: FullAuditRecordArgs<'_>) {
    if let Err(err) = emit_full_audit_record(args) {
        emit_stderr_warning(format!("sc-hooks: full audit degraded: {err}"));
    }
}

/// Emits the canonical `session.root_divergence` observability event.
///
/// # Errors
///
/// Returns an error when logger initialization fails or when the underlying
/// observability sink fails during emit or flush.
pub fn emit_root_divergence_event(args: RootDivergenceEventArgs<'_>) -> Result<(), CliError> {
    if matches!(args.observability.mode, ObservabilityMode::Off) {
        return Ok(());
    }

    let service = ServiceName::new(SERVICE_NAME)
        .map_err(|source| CliError::internal_with_source("invalid service name", source))?;
    let logger = logger(args.project_root, args.observability)?;
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
fn logger(
    project_root: &AiRootDir,
    observability: &ObservabilityConfig,
) -> Result<&'static Logger, CliError> {
    let effective_root = effective_logger_root(project_root)?;

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

    if should_force_observability_failure(ForcedObservabilityFailure::LoggerInit) {
        return Err(CliError::internal(
            "forced observability logger init failure",
        ));
    }

    let service = ServiceName::new(SERVICE_NAME).map_err(|source| {
        CliError::internal_with_source(
            "failed to initialize observability logger",
            ObservabilityInitError::InvalidServiceName { source },
        )
    })?;
    let config =
        default_logger_config(service, initialized_root, observability).map_err(|source| {
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

#[cfg(test)]
fn test_logger_root_override() -> &'static Mutex<Option<PathBuf>> {
    TEST_LOGGER_ROOT_OVERRIDE.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
fn effective_logger_root(project_root: &AiRootDir) -> Result<AiRootDir, CliError> {
    let override_root = test_logger_root_override()
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clone();
    let root = override_root.unwrap_or_else(crate::test_support::shared_observability_root);
    let _ = project_root;
    AiRootDir::new(root).map_err(|source| {
        CliError::internal_with_source("failed resolving shared test observability root", source)
    })
}

#[cfg(not(test))]
fn effective_logger_root(project_root: &AiRootDir) -> Result<AiRootDir, CliError> {
    Ok(project_root.clone())
}

fn default_logger_config(
    service: ServiceName,
    project_root: &AiRootDir,
    observability: &ObservabilityConfig,
) -> Result<LoggerConfig, CliError> {
    let root =
        sc_hooks_core::storage::observability_root_for(Some(project_root)).map_err(|source| {
            CliError::internal_with_source("failed resolving observability root", source)
        })?;
    let mut config = LoggerConfig::default_for(service, root.into_path_buf());
    config.level = LevelFilter::Info;
    if matches!(observability.mode, ObservabilityMode::Off) {
        config.enable_console_sink = false;
        config.enable_file_sink = false;
        return Ok(config);
    }
    config.enable_console_sink =
        env_flag("SC_HOOKS_ENABLE_CONSOLE_SINK").unwrap_or(observability.console_mirror);
    config.enable_file_sink = env_flag("SC_HOOKS_ENABLE_FILE_SINK").unwrap_or(true);
    Ok(config)
}

fn env_flag(key: &str) -> Option<bool> {
    let value = std::env::var(key).ok()?;
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => {
            emit_stderr_warning(format!(
                "warning: unrecognized value for {key}: {value:?} (expected 1/true/yes/on or 0/false/no/off)"
            ));
            None
        }
    }
}

fn forced_failures_enabled() -> bool {
    cfg!(debug_assertions)
}

pub(crate) fn emit_stderr_warning(message: impl AsRef<str>) {
    let message = message.as_ref();
    warn!("{message}");
    let _ = writeln!(std::io::stderr(), "{message}");
}

fn forced_observability_failure() -> Option<ForcedObservabilityFailure> {
    // RATIONALE: debug_assertions keeps forced-failure injection compiled out
    // of release builds so test-only failure paths cannot leak into shipped
    // binaries through environment variables.
    if !forced_failures_enabled() {
        return None;
    }

    std::env::var_os(TEST_MODE_ENV)?;

    match std::env::var(TEST_FORCE_OBSERVABILITY_FAILURE_ENV)
        .ok()?
        .as_str()
    {
        "logger_init" => Some(ForcedObservabilityFailure::LoggerInit),
        "emit" => Some(ForcedObservabilityFailure::Emit),
        "audit_append" => Some(ForcedObservabilityFailure::AuditAppend),
        "audit_prune" => Some(ForcedObservabilityFailure::AuditPrune),
        _ => None,
    }
}

fn should_force_observability_failure(kind: ForcedObservabilityFailure) -> bool {
    forced_observability_failure() == Some(kind)
}

pub(crate) fn build_debug_excerpt(
    observability: &ObservabilityConfig,
    text: &str,
) -> Option<DebugExcerpt> {
    if text.is_empty() {
        return None;
    }

    match observability.capture_stdio {
        CaptureStdio::None => None,
        CaptureStdio::Summary => Some(DebugExcerpt {
            excerpt: summary_excerpt(text),
            truncated: false,
            redacted: true,
        }),
        CaptureStdio::Bounded => match observability.redaction {
            RedactionMode::Strict => Some(DebugExcerpt {
                excerpt: summary_excerpt(text),
                truncated: false,
                redacted: true,
            }),
            RedactionMode::Permissive => Some(bounded_debug_excerpt(text, false)),
        },
    }
}

fn build_debug_payload_excerpt(
    observability: &ObservabilityConfig,
    payload: Option<&Value>,
) -> Option<DebugExcerpt> {
    if !observability.capture_payloads.is_enabled() {
        return None;
    }
    let payload = payload?;
    let serialized = serde_json::to_string(payload).ok()?;
    Some(match observability.redaction {
        RedactionMode::Strict => DebugExcerpt {
            excerpt: summary_excerpt(&serialized),
            truncated: false,
            redacted: true,
        },
        RedactionMode::Permissive => bounded_debug_excerpt(&serialized, false),
    })
}

fn bounded_debug_excerpt(text: &str, redacted: bool) -> DebugExcerpt {
    let truncated = text.chars().count() > DEBUG_EXCERPT_LIMIT;
    let excerpt = text.chars().take(DEBUG_EXCERPT_LIMIT).collect();
    DebugExcerpt {
        excerpt,
        truncated,
        redacted,
    }
}

fn summary_excerpt(text: &str) -> String {
    format!(
        "summary(chars={}, lines={})",
        text.chars().count(),
        text.lines().count().max(1)
    )
}

fn emit_full_audit_record(args: FullAuditRecordArgs<'_>) -> Result<(), CliError> {
    if !matches!(args.observability.mode, ObservabilityMode::Full) {
        return Ok(());
    }

    let state = full_audit_run_state(args.project_root, args.observability)?;
    let (
        config_source_summary,
        config_layer_resolution,
        decision_trace_summary,
        handler_stderr_excerpt,
        handler_stdout_excerpt,
        redaction_actions,
        payload_capture_state,
        payload_excerpt,
    ) = if matches!(state.profile, FullAuditProfile::Debug) {
        let payload_excerpt = build_debug_payload_excerpt(args.observability, args.payload);
        (
            Some(full_audit_config_source_summary(args.observability)),
            Some(full_audit_config_layer_resolution(args.observability)),
            Some(full_audit_decision_trace_summary(&args)),
            Some(full_audit_handler_excerpts(args.results, true)),
            Some(full_audit_handler_excerpts(args.results, false)),
            Some(full_audit_redaction_actions(args.observability)),
            Some(full_audit_payload_capture_state(
                args.observability,
                payload_excerpt.is_some(),
            )),
            payload_excerpt,
        )
    } else {
        (None, None, None, None, None, None, None, None)
    };
    let current_dir = std::env::current_dir()
        .ok()
        .map(|path| path.display().to_string())
        .filter(|path| path != &state.project_root);
    let record = FullAuditRecord {
        schema_version: 1,
        timestamp: utc_timestamp_now(),
        service: SERVICE_NAME,
        run_id: &state.run_id,
        invocation_id: &state.invocation_id,
        name: args.name,
        hook: args.hook,
        hook_event: args.event,
        mode: args.mode.as_str(),
        profile: state.profile.as_str(),
        project_root: &state.project_root,
        current_dir,
        pid: std::process::id(),
        outcome: args.outcome,
        stage: args.stage,
        handler_chain: args.handler_chain,
        handler_count: args.handler_chain.map(<[String]>::len),
        total_ms: args.total_ms.map(|ms| ms.min(u64::MAX as u128) as u64),
        exit: args.exit,
        error: args.error,
        ai_notification: args.ai_notification,
        degraded: args.degraded,
        config_source_summary,
        config_layer_resolution,
        decision_trace_summary,
        handler_stderr_excerpt,
        handler_stdout_excerpt,
        redaction_actions,
        payload_capture_state,
        payload_excerpt,
    };
    append_jsonl(&state.events_path, &record)
}

fn full_audit_config_source_summary(
    observability: &ObservabilityConfig,
) -> FullAuditConfigSourceSummary {
    FullAuditConfigSourceSummary {
        local_config_path: observability
            .debug_context
            .local_config_path
            .display()
            .to_string(),
        global_config_path: observability
            .debug_context
            .global_config_path
            .as_ref()
            .map(|path| path.display().to_string()),
        global_config_present: observability.debug_context.global_config_present,
        env_overrides: observability.debug_context.env_overrides.clone(),
    }
}

fn full_audit_config_layer_resolution(
    observability: &ObservabilityConfig,
) -> FullAuditConfigLayerResolution {
    FullAuditConfigLayerResolution {
        mode_source: observability.debug_context.mode_source.as_str(),
        full_profile_source: observability.debug_context.full_profile_source.as_str(),
        path_source: observability.debug_context.path_source.as_str(),
        console_mirror_source: observability.debug_context.console_mirror_source.as_str(),
        retain_runs_source: observability.debug_context.retain_runs_source.as_str(),
        retain_days_source: observability.debug_context.retain_days_source.as_str(),
        redaction_source: observability.debug_context.redaction_source.as_str(),
        capture_payloads_source: observability.debug_context.capture_payloads_source.as_str(),
        capture_stdio_source: observability.debug_context.capture_stdio_source.as_str(),
    }
}

fn full_audit_decision_trace_summary(args: &FullAuditRecordArgs<'_>) -> DecisionTrace {
    DecisionTrace {
        record: args.name,
        mode: args.mode.as_str(),
        profile: args.observability.full_profile.as_str(),
        outcome: args.outcome.to_string(),
        redaction: args.observability.redaction.as_str(),
        capture_stdio: args.observability.capture_stdio.as_str(),
        capture_payloads: args.observability.capture_payloads.is_enabled(),
        stage: args.stage.map(ToOwned::to_owned),
        result_count: args.results.map(<[HandlerResultRecord]>::len),
        handler_chain_count: args.handler_chain.map(<[String]>::len),
        degraded: args.degraded,
    }
}

fn full_audit_handler_excerpts(
    results: Option<&[HandlerResultRecord]>,
    use_stderr: bool,
) -> Vec<FullAuditHandlerExcerpt> {
    results
        .into_iter()
        .flat_map(|results| results.iter())
        .filter_map(|result| {
            let excerpt = if use_stderr {
                result.debug_stderr_excerpt.as_ref()
            } else {
                result.debug_stdout_excerpt.as_ref()
            }?;
            Some(FullAuditHandlerExcerpt {
                handler: result.handler.clone(),
                excerpt: excerpt.excerpt.clone(),
                truncated: excerpt.truncated,
                redacted: excerpt.redacted,
            })
        })
        .collect()
}

fn full_audit_redaction_actions(observability: &ObservabilityConfig) -> Vec<RedactionAction> {
    let mut actions = vec![RedactionAction::RedactionMode {
        mode: observability.redaction.as_str(),
    }];
    if !observability.capture_payloads.is_enabled() {
        actions.push(RedactionAction::PayloadCaptureDisabled);
    }
    actions.push(RedactionAction::StdioCapture {
        capture: observability.capture_stdio.as_str(),
    });
    if matches!(observability.redaction, RedactionMode::Strict) {
        actions.push(RedactionAction::StrictModeSummarizesSensitiveText);
    }
    actions
}

fn full_audit_payload_capture_state(
    observability: &ObservabilityConfig,
    payloads_included: bool,
) -> FullAuditPayloadCaptureState {
    FullAuditPayloadCaptureState {
        redaction: observability.redaction.as_str(),
        capture_payloads: observability.capture_payloads,
        capture_stdio: observability.capture_stdio.as_str(),
        payloads_included,
    }
}

fn full_audit_run_state(
    project_root: &AiRootDir,
    observability: &ObservabilityConfig,
) -> Result<&'static FullAuditRunState, CliError> {
    let root = resolve_full_audit_root(project_root, observability);
    if let Some(state) = FULL_AUDIT_RUN.get() {
        if state.root != root {
            return Err(CliError::internal_with_source(
                "full audit root mismatch",
                ObservabilityInitError::FullAuditRootMismatch {
                    initialized: state.root.clone(),
                    requested: root,
                },
            ));
        }
        return Ok(state);
    }

    fs::create_dir_all(root.join("runs")).map_err(|source| {
        CliError::internal_with_source(
            format!("failed preparing full audit root at {}", root.display()),
            source,
        )
    })?;
    let run_id = generate_run_id();
    let invocation_id = format!("{run_id}-invocation");
    let run_dir = root.join("runs").join(&run_id);
    fs::create_dir_all(&run_dir).map_err(|source| {
        CliError::internal_with_source(
            format!(
                "failed preparing full audit run directory at {}",
                run_dir.display()
            ),
            source,
        )
    })?;
    let state = FullAuditRunState {
        run_id,
        invocation_id,
        root: root.clone(),
        events_path: run_dir.join("events.jsonl"),
        meta_path: run_dir.join("meta.json"),
        project_root: project_root.as_path().display().to_string(),
        profile: observability.full_profile,
        started_at: utc_timestamp_now(),
    };
    write_full_audit_meta(&state)?;
    if let Err(err) = prune_full_audit_runs(&root.join("runs"), &state.run_id, observability) {
        // Pruning is best-effort: a failure to remove old runs should not block
        // the current dispatch or change its hook outcome.
        emit_stderr_warning(format!("sc-hooks: full audit degraded: {err}"));
    }
    Ok(FULL_AUDIT_RUN.get_or_init(|| state))
}

fn resolve_full_audit_root(
    project_root: &AiRootDir,
    observability: &ObservabilityConfig,
) -> PathBuf {
    if observability.path.is_absolute() {
        observability.path.clone()
    } else {
        project_root.as_path().join(&observability.path)
    }
}

fn write_full_audit_meta(state: &FullAuditRunState) -> Result<(), CliError> {
    if state.meta_path.exists() {
        return Ok(());
    }
    let meta = FullAuditMeta {
        schema_version: 1,
        service: SERVICE_NAME,
        run_id: &state.run_id,
        invocation_id: &state.invocation_id,
        profile: state.profile.as_str(),
        started_at: &state.started_at,
        project_root: &state.project_root,
        pid: std::process::id(),
    };
    let parent = state
        .meta_path
        .parent()
        .ok_or_else(|| CliError::internal("full audit meta path must have a parent directory"))?;
    let mut temp = NamedTempFile::new_in(parent).map_err(|source| {
        CliError::internal_with_source(
            format!(
                "failed creating temporary full audit meta file in {}",
                parent.display()
            ),
            source,
        )
    })?;
    serde_json::to_writer_pretty(temp.as_file_mut(), &meta).map_err(|source| {
        CliError::internal_with_source("failed serializing full audit meta file", source)
    })?;
    writeln!(temp.as_file_mut()).map_err(|source| {
        CliError::internal_with_source("failed finalizing full audit meta file", source)
    })?;
    temp.as_file_mut().sync_all().map_err(|source| {
        CliError::internal_with_source("failed syncing full audit meta file", source)
    })?;
    temp.persist(&state.meta_path).map_err(|source| {
        CliError::internal_with_source(
            format!(
                "failed persisting full audit meta file to {}",
                state.meta_path.display()
            ),
            source.error,
        )
    })?;
    Ok(())
}

fn append_jsonl(path: &std::path::Path, record: &impl Serialize) -> Result<(), CliError> {
    if should_force_observability_failure(ForcedObservabilityFailure::AuditAppend) {
        return Err(CliError::internal("forced full audit append failure"));
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| {
            CliError::internal_with_source(
                format!(
                    "failed opening full audit events file at {}",
                    path.display()
                ),
                source,
            )
        })?;
    serde_json::to_writer(&mut file, record).map_err(|source| {
        CliError::internal_with_source("failed serializing full audit record", source)
    })?;
    writeln!(file).map_err(|source| {
        CliError::internal_with_source("failed writing full audit newline", source)
    })?;
    file.sync_data().map_err(|source| {
        CliError::internal_with_source("failed syncing full audit events file", source)
    })?;
    Ok(())
}

#[derive(Debug)]
struct FullAuditRunDir {
    name: String,
    path: PathBuf,
    started_at: SystemTime,
    sort_key_nanos: u128,
}

fn prune_full_audit_runs(
    runs_root: &std::path::Path,
    current_run_id: &str,
    observability: &ObservabilityConfig,
) -> Result<(), CliError> {
    if should_force_observability_failure(ForcedObservabilityFailure::AuditPrune) {
        return Err(CliError::internal("forced full audit prune failure"));
    }
    if !runs_root.exists() {
        return Ok(());
    }

    let mut runs = list_full_audit_runs(runs_root)?;
    prune_runs_older_than(&mut runs, current_run_id, observability.retain_days)?;
    prune_runs_by_count(&runs, current_run_id, observability.retain_runs)
}

fn list_full_audit_runs(runs_root: &std::path::Path) -> Result<Vec<FullAuditRunDir>, CliError> {
    let mut runs = Vec::new();
    for entry in fs::read_dir(runs_root).map_err(|source| {
        CliError::internal_with_source(
            format!(
                "failed reading full audit runs directory at {}",
                runs_root.display()
            ),
            source,
        )
    })? {
        let entry = entry.map_err(|source| {
            CliError::internal_with_source("failed reading full audit run directory entry", source)
        })?;
        let file_type = entry.file_type().map_err(|source| {
            CliError::internal_with_source("failed reading full audit run directory type", source)
        })?;
        if !file_type.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let metadata = entry.metadata().map_err(|source| {
            CliError::internal_with_source("failed reading full audit run metadata", source)
        })?;
        let sort_key_nanos = parse_run_sort_key_nanos(&name).unwrap_or_else(|| {
            metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        });
        let started_at = parse_run_started_at(&name)
            .or_else(|| metadata.modified().ok())
            .unwrap_or(UNIX_EPOCH);
        runs.push(FullAuditRunDir {
            name,
            path,
            started_at,
            sort_key_nanos,
        });
    }
    Ok(runs)
}

fn parse_run_sort_key_nanos(name: &str) -> Option<u128> {
    name.split('-').next()?.parse().ok()
}

fn parse_run_started_at(name: &str) -> Option<SystemTime> {
    let nanos = parse_run_sort_key_nanos(name)?;
    let nanos = u64::try_from(nanos).ok()?;
    Some(UNIX_EPOCH + Duration::from_nanos(nanos))
}

fn prune_runs_older_than(
    runs: &mut Vec<FullAuditRunDir>,
    current_run_id: &str,
    retain_days: RetainDays,
) -> Result<(), CliError> {
    let cutoff = SystemTime::now()
        .checked_sub(retain_days.to_duration())
        .unwrap_or(UNIX_EPOCH);
    let mut kept = Vec::with_capacity(runs.len());
    for run in runs.drain(..) {
        if run.name != current_run_id && run.started_at < cutoff {
            fs::remove_dir_all(&run.path).map_err(|source| {
                CliError::internal_with_source(
                    format!("failed pruning full audit run at {}", run.path.display()),
                    source,
                )
            })?;
        } else {
            kept.push(run);
        }
    }
    *runs = kept;
    Ok(())
}

fn prune_runs_by_count(
    runs: &[FullAuditRunDir],
    current_run_id: &str,
    retain_runs: RetainRunCount,
) -> Result<(), CliError> {
    let mut runs = runs.iter().collect::<Vec<_>>();
    runs.sort_by(|left, right| {
        right
            .sort_key_nanos
            .cmp(&left.sort_key_nanos)
            .then_with(|| right.name.cmp(&left.name))
    });

    let keep_historical =
        usize::try_from(retain_runs.get().saturating_sub(1)).unwrap_or(usize::MAX);
    let mut kept_historical = 0usize;
    for run in runs {
        if run.name == current_run_id {
            continue;
        }
        if kept_historical < keep_historical {
            kept_historical += 1;
            continue;
        }
        fs::remove_dir_all(&run.path).map_err(|source| {
            CliError::internal_with_source(
                format!("failed pruning full audit run at {}", run.path.display()),
                source,
            )
        })?;
    }
    Ok(())
}

fn generate_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{nanos}-{}", std::process::id())
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
    use crate::config::CapturePayloads;
    use std::fs;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn observability_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct LoggerRootOverrideGuard {
        previous: Option<PathBuf>,
    }

    impl Drop for LoggerRootOverrideGuard {
        fn drop(&mut self) {
            *test_logger_root_override()
                .lock()
                .unwrap_or_else(|err| err.into_inner()) = self.previous.take();
        }
    }

    fn scoped_logger_root_override(path: PathBuf) -> LoggerRootOverrideGuard {
        let mut guard = test_logger_root_override()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let previous = (*guard).replace(path);
        LoggerRootOverrideGuard { previous }
    }

    fn emit_sample_dispatch(
        project_root: &AiRootDir,
        observability: &ObservabilityConfig,
    ) -> Result<(), CliError> {
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
                debug_stderr_excerpt: None,
                debug_stdout_excerpt: None,
            }],
            total_ms: 2,
            exit: sc_hooks_core::exit_codes::SUCCESS,
            ai_notification: None,
            project_root,
            observability,
            payload: None,
        })
    }

    fn shared_log_line_count(root: &std::path::Path) -> usize {
        fs::read_to_string(root.join(sc_hooks_core::OBSERVABILITY_LOG_PATH))
            .map(|rendered| rendered.lines().count())
            .unwrap_or(0)
    }

    fn debug_observability(
        redaction: RedactionMode,
        capture_stdio: CaptureStdio,
        capture_payloads: bool,
    ) -> ObservabilityConfig {
        ObservabilityConfig {
            mode: ObservabilityMode::Full,
            full_profile: FullAuditProfile::Debug,
            redaction,
            capture_stdio,
            capture_payloads: CapturePayloads::from(capture_payloads),
            ..ObservabilityConfig::default()
        }
    }

    #[test]
    fn emits_service_scoped_sc_observability_log_event() {
        let _lock: MutexGuard<'_, ()> = observability_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let root = crate::test_support::shared_observability_root();
        let project_root = AiRootDir::new(root.clone()).expect("root should be absolute");
        let _cwd = crate::test_support::scoped_current_dir(&root);
        let observability = ObservabilityConfig::default();

        emit_sample_dispatch(&project_root, &observability)
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

    #[test]
    fn off_mode_returns_before_logger_initialization() {
        let _lock: MutexGuard<'_, ()> = observability_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let root = crate::test_support::shared_observability_root();
        let project_root = AiRootDir::new(root.clone()).expect("root should be absolute");
        let _cwd = crate::test_support::scoped_current_dir(&root);

        emit_sample_dispatch(&project_root, &ObservabilityConfig::default())
            .expect("baseline observability event should initialize logger");
        let before = shared_log_line_count(&root);

        let mismatch_root = tempfile::tempdir().expect("tempdir should create");
        let _override = scoped_logger_root_override(mismatch_root.path().to_path_buf());
        let observability = ObservabilityConfig {
            mode: ObservabilityMode::Off,
            ..ObservabilityConfig::default()
        };

        emit_sample_dispatch(&project_root, &observability)
            .expect("off mode should return before logger initialization");

        assert_eq!(shared_log_line_count(&root), before);
    }

    #[test]
    fn reports_project_root_mismatch_for_cached_logger() {
        let _lock: MutexGuard<'_, ()> = observability_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let root = crate::test_support::shared_observability_root();
        let project_root = AiRootDir::new(root.clone()).expect("root should be absolute");
        let _cwd = crate::test_support::scoped_current_dir(&root);

        emit_sample_dispatch(&project_root, &ObservabilityConfig::default())
            .expect("baseline observability event should initialize logger");

        let initialized_root = LOGGER_ROOT
            .get()
            .map(|root| root.as_path().to_path_buf())
            .unwrap_or_else(|| root.clone());
        let mismatch_root = tempfile::tempdir().expect("tempdir should create");
        assert_ne!(
            mismatch_root.path(),
            initialized_root.as_path(),
            "mismatch root must differ from the initialized root"
        );
        let _override = scoped_logger_root_override(mismatch_root.path().to_path_buf());

        let err = emit_sample_dispatch(&project_root, &ObservabilityConfig::default())
            .expect_err("mismatched test root should fail");
        let rendered = err.to_string();
        assert!(rendered.contains("observability logger project root mismatch"));
        assert!(rendered.contains("project_root mismatch for cached logger"));
    }

    #[test]
    fn build_debug_excerpt_covers_all_stdio_redaction_combinations() {
        let text = "alpha\nbeta";
        assert!(
            build_debug_excerpt(
                &debug_observability(RedactionMode::Strict, CaptureStdio::Summary, false),
                ""
            )
            .is_none()
        );
        let cases = [
            (CaptureStdio::None, RedactionMode::Strict, None, None, None),
            (
                CaptureStdio::Summary,
                RedactionMode::Strict,
                Some("summary(chars=10, lines=2)"),
                Some(false),
                Some(true),
            ),
            (
                CaptureStdio::Bounded,
                RedactionMode::Strict,
                Some("summary(chars=10, lines=2)"),
                Some(false),
                Some(true),
            ),
            (
                CaptureStdio::None,
                RedactionMode::Permissive,
                None,
                None,
                None,
            ),
            (
                CaptureStdio::Summary,
                RedactionMode::Permissive,
                Some("summary(chars=10, lines=2)"),
                Some(false),
                Some(true),
            ),
            (
                CaptureStdio::Bounded,
                RedactionMode::Permissive,
                Some("alpha\nbeta"),
                Some(false),
                Some(false),
            ),
        ];

        for (capture_stdio, redaction, expected_excerpt, expected_truncated, expected_redacted) in
            cases
        {
            let observability = debug_observability(redaction, capture_stdio, false);
            let excerpt = build_debug_excerpt(&observability, text);
            match (excerpt, expected_excerpt) {
                (None, None) => {}
                (Some(excerpt), Some(expected_excerpt)) => {
                    assert_eq!(excerpt.excerpt, expected_excerpt);
                    assert_eq!(Some(excerpt.truncated), expected_truncated);
                    assert_eq!(Some(excerpt.redacted), expected_redacted);
                }
                other => panic!(
                    "unexpected excerpt outcome for {capture_stdio:?}/{redaction:?}: {other:?}"
                ),
            }
        }

        let long_text = "x".repeat(DEBUG_EXCERPT_LIMIT + 25);
        let bounded = build_debug_excerpt(
            &debug_observability(RedactionMode::Permissive, CaptureStdio::Bounded, false),
            &long_text,
        )
        .expect("bounded permissive excerpt should exist");
        assert!(bounded.truncated);
        assert_eq!(bounded.excerpt.chars().count(), DEBUG_EXCERPT_LIMIT);
    }

    #[test]
    fn build_debug_payload_excerpt_covers_capture_and_redaction_combinations() {
        let payload = serde_json::json!({"tool_input": {"command": "echo hi"}});
        let serialized = serde_json::to_string(&payload).expect("payload should serialize");
        let summary = summary_excerpt(&serialized);
        let cases = [
            (false, RedactionMode::Strict, None, None, None),
            (
                true,
                RedactionMode::Strict,
                Some(summary.as_str()),
                Some(false),
                Some(true),
            ),
            (false, RedactionMode::Permissive, None, None, None),
            (
                true,
                RedactionMode::Permissive,
                Some(serialized.as_str()),
                Some(false),
                Some(false),
            ),
        ];

        for (
            capture_payloads,
            redaction,
            expected_excerpt,
            expected_truncated,
            expected_redacted,
        ) in cases
        {
            let observability =
                debug_observability(redaction, CaptureStdio::Summary, capture_payloads);
            let excerpt = build_debug_payload_excerpt(&observability, Some(&payload));
            match (excerpt, expected_excerpt) {
                (None, None) => {}
                (Some(excerpt), Some(expected_excerpt)) => {
                    assert_eq!(excerpt.excerpt, expected_excerpt);
                    assert_eq!(Some(excerpt.truncated), expected_truncated);
                    assert_eq!(Some(excerpt.redacted), expected_redacted);
                }
                other => panic!(
                    "unexpected payload excerpt outcome for capture_payloads={capture_payloads} / {redaction:?}: {other:?}"
                ),
            }
        }

        let payload =
            serde_json::json!({"tool_input": {"command": "y".repeat(DEBUG_EXCERPT_LIMIT + 40)}});
        let observability =
            debug_observability(RedactionMode::Permissive, CaptureStdio::Summary, true);
        let excerpt = build_debug_payload_excerpt(&observability, Some(&payload))
            .expect("payload excerpt should exist when capture is enabled");
        assert!(excerpt.truncated);
        assert_eq!(excerpt.excerpt.chars().count(), DEBUG_EXCERPT_LIMIT);
    }
}
