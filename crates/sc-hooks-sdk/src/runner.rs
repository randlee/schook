//! Executable runner helpers for Rust-authored `sc-hooks` plugins.

use std::borrow::Cow;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use crate::result::{HookResult, error_from_hook_error};
use crate::traits::{AsyncHandler, SyncHandler};
use log::error;
use sc_hooks_core::context::HookContext;
use sc_hooks_core::errors::HookError;
use sc_hooks_core::events::HookType;
use thiserror::Error;

/// Standard executable entrypoint helper for Rust plugins.
pub struct PluginRunner;

/// Errors raised while constructing hook context for SDK-based plugins.
#[derive(Debug, Error)]
pub enum RunnerError {
    /// No hook type was available from the environment or payload.
    #[error("missing hook type in SC_HOOK_TYPE and payload")]
    MissingHookType,

    /// The hook name could not be resolved to a known `HookType`.
    #[error("unknown hook type `{name}`: {reason}")]
    UnknownHookType {
        /// Unrecognized hook name.
        name: String,
        /// Parser rejection reason.
        reason: String,
    },

    /// Stdin could not be read from the plugin process.
    #[error("failed to read stdin: {source}")]
    StdinRead {
        #[source]
        /// Underlying stdin read error.
        source: std::io::Error,
    },

    /// Stdin contained invalid JSON for hook-context construction.
    #[error("invalid JSON on stdin: {source}")]
    StdinParse {
        /// Excerpt of the unreadable payload.
        input_excerpt: String,
        #[source]
        /// Underlying JSON parser error.
        source: serde_json::Error,
    },
}

impl From<RunnerError> for HookError {
    fn from(err: RunnerError) -> Self {
        match err {
            RunnerError::MissingHookType => {
                HookError::invalid_context("missing hook type in SC_HOOK_TYPE and payload")
            }
            RunnerError::UnknownHookType { name, reason } => {
                HookError::invalid_context(format!("unknown hook type `{name}`: {reason}"))
            }
            RunnerError::StdinRead { source } => {
                HookError::internal_with_source("failed to read stdin", source)
            }
            RunnerError::StdinParse {
                input_excerpt,
                source,
            } => HookError::InvalidPayload {
                input_excerpt,
                source: Some(source),
            },
        }
    }
}

impl PluginRunner {
    /// Runs a synchronous handler from a standard `main()` function.
    ///
    /// # Errors
    ///
    /// This helper does not return Rust errors directly. Manifest rendering,
    /// stdin decoding, hook-context construction, and handler failures are
    /// converted into stderr output and the returned process exit code.
    pub fn run_sync<H: SyncHandler>(handler: &H) -> i32 {
        if is_manifest_request() {
            return print_manifest(&handler.manifest());
        }

        let input = match read_hook_context() {
            Ok(value) => value,
            Err(err) => {
                return write_result(&error_from_hook_error(&HookError::from(err)));
            }
        };
        let result = run_sync_with_context(handler, input);
        write_result(&result)
    }

    /// Runs an asynchronous handler from a standard `main()` function.
    ///
    /// # Errors
    ///
    /// This helper does not return Rust errors directly. Manifest rendering,
    /// stdin decoding, hook-context construction, and handler failures are
    /// converted into stderr output and the returned process exit code.
    pub fn run_async<H: AsyncHandler>(handler: &H) -> i32 {
        if is_manifest_request() {
            return print_manifest(&handler.manifest());
        }

        let input = match read_hook_context() {
            Ok(value) => value,
            Err(err) => {
                return write_result(&error_from_hook_error(&HookError::from(err)));
            }
        };
        let result = run_async_with_context(handler, input);
        write_result(&result)
    }
}

fn is_manifest_request() -> bool {
    std::env::args().any(|arg| arg == "--manifest")
}

fn render_manifest(
    manifest: &sc_hooks_core::manifest::Manifest,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(manifest)
}

fn print_manifest(manifest: &sc_hooks_core::manifest::Manifest) -> i32 {
    match render_manifest(manifest) {
        Ok(rendered) => {
            println!("{rendered}");
            sc_hooks_core::exit_codes::SUCCESS
        }
        Err(err) => {
            error!("failed to serialize manifest: {err}");
            sc_hooks_core::exit_codes::PLUGIN_ERROR
        }
    }
}

fn parse_json_input(input: &str) -> Result<serde_json::Value, RunnerError> {
    if input.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }

    serde_json::from_str::<serde_json::Value>(input).map_err(|source| RunnerError::StdinParse {
        input_excerpt: input.chars().take(120).collect(),
        source,
    })
}

fn read_json_stdin() -> Result<serde_json::Value, RunnerError> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|source| RunnerError::StdinRead { source })?;

    parse_json_input(&input)
}

fn build_hook_context(
    raw_input: serde_json::Value,
    hook_name: Option<String>,
    event_name: Option<String>,
    metadata_path: Option<PathBuf>,
) -> Result<HookContext, RunnerError> {
    let hook = resolve_hook_type(&raw_input, hook_name.as_deref())?;
    let event = resolve_event(&raw_input, event_name.as_deref());
    Ok(HookContext::new(hook, event, raw_input, metadata_path))
}

fn read_hook_context() -> Result<HookContext, RunnerError> {
    let raw_input = read_json_stdin()?;
    let hook_name = std::env::var("SC_HOOK_TYPE").ok();
    let event_name = std::env::var("SC_HOOK_EVENT").ok();
    let metadata_path = std::env::var_os("SC_HOOK_METADATA").map(PathBuf::from);
    build_hook_context(raw_input, hook_name, event_name, metadata_path)
}

fn resolve_hook_type(
    raw_input: &serde_json::Value,
    hook_name: Option<&str>,
) -> Result<HookType, RunnerError> {
    let hook_name = hook_name
        .map(Cow::Borrowed)
        .or_else(|| std::env::var("SC_HOOK_TYPE").ok().map(Cow::Owned))
        .or_else(|| {
            raw_input
                .get("hook")
                .and_then(|hook| hook.get("type"))
                .and_then(serde_json::Value::as_str)
                .map(Cow::Borrowed)
        })
        .ok_or(RunnerError::MissingHookType)?;

    HookType::from_str(hook_name.as_ref()).map_err(|err| RunnerError::UnknownHookType {
        name: hook_name.into_owned(),
        reason: err.to_string(),
    })
}

fn resolve_event(
    raw_input: &serde_json::Value,
    event_name: Option<&str>,
) -> Option<Cow<'static, str>> {
    event_name
        .map(str::to_owned)
        .or_else(|| std::env::var("SC_HOOK_EVENT").ok())
        .or_else(|| {
            raw_input
                .get("hook")
                .and_then(|hook| hook.get("event"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .map(Cow::Owned)
}

fn run_sync_with_context<H: SyncHandler>(handler: &H, input: HookContext) -> HookResult {
    match handler.handle(input) {
        Ok(result) => result,
        Err(error) => error_from_hook_error(&error),
    }
}

fn run_async_with_context<H: AsyncHandler>(handler: &H, input: HookContext) -> HookResult {
    match handler.handle_async(input) {
        Ok(result) => result.into_hook_result(),
        Err(error) => error_from_hook_error(&error),
    }
}

fn write_result_to<W: Write>(writer: &mut W, result: &HookResult) -> i32 {
    match serde_json::to_vec(result) {
        Ok(body) => {
            if let Err(err) = writer.write_all(&body) {
                error!("failed to write stdout: {err}");
                return sc_hooks_core::exit_codes::PLUGIN_ERROR;
            }
            if let Err(err) = writer.write_all(b"\n") {
                error!("failed to flush newline: {err}");
                return sc_hooks_core::exit_codes::PLUGIN_ERROR;
            }
            sc_hooks_core::exit_codes::SUCCESS
        }
        Err(err) => {
            error!("failed to serialize result: {err}");
            sc_hooks_core::exit_codes::PLUGIN_ERROR
        }
    }
}

fn write_result(result: &HookResult) -> i32 {
    let mut stdout = std::io::stdout();
    write_result_to(&mut stdout, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ManifestProvider;
    use sc_hooks_core::errors::HookError;
    use sc_hooks_core::manifest::{Manifest, ManifestMatcher};
    use sc_hooks_core::results::HookAction;
    use sc_hooks_core::{dispatch::DispatchMode, events::HookType};
    use std::collections::BTreeMap;
    use std::io;

    struct DummySync;
    struct DummyAsync;

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

    impl ManifestProvider for DummyAsync {
        fn manifest(&self) -> Manifest {
            DummySync.manifest()
        }
    }

    impl AsyncHandler for DummyAsync {
        fn handle_async(
            &self,
            _context: HookContext,
        ) -> Result<crate::result::AsyncResult, HookError> {
            Ok(crate::result::AsyncResult::with_system_message("done"))
        }
    }

    #[test]
    fn manifest_flag_detection_defaults_false_in_tests() {
        // This validates helper behavior without mutating global process args.
        assert!(!is_manifest_request());
    }

    #[test]
    fn read_json_stdin_empty_defaults_to_object() {
        // Unit-level coverage for default empty-object behavior.
        let parsed = parse_json_input("").expect("empty stdin should default");
        assert_eq!(parsed, serde_json::json!({}));
    }

    #[test]
    fn parse_json_input_reports_excerpt_on_invalid_payload() {
        let err = parse_json_input("{not json").expect_err("invalid json should fail");
        match err {
            RunnerError::StdinParse { input_excerpt, .. } => {
                assert!(input_excerpt.contains("{not json"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parse_json_input_parses_non_empty_valid_json() {
        let parsed =
            parse_json_input("{\"payload\":{\"ok\":true}}").expect("valid json should parse");
        assert_eq!(parsed["payload"]["ok"], serde_json::json!(true));
    }

    #[test]
    fn hook_type_resolution_prefers_explicit_argument() {
        let hook = resolve_hook_type(
            &serde_json::json!({"hook": {"type": "SessionEnd"}}),
            Some("SessionStart"),
        )
        .expect("hook type should parse");
        assert_eq!(hook, HookType::SessionStart);
    }

    #[test]
    fn hook_type_resolution_uses_payload_when_argument_missing() {
        let hook = resolve_hook_type(&serde_json::json!({"hook": {"type": "SessionEnd"}}), None)
            .expect("payload hook should parse");
        assert_eq!(hook, HookType::SessionEnd);
    }

    #[test]
    fn hook_type_resolution_returns_typed_error_for_unknown_hook() {
        let err = resolve_hook_type(&serde_json::json!({"hook": {"type": "NotAHook"}}), None)
            .expect_err("unknown hook should fail");
        assert!(matches!(
            err,
            RunnerError::UnknownHookType { name, .. } if name == "NotAHook"
        ));
    }

    #[test]
    fn hook_type_resolution_returns_missing_hook_type_error() {
        let err = resolve_hook_type(&serde_json::json!({"payload": {"session_id": "abc"}}), None)
            .expect_err("missing hook should fail");
        assert!(matches!(err, RunnerError::MissingHookType));
    }

    #[test]
    fn resolve_event_prefers_explicit_argument_then_payload() {
        let payload = serde_json::json!({"hook": {"event": "Write"}});
        assert_eq!(
            resolve_event(&payload, Some("Read")).as_deref(),
            Some("Read")
        );
        assert_eq!(resolve_event(&payload, None).as_deref(), Some("Write"));
    }

    #[test]
    fn resolve_event_returns_none_when_absent() {
        let payload = serde_json::json!({"payload": {"ok": true}});
        assert!(resolve_event(&payload, None).is_none());
    }

    #[test]
    fn build_hook_context_uses_explicit_inputs() {
        let metadata = Some(std::env::temp_dir().join("test-metadata.json"));
        let context = build_hook_context(
            serde_json::json!({"payload": true}),
            Some("SessionStart".to_string()),
            Some("Write".to_string()),
            metadata.clone(),
        )
        .expect("context should build");

        assert_eq!(context.hook, HookType::SessionStart);
        assert_eq!(context.event.as_deref(), Some("Write"));
        assert_eq!(context.metadata_path.as_deref(), metadata.as_deref());
    }

    #[test]
    fn runner_error_conversion_preserves_variant_behavior() {
        let errors = [
            (
                RunnerError::MissingHookType,
                "invalid context: missing hook type in SC_HOOK_TYPE and payload",
            ),
            (
                RunnerError::UnknownHookType {
                    name: "Nope".to_string(),
                    reason: "unknown hook".to_string(),
                },
                "invalid context: unknown hook type `Nope`: unknown hook",
            ),
        ];

        for (runner_err, expected) in errors {
            assert_eq!(HookError::from(runner_err).to_string(), expected);
        }
    }

    #[test]
    fn runner_error_stdin_variants_convert_to_hook_errors() {
        let io_hook_error = HookError::from(RunnerError::StdinRead {
            source: io::Error::other("boom"),
        });
        assert!(matches!(io_hook_error, HookError::Internal { .. }));

        let parse_error =
            serde_json::from_str::<serde_json::Value>("{oops").expect_err("fixture should fail");
        let parse_hook_error = HookError::from(RunnerError::StdinParse {
            input_excerpt: "{oops".to_string(),
            source: parse_error,
        });
        assert!(matches!(parse_hook_error, HookError::InvalidPayload { .. }));
    }

    #[test]
    fn render_manifest_serializes_manifest() {
        let rendered = render_manifest(&DummySync.manifest()).expect("manifest should render");
        assert!(rendered.contains("\"name\": \"dummy-sync\""));
    }

    #[test]
    fn run_sync_with_context_converts_handler_error() {
        struct FailingSync;

        impl ManifestProvider for FailingSync {
            fn manifest(&self) -> Manifest {
                DummySync.manifest()
            }
        }

        impl SyncHandler for FailingSync {
            fn handle(&self, _context: HookContext) -> Result<HookResult, HookError> {
                Err(HookError::invalid_context("bad context"))
            }
        }

        let result = run_sync_with_context(
            &FailingSync,
            HookContext::new(HookType::PreToolUse, None, serde_json::json!({}), None),
        );
        assert_eq!(result.action, HookAction::Error);
        assert_eq!(
            result.message.as_deref(),
            Some("invalid context: bad context")
        );
    }

    #[test]
    fn run_sync_with_context_preserves_successful_result() {
        struct SuccessfulSync;

        impl ManifestProvider for SuccessfulSync {
            fn manifest(&self) -> Manifest {
                DummySync.manifest()
            }
        }

        impl SyncHandler for SuccessfulSync {
            fn handle(&self, _context: HookContext) -> Result<HookResult, HookError> {
                Ok(crate::result::block("retryable"))
            }
        }

        let result = run_sync_with_context(
            &SuccessfulSync,
            HookContext::new(HookType::PreToolUse, None, serde_json::json!({}), None),
        );
        assert_eq!(result.action, HookAction::Block);
        assert_eq!(result.reason.as_deref(), Some("retryable"));
    }

    #[test]
    fn run_async_with_context_converts_success_and_error() {
        struct FailingAsync;

        impl ManifestProvider for FailingAsync {
            fn manifest(&self) -> Manifest {
                DummySync.manifest()
            }
        }

        impl AsyncHandler for FailingAsync {
            fn handle_async(
                &self,
                _context: HookContext,
            ) -> Result<crate::result::AsyncResult, HookError> {
                Err(HookError::internal("async fail"))
            }
        }

        let success = run_async_with_context(
            &DummyAsync,
            HookContext::new(HookType::PreToolUse, None, serde_json::json!({}), None),
        );
        assert_eq!(success.action, HookAction::Proceed);
        assert_eq!(success.system_message.as_deref(), Some("done"));

        let failure = run_async_with_context(
            &FailingAsync,
            HookContext::new(HookType::PreToolUse, None, serde_json::json!({}), None),
        );
        assert_eq!(failure.action, HookAction::Error);
        assert_eq!(
            failure.message.as_deref(),
            Some("internal hook error: async fail")
        );
    }

    #[test]
    fn write_result_to_writes_json_and_newline() {
        let mut output = Vec::new();
        let exit = write_result_to(&mut output, &crate::result::proceed());
        assert_eq!(exit, sc_hooks_core::exit_codes::SUCCESS);
        assert!(String::from_utf8(output).expect("utf8").ends_with('\n'));
    }

    #[test]
    fn write_result_to_returns_plugin_error_on_writer_failure() {
        struct FailWriter {
            fail_on_call: usize,
            writes: usize,
        }

        impl Write for FailWriter {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.writes += 1;
                if self.writes == self.fail_on_call {
                    Err(io::Error::other("write failed"))
                } else {
                    Ok(buf.len())
                }
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut body_fail = FailWriter {
            fail_on_call: 1,
            writes: 0,
        };
        let mut newline_fail = FailWriter {
            fail_on_call: 2,
            writes: 0,
        };

        assert_eq!(
            write_result_to(&mut body_fail, &crate::result::proceed()),
            sc_hooks_core::exit_codes::PLUGIN_ERROR
        );
        assert_eq!(
            write_result_to(&mut newline_fail, &crate::result::proceed()),
            sc_hooks_core::exit_codes::PLUGIN_ERROR
        );
    }

    #[test]
    fn hook_error_strings_render_for_result_conversion() {
        let result = error_from_hook_error(&HookError::invalid_context("missing"));
        assert_eq!(result.action, HookAction::Error);
        assert_eq!(result.message, Some("invalid context: missing".to_string()));
        assert_eq!(
            result.additional_context,
            Some("hook_error_kind=invalid_context".to_string())
        );
    }
}
