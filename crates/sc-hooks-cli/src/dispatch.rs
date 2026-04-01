use serde_json::Value;
use std::time::Instant;
use thiserror::Error;

use crate::config::ScHooksConfig;
use crate::errors::CliError;
use crate::metadata;
use crate::observability::{self, HandlerResultRecord};
use crate::resolution::ResolvedHandler;
use crate::session;
use crate::timeout::{TimeoutOutcome, resolve_timeout_ms, wait_with_timeout};
use log::error;
use sc_hooks_core::errors::RootDivergenceNotice;
use sc_hooks_core::session::AiRootDir;
use std::borrow::Cow;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

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
    project_root: &'a AiRootDir,
}

#[derive(Debug, Error)]
enum HookResultParseError {
    #[error("plugin produced empty stdout")]
    EmptyStdout,
    #[error("plugin produced malformed JSON on stdout: {0}")]
    MalformedFirstJson(serde_json::Error),
    #[error("plugin produced JSON that did not match HookResult: {0}")]
    InvalidHookResult(serde_json::Error),
    #[error("plugin produced invalid trailing stdout after first JSON object: {0}")]
    InvalidTrailingJson(serde_json::Error),
}

#[derive(Debug, Error)]
#[error(
    "failed to read plugin stderr after capturing {captured_bytes} bytes (partial stderr: {captured_excerpt:?})"
)]
struct StderrCaptureContextError {
    captured_bytes: usize,
    captured_excerpt: String,
    #[source]
    source: std::io::Error,
}

impl StderrCaptureContextError {
    fn new(captured: &[u8], source: std::io::Error) -> Self {
        let captured_excerpt = String::from_utf8_lossy(captured)
            .chars()
            .take(200)
            .collect();
        Self {
            captured_bytes: captured.len(),
            captured_excerpt,
            source,
        }
    }
}

#[derive(Debug, Error)]
#[error("{context}")]
struct PluginExecutionContextError {
    context: String,
    #[source]
    source: Option<BoxedError>,
}

impl PluginExecutionContextError {
    fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            source: None,
        }
    }

    fn with_source(
        context: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            context: context.into(),
            source: Some(Box::new(source)),
        }
    }
}

#[derive(Debug, Error)]
enum PluginTerminationError {
    #[error("plugin exited with status {status}{stderr_suffix}")]
    NonZeroExit { status: i32, stderr_suffix: String },
    #[cfg(unix)]
    #[error("plugin terminated by signal {signal}{stderr_suffix}")]
    Signaled { signal: i32, stderr_suffix: String },
    #[error("plugin terminated without an exit status{stderr_suffix}")]
    MissingStatus { stderr_suffix: String },
}

fn stderr_suffix(stderr: Option<&str>) -> String {
    match stderr {
        Some(stderr) if !stderr.is_empty() => format!("; stderr: {stderr}"),
        _ => String::new(),
    }
}

fn plugin_termination_error(
    status: std::process::ExitStatus,
    stderr: Option<&str>,
) -> PluginTerminationError {
    if let Some(code) = status.code() {
        return PluginTerminationError::NonZeroExit {
            status: code,
            stderr_suffix: stderr_suffix(stderr),
        };
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        if let Some(signal) = status.signal() {
            return PluginTerminationError::Signaled {
                signal,
                stderr_suffix: stderr_suffix(stderr),
            };
        }
    }

    PluginTerminationError::MissingStatus {
        stderr_suffix: stderr_suffix(stderr),
    }
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
        project_root: &prepared.project_root,
    };
    let mut log_results: Vec<HandlerResultRecord> = Vec::new();
    let mut async_additional_context = Vec::new();
    let mut async_system_message = Vec::new();

    for handler in handlers {
        let handler_started = Instant::now();
        let handler_name = &handler.name;
        if mode == sc_hooks_core::dispatch::DispatchMode::Async && handler.manifest.long_running {
            return Err(CliError::Resolution(
                crate::errors::ResolutionError::HandlerRejected {
                    plugin: handler_name.clone(),
                    reason: "manifest long_running=true is only supported for sync handlers"
                        .to_string(),
                    source: None,
                },
            ));
        }
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
            sc_hooks_sdk::manifest::ManifestError::ValidationRuleFailed {
                field,
                rule,
                actual,
            } => CliError::Validation(crate::errors::ValidationError::InvalidField {
                handler: handler_name.clone(),
                field,
                reason: format!("{rule} (actual {actual})"),
            }),
            sc_hooks_sdk::manifest::ManifestError::TypeValidationFailed {
                field,
                expected,
                actual,
            } => CliError::Validation(crate::errors::ValidationError::InvalidField {
                handler: handler_name.clone(),
                field,
                reason: format!("expected type {expected:?} (actual {actual})"),
            }),
            other => CliError::plugin_error_with_source(
                format!("failed to construct plugin input for `{handler_name}`"),
                other,
            ),
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
                disable_plugin_for_session(
                    prepared
                        .session_id
                        .as_ref()
                        .map(sc_hooks_core::session::SessionId::as_str),
                    handler_name,
                )?;
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
                emit_dispatch_log_with_fallback(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::plugin_error_with_source(ai_message, err));
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let body = serde_json::to_vec(&stdin_payload).map_err(|err| {
                CliError::plugin_error_with_source("failed to serialize stdin payload", err)
            })?;
            if let Err(err) = stdin.write_all(&body) {
                disable_plugin_for_session(
                    prepared
                        .session_id
                        .as_ref()
                        .map(sc_hooks_core::session::SessionId::as_str),
                    handler_name,
                )?;
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
                emit_dispatch_log_with_fallback(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::plugin_error_with_source(ai_message, err));
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
                disable_plugin_for_session(
                    prepared
                        .session_id
                        .as_ref()
                        .map(sc_hooks_core::session::SessionId::as_str),
                    handler_name,
                )?;
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
                    emit_dispatch_log_with_fallback(
                        &log_base,
                        &log_results,
                        started.elapsed().as_millis(),
                        sc_hooks_core::exit_codes::SUCCESS,
                        Some(ai_message.as_str()),
                    );
                    async_system_message.push(ai_message);
                    continue;
                }
                emit_dispatch_log_with_fallback(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::TIMEOUT,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::timeout(ai_message));
            }
            Err(err) => {
                disable_plugin_for_session(
                    prepared
                        .session_id
                        .as_ref()
                        .map(sc_hooks_core::session::SessionId::as_str),
                    handler_name,
                )?;
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
                emit_dispatch_log_with_fallback(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::plugin_error_with_source(ai_message, err));
            }
        };
        use std::io::Read;
        let mut stdout = Vec::new();
        if let Some(mut out) = child.stdout.take()
            && let Err(err) = out.read_to_end(&mut stdout)
        {
            disable_plugin_for_session(
                prepared
                    .session_id
                    .as_ref()
                    .map(sc_hooks_core::session::SessionId::as_str),
                handler_name,
            )?;
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
            emit_dispatch_log_with_fallback(
                &log_base,
                &log_results,
                started.elapsed().as_millis(),
                sc_hooks_core::exit_codes::PLUGIN_ERROR,
                Some(ai_message.as_str()),
            );
            return Err(CliError::plugin_error_with_source(ai_message, err));
        }

        let mut stderr = Vec::new();
        if let Some(mut err) = child.stderr.take()
            && let Err(read_err) = err.read_to_end(&mut stderr)
        {
            disable_plugin_for_session(
                prepared
                    .session_id
                    .as_ref()
                    .map(sc_hooks_core::session::SessionId::as_str),
                handler_name,
            )?;
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
            emit_dispatch_log_with_fallback(
                &log_base,
                &log_results,
                started.elapsed().as_millis(),
                sc_hooks_core::exit_codes::PLUGIN_ERROR,
                Some(ai_message.as_str()),
            );
            return Err(CliError::plugin_error_with_source(
                ai_message,
                StderrCaptureContextError::new(&stderr, read_err),
            ));
        }

        let stdout_text = String::from_utf8_lossy(&stdout);
        let stderr_text =
            (!stderr.is_empty()).then(|| String::from_utf8_lossy(&stderr).into_owned());
        let stderr_ref = stderr_text.as_deref();

        if !status.success() {
            disable_plugin_for_session(
                prepared
                    .session_id
                    .as_ref()
                    .map(sc_hooks_core::session::SessionId::as_str),
                handler_name,
            )?;
            let ai_message = ai_notification(
                handler_name,
                "non-zero-exit",
                "inspect plugin stderr and run 'sc-hooks test <plugin>'.",
            );
            log_results.push(error_result(
                handler_name,
                handler_started.elapsed().as_millis(),
                "non_zero_exit",
                stderr_text.clone(),
                Some(true),
            ));
            emit_dispatch_log_with_fallback(
                &log_base,
                &log_results,
                started.elapsed().as_millis(),
                sc_hooks_core::exit_codes::PLUGIN_ERROR,
                Some(ai_message.as_str()),
            );
            return Err(plugin_error_with_context(
                ai_message,
                Some(PluginExecutionContextError::with_source(
                    "plugin execution failed",
                    plugin_termination_error(status, stderr_ref),
                )),
            ));
        }

        let (parsed, warning) = match parse_first_hook_result(&stdout_text) {
            Ok(parsed) => parsed,
            Err(err) => {
                disable_plugin_for_session(
                    prepared
                        .session_id
                        .as_ref()
                        .map(sc_hooks_core::session::SessionId::as_str),
                    handler_name,
                )?;
                let ai_message = ai_notification(
                    handler_name,
                    "invalid-json",
                    "ensure plugin writes a single valid JSON object to stdout.",
                );
                log_results.push(error_result(
                    handler_name,
                    handler_started.elapsed().as_millis(),
                    "invalid_json",
                    Some(format!(
                        "stdout={stdout_text}; stderr={}; {err}",
                        stderr_ref.unwrap_or("")
                    )),
                    Some(true),
                ));
                emit_dispatch_log_with_fallback(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(CliError::plugin_error_with_source(ai_message, err));
            }
        };

        let mut parsed = parsed;
        let (additional_context, root_divergence) =
            split_root_divergence_context(parsed.additional_context.take());
        let warning = merge_warnings(
            warning,
            root_divergence
                .as_ref()
                .map(RootDivergenceNotice::warning_message),
        );
        if let Some(notice) = root_divergence.as_ref() {
            emit_root_divergence_log_with_fallback(log_base.project_root, notice);
        }

        match parsed.action {
            sc_hooks_core::results::HookAction::Proceed => {
                log_results.push(HandlerResultRecord {
                    handler: handler_name.clone(),
                    action: Cow::Borrowed("proceed"),
                    ms: handler_started.elapsed().as_millis(),
                    error_type: None,
                    stderr: stderr_text.clone(),
                    warning,
                    disabled: None,
                });

                if mode == sc_hooks_core::dispatch::DispatchMode::Async {
                    if let Some(context) = additional_context {
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
                        prepared
                            .session_id
                            .as_ref()
                            .map(sc_hooks_core::session::SessionId::as_str),
                        handler_name,
                    )?;
                    let ai_message = ai_notification(
                        handler_name,
                        "async-block",
                        "update plugin to return proceed/error only when mode=async.",
                    );
                    log_results.push(error_result(
                        handler_name,
                        handler_started.elapsed().as_millis(),
                        "async_block",
                        stderr_text.clone(),
                        Some(true),
                    ));
                    emit_dispatch_log_with_fallback(
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
                    action: Cow::Borrowed("block"),
                    ms: handler_started.elapsed().as_millis(),
                    error_type: None,
                    stderr: stderr_text.clone(),
                    warning,
                    disabled: None,
                });
                emit_dispatch_log_with_fallback(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::BLOCKED,
                    None,
                );
                return Ok(DispatchOutcome::Blocked { reason });
            }
            sc_hooks_core::results::HookAction::Error => {
                disable_plugin_for_session(
                    prepared
                        .session_id
                        .as_ref()
                        .map(sc_hooks_core::session::SessionId::as_str),
                    handler_name,
                )?;
                let ai_message = ai_notification(
                    handler_name,
                    "action-error",
                    "fix plugin logic and run 'sc-hooks test <plugin>'.",
                );
                let action_error_message = parsed
                    .message
                    .unwrap_or_else(|| "plugin returned action=error".to_string());
                log_results.push(error_result(
                    handler_name,
                    handler_started.elapsed().as_millis(),
                    "action_error",
                    Some(action_error_message.clone()),
                    Some(true),
                ));
                emit_dispatch_log_with_fallback(
                    &log_base,
                    &log_results,
                    started.elapsed().as_millis(),
                    sc_hooks_core::exit_codes::PLUGIN_ERROR,
                    Some(ai_message.as_str()),
                );
                return Err(plugin_error_with_context(
                    ai_message,
                    stderr_text
                        .as_deref()
                        .map(|stderr| {
                            PluginExecutionContextError::new(format!("plugin stderr: {stderr}"))
                        })
                        .or_else(|| Some(PluginExecutionContextError::new(action_error_message))),
                ));
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

fn disable_plugin_for_session(
    session_id: Option<&str>,
    handler_name: &str,
) -> Result<(), CliError> {
    session::mark_plugin_disabled(session_id, handler_name, "runtime-error").map_err(|source| {
        CliError::internal_with_source(
            format!("failed persisting disabled state for `{handler_name}`"),
            source,
        )
    })
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
        project_root: base.project_root,
    })
}

fn emit_dispatch_log_with_fallback(
    base: &DispatchLogBase<'_>,
    results: &[HandlerResultRecord],
    total_ms: u128,
    exit: i32,
    ai_notification: Option<&str>,
) {
    if let Err(err) = emit_dispatch_log(base, results, total_ms, exit, ai_notification) {
        emit_observability_stderr_fallback(&err);
    }
}

fn emit_root_divergence_log(
    project_root: &AiRootDir,
    notice: &RootDivergenceNotice,
) -> Result<(), CliError> {
    observability::emit_root_divergence_event(observability::RootDivergenceEventArgs {
        notice,
        project_root,
    })
}

fn emit_root_divergence_log_with_fallback(project_root: &AiRootDir, notice: &RootDivergenceNotice) {
    if let Err(err) = emit_root_divergence_log(project_root, notice) {
        emit_observability_stderr_fallback(&err);
    }
}

fn emit_observability_stderr_fallback(err: &CliError) {
    error!("sc-hooks: failed emitting dispatch observability event: {err}");
}

fn split_root_divergence_context(
    additional_context: Option<String>,
) -> (Option<String>, Option<RootDivergenceNotice>) {
    let Some(additional_context) = additional_context else {
        return (None, None);
    };

    match RootDivergenceNotice::decode(&additional_context) {
        Some(notice) => (None, Some(notice)),
        None => (Some(additional_context), None),
    }
}

fn merge_warnings(existing: Option<String>, additional: Option<String>) -> Option<String> {
    match (existing, additional) {
        (Some(existing), Some(additional)) => Some(format!("{existing}; {additional}")),
        (Some(existing), None) => Some(existing),
        (None, Some(additional)) => Some(additional),
        (None, None) => None,
    }
}

fn parse_first_hook_result(
    stdout_text: &str,
) -> Result<(sc_hooks_core::results::HookResult, Option<String>), HookResultParseError> {
    let mut stream =
        serde_json::Deserializer::from_str(stdout_text).into_iter::<serde_json::Value>();
    let Some(first) = stream.next() else {
        return Err(HookResultParseError::EmptyStdout);
    };
    let first = first.map_err(HookResultParseError::MalformedFirstJson)?;
    let parsed = serde_json::from_value::<sc_hooks_core::results::HookResult>(first)
        .map_err(HookResultParseError::InvalidHookResult)?;

    let warning = match stream.next() {
        Some(Ok(_)) => {
            Some("plugin produced multiple JSON objects; only first object was used".to_string())
        }
        Some(Err(err)) => return Err(HookResultParseError::InvalidTrailingJson(err)),
        None => None,
    };

    Ok((parsed, warning))
}

fn error_result(
    handler_name: &str,
    ms: u128,
    error_type: &'static str,
    stderr: Option<String>,
    disabled: Option<bool>,
) -> HandlerResultRecord {
    HandlerResultRecord {
        handler: handler_name.to_string(),
        action: Cow::Borrowed("error"),
        ms,
        error_type: Some(Cow::Borrowed(error_type)),
        stderr,
        warning: None,
        disabled,
    }
}

fn plugin_error_with_context(
    message: String,
    context: Option<PluginExecutionContextError>,
) -> CliError {
    match context {
        Some(context) => CliError::plugin_error_with_source(message, context),
        None => CliError::plugin_error(message),
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
    use std::path::{Path, PathBuf};
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
            err.to_string().contains("invalid trailing stdout"),
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
    fn dispatch_rejects_async_long_running_before_spawn() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PostToolUse = ["notify"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let handler = resolution::ResolvedHandler {
            name: "notify".to_string(),
            executable_path: PathBuf::from(".sc-hooks/plugins/notify"),
            manifest: sc_hooks_core::manifest::Manifest {
                contract_version: 1,
                name: "notify".to_string(),
                mode: sc_hooks_core::dispatch::DispatchMode::Async,
                hooks: vec!["PostToolUse".to_string()],
                matchers: vec!["*".to_string()],
                payload_conditions: Vec::new(),
                timeout_ms: None,
                long_running: true,
                response_time: None,
                requires: std::collections::BTreeMap::new(),
                optional: std::collections::BTreeMap::new(),
                sandbox: None,
                description: Some("wait for remote ack".to_string()),
            },
        };

        let err = execute_chain(
            &[handler],
            &cfg,
            "PostToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Async,
            None,
        )
        .expect_err("dispatch should reject async long_running manifests before spawn");

        assert!(matches!(
            err,
            CliError::Resolution(crate::errors::ResolutionError::HandlerRejected {
                plugin,
                reason,
                source: _
            })
                if plugin == "notify" && reason.contains("long_running=true")
        ));
    }

    #[test]
    fn integration_dispatch_writes_structured_log_entry() {
        let root = test_support::shared_observability_root();
        let _cwd = test_support::scoped_current_dir(&root);

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

        let log_path = root.join(".sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl");
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
    fn plugin_only_chain_completes_under_500ms_median() {
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
            median < Duration::from_millis(500),
            "median plugin chain runtime {median:?} exceeded 500ms target"
        );
    }
}
