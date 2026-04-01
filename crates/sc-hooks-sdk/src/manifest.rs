use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde_json::{Map, Value};
use thiserror::Error;

use sc_hooks_core::manifest::{FieldRequirement, Manifest};
use sc_hooks_core::validation::{FieldType, parse_validation_rule};

/// Highest manifest contract version understood by the current host.
pub const HOST_CONTRACT_VERSION: u32 = 1;
const STDERR_EXCERPT_LIMIT: usize = 4096;

/// Returns whether a plugin contract version is compatible with the host.
pub fn is_contract_compatible(host_version: u32, plugin_version: u32) -> bool {
    plugin_version <= host_version
}

#[derive(Debug, Error)]
/// Errors produced while parsing or validating plugin manifests.
pub enum ManifestError {
    /// Manifest JSON could not be parsed.
    #[error("invalid manifest JSON: {source}")]
    Parse {
        #[source]
        /// Underlying serde parser error.
        source: serde_json::Error,
    },

    /// Manifest JSON could not be serialized.
    #[error("failed to serialize manifest JSON: {source}")]
    Serialize {
        #[source]
        /// Underlying serde serializer error.
        source: serde_json::Error,
    },

    /// `name` was empty or whitespace-only.
    #[error("manifest field `name` must be non-empty")]
    EmptyName,

    /// No hooks were declared.
    #[error("manifest must declare at least one hook")]
    EmptyHooks,

    /// No matchers were declared.
    #[error("manifest must declare at least one matcher")]
    EmptyMatchers,

    /// `timeout_ms` was zero.
    #[error("manifest timeout_ms must be greater than zero when set")]
    InvalidTimeout,

    /// Response-time minimum exceeded the maximum.
    #[error("manifest response_time.min_ms must be <= response_time.max_ms")]
    InvalidResponseTimeRange,

    /// `long_running=true` was set without a description.
    #[error("manifest long_running=true requires a non-empty description")]
    MissingLongRunningDescription,

    /// `long_running=true` was used on an async handler.
    #[error("manifest long_running=true is only supported for sync handlers")]
    AsyncLongRunningUnsupported,

    /// Plugin and host contract versions are incompatible.
    #[error(
        "manifest contract_version {plugin_version} is incompatible with host version {host_version}"
    )]
    IncompatibleContractVersion {
        /// Host contract version.
        host_version: u32,
        /// Plugin-declared contract version.
        plugin_version: u32,
    },

    /// A manifest field used an unknown validation rule string.
    #[error("manifest field `{field}` has unknown validation rule `{rule}`")]
    UnknownValidationRule {
        /// Manifest field path that referenced the rule.
        field: String,
        /// Unknown rule string.
        rule: String,
    },

    /// Required metadata was missing from host input.
    #[error("missing required metadata field `{field}`")]
    MissingRequiredField {
        /// Missing field path.
        field: String,
    },

    /// Metadata value failed a declared validation rule.
    #[error("metadata field `{field}` failed validation `{rule}` for value {actual}")]
    ValidationRuleFailed {
        /// Field path that failed validation.
        field: String,
        /// Rule string that failed.
        rule: String,
        /// Actual serialized value that failed validation.
        actual: String,
    },

    /// Metadata value failed a declared type check.
    #[error("metadata field `{field}` failed type check `{expected:?}` for value {actual}")]
    TypeValidationFailed {
        /// Field path that failed the type check.
        field: String,
        /// Expected field type.
        expected: FieldType,
        /// Actual serialized value that failed the type check.
        actual: String,
    },

    /// Payload-condition schema was invalid.
    #[error("payload conditions invalid: {source}")]
    PayloadConditions {
        #[source]
        /// Underlying payload-condition validation error.
        source: crate::conditions::ConditionError,
    },

    /// Dot-path expansion collided with a non-object value.
    #[error("manifest field path `{path}` collides with a non-object value")]
    PathCollision {
        /// Dot-separated field path that collided.
        path: String,
    },
}

#[derive(Debug, Error)]
/// Errors produced while invoking a plugin executable for `--manifest`.
pub enum ManifestLoadError {
    /// The executable could not be spawned.
    #[error("failed to run plugin manifest command `{path}`: {source}")]
    Spawn {
        /// Plugin executable path.
        path: String,
        /// Underlying process-spawn error.
        source: std::io::Error,
    },

    /// The executable returned non-zero from `--manifest`.
    #[error("plugin `{path}` returned non-zero on --manifest: status={status}, stderr={stderr}")]
    NonZeroExit {
        /// Plugin executable path.
        path: String,
        /// Exit status code.
        status: i32,
        /// Captured stderr output.
        stderr: String,
    },

    /// The executable terminated from a signal while serving `--manifest`.
    #[error("plugin `{path}` terminated by signal {signal} on --manifest: stderr={stderr}")]
    TerminatedBySignal {
        /// Plugin executable path.
        path: String,
        /// Signal number reported by the operating system.
        signal: i32,
        /// Captured stderr output.
        stderr: String,
    },

    /// The executable terminated without an exit code while serving `--manifest`.
    #[error("plugin `{path}` terminated without an exit status on --manifest: stderr={stderr}")]
    Terminated {
        /// Plugin executable path.
        path: String,
        /// Captured stderr output.
        stderr: String,
    },

    /// Manifest loading succeeded but parsing/validation failed.
    #[error(transparent)]
    Manifest(#[from] ManifestError),
}

/// Parses and validates a manifest from a JSON string.
pub fn parse_manifest_str(input: &str) -> Result<Manifest, ManifestError> {
    let manifest = serde_json::from_str::<Manifest>(input)
        .map_err(|source| ManifestError::Parse { source })?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

/// Loads, parses, and validates a manifest from a plugin executable.
pub fn load_manifest_from_executable(path: &Path) -> Result<Manifest, ManifestLoadError> {
    let output = Command::new(path)
        .arg("--manifest")
        .output()
        .map_err(|source| ManifestLoadError::Spawn {
            path: path.display().to_string(),
            source,
        })?;

    if !output.status.success() {
        let path = path.display().to_string();
        let stderr = capped_stderr(&output.stderr);
        if let Some(status) = output.status.code() {
            return Err(ManifestLoadError::NonZeroExit {
                path,
                status,
                stderr,
            });
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;

            if let Some(signal) = output.status.signal() {
                return Err(ManifestLoadError::TerminatedBySignal {
                    path,
                    signal,
                    stderr,
                });
            }
        }
        return Err(ManifestLoadError::Terminated { path, stderr });
    }

    parse_manifest_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(ManifestLoadError::Manifest)
}

fn capped_stderr(stderr: &[u8]) -> String {
    let rendered = String::from_utf8_lossy(stderr);
    let mut excerpt = String::new();
    let mut truncated = false;
    for (index, ch) in rendered.chars().enumerate() {
        if index == STDERR_EXCERPT_LIMIT {
            truncated = true;
            break;
        }
        excerpt.push(ch);
    }
    if truncated {
        excerpt.push_str("…[truncated]");
    }
    excerpt
}

/// Validates manifest invariants required by the current host contract.
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

    if manifest.long_running && manifest.mode == sc_hooks_core::dispatch::DispatchMode::Async {
        return Err(ManifestError::AsyncLongRunningUnsupported);
    }

    validate_field_specs(&manifest.requires)?;
    validate_field_specs(&manifest.optional)?;
    crate::conditions::validate_payload_conditions(&manifest.payload_conditions)
        .map_err(|source| ManifestError::PayloadConditions { source })?;

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

/// Builds the filtered stdin payload passed from the host to a plugin process.
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
        set_value_by_path(&mut root, field, value)?;
    }

    for (field, spec) in &manifest.optional {
        if let Some(value) = get_value_by_path(metadata, field) {
            validate_field_value(field, value, spec)?;
            set_value_by_path(&mut root, field, value)?;
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
            expected: spec.field_type,
            actual: value.to_string(),
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
        actual: value.to_string(),
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

fn set_value_by_path(
    root: &mut Map<String, Value>,
    path: &str,
    value: &Value,
) -> Result<(), ManifestError> {
    let mut segments = path.split('.').peekable();
    let mut current = root;

    while let Some(segment) = segments.next() {
        if segments.peek().is_none() {
            current.insert(segment.to_string(), value.clone());
            return Ok(());
        }

        if !current.contains_key(segment) {
            current.insert(segment.to_string(), Value::Object(Map::new()));
        }

        let next = current
            .get_mut(segment)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| ManifestError::PathCollision {
                path: path.to_string(),
            })?;
        current = next;
    }

    Ok(())
}

#[derive(Debug, Clone)]
/// Fluent helper for constructing valid plugin manifests in tests and binaries.
pub struct ManifestBuilder {
    manifest: Manifest,
}

impl ManifestBuilder {
    /// Starts a new manifest builder with default host contract values.
    pub fn new(name: impl Into<String>, mode: sc_hooks_core::dispatch::DispatchMode) -> Self {
        Self {
            manifest: Manifest {
                contract_version: HOST_CONTRACT_VERSION,
                name: name.into(),
                mode,
                hooks: Vec::new(),
                matchers: vec!["*".to_string()],
                payload_conditions: Vec::new(),
                timeout_ms: None,
                long_running: false,
                response_time: None,
                requires: BTreeMap::new(),
                optional: BTreeMap::new(),
                sandbox: None,
                description: None,
            },
        }
    }

    /// Replaces the hook list.
    pub fn hooks(mut self, hooks: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.manifest.hooks = hooks.into_iter().map(Into::into).collect();
        self
    }

    /// Replaces the matcher list.
    pub fn matchers(mut self, matchers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.manifest.matchers = matchers.into_iter().map(Into::into).collect();
        self
    }

    /// Sets an explicit timeout in milliseconds.
    pub fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.manifest.timeout_ms = Some(timeout_ms);
        self
    }

    /// Marks the manifest as long-running and records its required description.
    pub fn long_running(mut self, description: impl Into<String>) -> Self {
        self.manifest.long_running = true;
        self.manifest.description = Some(description.into());
        self
    }

    /// Sets the expected minimum and maximum response times.
    pub fn response_time(mut self, min_ms: u64, max_ms: u64) -> Self {
        self.manifest.response_time =
            Some(sc_hooks_core::manifest::ResponseTimeRange { min_ms, max_ms });
        self
    }

    /// Adds a required metadata field declaration.
    pub fn require_field(
        mut self,
        path: impl Into<String>,
        field_type: FieldType,
        validate: Option<impl Into<String>>,
    ) -> Self {
        self.manifest.requires.insert(
            path.into(),
            FieldRequirement {
                field_type,
                validate: validate.map(Into::into),
            },
        );
        self
    }

    /// Adds an optional metadata field declaration.
    pub fn optional_field(
        mut self,
        path: impl Into<String>,
        field_type: FieldType,
        validate: Option<impl Into<String>>,
    ) -> Self {
        self.manifest.optional.insert(
            path.into(),
            FieldRequirement {
                field_type,
                validate: validate.map(Into::into),
            },
        );
        self
    }

    /// Validates and returns the constructed manifest.
    pub fn build(self) -> Result<Manifest, ManifestError> {
        validate_manifest(&self.manifest)?;
        Ok(self.manifest)
    }

    /// Validates and pretty-serializes the constructed manifest.
    pub fn build_json(self) -> Result<String, ManifestError> {
        let manifest = self.build()?;
        serde_json::to_string_pretty(&manifest)
            .map_err(|source| ManifestError::Serialize { source })
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
        let repo_path = std::env::temp_dir().join("repo");
        let repo_path = repo_path.to_string_lossy().to_string();
        let metadata = serde_json::json!({
            "repo": {"path": repo_path, "branch": "main"},
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
                "repo": {"path": repo_path},
                "team": {"name": "cal"},
                "hook": {"type": "PreToolUse", "event": "Bash"},
                "payload": {"tool_input": {"command": "atm send"}}
            })
        );
    }

    #[test]
    fn manifest_builder_creates_valid_manifest() {
        let manifest = ManifestBuilder::new("notify", sc_hooks_core::dispatch::DispatchMode::Async)
            .hooks(["PostToolUse"])
            .matchers(["Write", "Bash"])
            .response_time(100, 1000)
            .optional_field("team.name", FieldType::String, Some("non_empty"))
            .build()
            .expect("builder should produce valid manifest");
        assert_eq!(manifest.name, "notify");
        assert_eq!(manifest.mode, sc_hooks_core::dispatch::DispatchMode::Async);
        assert_eq!(manifest.hooks, vec!["PostToolUse".to_string()]);
    }

    #[test]
    fn rejects_async_long_running_manifest() {
        let err = ManifestBuilder::new("notify", sc_hooks_core::dispatch::DispatchMode::Async)
            .hooks(["PostToolUse"])
            .long_running("wait for remote ack")
            .build()
            .expect_err("async long_running should be rejected");
        assert!(matches!(err, ManifestError::AsyncLongRunningUnsupported));
    }

    #[test]
    fn rejects_long_running_manifest_without_description() {
        let err = ManifestBuilder::new("notify", sc_hooks_core::dispatch::DispatchMode::Sync)
            .hooks(["PostToolUse"])
            .long_running("   ")
            .build()
            .expect_err("long_running manifest should require a non-empty description");
        assert!(matches!(err, ManifestError::MissingLongRunningDescription));
    }

    #[test]
    fn set_value_by_path_rejects_non_object_path_collision() {
        let mut root = Map::new();
        root.insert("team".to_string(), Value::String("ops".to_string()));

        let err = set_value_by_path(
            &mut root,
            "team.name",
            &Value::String("calibration".to_string()),
        )
        .expect_err("non-object path collisions should not panic");

        assert!(matches!(err, ManifestError::PathCollision { path } if path == "team.name"));
    }

    #[test]
    fn serialize_variant_mentions_manifest_json() {
        let err = ManifestError::Serialize {
            source: serde_json::from_str::<Value>("not-json")
                .expect_err("fixture should produce a serde error"),
        };

        assert!(err.to_string().contains("serialize manifest JSON"));
    }
}
