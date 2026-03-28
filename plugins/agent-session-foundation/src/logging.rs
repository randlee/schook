use sc_hooks_core::errors::HookError;
use sc_hooks_core::session::CanonicalSessionRecord;
use sc_observability::{Logger, LoggerConfig};
use sc_observability_types::{
    ActionName, Level, LevelFilter, LogEvent, ProcessIdentity, ServiceName, TargetCategory,
};
use serde_json::{Map, Value};

use crate::state::PersistOutcome;

pub fn emit_session_log(
    record: &CanonicalSessionRecord,
    persist: PersistOutcome,
    hook_event: &str,
) -> Result<(), HookError> {
    let service = ServiceName::new("sc-hooks")
        .map_err(|err| HookError::internal(format!("invalid service name: {err}")))?;
    let target = TargetCategory::new("hook")
        .map_err(|err| HookError::internal(format!("invalid log target: {err}")))?;
    let action = ActionName::new("session.state")
        .map_err(|err| HookError::internal(format!("invalid log action: {err}")))?;

    let root = record
        .ai_root_dir
        .as_path()
        .join(sc_hooks_core::OBSERVABILITY_ROOT);
    let mut config = LoggerConfig::default_for(service.clone(), root);
    config.level = LevelFilter::Info;
    config.enable_console_sink = false;
    config.enable_file_sink = true;

    let logger = Logger::new(config)
        .map_err(|err| HookError::internal(format!("failed to initialize logger: {err}")))?;

    let mut fields = Map::new();
    fields.insert("hook_event".to_string(), Value::String(hook_event.to_string()));
    fields.insert("session_id".to_string(), Value::String(record.session_id.to_string()));
    fields.insert("active_pid".to_string(), Value::from(record.active_pid.get()));
    fields.insert(
        "ai_root_dir".to_string(),
        Value::String(record.ai_root_dir.to_string()),
    );
    fields.insert(
        "ai_current_dir".to_string(),
        Value::String(record.ai_current_dir.to_string()),
    );
    fields.insert(
        "agent_state_after".to_string(),
        serde_json::to_value(record.agent_state)
            .map_err(|err| HookError::internal(format!("failed to serialize state: {err}")))?,
    );
    fields.insert(
        "persist_outcome".to_string(),
        Value::String(match persist {
            PersistOutcome::Created => "created",
            PersistOutcome::Updated => "updated",
            PersistOutcome::Unchanged => "unchanged",
        }.to_string()),
    );
    fields.insert("state_revision".to_string(), Value::from(record.state_revision));

    let event = LogEvent {
        version: sc_observability_types::constants::OBSERVATION_ENVELOPE_VERSION.to_string(),
        timestamp: sc_observability_types::Timestamp::now_utc(),
        level: Level::Info,
        service,
        target,
        action,
        message: Some(format!(
            "session state hook={} session_id={} persist={}",
            hook_event, record.session_id, fields["persist_outcome"]
        )),
        identity: ProcessIdentity {
            hostname: None,
            pid: Some(std::process::id()),
        },
        trace: None,
        request_id: None,
        correlation_id: None,
        outcome: Some("proceed".to_string()),
        diagnostic: None,
        state_transition: None,
        fields,
    };

    logger
        .emit(event)
        .map_err(|err| HookError::internal(format!("failed emitting session log: {err}")))?;
    logger
        .flush()
        .map_err(|err| HookError::internal(format!("failed flushing session log: {err}")))?;
    logger
        .shutdown()
        .map_err(|err| HookError::internal(format!("failed shutting down session log: {err}")))?;
    Ok(())
}
