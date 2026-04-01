//! Convenience helpers for authoring `sc-hooks` plugins in Rust.

/// Payload-condition helper APIs.
pub mod conditions;
/// Manifest loading, validation, and construction helpers.
pub mod manifest;
/// Hook-result construction helpers.
pub mod result;
/// Executable runner helpers for Rust plugins.
pub mod runner;
/// Public plugin trait surfaces.
pub mod traits;
