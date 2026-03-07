use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde_json::{Map, Value};
use thiserror::Error;

use sc_hooks_core::manifest::{FieldRequirement, Manifest};
use sc_hooks_core::validation::{FieldType, parse_validation_rule};

pub const HOST_CONTRACT_VERSION: u32 = 1;

pub fn is_contract_compatible(host_version: u32, plugin_version: u32) -> bool {
    plugin_version <= host_version
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("invalid manifest JSON: {0}")]
    Parse(String),

    #[error("manifest field `name` must be non-empty")]
    EmptyName,

    #[error("manifest must declare at least one hook")]
    EmptyHooks,

    #[error("manifest must declare at least one matcher")]
    EmptyMatchers,

    #[error("manifest timeout_ms must be greater than zero when set")]
    InvalidTimeout,

    #[error("manifest response_time.min_ms must be <= response_time.max_ms")]
    InvalidResponseTimeRange,

    #[error("manifest long_running=true requires a non-empty description")]
    MissingLongRunningDescription,

    #[error(
        "manifest contract_version {plugin_version} is incompatible with host version {host_version}"
    )]
    IncompatibleContractVersion {
        host_version: u32,
        plugin_version: u32,
    },

    #[error("manifest field `{field}` has unknown validation rule `{rule}`")]
    UnknownValidationRule { field: String, rule: String },

    #[error("missing required metadata field `{field}`")]
    MissingRequiredField { field: String },

    #[error("metadata field `{field}` failed validation `{rule}`")]
    ValidationRuleFailed { field: String, rule: String },

    #[error("metadata field `{field}` failed type check `{expected:?}")]
    TypeValidationFailed { field: String, expected: FieldType },

    #[error("payload conditions invalid: {0}")]
    PayloadConditions(String),
}

#[derive(Debug, Error)]
pub enum ManifestLoadError {
    #[error("failed to run plugin manifest command `{path}`: {source}")]
    Spawn {
        path: String,
        source: std::io::Error,
    },

    #[error("plugin `{path}` returned non-zero on --manifest: status={status}, stderr={stderr}")]
    NonZero {
        path: String,
        status: i32,
        stderr: String,
    },

    #[error(transparent)]
    Manifest(#[from] ManifestError),
}

pub fn parse_manifest_str(input: &str) -> Result<Manifest, ManifestError> {
    let manifest = serde_json::from_str::<Manifest>(input)
        .map_err(|err| ManifestError::Parse(err.to_string()))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn load_manifest_from_executable(path: &Path) -> Result<Manifest, ManifestLoadError> {
    let output = Command::new(path)
        .arg("--manifest")
        .output()
        .map_err(|source| ManifestLoadError::Spawn {
            path: path.display().to_string(),
            source,
        })?;

    if !output.status.success() {
        return Err(ManifestLoadError::NonZero {
            path: path.display().to_string(),
            status: output.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    parse_manifest_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(ManifestLoadError::Manifest)
}

pub fn validate_manifest(manifest: &Manifest) -> Result<(), ManifestError> {
    if manifest.name.trim().is_empty() {
        return Err(ManifestError::EmptyName);
    }

    if manifest.hooks.is_empty() {
        return Err(ManifestError::EmptyHooks);
    }

    if manifest.matchers.is_empty() {
        return Err(ManifestError::EmptyMatchers);
    }

    if !is_contract_compatible(HOST_CONTRACT_VERSION, manifest.contract_version) {
        return Err(ManifestError::IncompatibleContractVersion {
            host_version: HOST_CONTRACT_VERSION,
            plugin_version: manifest.contract_version,
        });
    }

    if matches!(manifest.timeout_ms, Some(0)) {
        return Err(ManifestError::InvalidTimeout);
    }

    if let Some(response_time) = &manifest.response_time
        && response_time.min_ms > response_time.max_ms
    {
        return Err(ManifestError::InvalidResponseTimeRange);
    }

    if manifest.long_running
        && manifest
            .description
            .as_ref()
            .map(|description| description.trim().is_empty())
            .unwrap_or(true)
    {
        return Err(ManifestError::MissingLongRunningDescription);
    }

    validate_field_specs(&manifest.requires)?;
    validate_field_specs(&manifest.optional)?;
    crate::conditions::validate_payload_conditions(&manifest.payload_conditions)
        .map_err(|err| ManifestError::PayloadConditions(err.to_string()))?;

    Ok(())
}

fn validate_field_specs(fields: &BTreeMap<String, FieldRequirement>) -> Result<(), ManifestError> {
    for (field, spec) in fields {
        if let Some(rule) = &spec.validate
            && parse_validation_rule(rule).is_none()
        {
            return Err(ManifestError::UnknownValidationRule {
                field: field.clone(),
                rule: rule.clone(),
            });
        }
    }

    Ok(())
}

pub fn build_plugin_input(
    manifest: &Manifest,
    metadata: &Value,
    hook_type: &str,
    event: Option<&str>,
    payload: Option<&Value>,
) -> Result<Value, ManifestError> {
    let mut root = Map::new();

    for (field, spec) in &manifest.requires {
        let value = get_value_by_path(metadata, field).ok_or_else(|| {
            ManifestError::MissingRequiredField {
                field: field.clone(),
            }
        })?;

        validate_field_value(field, value, spec)?;
        set_value_by_path(&mut root, field, value.clone());
    }

    for (field, spec) in &manifest.optional {
        if let Some(value) = get_value_by_path(metadata, field) {
            validate_field_value(field, value, spec)?;
            set_value_by_path(&mut root, field, value.clone());
        }
    }

    let mut hook = Map::new();
    hook.insert("type".to_string(), Value::String(hook_type.to_string()));
    if let Some(event) = event {
        hook.insert("event".to_string(), Value::String(event.to_string()));
    }
    root.insert("hook".to_string(), Value::Object(hook));

    if let Some(payload) = payload {
        root.insert("payload".to_string(), payload.clone());
    }

    Ok(Value::Object(root))
}

fn validate_field_value(
    field: &str,
    value: &Value,
    spec: &FieldRequirement,
) -> Result<(), ManifestError> {
    if !type_matches(value, &spec.field_type) {
        return Err(ManifestError::TypeValidationFailed {
            field: field.to_string(),
            expected: spec.field_type.clone(),
        });
    }

    if let Some(rule) = &spec.validate {
        validate_rule(field, value, rule)?;
    }

    Ok(())
}

fn type_matches(value: &Value, expected: &FieldType) -> bool {
    match expected {
        FieldType::Any => true,
        FieldType::String => value.is_string(),
        FieldType::Number => value.is_number(),
        FieldType::Integer => value.as_i64().is_some() || value.as_u64().is_some(),
        FieldType::Boolean => value.is_boolean(),
        FieldType::Object => value.is_object(),
        FieldType::Array => value.is_array(),
    }
}

fn validate_rule(field: &str, value: &Value, rule: &str) -> Result<(), ManifestError> {
    let fail = |name: &str| ManifestError::ValidationRuleFailed {
        field: field.to_string(),
        rule: name.to_string(),
    };

    match rule {
        "non_empty" => {
            let Some(text) = value.as_str() else {
                return Err(fail("non_empty"));
            };
            if text.trim().is_empty() {
                return Err(fail("non_empty"));
            }
        }
        "dir_exists" => {
            let Some(path) = value.as_str() else {
                return Err(fail("dir_exists"));
            };
            if !Path::new(path).is_dir() {
                return Err(fail("dir_exists"));
            }
        }
        "file_exists" => {
            let Some(path) = value.as_str() else {
                return Err(fail("file_exists"));
            };
            if !Path::new(path).is_file() {
                return Err(fail("file_exists"));
            }
        }
        "path_resolves" => {
            let Some(path) = value.as_str() else {
                return Err(fail("path_resolves"));
            };
            if std::fs::canonicalize(path).is_err() {
                return Err(fail("path_resolves"));
            }
        }
        "positive_int" => {
            let Some(number) = value.as_i64().or_else(|| value.as_u64().map(|n| n as i64)) else {
                return Err(fail("positive_int"));
            };
            if number <= 0 {
                return Err(fail("positive_int"));
            }
        }
        _ if rule.starts_with("one_of:") => {
            let Some(actual) = value.as_str() else {
                return Err(fail("one_of"));
            };
            let allowed = rule
                .trim_start_matches("one_of:")
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .collect::<Vec<_>>();
            if !allowed.iter().any(|entry| entry == &actual) {
                return Err(fail("one_of"));
            }
        }
        _ => {
            return Err(ManifestError::UnknownValidationRule {
                field: field.to_string(),
                rule: rule.to_string(),
            });
        }
    }

    Ok(())
}

fn get_value_by_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current.as_object()?.get(segment)?;
    }
    Some(current)
}

fn set_value_by_path(root: &mut Map<String, Value>, path: &str, value: Value) {
    let mut segments = path.split('.').peekable();
    let mut current = root;

    while let Some(segment) = segments.next() {
        if segments.peek().is_none() {
            current.insert(segment.to_string(), value);
            return;
        }

        if !current.contains_key(segment) {
            current.insert(segment.to_string(), Value::Object(Map::new()));
        }

        let next = current
            .get_mut(segment)
            .and_then(Value::as_object_mut)
            .expect("manifest path collision with non-object value");
        current = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_json() -> &'static str {
        r#"{
  "contract_version": 1,
  "name": "guard-paths",
  "mode": "sync",
  "hooks": ["PreToolUse"],
  "matchers": ["Write", "Bash"],
  "payload_conditions": [{"path":"tool_input.command","op":"contains","value":"atm"}],
  "timeout_ms": 5000,
  "long_running": false,
  "requires": {
    "repo.path": { "type": "string", "validate": "non_empty" }
  },
  "optional": {
    "team.name": { "type": "string" }
  }
}"#
    }

    #[test]
    fn parses_manifest() {
        let manifest = parse_manifest_str(manifest_json()).expect("manifest should parse");
        assert_eq!(manifest.contract_version, 1);
        assert_eq!(manifest.name, "guard-paths");
    }

    #[test]
    fn rejects_newer_contract_version() {
        let json = manifest_json().replace("\"contract_version\": 1", "\"contract_version\": 2");
        let err = parse_manifest_str(&json).expect_err("newer contract version should fail");
        assert!(matches!(
            err,
            ManifestError::IncompatibleContractVersion {
                host_version: 1,
                plugin_version: 2,
            }
        ));
    }

    #[test]
    fn contract_compatibility_allows_downward_and_equal() {
        assert!(is_contract_compatible(1, 1));
        assert!(is_contract_compatible(2, 1));
        assert!(!is_contract_compatible(1, 2));
    }

    #[test]
    fn builds_filtered_input_plus_hook_and_payload_passthrough() {
        let manifest = parse_manifest_str(manifest_json()).expect("manifest should parse");
        let metadata = serde_json::json!({
            "repo": {"path": "/tmp/repo", "branch": "main"},
            "team": {"name": "cal"},
            "ignored": true
        });
        let payload = serde_json::json!({"tool_input": {"command": "atm send"}});

        let input = build_plugin_input(
            &manifest,
            &metadata,
            "PreToolUse",
            Some("Bash"),
            Some(&payload),
        )
        .expect("input should build");

        assert_eq!(
            input,
            serde_json::json!({
                "repo": {"path": "/tmp/repo"},
                "team": {"name": "cal"},
                "hook": {"type": "PreToolUse", "event": "Bash"},
                "payload": {"tool_input": {"command": "atm send"}}
            })
        );
    }
}
