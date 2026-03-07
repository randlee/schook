use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HookAction {
    Proceed,
    Block,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookResult {
    pub action: HookAction,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default, rename = "additionalContext")]
    pub additional_context: Option<String>,
    #[serde(default, rename = "systemMessage")]
    pub system_message: Option<String>,
}
