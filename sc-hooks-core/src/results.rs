use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// Public hook action returned by a plugin.
pub enum HookAction {
    /// Continue executing the hook chain.
    Proceed,
    /// Block the host action.
    Block,
    /// Treat the hook as failed.
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Host/plugin result envelope exchanged over stdout JSON.
pub struct HookResult {
    /// Requested hook action.
    pub action: HookAction,
    #[serde(default)]
    /// Optional block reason.
    pub reason: Option<String>,
    #[serde(default)]
    /// Optional error message.
    pub message: Option<String>,
    #[serde(default, rename = "additionalContext")]
    /// Additional Claude context appended on async proceed flows.
    pub additional_context: Option<String>,
    #[serde(default, rename = "systemMessage")]
    /// Additional Claude system message appended on async proceed flows.
    pub system_message: Option<String>,
}
