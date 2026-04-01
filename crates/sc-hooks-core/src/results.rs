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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::dispatch::DispatchMode;
    use crate::manifest::{FieldRequirement, Manifest};
    use crate::validation::FieldType;

    #[test]
    fn hook_result_unknown_fields_are_dropped_on_round_trip() {
        let payload = json!({
            "action": "proceed",
            "reason": "ok",
            "unknown_field": "ignored"
        });

        let result: HookResult =
            serde_json::from_value(payload).expect("hook result should deserialize");
        let serialized = serde_json::to_value(result).expect("hook result should serialize");

        assert_eq!(serialized["action"], "proceed");
        assert_eq!(serialized["reason"], "ok");
        assert!(serialized.get("unknown_field").is_none());
    }

    #[test]
    fn manifest_unknown_fields_are_dropped_on_round_trip() {
        let mut requires = BTreeMap::new();
        requires.insert(
            "repo.root".to_string(),
            FieldRequirement {
                field_type: FieldType::String,
                validate: Some("non_empty".to_string()),
            },
        );

        let payload = json!({
            "contract_version": 1,
            "name": "demo",
            "mode": "sync",
            "hooks": ["PreToolUse"],
            "matchers": ["*"],
            "requires": requires,
            "unknown_field": "ignored"
        });

        let manifest: Manifest =
            serde_json::from_value(payload).expect("manifest should deserialize");
        let serialized = serde_json::to_value(manifest).expect("manifest should serialize");

        assert_eq!(serialized["name"], "demo");
        assert_eq!(
            serialized["mode"],
            serde_json::to_value(DispatchMode::Sync).unwrap()
        );
        assert!(serialized.get("unknown_field").is_none());
    }
}
