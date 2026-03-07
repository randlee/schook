use serde_json::Value;

use crate::builtins;
use crate::config::ScHooksConfig;
use crate::errors::CliError;
use crate::metadata;
use crate::resolution::{BuiltinHandler, HandlerTarget, ResolvedHandler};
use crate::session;
use crate::timeout::{TimeoutOutcome, resolve_timeout_ms, wait_with_timeout};

#[derive(Debug)]
pub enum DispatchOutcome {
    Proceed,
    Blocked { reason: String },
}

pub fn execute_chain(
    handlers: &[ResolvedHandler],
    config: &ScHooksConfig,
    hook: &str,
    event: Option<&str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
    payload: Option<&Value>,
) -> Result<DispatchOutcome, CliError> {
    let prepared = metadata::prepare_for_dispatch(config, hook, event, payload)?;
    let mut async_additional_context = Vec::new();
    let mut async_system_message = Vec::new();

    for handler in handlers {
        let handler_name = &handler.name;
        match &handler.target {
            HandlerTarget::Builtin(builtin) => {
                run_builtin(builtin, config, hook, event, mode)?;
            }
            HandlerTarget::Plugin(plugin) => {
                let stdin_payload = sc_hooks_sdk::manifest::build_plugin_input(
                    &plugin.manifest,
                    &prepared.metadata,
                    hook,
                    event,
                    payload,
                )
                .map_err(|err| match err {
                    sc_hooks_sdk::manifest::ManifestError::MissingRequiredField { field } => {
                        CliError::Validation(crate::errors::ValidationError::MissingField {
                            handler: handler_name.clone(),
                            field,
                        })
                    }
                    sc_hooks_sdk::manifest::ManifestError::ValidationRuleFailed { field, rule } => {
                        CliError::Validation(crate::errors::ValidationError::InvalidField {
                            handler: handler_name.clone(),
                            field,
                            reason: rule,
                        })
                    }
                    sc_hooks_sdk::manifest::ManifestError::TypeValidationFailed {
                        field,
                        expected,
                    } => CliError::Validation(crate::errors::ValidationError::InvalidField {
                        handler: handler_name.clone(),
                        field,
                        reason: format!("expected type {expected:?}"),
                    }),
                    other => CliError::PluginError {
                        message: format!(
                            "failed to construct plugin input for `{handler_name}`: {other}"
                        ),
                    },
                })?;

                let mut command = std::process::Command::new(&plugin.executable_path);
                metadata::inject_env_vars(&mut command, &prepared.env);
                let mut child = command
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|err| {
                        disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                        CliError::PluginError {
                            message: format!("failed to spawn `{handler_name}`: {err}"),
                        }
                    })?;

                if let Some(mut stdin) = child.stdin.take() {
                    let body = serde_json::to_vec(&stdin_payload).map_err(|err| {
                        CliError::PluginError {
                            message: format!("failed to serialize stdin payload: {err}"),
                        }
                    })?;
                    use std::io::Write;
                    stdin.write_all(&body).map_err(|err| {
                        disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                        CliError::PluginError {
                            message: format!("failed to write stdin payload: {err}"),
                        }
                    })?;
                }

                let timeout_ms = resolve_timeout_ms(
                    mode,
                    plugin.manifest.timeout_ms,
                    plugin.manifest.long_running,
                );
                let status = match wait_with_timeout(&mut child, timeout_ms).map_err(|err| {
                    disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                    CliError::PluginError {
                        message: format!("failed while waiting for `{handler_name}`: {err}"),
                    }
                })? {
                    TimeoutOutcome::TimedOut => {
                        disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                        return Err(CliError::Timeout {
                            message: format!("plugin `{handler_name}` exceeded timeout"),
                        });
                    }
                    TimeoutOutcome::Completed(status) => status,
                };

                use std::io::Read;
                let mut stdout = Vec::new();
                if let Some(mut out) = child.stdout.take() {
                    out.read_to_end(&mut stdout).map_err(|err| {
                        disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                        CliError::PluginError {
                            message: format!("failed to read plugin stdout: {err}"),
                        }
                    })?;
                }
                let mut stderr = Vec::new();
                if let Some(mut err) = child.stderr.take() {
                    err.read_to_end(&mut stderr).map_err(|read_err| {
                        disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                        CliError::PluginError {
                            message: format!("failed to read plugin stderr: {read_err}"),
                        }
                    })?;
                }

                let stdout_text = String::from_utf8_lossy(&stdout);
                let stderr_text = String::from_utf8_lossy(&stderr);

                if !status.success() {
                    disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                    return Err(CliError::PluginError {
                        message: format!(
                            "plugin `{}` exited non-zero status {:?}; stderr={}",
                            handler_name,
                            status.code(),
                            stderr_text
                        ),
                    });
                }

                let parsed = serde_json::from_str::<sc_hooks_core::results::HookResult>(
                    &stdout_text,
                )
                .map_err(|err| {
                    disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                    CliError::PluginError {
                        message: format!(
                            "plugin `{handler_name}` returned invalid JSON: {err}; stderr={stderr_text}"
                        ),
                    }
                })?;

                match parsed.action {
                    sc_hooks_core::results::HookAction::Proceed => {
                        if mode == sc_hooks_core::dispatch::DispatchMode::Async {
                            if let Some(context) = parsed.additional_context {
                                async_additional_context.push(context);
                            }
                            if let Some(message) = parsed.system_message {
                                async_system_message.push(message);
                            }
                        }
                    }
                    sc_hooks_core::results::HookAction::Block => {
                        if mode == sc_hooks_core::dispatch::DispatchMode::Async {
                            disable_plugin_for_session(
                                prepared.session_id.as_deref(),
                                handler_name,
                            );
                            return Err(CliError::PluginError {
                                message: format!(
                                    "{handler_name}: async plugins cannot return block action"
                                ),
                            });
                        }
                        let reason = parsed
                            .reason
                            .unwrap_or_else(|| "plugin blocked without reason".to_string());
                        return Ok(DispatchOutcome::Blocked { reason });
                    }
                    sc_hooks_core::results::HookAction::Error => {
                        disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                        let message = parsed
                            .message
                            .unwrap_or_else(|| "plugin reported error".to_string());
                        return Err(CliError::PluginError {
                            message: format!("{handler_name}: {message}"),
                        });
                    }
                }
            }
        }
    }

    if mode == sc_hooks_core::dispatch::DispatchMode::Async
        && (!async_additional_context.is_empty() || !async_system_message.is_empty())
    {
        let output = serde_json::json!({
            "additionalContext": if async_additional_context.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(async_additional_context.join("\n---\n"))
            },
            "systemMessage": if async_system_message.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(async_system_message.join("\n"))
            }
        });
        println!("{output}");
    }

    Ok(DispatchOutcome::Proceed)
}

fn disable_plugin_for_session(session_id: Option<&str>, handler_name: &str) {
    let _ = session::mark_plugin_disabled(session_id, handler_name);
}

fn run_builtin(
    builtin: &BuiltinHandler,
    config: &ScHooksConfig,
    hook: &str,
    event: Option<&str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
) -> Result<(), CliError> {
    match builtin {
        BuiltinHandler::Log => {
            builtins::log::write_entry(&config.logging.hook_log, hook, event, mode)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::resolution;
    use crate::test_support;
    use std::fs;
    use std::path::Path;

    fn make_plugin(path: &Path, manifest: &str, runtime_output: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("plugin parent directory should exist");
        }

        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--manifest\" ]; then\n  cat <<'JSON'\n{manifest}\nJSON\n  exit 0\nfi\ncat >/dev/null\ncat <<'JSON'\n{runtime_output}\nJSON\n"
        );
        fs::write(path, script).expect("plugin script should be writable");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)
                .expect("plugin metadata should be readable")
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).expect("plugin should be executable");
        }
    }

    #[test]
    fn dispatch_executes_plugin_and_returns_proceed() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch");

        make_plugin(
            Path::new(".sc-hooks/plugins/guard-paths"),
            r#"{"contract_version":1,"name":"guard-paths","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
            r#"{"action":"proceed"}"#,
        );

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let handlers = resolution::resolve_chain(
            &cfg,
            "PreToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Sync,
            None,
            None,
            &std::collections::BTreeSet::new(),
        )
        .expect("resolution should succeed");

        let outcome = execute_chain(
            &handlers,
            &cfg,
            "PreToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Sync,
            None,
        )
        .expect("dispatch should succeed");

        assert!(matches!(outcome, DispatchOutcome::Proceed));
        std::env::set_current_dir(original).expect("cwd should restore");
    }
}
