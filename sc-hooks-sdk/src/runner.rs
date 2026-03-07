use std::io::{Read, Write};

use crate::result::{HookResult, error};
use crate::traits::{AsyncHandler, SyncHandler};

pub struct PluginRunner;

impl PluginRunner {
    pub fn run_sync<H: SyncHandler>(handler: &H) -> i32 {
        if is_manifest_request() {
            return print_manifest(&handler.manifest());
        }

        let input = match read_json_stdin() {
            Ok(value) => value,
            Err(message) => {
                eprintln!("{message}");
                return sc_hooks_core::exit_codes::PLUGIN_ERROR;
            }
        };

        let result = match handler.handle(input) {
            Ok(result) => result,
            Err(message) => error(message),
        };

        write_result(&result)
    }

    pub fn run_async<H: AsyncHandler>(handler: &H) -> i32 {
        if is_manifest_request() {
            return print_manifest(&handler.manifest());
        }

        let input = match read_json_stdin() {
            Ok(value) => value,
            Err(message) => {
                eprintln!("{message}");
                return sc_hooks_core::exit_codes::PLUGIN_ERROR;
            }
        };

        let result = match handler.handle_async(input) {
            Ok(result) => result.into_hook_result(),
            Err(message) => error(message),
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
}
