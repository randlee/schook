//! PostToolUse(Bash) output gate for fenced JSON extraction and schema checks.
//! Blocks malformed or non-conforming structured output with exact retryable
//! reasons so callers can rerun the tool successfully.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use sc_hooks_core::context::HookContext;
use sc_hooks_core::dispatch::DispatchMode;
use sc_hooks_core::errors::HookError;
use sc_hooks_core::events::HookType;
use sc_hooks_core::manifest::Manifest;
use sc_hooks_core::results::HookResult;
use sc_hooks_core::tools::ToolName;
use sc_hooks_sdk::result::{block, proceed};
use sc_hooks_sdk::traits::{ManifestProvider, SyncHandler};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct ToolOutputGatesHandler;

#[derive(Debug, Deserialize)]
struct BashToolInput {
    #[serde(rename = "command")]
    _command: String,
    #[serde(default, rename = "description")]
    _description: Option<String>,
    #[serde(default, alias = "json_schema", alias = "output_schema")]
    schema: Option<Value>,
    #[serde(default)]
    file_path: Option<PathBuf>,
    #[serde(default)]
    output_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct BashToolResponse {
    #[serde(default)]
    stdout: Option<String>,
    #[serde(default, rename = "stderr")]
    _stderr: Option<String>,
    #[serde(rename = "interrupted")]
    _interrupted: bool,
    #[serde(default, rename = "isImage")]
    _is_image: Option<bool>,
    #[serde(default, rename = "noOutputExpected")]
    _no_output_expected: Option<bool>,
    #[serde(default)]
    file_path: Option<PathBuf>,
    #[serde(default)]
    output_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct PostToolUseBashPayload {
    tool_name: String,
    #[serde(rename = "tool_input")]
    tool_input: BashToolInput,
    tool_response: BashToolResponse,
}

impl ManifestProvider for ToolOutputGatesHandler {
    fn manifest(&self) -> Manifest {
        Manifest {
            contract_version: 1,
            name: "tool-output-gates".to_string(),
            mode: DispatchMode::Sync,
            hooks: vec!["PostToolUse".to_string()],
            matchers: vec!["Bash".to_string()],
            payload_conditions: Vec::new(),
            timeout_ms: Some(2_000),
            long_running: false,
            response_time: None,
            requires: BTreeMap::new(),
            optional: BTreeMap::new(),
            sandbox: None,
            description: Some(
                "Extracts fenced JSON from tool output, validates it, and blocks with retryable failures."
                    .to_string(),
            ),
        }
    }
}

impl SyncHandler for ToolOutputGatesHandler {
    fn handle(&self, context: HookContext) -> Result<HookResult, HookError> {
        if context.hook != HookType::PostToolUse {
            return Ok(proceed());
        }

        let payload: PostToolUseBashPayload = context.payload()?;
        let tool_name = ToolName::new(payload.tool_name.clone())?;
        if tool_name.as_str() != "Bash" {
            return Ok(proceed());
        }

        let Some(schema) = resolve_schema(&payload)? else {
            return Ok(proceed());
        };
        let stdout = payload.tool_response.stdout.as_deref().unwrap_or("");
        let json_value = match extract_single_fenced_json(stdout) {
            Ok(value) => value,
            Err(reason) => return Ok(block(reason)),
        };
        if let Err(reason) = validate_against_schema(&schema, &json_value, "$") {
            return Ok(block(format!(
                "Tool output blocked: {reason}. Retry by emitting exactly one fenced `json` block that matches the declared schema."
            )));
        }

        Ok(proceed())
    }
}

fn resolve_schema(payload: &PostToolUseBashPayload) -> Result<Option<Value>, HookError> {
    if let Some(schema) = payload.tool_input.schema.as_ref() {
        return parse_inline_schema(schema).map(Some);
    }

    if let Some(schema) = discover_sibling_schema(payload)? {
        return Ok(Some(schema));
    }

    let Some(schema_path) = std::env::var_os("SC_HOOK_JSON_SCHEMA_PATH") else {
        return Ok(None);
    };
    let schema_path = PathBuf::from(schema_path);
    load_schema(&schema_path).map(Some)
}

fn parse_inline_schema(schema: &Value) -> Result<Value, HookError> {
    match schema {
        Value::Object(_) | Value::Bool(_) => Ok(schema.clone()),
        Value::String(body) => {
            serde_json::from_str(body).map_err(|source| HookError::InvalidPayload {
                input_excerpt: body.chars().take(120).collect(),
                source: Some(source),
            })
        }
        other => Err(HookError::validation(
            "tool_input.schema",
            format!(
                "expected embedded schema object/bool or JSON string, found {}",
                json_type_name(other)
            ),
        )),
    }
}

fn discover_sibling_schema(payload: &PostToolUseBashPayload) -> Result<Option<Value>, HookError> {
    for output_path in referenced_output_paths(payload) {
        for schema_path in sibling_schema_candidates(&output_path) {
            if schema_path.exists() {
                return load_schema(&schema_path).map(Some);
            }
        }
    }
    Ok(None)
}

fn referenced_output_paths(payload: &PostToolUseBashPayload) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for path in [
        payload.tool_input.file_path.as_ref(),
        payload.tool_input.output_path.as_ref(),
        payload.tool_response.file_path.as_ref(),
        payload.tool_response.output_path.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        push_candidate_path(&mut candidates, path);
    }

    if let Some(stdout) = payload.tool_response.stdout.as_deref() {
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            push_candidate_path(&mut candidates, Path::new(trimmed));
        }
    }

    candidates
}

fn push_candidate_path(paths: &mut Vec<PathBuf>, candidate: &Path) {
    if !candidate.exists() {
        return;
    }
    let owned = candidate.to_path_buf();
    if !paths.iter().any(|existing| existing == &owned) {
        paths.push(owned);
    }
}

fn sibling_schema_candidates(path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        candidates.push(path.with_file_name(format!("{stem}.schema.json")));
    }
    if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
        let pathbuf = path.with_file_name(format!("{name}.schema.json"));
        if !candidates.iter().any(|existing| existing == &pathbuf) {
            candidates.push(pathbuf);
        }
    }
    candidates
}

fn load_schema(path: &Path) -> Result<Value, HookError> {
    let body = fs::read_to_string(path)
        .map_err(|source| HookError::state_io(path.to_path_buf(), source))?;
    serde_json::from_str(&body).map_err(|source| HookError::InvalidPayload {
        input_excerpt: body.chars().take(120).collect(),
        source: Some(source),
    })
}

fn extract_single_fenced_json(stdout: &str) -> Result<Value, String> {
    let blocks = extract_fenced_json_blocks(stdout);
    match blocks.len() {
        0 => Err("Tool output blocked: expected exactly one fenced `json` block, found none. Retry by printing one ```json ... ``` block to stdout.".to_string()),
        1 => serde_json::from_str::<Value>(&blocks[0]).map_err(|err| {
            format!(
                "Tool output blocked: fenced `json` block is invalid JSON ({err}). Retry by emitting exactly one fenced `json` block that matches the declared schema."
            )
        }),
        count => Err(format!(
            "Tool output blocked: expected exactly one fenced `json` block, found {count}. Retry by emitting a single ```json ... ``` block to stdout."
        )),
    }
}

fn extract_fenced_json_blocks(stdout: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut inside = false;
    let mut current = String::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !inside && trimmed == "```json" {
            inside = true;
            current.clear();
            continue;
        }
        if inside && trimmed == "```" {
            blocks.push(current.trim().to_string());
            inside = false;
            current.clear();
            continue;
        }
        if inside {
            current.push_str(line);
            current.push('\n');
        }
    }
    blocks
}

fn validate_against_schema(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    if let Some(expected_type) = schema.get("type").and_then(Value::as_str) {
        validate_type(expected_type, value, path)?;
    }

    if let Some(enum_values) = schema.get("enum").and_then(Value::as_array)
        && !enum_values.iter().any(|candidate| candidate == value)
    {
        return Err(format!("{path} must be one of the schema enum values"));
    }

    if schema.get("type").and_then(Value::as_str) == Some("object") {
        let object = value
            .as_object()
            .ok_or_else(|| format!("{path} must be an object"))?;
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required.iter().filter_map(Value::as_str) {
                if !object.contains_key(key) {
                    return Err(format!("{path}.{key} is required"));
                }
            }
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (key, property_schema) in properties {
                if let Some(field_value) = object.get(key) {
                    validate_against_schema(
                        property_schema,
                        field_value,
                        &format!("{path}.{key}"),
                    )?;
                }
            }
        }
    }

    if schema.get("type").and_then(Value::as_str) == Some("array")
        && let Some(items_schema) = schema.get("items")
    {
        let items = value
            .as_array()
            .ok_or_else(|| format!("{path} must be an array"))?;
        for (index, item) in items.iter().enumerate() {
            validate_against_schema(items_schema, item, &format!("{path}[{index}]"))?;
        }
    }

    Ok(())
}

fn validate_type(expected_type: &str, value: &Value, path: &str) -> Result<(), String> {
    let valid = match expected_type {
        "object" => value.is_object(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "array" => value.is_array(),
        _ => true,
    };

    if valid {
        Ok(())
    } else {
        Err(format!("{path} must be {expected_type}"))
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        key: &'static str,
        original: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set_path(key: &'static str, value: &Path) -> Self {
            let original = std::env::var_os(key);
            // SAFETY: tests serialize env mutation with a process-wide mutex.
            unsafe { std::env::set_var(key, value) };
            Self { key, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.original {
                // SAFETY: tests serialize env mutation with a process-wide mutex.
                unsafe { std::env::set_var(self.key, value) };
            } else {
                // SAFETY: tests serialize env mutation with a process-wide mutex.
                unsafe { std::env::remove_var(self.key) };
            }
        }
    }

    fn bash_context(stdout: &str, tool_name: &str) -> HookContext {
        HookContext::new(
            HookType::PostToolUse,
            Some(tool_name.to_string()),
            serde_json::json!({
                "payload": {
                    "tool_name": tool_name,
                    "tool_input": {
                        "command": "echo",
                        "description": "Emit structured output"
                    },
                    "tool_response": {
                        "stdout": stdout,
                        "stderr": "",
                        "interrupted": false,
                        "isImage": false,
                        "noOutputExpected": false
                    }
                }
            }),
            None,
        )
    }

    fn write_schema(temp: &tempfile::TempDir) -> std::path::PathBuf {
        let path = temp.path().join("schema.json");
        fs::write(
            &path,
            serde_json::json!({
                "type": "object",
                "required": ["status"],
                "properties": {
                    "status": { "type": "string" },
                    "ok": { "type": "boolean" }
                }
            })
            .to_string(),
        )
        .expect("write schema");
        path
    }

    fn bash_context_with_payload(payload: Value) -> HookContext {
        HookContext::new(
            HookType::PostToolUse,
            Some("Bash".to_string()),
            serde_json::json!({ "payload": payload }),
            None,
        )
    }

    #[test]
    fn post_tool_use_bash_routes_through_gate_and_validates_success() {
        let _guard = test_lock().lock().expect("lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let schema_path = write_schema(&temp);
        let _env = EnvGuard::set_path("SC_HOOK_JSON_SCHEMA_PATH", &schema_path);

        let handler = ToolOutputGatesHandler;
        let result = handler
            .handle(bash_context(
                "```json\n{\"status\":\"ok\",\"ok\":true}\n```",
                "Bash",
            ))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);
    }

    #[test]
    fn invalid_output_returns_exact_retryable_reason() {
        let _guard = test_lock().lock().expect("lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let schema_path = write_schema(&temp);
        let _env = EnvGuard::set_path("SC_HOOK_JSON_SCHEMA_PATH", &schema_path);

        let handler = ToolOutputGatesHandler;
        let result = handler
            .handle(bash_context("plain text only", "Bash"))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Block);
        assert_eq!(
            result.reason.as_deref(),
            Some(
                "Tool output blocked: expected exactly one fenced `json` block, found none. Retry by printing one ```json ... ``` block to stdout."
            )
        );
    }

    #[test]
    fn multiple_json_blocks_are_rejected() {
        let _guard = test_lock().lock().expect("lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let schema_path = write_schema(&temp);
        let _env = EnvGuard::set_path("SC_HOOK_JSON_SCHEMA_PATH", &schema_path);

        let handler = ToolOutputGatesHandler;
        let result = handler
            .handle(bash_context(
                "```json\n{\"status\":\"ok\"}\n```\n```json\n{\"status\":\"dup\"}\n```",
                "Bash",
            ))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Block);
        assert_eq!(
            result.reason.as_deref(),
            Some(
                "Tool output blocked: expected exactly one fenced `json` block, found 2. Retry by emitting a single ```json ... ``` block to stdout."
            )
        );
    }

    #[test]
    fn schema_validation_failures_are_retryable() {
        let _guard = test_lock().lock().expect("lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let schema_path = write_schema(&temp);
        let _env = EnvGuard::set_path("SC_HOOK_JSON_SCHEMA_PATH", &schema_path);

        let handler = ToolOutputGatesHandler;
        let result = handler
            .handle(bash_context("```json\n{\"ok\":true}\n```", "Bash"))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Block);
        assert_eq!(
            result.reason.as_deref(),
            Some(
                "Tool output blocked: $.status is required. Retry by emitting exactly one fenced `json` block that matches the declared schema."
            )
        );
    }

    #[test]
    fn non_bash_payloads_are_ignored() {
        let handler = ToolOutputGatesHandler;
        let result = handler
            .handle(bash_context("```json\n{\"status\":\"ok\"}\n```", "Agent"))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);
    }

    #[test]
    fn missing_schema_path_is_a_noop() {
        let _guard = test_lock().lock().expect("lock");
        // SAFETY: tests serialize env mutation with a process-wide mutex.
        unsafe { std::env::remove_var("SC_HOOK_JSON_SCHEMA_PATH") };

        let handler = ToolOutputGatesHandler;
        let result = handler
            .handle(bash_context("```json\n{\"status\":\"ok\"}\n```", "Bash"))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);
    }

    #[test]
    fn inline_schema_definition_takes_priority() {
        let _guard = test_lock().lock().expect("lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let fallback_schema_path = write_schema(&temp);
        let _env = EnvGuard::set_path("SC_HOOK_JSON_SCHEMA_PATH", &fallback_schema_path);

        let handler = ToolOutputGatesHandler;
        let result = handler
            .handle(bash_context_with_payload(serde_json::json!({
                "tool_name": "Bash",
                "tool_input": {
                    "command": "echo",
                    "description": "Emit structured output",
                    "schema": {
                        "type": "object",
                        "required": ["state"],
                        "properties": {
                            "state": { "type": "string" }
                        }
                    }
                },
                "tool_response": {
                    "stdout": "```json\n{\"state\":\"ok\"}\n```",
                    "stderr": "",
                    "interrupted": false,
                    "isImage": false,
                    "noOutputExpected": false
                }
            })))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);
    }

    #[test]
    fn sibling_schema_file_is_discovered_from_referenced_output_path() {
        let _guard = test_lock().lock().expect("lock");
        // SAFETY: tests serialize env mutation with a process-wide mutex.
        unsafe { std::env::remove_var("SC_HOOK_JSON_SCHEMA_PATH") };
        let temp = tempfile::tempdir().expect("tempdir");
        let output_path = temp.path().join("report.json");
        fs::write(&output_path, "{}").expect("write output");
        let sibling_schema = temp.path().join("report.schema.json");
        fs::write(
            &sibling_schema,
            serde_json::json!({
                "type": "object",
                "required": ["status"],
                "properties": {
                    "status": { "type": "string" }
                }
            })
            .to_string(),
        )
        .expect("write schema");

        let handler = ToolOutputGatesHandler;
        let result = handler
            .handle(bash_context_with_payload(serde_json::json!({
                "tool_name": "Bash",
                "tool_input": {
                    "command": "cat",
                    "description": "Render report"
                },
                "tool_response": {
                    "stdout": format!("{}\n```json\n{{\"status\":\"ok\"}}\n```", output_path.display()),
                    "stderr": "",
                    "interrupted": false,
                    "isImage": false,
                    "noOutputExpected": false,
                    "file_path": output_path
                }
            })))
            .expect("handler result");

        assert_eq!(result.action, sc_hooks_core::results::HookAction::Proceed);
    }

    #[test]
    fn enum_validation_failures_are_retryable() {
        let schema = serde_json::json!({
            "type": "string",
            "enum": ["ok", "warn"]
        });

        let err = validate_against_schema(&schema, &serde_json::json!("bad"), "$")
            .expect_err("enum mismatch should fail");
        assert_eq!(err, "$ must be one of the schema enum values");
    }

    #[test]
    fn array_item_validation_is_enforced() {
        let schema = serde_json::json!({
            "type": "array",
            "items": { "type": "integer" }
        });

        let err = validate_against_schema(&schema, &serde_json::json!([1, "oops"]), "$")
            .expect_err("mixed array should fail");
        assert_eq!(err, "$[1] must be integer");
    }
}
