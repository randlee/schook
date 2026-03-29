use sc_hooks_core::context::HookContext;
use sc_hooks_core::errors::HookError;
use sc_hooks_core::manifest::Manifest;

use crate::result::{AsyncResult, HookResult};

// These traits remain intentionally unsealed because runtime handler
// implementations live in sibling plugin crates across the workspace rather than
// inside sc-hooks-sdk itself.
pub trait ManifestProvider {
    fn manifest(&self) -> Manifest;
}

// Intentionally unsealed for cross-crate plugin implementations.
pub trait SyncHandler: ManifestProvider {
    fn handle(&self, context: HookContext) -> Result<HookResult, HookError>;
}

// Intentionally unsealed for cross-crate plugin implementations.
pub trait AsyncHandler: ManifestProvider {
    fn handle_async(&self, context: HookContext) -> Result<AsyncResult, HookError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_hooks_core::dispatch::DispatchMode;
    use sc_hooks_core::manifest::Manifest;
    use std::collections::BTreeMap;

    struct DummySync;

    impl ManifestProvider for DummySync {
        fn manifest(&self) -> Manifest {
            Manifest {
                contract_version: 1,
                name: "dummy-sync".to_string(),
                mode: DispatchMode::Sync,
                hooks: vec!["PreToolUse".to_string()],
                matchers: vec!["Write".to_string()],
                payload_conditions: Vec::new(),
                timeout_ms: Some(1_000),
                long_running: false,
                response_time: None,
                requires: BTreeMap::new(),
                optional: BTreeMap::new(),
                sandbox: None,
                description: None,
            }
        }
    }

    impl SyncHandler for DummySync {
        fn handle(&self, _context: HookContext) -> Result<HookResult, HookError> {
            Ok(crate::result::proceed())
        }
    }

    #[test]
    fn sync_handler_trait_is_usable() {
        let handler = DummySync;
        let output = handler
            .handle(HookContext::new(
                sc_hooks_core::events::HookType::PreToolUse,
                Some("Write".to_string()),
                serde_json::json!({}),
                None,
            ))
            .expect("sync handler should succeed");
        assert_eq!(output.action, sc_hooks_core::results::HookAction::Proceed);
    }
}
