//! ATM-specific relay and identity-file handling layered on top of the generic
//! session foundation and gate plugins.
//!
//! This crate owns the ATM-only portions of the hook runtime:
//! - Bash identity-file writes for `atm` command execution
//! - permission-request relay parsing and validation
//! - stop / teammate-idle relay event emission
//! - identity-file cleanup on stop paths
//! - ATM metadata enrichment on top of the canonical session record
//!
//! Relay handling follows a four-stage pipeline so parsing, validation, routing,
//! and side effects stay explicit and testable:
//! - `RawRequest<T>` captures the raw payload plus resolved ATM routing
//! - `ValidatedRequest<T>` carries a payload that passed shape/content checks
//! - `RelayDecision` describes the relay event, state update, and cleanup work
//! - `RelayResult` records the side-effect application outcome

use sc_hooks_core::context::HookContext;
use sc_hooks_core::dispatch::DispatchMode;
use sc_hooks_core::errors::HookError;
use sc_hooks_core::events::HookType;
use sc_hooks_core::manifest::{Manifest, ManifestMatcher};
use sc_hooks_core::results::HookResult;
use sc_hooks_core::session::{AgentState, CanonicalSessionRecord, SessionId, utc_timestamp_now};
use sc_hooks_core::storage::{SessionStore, resolve_state_root};
use sc_hooks_core::tools::ToolName;
use sc_hooks_sdk::result::proceed;
use sc_hooks_sdk::traits::{ManifestProvider, SyncHandler};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Sync hook handler that layers ATM relay behavior onto the generic hook
/// runtime without redefining canonical session ownership.
#[derive(Debug, Default)]
pub struct AtmExtensionHandler;

#[derive(Debug)]
struct RawRequest<T> {
    raw_payload: T,
    relay: RelayContext,
}

#[derive(Debug)]
struct ValidatedRequest<T> {
    validated: T,
    relay: RelayContext,
}

#[derive(Debug)]
struct RelayContext {
    routing: AtmRouting,
    process_id: u32,
}

#[derive(Debug)]
struct RelayDecision {
    relay: RelayContext,
    state_update: RecordUpdate,
    event_body: Value,
    cleanup_identity_file: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RelayResult {
    Applied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AtmRouting {
    team: String,
    identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SuggestionType(String);

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuleContent(String);

impl SuggestionType {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl RuleContent {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionSuggestionRule {
    tool_name: ToolName,
    rule_content: RuleContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionSuggestion {
    suggestion_type: SuggestionType,
    behavior: Option<String>,
    destination: Option<String>,
    mode: Option<String>,
    rules: Vec<PermissionSuggestionRule>,
}

#[derive(Debug)]
struct ValidatedPermissionRequest {
    session_id: SessionId,
    tool_name: ToolName,
    tool_input: Value,
    permission_suggestions: Vec<PermissionSuggestion>,
}

#[derive(Debug)]
struct ValidatedStopRequest {
    session_id: SessionId,
}

#[derive(Debug, Clone, Copy)]
struct RecordUpdate {
    hook_event: &'static str,
    state_reason: &'static str,
    agent_state: Option<AgentState>,
}

#[derive(Debug, Deserialize)]
struct BashToolInput {
    command: String,
}

#[derive(Debug, Deserialize)]
struct BashPayload {
    #[serde(rename = "session_id")]
    _session_id: String,
    tool_name: String,
    tool_input: BashToolInput,
}

#[derive(Debug, Deserialize)]
struct PermissionRequestPayload {
    session_id: String,
    tool_name: String,
    tool_input: Value,
    #[serde(default)]
    permission_suggestions: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct StopPayload {
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct TeammateIdlePayload {
    #[serde(default)]
    session_id: Option<String>,
}

impl ManifestProvider for AtmExtensionHandler {
    fn manifest(&self) -> Manifest {
        Manifest {
            contract_version: 1,
            name: "atm-extension".to_string(),
            mode: DispatchMode::Sync,
            hooks: vec![
                HookType::PreToolUse,
                HookType::PostToolUse,
                HookType::PermissionRequest,
                HookType::Stop,
                HookType::TeammateIdle,
                HookType::SubagentStop,
                HookType::Notification,
            ],
            matchers: vec![ManifestMatcher::from("*")],
            payload_conditions: Vec::new(),
            timeout_ms: Some(2_000),
            long_running: false,
            response_time: None,
            requires: BTreeMap::new(),
            optional: BTreeMap::new(),
            sandbox: None,
            description: Some(
                "Layers ATM routing, relay events, and identity-file handling onto the canonical session state. Notification remains deferred until a verified payload is captured."
                    .to_string(),
            ),
        }
    }
}

impl SyncHandler for AtmExtensionHandler {
    fn handle(&self, context: HookContext) -> Result<HookResult, HookError> {
        match context.hook {
            HookType::PreToolUse => handle_pre_tool_use(context),
            HookType::PostToolUse => handle_post_tool_use(context),
            HookType::PermissionRequest => handle_permission_request(context),
            HookType::Stop => handle_stop(context),
            HookType::TeammateIdle | HookType::SubagentStop => handle_teammate_idle(context),
            HookType::Notification => Ok(proceed()),
            HookType::PreCompact
            | HookType::PostCompact
            | HookType::SessionStart
            | HookType::SessionEnd => Ok(proceed()),
            _ => {
                // Keep future hook additions fail-open here; ATM-specific relay work
                // should only promote a new surface after the payload is captured.
                let hook_type = context.hook;
                log::debug!("sc-hooks atm-extension: unhandled hook_type={hook_type} proceeding");
                Ok(proceed())
            }
        }
    }
}

fn handle_pre_tool_use(context: HookContext) -> Result<HookResult, HookError> {
    let payload: BashPayload = context.payload()?;
    if payload.tool_name != "Bash" {
        return Ok(proceed());
    }

    let Some((store, record)) = load_record_for_context(&context)? else {
        return Ok(proceed());
    };
    let payload_value = context.payload_value()?;
    let Some(routing) = resolve_atm_routing(payload_value, Some(&record)) else {
        return Ok(proceed());
    };

    persist_atm_update(
        &store,
        record.clone(),
        &routing,
        RecordUpdate {
            hook_event: "PreToolUse",
            state_reason: "atm_identity_write",
            agent_state: None,
        },
    )?;

    if is_atm_invocation(&payload.tool_input.command) {
        let identity_file = identity_file_path(record.active_pid().get());
        if let Err(err) = write_identity_file(&identity_file, &record, &routing) {
            let identity_file = identity_file.display();
            log::error!("atm-extension: failed to write identity_file={identity_file} error={err}");
        }
    }

    Ok(proceed())
}

fn handle_post_tool_use(context: HookContext) -> Result<HookResult, HookError> {
    let payload: BashPayload = context.payload()?;
    if payload.tool_name != "Bash" {
        return Ok(proceed());
    }

    let Some((store, record)) = load_record_for_context(&context)? else {
        return Ok(proceed());
    };
    let payload_value = context.payload_value()?;
    let Some(routing) = resolve_atm_routing(payload_value, Some(&record)) else {
        return Ok(proceed());
    };

    persist_atm_update(
        &store,
        record.clone(),
        &routing,
        RecordUpdate {
            hook_event: "PostToolUse",
            state_reason: "atm_identity_cleanup",
            agent_state: None,
        },
    )?;

    if is_atm_invocation(&payload.tool_input.command) {
        let identity_file = identity_file_path(record.active_pid().get());
        if let Err(err) = delete_identity_file(&identity_file) {
            let identity_file = identity_file.display();
            log::error!(
                "atm-extension: failed to delete identity_file={identity_file} error={err}"
            );
        }
    }

    Ok(proceed())
}

fn handle_permission_request(context: HookContext) -> Result<HookResult, HookError> {
    let payload: PermissionRequestPayload = context.payload()?;
    let Some((store, record)) = load_record_for_context(&context)? else {
        return Ok(proceed());
    };
    let payload_value = context.payload_value()?;
    let Some(routing) = resolve_atm_routing(payload_value, Some(&record)) else {
        return Ok(proceed());
    };

    let raw_request = RawRequest {
        raw_payload: payload,
        relay: RelayContext {
            routing,
            process_id: record.active_pid().get(),
        },
    };
    let validated = validate_permission_request(raw_request)?;
    let decision = permission_relay_decision(validated);
    let _result = execute_relay(&store, record, decision)?;

    Ok(proceed())
}

fn handle_stop(context: HookContext) -> Result<HookResult, HookError> {
    let payload: StopPayload = context.payload()?;
    let Some((store, record)) = load_record_for_context(&context)? else {
        return Ok(proceed());
    };
    let payload_value = context.payload_value()?;
    let Some(routing) = resolve_atm_routing(payload_value, Some(&record)) else {
        return Ok(proceed());
    };

    let raw_request = RawRequest {
        raw_payload: payload,
        relay: RelayContext {
            routing,
            process_id: record.active_pid().get(),
        },
    };
    let validated = validate_stop_request(raw_request)?;
    let decision = stop_relay_decision(validated);
    let _result = execute_relay(&store, record, decision)?;

    Ok(proceed())
}

fn handle_teammate_idle(context: HookContext) -> Result<HookResult, HookError> {
    let payload: TeammateIdlePayload = context.payload()?;
    let loaded = load_record_for_context(&context)?;
    let record_ref = loaded.as_ref().map(|(_, record)| record);
    let payload_value = context.payload_value()?;
    let Some(routing) = resolve_atm_routing(payload_value, record_ref) else {
        return Ok(proceed());
    };

    let process_id = record_ref
        .map(|record| record.active_pid().get())
        .or_else(resolve_process_id_from_env)
        .unwrap_or_else(std::process::id);

    if let Some((store, record)) = loaded {
        persist_atm_update(
            &store,
            record,
            &routing,
            RecordUpdate {
                hook_event: "TeammateIdle",
                state_reason: "teammate_idle",
                agent_state: Some(AgentState::Idle),
            },
        )?;
    }

    append_relay_event(
        relay_event_root(),
        json!({
            "event": "teammate_idle",
            "session_id": payload.session_id,
            "process_id": process_id,
            "agent": routing.identity,
            "team": routing.team,
            "received_at": utc_timestamp_now(),
            "source": {"kind": "claude_hook"},
        }),
    );

    Ok(proceed())
}

fn load_record_for_context(
    context: &HookContext,
) -> Result<Option<(SessionStore, CanonicalSessionRecord)>, HookError> {
    let state_root = resolve_state_root()?;
    let store = SessionStore::new(state_root);
    let Some(session_id) = context
        .payload_value()?
        .get("session_id")
        .and_then(Value::as_str)
    else {
        return Ok(None);
    };

    let session_id = SessionId::new(session_id.to_string())?;
    let Some(record) = store.load(&session_id)? else {
        return Ok(None);
    };
    Ok(Some((store, record)))
}

fn resolve_atm_routing(
    payload: &Value,
    record: Option<&CanonicalSessionRecord>,
) -> Option<AtmRouting> {
    let payload_tool_input = payload.get("tool_input").and_then(Value::as_object);
    let existing = record.and_then(existing_atm_extension);
    let config =
        record.and_then(|existing_record| load_atm_config(existing_record.ai_root_dir().as_path()));

    let team = first_nonempty([
        string_field(payload, "team_name"),
        string_field(payload, "team"),
        payload_tool_input.and_then(|tool_input| string_field_from_map(tool_input, "team_name")),
        std::env::var("ATM_TEAM")
            .ok()
            .filter(|value| !value.trim().is_empty()),
        existing.as_ref().map(|routing| routing.team.clone()),
        config
            .as_ref()
            .and_then(|atm_config| atm_config.default_team.clone()),
    ])?;

    let identity = first_nonempty([
        string_field(payload, "teammate_name"),
        string_field(payload, "name"),
        string_field(payload, "agent"),
        payload_tool_input.and_then(|tool_input| string_field_from_map(tool_input, "name")),
        std::env::var("ATM_IDENTITY")
            .ok()
            .filter(|value| !value.trim().is_empty()),
        existing.as_ref().map(|routing| routing.identity.clone()),
        config.and_then(|atm_config| atm_config.identity),
    ])?;

    Some(AtmRouting { team, identity })
}

fn existing_atm_extension(record: &CanonicalSessionRecord) -> Option<AtmRouting> {
    let atm = record.extension("atm")?.as_object()?;
    Some(AtmRouting {
        team: atm.get("atm_team")?.as_str()?.to_string(),
        identity: atm.get("atm_identity")?.as_str()?.to_string(),
    })
}

#[derive(Debug)]
struct AtmConfig {
    default_team: Option<String>,
    identity: Option<String>,
}

fn load_atm_config(ai_root_dir: &Path) -> Option<AtmConfig> {
    let path = ai_root_dir.join(".atm.toml");
    let body = fs::read_to_string(path).ok()?;
    let parsed = toml::from_str::<toml::Value>(&body).ok()?;
    let core = parsed.get("core")?.as_table()?;
    Some(AtmConfig {
        default_team: core
            .get("default_team")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        identity: core
            .get("identity")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
    })
}

fn string_field(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn string_field_from_map(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn first_nonempty<const N: usize>(candidates: [Option<String>; N]) -> Option<String> {
    candidates
        .into_iter()
        .flatten()
        .find(|candidate| !candidate.trim().is_empty())
}

fn persist_atm_update(
    store: &SessionStore,
    record: CanonicalSessionRecord,
    routing: &AtmRouting,
    update: RecordUpdate,
) -> Result<(), HookError> {
    if record.agent_state() == AgentState::Ended {
        return Ok(());
    }

    let atm_extension = json!({
        "atm_team": routing.team,
        "atm_identity": routing.identity,
    });

    let mut active = match record.try_into_active() {
        Ok(record) => record,
        Err(_) => return Ok(()),
    };

    let mut material_changed = active.set_extension("atm", atm_extension)?;

    if let Some(agent_state) = update.agent_state
        && active.agent_state() != agent_state
    {
        material_changed = true;
    }

    if active.last_hook_event() != update.hook_event {
        material_changed = true;
    }

    if active.state_reason() != update.state_reason {
        material_changed = true;
    }

    if !material_changed {
        return Ok(());
    }

    let now = utc_timestamp_now();
    let active_pid = active.active_pid();
    let ai_current_dir = active.ai_current_dir().clone();
    let session_start_source = active.session_start_source();
    let agent_state = update.agent_state.unwrap_or(active.agent_state());
    let ended_at = active.ended_at().cloned();
    let record = active
        .apply_hook_update(
            active_pid,
            ai_current_dir,
            session_start_source,
            agent_state,
            now,
            update.hook_event,
            update.state_reason,
            ended_at,
        )?
        .into_record();
    store.persist(&record)?;

    Ok(())
}

fn validate_permission_request(
    raw: RawRequest<PermissionRequestPayload>,
) -> Result<ValidatedRequest<ValidatedPermissionRequest>, HookError> {
    let session_id = SessionId::new(raw.raw_payload.session_id)?;
    let tool_name = ToolName::new(raw.raw_payload.tool_name)?;
    let permission_suggestions = raw
        .raw_payload
        .permission_suggestions
        .as_ref()
        .map(parse_permission_suggestions)
        .transpose()?
        .unwrap_or_default();

    Ok(ValidatedRequest {
        validated: ValidatedPermissionRequest {
            session_id,
            tool_name,
            tool_input: raw.raw_payload.tool_input,
            permission_suggestions,
        },
        relay: raw.relay,
    })
}

fn validate_stop_request(
    raw: RawRequest<StopPayload>,
) -> Result<ValidatedRequest<ValidatedStopRequest>, HookError> {
    Ok(ValidatedRequest {
        validated: ValidatedStopRequest {
            session_id: SessionId::new(raw.raw_payload.session_id)?,
        },
        relay: raw.relay,
    })
}

fn permission_relay_decision(
    request: ValidatedRequest<ValidatedPermissionRequest>,
) -> RelayDecision {
    let process_id = request.relay.process_id;
    let body = json!({
        "event": "permission_request",
        "session_id": request.validated.session_id,
        "process_id": process_id,
        "agent": request.relay.routing.identity,
        "team": request.relay.routing.team,
        "tool_name": request.validated.tool_name.as_str(),
        "tool_input": request.validated.tool_input,
        "permission_suggestions": render_permission_suggestions(&request.validated.permission_suggestions),
        "source": {"kind": "claude_hook"},
    });
    RelayDecision {
        relay: request.relay,
        state_update: RecordUpdate {
            hook_event: "PermissionRequest",
            state_reason: "permission_requested",
            agent_state: Some(AgentState::AwaitingPermission),
        },
        event_body: body,
        cleanup_identity_file: false,
    }
}

fn stop_relay_decision(request: ValidatedRequest<ValidatedStopRequest>) -> RelayDecision {
    let process_id = request.relay.process_id;
    let body = json!({
        "event": "stop",
        "session_id": request.validated.session_id,
        "process_id": process_id,
        "agent": request.relay.routing.identity,
        "team": request.relay.routing.team,
        "source": {"kind": "claude_hook"},
    });
    RelayDecision {
        relay: request.relay,
        state_update: RecordUpdate {
            hook_event: "Stop",
            state_reason: "relay_stop",
            agent_state: Some(AgentState::Idle),
        },
        event_body: body,
        cleanup_identity_file: true,
    }
}

fn execute_relay(
    store: &SessionStore,
    record: CanonicalSessionRecord,
    decision: RelayDecision,
) -> Result<RelayResult, HookError> {
    persist_atm_update(
        store,
        record,
        &decision.relay.routing,
        decision.state_update,
    )?;

    append_relay_event(relay_event_root(), decision.event_body);

    if decision.cleanup_identity_file {
        let identity_file = identity_file_path(decision.relay.process_id);
        if let Err(err) = delete_identity_file(&identity_file) {
            let identity_file = identity_file.display();
            log::error!(
                "atm-extension: failed to delete identity_file={identity_file} error={err}"
            );
        }
    }
    Ok(RelayResult::Applied)
}

fn relay_event_root() -> Option<PathBuf> {
    std::env::var_os("ATM_HOME")
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
}

fn append_relay_event(root: Option<PathBuf>, event: Value) {
    let Some(root) = root else {
        return;
    };
    let events_path = root
        .join(".atm")
        .join("daemon")
        .join("hooks")
        .join("events.jsonl");
    let result = (|| -> std::io::Result<()> {
        if let Some(parent) = events_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&events_path)?;
        writeln!(file, "{}", serde_json::to_string(&event)?)?;
        Ok(())
    })();

    if let Err(err) = result {
        let relay_event_path = events_path.display();
        log::error!(
            "atm-extension: failed to append relay_event_path={relay_event_path} error={err}"
        );
    }
}

fn identity_file_path(active_pid: u32) -> PathBuf {
    // Tests override ATM_HOOK_TMP_DIR to inject a writable temp path without
    // touching TMPDIR (which tempfile::tempdir() reads, causing race conditions
    // when tests call tempdir() before the EnvGuard mutex is acquired).
    let base = std::env::var_os("ATM_HOOK_TMP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join(format!("atm-hook-{active_pid}.json"))
}

fn write_identity_file(
    path: &Path,
    record: &CanonicalSessionRecord,
    routing: &AtmRouting,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let created_at = utc_timestamp_now();
    let rendered = serde_json::to_string(&json!({
        "pid": record.active_pid().get(),
        "session_id": record.session_id().as_str(),
        "agent_name": routing.identity,
        "team_name": routing.team,
        "created_at": created_at.as_str(),
    }))?;
    fs::write(path, rendered)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn delete_identity_file(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn is_atm_invocation(command: &str) -> bool {
    let tokens = shell_words::split(command).unwrap_or_else(|err| {
        log::warn!(
            "[atm-extension] shell_words parse error command={command:?} error={err}; falling back to whitespace split"
        );
        command
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>()
    });

    tokens.iter().any(|token| {
        token == "atm"
            || token.ends_with("/atm")
            || token.ends_with("\\atm")
            || token == "atm.exe"
            || token.ends_with("/atm.exe")
            || token.ends_with("\\atm.exe")
    })
}

fn resolve_process_id_from_env() -> Option<u32> {
    std::env::var("SC_HOOK_AGENT_PID")
        .ok()
        .and_then(|raw| raw.parse::<u32>().ok())
        .filter(|value| *value > 0)
}

fn parse_permission_suggestions(value: &Value) -> Result<Vec<PermissionSuggestion>, HookError> {
    let suggestions = value.as_array().ok_or_else(|| {
        HookError::validation("permission_suggestions", "must be an array when present")
    })?;
    let mut parsed = Vec::with_capacity(suggestions.len());
    for (index, suggestion) in suggestions.iter().enumerate() {
        let suggestion_object = suggestion.as_object().ok_or_else(|| {
            HookError::validation(
                format!("permission_suggestions[{index}]"),
                "must be an object",
            )
        })?;
        let suggestion_type = SuggestionType::new(
            required_string_field(suggestion_object, index, "type")?,
            format!("permission_suggestions[{index}].type"),
        )?;
        let destination = optional_string_field(suggestion_object, index, "destination")?;
        let mode = optional_string_field(suggestion_object, index, "mode")?;
        let behavior = optional_string_field(suggestion_object, index, "behavior")?;
        let rules = parse_permission_rules(suggestion_object, index)?;

        parsed.push(PermissionSuggestion {
            suggestion_type,
            behavior,
            destination,
            mode,
            rules,
        });
    }
    Ok(parsed)
}

fn parse_permission_rules(
    suggestion_object: &Map<String, Value>,
    suggestion_index: usize,
) -> Result<Vec<PermissionSuggestionRule>, HookError> {
    let Some(rules) = suggestion_object.get("rules") else {
        return Ok(Vec::new());
    };
    let rules = rules.as_array().ok_or_else(|| {
        HookError::validation(
            format!("permission_suggestions[{suggestion_index}].rules"),
            "must be an array",
        )
    })?;
    let mut parsed = Vec::with_capacity(rules.len());
    for (rule_index, rule) in rules.iter().enumerate() {
        let rule_object = rule.as_object().ok_or_else(|| {
            HookError::validation(
                format!("permission_suggestions[{suggestion_index}].rules[{rule_index}]"),
                "must be an object",
            )
        })?;
        let tool_name = parse_tool_name(
            required_rule_string_field(rule_object, suggestion_index, rule_index, "toolName")?,
            format!("permission_suggestions[{suggestion_index}].rules[{rule_index}].toolName"),
        )?;
        let rule_content = RuleContent::new(
            required_rule_string_field(rule_object, suggestion_index, rule_index, "ruleContent")?,
            format!("permission_suggestions[{suggestion_index}].rules[{rule_index}].ruleContent"),
        )?;
        parsed.push(PermissionSuggestionRule {
            tool_name,
            rule_content,
        });
    }
    Ok(parsed)
}

fn optional_string_field(
    suggestion_object: &Map<String, Value>,
    suggestion_index: usize,
    field_name: &str,
) -> Result<Option<String>, HookError> {
    let Some(value) = suggestion_object.get(field_name) else {
        return Ok(None);
    };
    let Some(value) = value.as_str() else {
        return Err(HookError::validation(
            format!("permission_suggestions[{suggestion_index}].{field_name}"),
            "must be a string",
        ));
    };
    Ok(Some(value.to_string()))
}

fn required_string_field(
    object: &Map<String, Value>,
    suggestion_index: usize,
    field_name: &str,
) -> Result<String, HookError> {
    let Some(value) = object.get(field_name) else {
        return Err(HookError::validation(
            format!("permission_suggestions[{suggestion_index}].{field_name}"),
            "is required",
        ));
    };
    let Some(value) = value.as_str() else {
        return Err(HookError::validation(
            format!("permission_suggestions[{suggestion_index}].{field_name}"),
            "must be a string",
        ));
    };
    Ok(value.to_string())
}

fn required_rule_string_field(
    object: &Map<String, Value>,
    suggestion_index: usize,
    rule_index: usize,
    field_name: &str,
) -> Result<String, HookError> {
    let Some(value) = object.get(field_name) else {
        return Err(HookError::validation(
            format!("permission_suggestions[{suggestion_index}].rules[{rule_index}].{field_name}"),
            "is required",
        ));
    };
    let Some(value) = value.as_str() else {
        return Err(HookError::validation(
            format!("permission_suggestions[{suggestion_index}].rules[{rule_index}].{field_name}"),
            "must be a string",
        ));
    };
    Ok(value.to_string())
}

impl SuggestionType {
    fn new(value: String, field: String) -> Result<Self, HookError> {
        new_nonempty_string(value, field).map(Self)
    }
}

impl RuleContent {
    fn new(value: String, field: String) -> Result<Self, HookError> {
        new_nonempty_string(value, field).map(Self)
    }
}

fn parse_tool_name(value: String, field: String) -> Result<ToolName, HookError> {
    let value = new_nonempty_string(value, field.clone())?;
    ToolName::new(value)
        .map_err(|err| HookError::validation(field, format!("must be a non-empty string: {err}")))
}

fn new_nonempty_string(value: String, field: String) -> Result<String, HookError> {
    if value.trim().is_empty() {
        return Err(HookError::validation(field, "must be a non-empty string"));
    }
    Ok(value)
}

fn render_permission_suggestions(suggestions: &[PermissionSuggestion]) -> Value {
    Value::Array(
        suggestions
            .iter()
            .map(|suggestion| {
                json!({
                    "type": suggestion.suggestion_type.as_str(),
                    "behavior": suggestion.behavior,
                    "destination": suggestion.destination,
                    "mode": suggestion.mode,
                    "rules": suggestion.rules.iter().map(|rule| {
                        json!({
                            "toolName": rule.tool_name.as_str(),
                            "ruleContent": rule.rule_content.as_str(),
                        })
                    }).collect::<Vec<_>>(),
                })
            })
            .collect(),
    )
}
