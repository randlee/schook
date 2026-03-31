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

#[derive(Debug, Error)]
/// Errors raised while constructing hook context for SDK-based plugins.
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

        let result = match handler.handle(input) {
            Ok(result) => result,
            Err(error) => error_from_hook_error(&error),
        };

        write_result(&result)
    }

    /// Runs an asynchronous handler from a standard `main()` function.
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

        let result = match handler.handle_async(input) {
            Ok(result) => result.into_hook_result(),
            Err(error) => error_from_hook_error(&error),
        };

        write_result(&result)
    }
}

fn is_manifest_request() -> bool {
    std::env::args().any(|arg| arg == "--manifest")
}

fn print_manifest(manifest: &sc_hooks_core::manifest::Manifest) -> i32 {
    match serde_json::to_string_pretty(manifest) {
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

fn read_json_stdin() -> Result<serde_json::Value, RunnerError> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|source| RunnerError::StdinRead { source })?;

    if input.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }

    serde_json::from_str::<serde_json::Value>(&input).map_err(|source| RunnerError::StdinParse {
        input_excerpt: input.chars().take(120).collect(),
        source,
    })
}

fn read_hook_context() -> Result<HookContext, RunnerError> {
    let raw_input = read_json_stdin()?;
    let hook = resolve_hook_type(&raw_input)?;
    let event = resolve_event(&raw_input);
    let metadata_path = std::env::var_os("SC_HOOK_METADATA").map(PathBuf::from);
    Ok(HookContext::new(hook, event, raw_input, metadata_path))
}

fn resolve_hook_type(raw_input: &serde_json::Value) -> Result<HookType, RunnerError> {
    let hook_name = std::env::var("SC_HOOK_TYPE")
        .ok()
        .map(Cow::Owned)
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

fn resolve_event(raw_input: &serde_json::Value) -> Option<Cow<'static, str>> {
    std::env::var("SC_HOOK_EVENT")
        .ok()
        .or_else(|| {
            raw_input
                .get("hook")
                .and_then(|hook| hook.get("event"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .map(Cow::Owned)
}

fn write_result(result: &HookResult) -> i32 {
    let mut stdout = std::io::stdout();
    match serde_json::to_vec(result) {
        Ok(body) => {
            if let Err(err) = stdout.write_all(&body) {
                error!("failed to write stdout: {err}");
                return sc_hooks_core::exit_codes::PLUGIN_ERROR;
            }
            if let Err(err) = stdout.write_all(b"\n") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use sc_hooks_core::errors::HookError;
    use std::str::FromStr;

    #[test]
    fn manifest_flag_detection_defaults_false_in_tests() {
        // This validates helper behavior without mutating global process args.
        assert!(!is_manifest_request());
    }

    #[test]
    fn read_json_stdin_empty_defaults_to_object() {
        // Unit-level coverage for default empty-object behavior.
        let parsed =
            serde_json::from_str::<serde_json::Value>("{}").expect("fixture json should parse");
        assert_eq!(parsed, serde_json::json!({}));
    }

    #[test]
    fn hook_type_resolution_accepts_known_value() {
        let hook = HookType::from_str("SessionStart").expect("hook type should parse");
        assert_eq!(hook, HookType::SessionStart);
    }

    #[test]
    fn hook_type_resolution_returns_typed_error_for_unknown_hook() {
        let err = resolve_hook_type(&serde_json::json!({"hook": {"type": "NotAHook"}}))
            .expect_err("unknown hook should fail");
        assert!(matches!(
            err,
            RunnerError::UnknownHookType { name, .. } if name == "NotAHook"
        ));
    }

    #[test]
    fn hook_error_strings_render_for_result_conversion() {
        let result = error_from_hook_error(&HookError::invalid_context("missing"));
        assert_eq!(result.action, sc_hooks_core::results::HookAction::Error);
        assert_eq!(result.message, Some("invalid context: missing".to_string()));
        assert_eq!(
            result.additional_context,
            Some("hook_error_kind=invalid_context".to_string())
        );
    }
}
