use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::errors::HookError;
use crate::events::HookType;

#[derive(Debug, Clone, PartialEq)]
pub struct HookContext {
    pub hook: HookType,
    pub event: Option<String>,
    raw_input: Value,
    pub metadata_path: Option<PathBuf>,
}

impl HookContext {
    pub fn new(
        hook: HookType,
        event: Option<String>,
        raw_input: Value,
        metadata_path: Option<PathBuf>,
    ) -> Self {
        Self {
            hook,
            event,
            raw_input,
            metadata_path,
        }
    }

    pub fn payload_value(&self) -> Result<&Value, HookError> {
        self.raw_input
            .get("payload")
            .ok_or_else(|| HookError::validation("payload", "missing payload object"))
    }

    pub fn payload<T: DeserializeOwned>(&self) -> Result<T, HookError> {
        let payload = self.payload_value()?;
        serde_json::from_value(payload.clone()).map_err(|source| HookError::InvalidPayload {
            input_excerpt: excerpt(payload),
            source: Some(source),
        })
    }
}

fn excerpt(value: &Value) -> String {
    let rendered = match serde_json::to_string(value) {
        Ok(body) => body,
        Err(err) => format!("<unrenderable: {err}>"),
    };
    rendered.chars().take(120).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_deserializes_from_raw_input() {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct Payload {
            session_id: String,
        }

        let context = HookContext::new(
            HookType::SessionStart,
            None,
            serde_json::json!({
                "hook": { "type": "SessionStart" },
                "payload": { "session_id": "abc" }
            }),
            None,
        );

        let payload: Payload = context.payload().expect("payload should deserialize");
        assert_eq!(
            payload,
            Payload {
                session_id: "abc".to_string()
            }
        );
    }
}
