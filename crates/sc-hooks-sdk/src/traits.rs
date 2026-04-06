use sc_hooks_core::context::HookContext;
use sc_hooks_core::errors::HookError;
use sc_hooks_core::manifest::Manifest;

use crate::result::{AsyncResult, HookResult};

/// Public manifest provider surface used by runtime plugin crates.
///
/// This trait remains intentionally unsealed because handler implementations
/// live in sibling workspace crates rather than inside `sc-hooks-sdk` itself;
/// see `SEAL-001` in `docs/implementation-gaps.md`.
pub trait ManifestProvider {
    /// Returns the manifest advertised by this handler.
    fn manifest(&self) -> Manifest;
}

/// Sync handler contract for runtime plugin crates.
///
/// This trait remains intentionally unsealed so sibling workspace crates can
/// implement the host-facing trait surface; see `SEAL-001` in
/// `docs/implementation-gaps.md`.
pub trait SyncHandler: ManifestProvider {
    /// Handles one synchronous hook invocation.
    fn handle(&self, context: HookContext) -> Result<HookResult, HookError>;
}

/// Async handler contract for runtime plugin crates.
///
/// This trait remains intentionally unsealed so sibling workspace crates can
/// implement the host-facing trait surface; see `SEAL-001` in
/// `docs/implementation-gaps.md`.
pub trait AsyncHandler: ManifestProvider {
    /// Handles one asynchronous hook invocation.
    fn handle_async(&self, context: HookContext) -> Result<AsyncResult, HookError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_hooks_core::dispatch::DispatchMode;
    use sc_hooks_core::events::HookType;
    use sc_hooks_core::manifest::{Manifest, ManifestMatcher};
    use std::collections::BTreeMap;

    struct DummySync;

    impl ManifestProvider for DummySync {
        fn manifest(&self) -> Manifest {
            Manifest {
                contract_version: 1,
                name: "dummy-sync".to_string(),
                mode: DispatchMode::Sync,
                hooks: vec![HookType::PreToolUse],
                matchers: vec![ManifestMatcher::from("Write")],
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
                Some(std::borrow::Cow::Borrowed("Write")),
                serde_json::json!({}),
                None,
            ))
            .expect("sync handler should succeed");
        assert_eq!(output.action, sc_hooks_core::results::HookAction::Proceed);
    }
}
