use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::conditions::PayloadCondition;
use crate::dispatch::DispatchMode;
use crate::validation::FieldType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseTimeRange {
    pub min_ms: u64,
    pub max_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SandboxSpec {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub needs_network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldRequirement {
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub validate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub contract_version: u32,
    pub name: String,
    pub mode: DispatchMode,
    pub hooks: Vec<String>,
    pub matchers: Vec<String>,
    #[serde(default)]
    pub payload_conditions: Vec<PayloadCondition>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub long_running: bool,
    #[serde(default)]
    pub response_time: Option<ResponseTimeRange>,
    pub requires: BTreeMap<String, FieldRequirement>,
    #[serde(default)]
    pub optional: BTreeMap<String, FieldRequirement>,
    #[serde(default)]
    pub sandbox: Option<SandboxSpec>,
    #[serde(default)]
    pub description: Option<String>,
}
