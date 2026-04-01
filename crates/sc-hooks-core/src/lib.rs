//! Shared runtime types for the `sc-hooks` host, SDK, and plugins.

/// Payload-condition operators and evaluators.
pub mod conditions;
/// Raw payload/context helper types.
pub mod context;
/// Dispatch-mode enums shared across crates.
pub mod dispatch;
/// Shared hook error types.
pub mod errors;
/// Hook and matcher event enums.
pub mod events;
/// CLI exit-code constants and rendered documentation.
pub mod exit_codes;
/// Manifest schema types.
pub mod manifest;
/// Hook result schema types.
pub mod results;
/// Canonical session-state model and invariants.
pub mod session;
/// Session-state storage helpers.
pub mod storage;
/// Tool-name wrappers and helpers.
pub mod tools;
/// Validation-rule parsing and shared validators.
pub mod validation;

pub use session::SessionStartSource;

/// Default observability root used by the current CLI integration.
pub const OBSERVABILITY_ROOT: &str = ".sc-hooks/observability";
/// Default JSONL dispatch log path used by the current CLI integration.
pub const OBSERVABILITY_LOG_PATH: &str = ".sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl";

#[cfg(test)]
mod tests {
    use super::exit_codes;

    #[test]
    fn exit_code_table_has_expected_range() {
        let codes: Vec<i32> = exit_codes::all().iter().map(|entry| entry.code).collect();
        assert_eq!(codes, (0..=10).collect::<Vec<_>>());
    }
}
