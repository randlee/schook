use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::conditions::PayloadCondition;
use crate::dispatch::DispatchMode;
use crate::events::HookType;
use crate::validation::FieldType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Expected async response-time range advertised by a plugin manifest.
pub struct ResponseTimeRange {
    /// Minimum expected response time in milliseconds.
    pub min_ms: u64,
    /// Maximum expected response time in milliseconds.
    pub max_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
/// Sandbox capabilities requested by a plugin manifest.
pub struct SandboxSpec {
    #[serde(default)]
    /// Additional filesystem paths required by the plugin.
    pub paths: Vec<String>,
    #[serde(default)]
    /// Whether the plugin requires network access.
    pub needs_network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Type and validation requirements for one metadata field path.
pub struct FieldRequirement {
    #[serde(rename = "type")]
    /// Expected JSON type for the field.
    pub field_type: FieldType,
    #[serde(default)]
    /// Optional validation rule string applied after the type check.
    pub validate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "String", into = "String")]
/// Forward-compatible matcher declaration captured from plugin manifests.
pub struct ManifestMatcher(String);

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("manifest matcher must be non-empty")]
pub struct ManifestMatcherError;

impl ManifestMatcher {
    pub fn new(value: impl Into<String>) -> Result<Self, ManifestMatcherError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ManifestMatcherError);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ManifestMatcher {
    type Error = ManifestMatcherError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ManifestMatcher> for String {
    fn from(value: ManifestMatcher) -> Self {
        value.0
    }
}

impl From<&str> for ManifestMatcher {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Complete manifest schema emitted by a plugin executable.
pub struct Manifest {
    /// Contract version understood by the plugin.
    pub contract_version: u32,
    /// Plugin name used for runtime resolution and logging.
    pub name: String,
    /// Dispatch mode declared by the plugin.
    pub mode: DispatchMode,
    /// Hook names handled by the plugin.
    pub hooks: Vec<HookType>,
    /// Matcher/event names accepted by the plugin.
    pub matchers: Vec<ManifestMatcher>,
    #[serde(default)]
    /// Additional payload-condition filters applied before spawn.
    pub payload_conditions: Vec<PayloadCondition>,
    #[serde(default)]
    /// Optional timeout override in milliseconds.
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    /// Whether the plugin is allowed to outlive the default sync timeout.
    pub long_running: bool,
    #[serde(default)]
    /// Expected async response-time bounds.
    pub response_time: Option<ResponseTimeRange>,
    /// Required metadata fields copied into plugin stdin.
    pub requires: BTreeMap<String, FieldRequirement>,
    #[serde(default)]
    /// Optional metadata fields copied into plugin stdin when present.
    pub optional: BTreeMap<String, FieldRequirement>,
    #[serde(default)]
    /// Optional sandbox request block.
    pub sandbox: Option<SandboxSpec>,
    #[serde(default)]
    /// Human-readable manifest description.
    pub description: Option<String>,
}
