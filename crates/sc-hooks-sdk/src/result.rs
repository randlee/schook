//! Result helpers for Rust-authored `sc-hooks` plugins.

use serde::{Deserialize, Serialize};

use sc_hooks_core::errors::HookError;
pub use sc_hooks_core::results::{HookAction, HookResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Helper value for async handlers that want to return context/message additions.
pub struct AsyncResult {
    /// Additional context appended to Claude after the async hook finishes.
    pub additional_context: Option<String>,
    /// System message content appended to Claude after the async hook finishes.
    pub system_message: Option<String>,
}

impl AsyncResult {
    /// Returns an empty async result with no additional context.
    pub fn empty() -> Self {
        Self {
            additional_context: None,
            system_message: None,
        }
    }

    /// Returns an async result carrying only `additional_context`.
    pub fn with_context(context: impl Into<String>) -> Self {
        Self {
            additional_context: Some(context.into()),
            system_message: None,
        }
    }

    /// Returns an async result carrying only `system_message`.
    pub fn with_system_message(message: impl Into<String>) -> Self {
        Self {
            additional_context: None,
            system_message: Some(message.into()),
        }
    }

    /// Converts the async helper into a standard `HookResult`.
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

/// Builds a `proceed` hook result with no extra fields.
pub fn proceed() -> HookResult {
    HookResult {
        action: HookAction::Proceed,
        reason: None,
        message: None,
        additional_context: None,
        system_message: None,
    }
}

/// Builds a blocking hook result with a retryable reason string.
pub fn block(reason: impl Into<String>) -> HookResult {
    HookResult {
        action: HookAction::Block,
        reason: Some(reason.into()),
        message: None,
        additional_context: None,
        system_message: None,
    }
}

/// Builds an error hook result with a message.
pub fn error(message: impl Into<String>) -> HookResult {
    HookResult {
        action: HookAction::Error,
        reason: None,
        message: Some(message.into()),
        additional_context: None,
        system_message: None,
    }
}

/// Converts a typed `HookError` into the public `HookResult` error shape.
pub fn error_from_hook_error(error: &HookError) -> HookResult {
    let kind = match error {
        HookError::InvalidPayload { .. } => "invalid_payload",
        HookError::InvalidContext { .. } => "invalid_context",
        HookError::StateIo { .. } => "state_io",
        HookError::Validation { .. } => "validation",
        HookError::RootDivergence { .. } => "root_divergence",
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
    use sc_hooks_core::errors::HookError;

    #[test]
    fn async_result_converts_to_proceed_hook_result() {
        let result = AsyncResult::with_context("hello").into_hook_result();
        assert_eq!(result.action, HookAction::Proceed);
        assert_eq!(result.additional_context, Some("hello".to_string()));
    }

    #[test]
    fn async_result_helpers_cover_empty_and_system_message() {
        assert_eq!(AsyncResult::empty().additional_context, None);
        assert_eq!(
            AsyncResult::with_system_message("system").system_message,
            Some("system".to_string())
        );
    }

    #[test]
    fn hook_result_constructors_set_expected_fields() {
        let proceed_result = proceed();
        assert_eq!(proceed_result.action, HookAction::Proceed);
        assert_eq!(proceed_result.reason, None);

        let block_result = block("retry");
        assert_eq!(block_result.action, HookAction::Block);
        assert_eq!(block_result.reason.as_deref(), Some("retry"));

        let error_result = error("boom");
        assert_eq!(error_result.action, HookAction::Error);
        assert_eq!(error_result.message.as_deref(), Some("boom"));
    }

    #[test]
    fn error_from_hook_error_maps_all_hook_error_kinds() {
        let invalid_payload = HookError::InvalidPayload {
            input_excerpt: "{oops".to_string(),
            source: None,
        };
        let invalid_context = HookError::invalid_context("ctx");
        let state_path = std::env::temp_dir().join("state.json");
        let state_io = HookError::state_io(state_path, std::io::Error::other("disk"));
        let validation = HookError::validation("field", "bad");
        let root_divergence = HookError::root_divergence(
            sc_hooks_core::session::AiRootDir::new("/repo").expect("root"),
            "/other",
            sc_hooks_core::events::HookType::SessionStart,
        );
        let internal = HookError::internal("internal");

        let cases = [
            (invalid_payload, "hook_error_kind=invalid_payload"),
            (invalid_context, "hook_error_kind=invalid_context"),
            (state_io, "hook_error_kind=state_io"),
            (validation, "hook_error_kind=validation"),
            (root_divergence, "hook_error_kind=root_divergence"),
            (internal, "hook_error_kind=internal"),
        ];

        for (error, expected_context) in cases {
            let result = error_from_hook_error(&error);
            assert_eq!(result.action, HookAction::Error);
            assert_eq!(result.additional_context.as_deref(), Some(expected_context));
        }
    }
}
