use sc_hooks_core::manifest::Manifest;

use crate::result::{AsyncResult, HookResult};

pub trait ManifestProvider {
    fn manifest(&self) -> Manifest;
}

pub trait SyncHandler: ManifestProvider {
    fn handle(&self, input: serde_json::Value) -> Result<HookResult, String>;
}

pub trait AsyncHandler: ManifestProvider {
    fn handle_async(&self, input: serde_json::Value) -> Result<AsyncResult, String>;
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
        fn handle(&self, _input: serde_json::Value) -> Result<HookResult, String> {
            Ok(crate::result::proceed())
        }
    }

    #[test]
    fn sync_handler_trait_is_usable() {
        let handler = DummySync;
        let output = handler
            .handle(serde_json::json!({}))
            .expect("sync handler should succeed");
        assert_eq!(output.action, sc_hooks_core::results::HookAction::Proceed);
    }
}
