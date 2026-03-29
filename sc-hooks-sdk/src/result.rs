use serde::{Deserialize, Serialize};

use sc_hooks_core::errors::HookError;
pub use sc_hooks_core::results::{HookAction, HookResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsyncResult {
    pub additional_context: Option<String>,
    pub system_message: Option<String>,
}

impl AsyncResult {
    pub fn empty() -> Self {
        Self {
            additional_context: None,
            system_message: None,
        }
    }

    pub fn with_context(context: impl Into<String>) -> Self {
        Self {
            additional_context: Some(context.into()),
            system_message: None,
        }
    }

    pub fn with_system_message(message: impl Into<String>) -> Self {
        Self {
            additional_context: None,
            system_message: Some(message.into()),
        }
    }

    pub fn into_hook_result(self) -> HookResult {
        HookResult {
            action: HookAction::Proceed,
            reason: None,
            message: None,
            additional_context: self.additional_context,
            system_message: self.system_message,
        }
    }
}

pub fn proceed() -> HookResult {
    HookResult {
        action: HookAction::Proceed,
        reason: None,
        message: None,
        additional_context: None,
        system_message: None,
    }
}

pub fn block(reason: impl Into<String>) -> HookResult {
    HookResult {
        action: HookAction::Block,
        reason: Some(reason.into()),
        message: None,
        additional_context: None,
        system_message: None,
    }
}

pub fn error(message: impl Into<String>) -> HookResult {
    HookResult {
        action: HookAction::Error,
        reason: None,
        message: Some(message.into()),
        additional_context: None,
        system_message: None,
    }
}

pub fn error_from_hook_error(error: &HookError) -> HookResult {
    let kind = match error {
        HookError::InvalidPayload { .. } => "invalid_payload",
        HookError::InvalidContext { .. } => "invalid_context",
        HookError::StateIo { .. } => "state_io",
        HookError::Validation { .. } => "validation",
        HookError::Internal { .. } => "internal",
    };

    HookResult {
        action: HookAction::Error,
        reason: None,
        message: Some(error.to_string()),
        additional_context: Some(format!("hook_error_kind={kind}")),
        system_message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_result_converts_to_proceed_hook_result() {
        let result = AsyncResult::with_context("hello").into_hook_result();
        assert_eq!(result.action, HookAction::Proceed);
        assert_eq!(result.additional_context, Some("hello".to_string()));
    }
}
