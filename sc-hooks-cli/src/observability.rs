use std::path::PathBuf;

use sc_observability::{Logger, LoggerConfig};
use sc_observability_types::{
    ActionName, Level, LevelFilter, LogEvent, ProcessIdentity, ServiceName, TargetCategory,
};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::errors::CliError;

const OBSERVABILITY_ROOT: &str = ".sc-hooks/observability";
const SERVICE_NAME: &str = "sc-hooks";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HandlerResultRecord {
    pub handler: String,
    pub action: String,
    pub ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

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
}

pub fn emit_dispatch_event(args: DispatchEventArgs<'_>) -> Result<(), CliError> {
    let service = ServiceName::new(SERVICE_NAME)
        .map_err(|err| CliError::internal(format!("invalid service name: {err}")))?;
    let target = TargetCategory::new("hook")
        .map_err(|err| CliError::internal(format!("invalid log target: {err}")))?;
    let action = ActionName::new("dispatch.complete")
        .map_err(|err| CliError::internal(format!("invalid log action: {err}")))?;

    let logger = Logger::new(default_logger_config(service.clone()))
        .map_err(|err| CliError::internal(format!("failed to initialize observability: {err}")))?;

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
        serde_json::to_value(args.handler_chain)
            .map_err(|err| CliError::internal(format!("failed to serialize handlers: {err}")))?,
    );
    fields.insert(
        "results".to_string(),
        serde_json::to_value(args.results)
            .map_err(|err| CliError::internal(format!("failed to serialize results: {err}")))?,
    );
    fields.insert("total_ms".to_string(), Value::from(args.total_ms as u64));
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

    logger
        .emit(event)
        .map_err(|err| CliError::internal(format!("failed emitting observability event: {err}")))?;
    logger
        .flush()
        .map_err(|err| CliError::internal(format!("failed flushing observability event: {err}")))?;
    logger.shutdown().map_err(|err| {
        CliError::internal(format!("failed shutting down observability logger: {err}"))
    })?;
    Ok(())
}

fn default_logger_config(service: ServiceName) -> LoggerConfig {
    let mut config = LoggerConfig::default_for(service, PathBuf::from(OBSERVABILITY_ROOT));
    config.level = LevelFilter::Info;
    config.enable_console_sink = false;
    config.enable_file_sink = true;
    config
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
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = crate::test_support::scoped_current_dir(temp.path());

        emit_dispatch_event(DispatchEventArgs {
            hook: "PreToolUse",
            event: Some("Write"),
            matcher: "Write",
            mode: sc_hooks_core::dispatch::DispatchMode::Sync,
            handler_chain: &["guard-paths".to_string()],
            results: &[HandlerResultRecord {
                handler: "guard-paths".to_string(),
                action: "proceed".to_string(),
                ms: 2,
                error_type: None,
                stderr: None,
                warning: None,
                disabled: None,
            }],
            total_ms: 2,
            exit: sc_hooks_core::exit_codes::SUCCESS,
            ai_notification: None,
        })
        .expect("observability event should emit");

        let path = temp
            .path()
            .join(".sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl");
        let rendered = fs::read_to_string(path).expect("log should be readable");
        let line = rendered.lines().next().expect("log line should exist");
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
