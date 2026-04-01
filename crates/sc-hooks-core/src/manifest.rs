use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::conditions::PayloadCondition;
use crate::dispatch::DispatchMode;
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
    pub hooks: Vec<String>,
    /// Matcher/event names accepted by the plugin.
    pub matchers: Vec<String>,
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
