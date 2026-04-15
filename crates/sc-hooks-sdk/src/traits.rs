//! Public trait contracts for Rust-authored `sc-hooks` plugins.

use sc_hooks_core::context::HookContext;
use sc_hooks_core::errors::HookError;
use sc_hooks_core::manifest::Manifest;

use crate::result::{AsyncResult, HookResult};

#[doc(hidden)]
pub(crate) mod private {
    pub trait Sealed {}
}

#[doc(hidden)]
pub use private::Sealed as RuntimePluginSealed;

/// Public manifest provider surface used by runtime plugin crates.
///
/// Source-owned runtime crates implement the SDK-owned sealed marker before
/// implementing this trait; see `SEAL-001` in `docs/implementation-gaps.md`.
pub trait ManifestProvider: private::Sealed {
    /// Returns the manifest advertised by this handler.
    fn manifest(&self) -> Manifest;
}

/// Sync handler contract for runtime plugin crates.
///
/// Source-owned runtime crates implement the SDK-owned sealed marker before
/// implementing this trait; see `SEAL-001` in `docs/implementation-gaps.md`.
pub trait SyncHandler: ManifestProvider + private::Sealed {
    /// Handles one synchronous hook invocation.
    fn handle(&self, context: HookContext) -> Result<HookResult, HookError>;
}

/// Async handler contract for runtime plugin crates.
///
/// Source-owned runtime crates implement the SDK-owned sealed marker before
/// implementing this trait; see `SEAL-001` in `docs/implementation-gaps.md`.
pub trait AsyncHandler: ManifestProvider + private::Sealed {
    /// Handles one asynchronous hook invocation.
    fn handle_async(&self, context: HookContext) -> Result<AsyncResult, HookError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_hooks_core::dispatch::DispatchMode;
    use sc_hooks_core::events::HookType;
    use sc_hooks_core::manifest::{Manifest, ManifestMatcher};
    use sc_hooks_core::results::HookAction;
    use std::collections::BTreeMap;

    struct DummySync;

    impl private::Sealed for DummySync {}

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

    struct DummyAsync;

    impl private::Sealed for DummyAsync {}

    impl ManifestProvider for DummyAsync {
        fn manifest(&self) -> Manifest {
            DummySync.manifest()
        }
    }

    impl AsyncHandler for DummyAsync {
        fn handle_async(&self, _context: HookContext) -> Result<AsyncResult, HookError> {
            Ok(AsyncResult::with_context("async-context"))
        }
    }

    #[test]
    fn manifest_provider_returns_expected_manifest_shape() {
        let manifest = DummySync.manifest();
        assert_eq!(manifest.name, "dummy-sync");
        assert_eq!(manifest.mode, DispatchMode::Sync);
        assert_eq!(manifest.matchers[0].as_str(), "Write");
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
        assert_eq!(output.action, HookAction::Proceed);
    }

    #[test]
    fn async_handler_trait_is_usable() {
        let handler = DummyAsync;
        let output = handler
            .handle_async(HookContext::new(
                HookType::PreToolUse,
                Some(std::borrow::Cow::Borrowed("Write")),
                serde_json::json!({}),
                None,
            ))
            .expect("async handler should succeed");
        assert_eq!(output, AsyncResult::with_context("async-context"));
    }
}
