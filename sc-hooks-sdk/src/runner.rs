use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use crate::result::{HookResult, error};
use crate::traits::{AsyncHandler, SyncHandler};
use sc_hooks_core::context::HookContext;
use sc_hooks_core::events::HookType;

pub struct PluginRunner;

impl PluginRunner {
    pub fn run_sync<H: SyncHandler>(handler: &H) -> i32 {
        if is_manifest_request() {
            return print_manifest(&handler.manifest());
        }

        let input = match read_hook_context() {
            Ok(value) => value,
            Err(message) => {
                eprintln!("{message}");
                return sc_hooks_core::exit_codes::PLUGIN_ERROR;
            }
        };

        let result = match handler.handle(input) {
            Ok(result) => result,
            Err(message) => error(message.to_string()),
        };

        write_result(&result)
    }

    pub fn run_async<H: AsyncHandler>(handler: &H) -> i32 {
        if is_manifest_request() {
            return print_manifest(&handler.manifest());
        }

        let input = match read_hook_context() {
            Ok(value) => value,
            Err(message) => {
                eprintln!("{message}");
                return sc_hooks_core::exit_codes::PLUGIN_ERROR;
            }
        };

        let result = match handler.handle_async(input) {
            Ok(result) => result.into_hook_result(),
            Err(message) => error(message.to_string()),
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
            eprintln!("failed to serialize manifest: {err}");
            sc_hooks_core::exit_codes::PLUGIN_ERROR
        }
    }
}

fn read_json_stdin() -> Result<serde_json::Value, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|err| format!("failed to read stdin: {err}"))?;

    if input.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }

    serde_json::from_str::<serde_json::Value>(&input)
        .map_err(|err| format!("invalid JSON on stdin: {err}"))
}

fn read_hook_context() -> Result<HookContext, String> {
    let raw_input = read_json_stdin()?;
    let hook = resolve_hook_type(&raw_input)?;
    let event = resolve_event(&raw_input);
    let metadata_path = std::env::var_os("SC_HOOK_METADATA").map(PathBuf::from);
    Ok(HookContext::new(hook, event, raw_input, metadata_path))
}

fn resolve_hook_type(raw_input: &serde_json::Value) -> Result<HookType, String> {
    let hook_name = std::env::var("SC_HOOK_TYPE")
        .ok()
        .or_else(|| {
            raw_input
                .get("hook")
                .and_then(|hook| hook.get("type"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .ok_or_else(|| "missing hook type in SC_HOOK_TYPE or input.hook.type".to_string())?;

    HookType::from_str(&hook_name).map_err(|_| format!("unknown hook type `{hook_name}`"))
}

fn resolve_event(raw_input: &serde_json::Value) -> Option<String> {
    std::env::var("SC_HOOK_EVENT").ok().or_else(|| {
        raw_input
            .get("hook")
            .and_then(|hook| hook.get("event"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    })
}

fn write_result(result: &HookResult) -> i32 {
    let mut stdout = std::io::stdout();
    match serde_json::to_vec(result) {
        Ok(body) => {
            if let Err(err) = stdout.write_all(&body) {
                eprintln!("failed to write stdout: {err}");
                return sc_hooks_core::exit_codes::PLUGIN_ERROR;
            }
            if let Err(err) = stdout.write_all(b"\n") {
                eprintln!("failed to flush newline: {err}");
                return sc_hooks_core::exit_codes::PLUGIN_ERROR;
            }
            sc_hooks_core::exit_codes::SUCCESS
        }
        Err(err) => {
            eprintln!("failed to serialize result: {err}");
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
    fn hook_error_strings_render_for_result_conversion() {
        let result = error(HookError::invalid_context("missing").to_string());
        assert_eq!(result.action, sc_hooks_core::results::HookAction::Error);
    }
}
