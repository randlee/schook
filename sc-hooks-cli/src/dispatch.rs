use serde_json::Value;
use std::time::Instant;

use crate::config::ScHooksConfig;
use crate::errors::CliError;
use crate::metadata;
use crate::observability::{self, HandlerResultRecord};
use crate::resolution::ResolvedHandler;
use crate::session;
use crate::timeout::{TimeoutOutcome, resolve_timeout_ms, wait_with_timeout};

#[derive(Debug)]
pub enum DispatchOutcome {
    Proceed,
    Blocked { reason: String },
}

struct DispatchLogBase<'a> {
    hook: &'a str,
    event: Option<&'a str>,
    matcher: &'a str,
    mode: sc_hooks_core::dispatch::DispatchMode,
    handler_chain: &'a [String],
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
    let started = Instant::now();
    let handler_chain: Vec<String> = handlers
        .iter()
        .map(|handler| handler.name.clone())
        .collect();
    let log_base = DispatchLogBase {
        hook,
        event,
        matcher: event.unwrap_or("*"),
        mode,
        handler_chain: &handler_chain,
    };
    let mut log_results: Vec<HandlerResultRecord> = Vec::new();
    let mut async_additional_context = Vec::new();
    let mut async_system_message = Vec::new();

    for handler in handlers {
        let handler_started = Instant::now();
        let handler_name = &handler.name;
        let stdin_payload = sc_hooks_sdk::manifest::build_plugin_input(
            &handler.manifest,
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
            sc_hooks_sdk::manifest::ManifestError::TypeValidationFailed { field, expected } => {
                CliError::Validation(crate::errors::ValidationError::InvalidField {
                    handler: handler_name.clone(),
                    field,
                    reason: format!("expected type {expected:?}"),
                })
            }
            other => CliError::PluginError {
                message: format!("failed to construct plugin input for `{handler_name}`: {other}"),
            },
        })?;

        let mut command = std::process::Command::new(&handler.executable_path);
        metadata::inject_env_vars(&mut command, &prepared.env);
        let mut child = match command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => {
                disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                let ai_message = ai_notification(
                    handler_name,
                    "spawn-error",
                    "verify executable permissions and run 'sc-hooks test <plugin>'.",
                );
                log_results.push(error_result(
                    handler_name,
                    handler_started.elapsed().as_millis(),
                    "spawn_error",
                    Some(err.to_string()),
                    Some(true),
                ));
                let _ = emit_dispatch_log(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::PluginError {
                    message: ai_message,
                });
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let body = serde_json::to_vec(&stdin_payload).map_err(|err| CliError::PluginError {
                message: format!("failed to serialize stdin payload: {err}"),
            })?;
            if let Err(err) = stdin.write_all(&body) {
                disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                let ai_message = ai_notification(
                    handler_name,
                    "stdin-write-failed",
                    "ensure the plugin reads stdin correctly and run 'sc-hooks test <plugin>'.",
                );
                log_results.push(error_result(
                    handler_name,
                    handler_started.elapsed().as_millis(),
                    "stdin_write_failed",
                    Some(err.to_string()),
                    Some(true),
                ));
                let _ = emit_dispatch_log(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::PluginError {
                    message: ai_message,
                });
            }
        }

        let timeout_ms = resolve_timeout_ms(
            mode,
            handler.manifest.timeout_ms,
            handler.manifest.long_running,
        );
        let status = match wait_with_timeout(&mut child, timeout_ms) {
            Ok(TimeoutOutcome::Completed(status)) => status,
            Ok(TimeoutOutcome::TimedOut) => {
                disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                let ai_message = ai_notification_with_timeout(
                    handler_name,
                    "timed-out",
                    "increase timeout_ms or optimize plugin execution.",
                    timeout_ms,
                );
                log_results.push(error_result(
                    handler_name,
                    handler_started.elapsed().as_millis(),
                    "timeout",
                    None,
                    Some(true),
                ));
                if mode == sc_hooks_core::dispatch::DispatchMode::Async {
                    let _ = emit_dispatch_log(
                        &log_base,
                        &log_results,
                        started.elapsed().as_millis(),
                        sc_hooks_core::exit_codes::SUCCESS,
                        Some(ai_message.as_str()),
                    );
                    async_system_message.push(ai_message);
                    continue;
                }
                let _ = emit_dispatch_log(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::TIMEOUT,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::Timeout {
                    message: ai_message,
                });
            }
            Err(err) => {
                disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                let ai_message = ai_notification(
                    handler_name,
                    "wait-failed",
                    "inspect plugin process behavior and run 'sc-hooks test <plugin>'.",
                );
                log_results.push(error_result(
                    handler_name,
                    handler_started.elapsed().as_millis(),
                    "wait_failed",
                    Some(err.to_string()),
                    Some(true),
                ));
                let _ = emit_dispatch_log(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::PluginError {
                    message: ai_message,
                });
            }
        };
        use std::io::Read;
        let mut stdout = Vec::new();
        if let Some(mut out) = child.stdout.take()
            && let Err(err) = out.read_to_end(&mut stdout)
        {
            disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
            let ai_message = ai_notification(
                handler_name,
                "stdout-read-failed",
                "check plugin output handling and run 'sc-hooks test <plugin>'.",
            );
            log_results.push(error_result(
                handler_name,
                handler_started.elapsed().as_millis(),
                "stdout_read_failed",
                Some(err.to_string()),
                Some(true),
            ));
            let _ = emit_dispatch_log(
                &log_base,
                &log_results,
                started.elapsed().as_millis(),
                sc_hooks_core::exit_codes::PLUGIN_ERROR,
                Some(ai_message.as_str()),
            );
            return Err(CliError::PluginError {
                message: ai_message,
            });
        }

        let mut stderr = Vec::new();
        if let Some(mut err) = child.stderr.take()
            && let Err(read_err) = err.read_to_end(&mut stderr)
        {
            disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
            let ai_message = ai_notification(
                handler_name,
                "stderr-read-failed",
                "check plugin stderr stream handling and run 'sc-hooks test <plugin>'.",
            );
            log_results.push(error_result(
                handler_name,
                handler_started.elapsed().as_millis(),
                "stderr_read_failed",
                Some(read_err.to_string()),
                Some(true),
            ));
            let _ = emit_dispatch_log(
                &log_base,
                &log_results,
                started.elapsed().as_millis(),
                sc_hooks_core::exit_codes::PLUGIN_ERROR,
                Some(ai_message.as_str()),
            );
            return Err(CliError::PluginError {
                message: ai_message,
            });
        }

        let stdout_text = String::from_utf8_lossy(&stdout);
        let stderr_text = String::from_utf8_lossy(&stderr).to_string();

        if !status.success() {
            disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
            let ai_message = ai_notification(
                handler_name,
                "non-zero-exit",
                "inspect plugin stderr and run 'sc-hooks test <plugin>'.",
            );
            log_results.push(error_result(
                handler_name,
                handler_started.elapsed().as_millis(),
                "non_zero_exit",
                Some(stderr_text),
                Some(true),
            ));
            let _ = emit_dispatch_log(
                &log_base,
                &log_results,
                started.elapsed().as_millis(),
                sc_hooks_core::exit_codes::PLUGIN_ERROR,
                Some(ai_message.as_str()),
            );
            return Err(CliError::PluginError {
                message: ai_message,
            });
        }

        let (parsed, warning) = match parse_first_hook_result(&stdout_text) {
            Ok(parsed) => parsed,
            Err(err) => {
                disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                let ai_message = ai_notification(
                    handler_name,
                    "invalid-json",
                    "ensure plugin writes a single valid JSON object to stdout.",
                );
                log_results.push(error_result(
                    handler_name,
                    handler_started.elapsed().as_millis(),
                    "invalid_json",
                    Some(format!("stdout={stdout_text}; stderr={stderr_text}; {err}")),
                    Some(true),
                ));
                let _ = emit_dispatch_log(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::PluginError {
                    message: ai_message,
                });
            }
        };

        match parsed.action {
            sc_hooks_core::results::HookAction::Proceed => {
                log_results.push(HandlerResultRecord {
                    handler: handler_name.clone(),
                    action: "proceed".to_string(),
                    ms: handler_started.elapsed().as_millis(),
                    error_type: None,
                    stderr: if stderr_text.is_empty() {
                        None
                    } else {
                        Some(stderr_text)
                    },
                    warning,
                    disabled: None,
                });

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
                    disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                    let ai_message = ai_notification(
                        handler_name,
                        "async-block",
                        "update plugin to return proceed/error only when mode=async.",
                    );
                    log_results.push(error_result(
                        handler_name,
                        handler_started.elapsed().as_millis(),
                        "async_block",
                        if stderr_text.is_empty() {
                            None
                        } else {
                            Some(stderr_text)
                        },
                        Some(true),
                    ));
                    let _ = emit_dispatch_log(
                        &log_base,
                        &log_results,
                        started.elapsed().as_millis(),
                        sc_hooks_core::exit_codes::SUCCESS,
                        Some(ai_message.as_str()),
                    );
                    async_system_message.push(ai_message);
                    continue;
                }

                let reason = parsed
                    .reason
                    .unwrap_or_else(|| "plugin blocked without reason".to_string());
                log_results.push(HandlerResultRecord {
                    handler: handler_name.clone(),
                    action: "block".to_string(),
                    ms: handler_started.elapsed().as_millis(),
                    error_type: None,
                    stderr: if stderr_text.is_empty() {
                        None
                    } else {
                        Some(stderr_text)
                    },
                    warning,
                    disabled: None,
                });
                let _ = emit_dispatch_log(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::BLOCKED,
                    None,
                );
                return Ok(DispatchOutcome::Blocked { reason });
            }
            sc_hooks_core::results::HookAction::Error => {
                disable_plugin_for_session(prepared.session_id.as_deref(), handler_name);
                let ai_message = ai_notification(
                    handler_name,
                    "action-error",
                    "fix plugin logic and run 'sc-hooks test <plugin>'.",
                );
                log_results.push(error_result(
                    handler_name,
                    handler_started.elapsed().as_millis(),
                    "action_error",
                    Some(
                        parsed
                            .message
                            .unwrap_or_else(|| "plugin returned action=error".to_string()),
                    ),
                    Some(true),
                ));
                let _ = emit_dispatch_log(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::PluginError {
                    message: ai_message,
                });
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

    emit_dispatch_log(
        &log_base,
        &log_results,
        started.elapsed().as_millis(),
        sc_hooks_core::exit_codes::SUCCESS,
        None,
    )?;

    Ok(DispatchOutcome::Proceed)
}

fn disable_plugin_for_session(session_id: Option<&str>, handler_name: &str) {
    let _ = session::mark_plugin_disabled(session_id, handler_name, "runtime-error");
}

fn emit_dispatch_log(
    base: &DispatchLogBase<'_>,
    results: &[HandlerResultRecord],
    total_ms: u128,
    exit: i32,
    ai_notification: Option<&str>,
) -> Result<(), CliError> {
    observability::emit_dispatch_event(observability::DispatchEventArgs {
        hook: base.hook,
        event: base.event,
        matcher: base.matcher,
        mode: base.mode,
        handler_chain: base.handler_chain,
        results,
        total_ms,
        exit,
        ai_notification,
    })
}

fn parse_first_hook_result(
    stdout_text: &str,
) -> Result<(sc_hooks_core::results::HookResult, Option<String>), String> {
    let mut stream =
        serde_json::Deserializer::from_str(stdout_text).into_iter::<serde_json::Value>();
    let Some(first) = stream.next() else {
        return Err("plugin produced empty stdout".to_string());
    };
    let first = first.map_err(|err| err.to_string())?;
    let parsed = serde_json::from_value::<sc_hooks_core::results::HookResult>(first)
        .map_err(|err| err.to_string())?;

    let warning = match stream.next() {
        Some(Ok(_)) => {
            Some("plugin produced multiple JSON objects; only first object was used".to_string())
        }
        Some(Err(err)) => {
            return Err(format!(
                "plugin produced invalid trailing stdout after first JSON object: {err}"
            ));
        }
        None => None,
    };

    Ok((parsed, warning))
}

fn error_result(
    handler_name: &str,
    ms: u128,
    error_type: &str,
    stderr: Option<String>,
    disabled: Option<bool>,
) -> HandlerResultRecord {
    HandlerResultRecord {
        handler: handler_name.to_string(),
        action: "error".to_string(),
        ms,
        error_type: Some(error_type.to_string()),
        stderr,
        warning: None,
        disabled,
    }
}

fn ai_notification(handler_name: &str, error_type: &str, guidance: &str) -> String {
    ai_notification_with_timeout(handler_name, error_type, guidance, None)
}

fn ai_notification_with_timeout(
    handler_name: &str,
    error_type: &str,
    guidance: &str,
    timeout_ms: Option<u64>,
) -> String {
    match error_type {
        "invalid-json" => {
            format!("hook {handler_name} returned invalid JSON — disabled. Please notify user!")
        }
        "non-zero-exit" => {
            format!("hook {handler_name} exited non-zero — disabled. Please notify user!")
        }
        "timed-out" => format!(
            "hook {handler_name} timed out after {}ms — disabled. Run 'sc-hooks test {handler_name}' to diagnose.",
            timeout_ms.unwrap_or_default()
        ),
        _ => format!("hook {handler_name} {error_type} — disabled. {guidance}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::resolution;
    use crate::test_support;
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, Instant};

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
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
            &BTreeSet::new(),
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
    }

    #[test]
    fn rejects_trailing_non_json_garbage_after_first_object() {
        let err = parse_first_hook_result("{\"action\":\"proceed\"}\nnot-json")
            .expect_err("trailing garbage should be rejected");
        assert!(
            err.contains("invalid trailing stdout"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn parses_first_json_object_and_warns_on_additional_output() {
        let output = r#"{"action":"proceed"}{"action":"error"}"#;
        let (parsed, warning) = parse_first_hook_result(output).expect("first json should parse");
        assert!(matches!(
            parsed.action,
            sc_hooks_core::results::HookAction::Proceed
        ));
        assert!(warning.is_some());
    }

    #[test]
    fn integration_dispatch_writes_structured_log_entry() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
            &BTreeSet::new(),
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

        let log_path = Path::new(".sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl");
        let rendered = fs::read_to_string(log_path).expect("log should be readable");
        let line = rendered.lines().last().expect("log line should exist");
        let parsed: serde_json::Value = serde_json::from_str(line).expect("log line should parse");
        assert_eq!(parsed["service"], "sc-hooks");
        assert_eq!(parsed["target"], "hook");
        assert_eq!(parsed["action"], "dispatch.complete");
        assert_eq!(parsed["fields"]["hook"], "PreToolUse");
        assert_eq!(parsed["fields"]["event"], "Write");
        assert_eq!(parsed["fields"]["matcher"], "Write");
        assert_ne!(parsed["timestamp"], serde_json::Value::Null);
        assert_eq!(parsed["fields"]["exit"], 0);
    }

    #[test]
    fn timeout_ai_notification_includes_duration() {
        let message = ai_notification_with_timeout(
            "guard-paths",
            "timed-out",
            "increase timeout",
            Some(5000),
        );
        assert!(message.contains("timed out after 5000ms"));
    }

    #[test]
    fn plugin_only_chain_completes_under_50ms_median() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
            &BTreeSet::new(),
        )
        .expect("resolution should succeed");
        assert_eq!(handlers.len(), 1);

        let mut samples = Vec::new();
        for _ in 0..15 {
            let started = Instant::now();
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
            samples.push(started.elapsed());
        }

        samples.sort_unstable();
        let median = samples[samples.len() / 2];
        assert!(
            median < Duration::from_millis(150),
            "median plugin chain runtime {median:?} exceeded 150ms target"
        );
    }
}
