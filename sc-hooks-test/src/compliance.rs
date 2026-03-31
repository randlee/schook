use std::path::Path;
use std::process::{Command, Stdio};

use serde::Serialize;

mod private {
    pub trait Sealed {}
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
/// Result of a single compliance assertion.
pub struct ComplianceCheck {
    /// Human-readable check name.
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Optional detail text for failures or supporting evidence.
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
/// Summary report for manifest/protocol compliance checks against one plugin.
pub struct ComplianceReport {
    /// Display path to the plugin executable under test.
    pub plugin_path: String,
    /// Individual checks performed for the plugin.
    pub checks: Vec<ComplianceCheck>,
}

impl ComplianceReport {
    /// Returns whether every recorded check passed.
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|check| check.passed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Shared host-behavior scenarios exercised by the compliance harness.
pub enum ContractScenario {
    /// Dispatch with no payload object.
    AbsentPayload,
    /// Plugin returns invalid stdout.
    InvalidOutput,
    /// Plugin returns multiple JSON objects.
    MultipleJsonObjects,
    /// Async plugin attempts to block.
    AsyncBlockMisuse,
    /// Matcher mismatch skips the plugin cleanly.
    MatcherFiltering,
    /// Plugin exceeds its timeout.
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Captured host outcome for a contract-behavior scenario.
pub struct ContractScenarioResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Last observability log line, when one was emitted.
    pub last_log_line: Option<String>,
    /// Serialized session state after the scenario, when present.
    pub session_state: Option<String>,
    /// Whether the scenario-specific marker file exists.
    pub marker_exists: bool,
}

/// Sealed probe interface for exercising the real host dispatch path against
/// the shared compliance scenarios without exposing arbitrary external probe
/// implementations. Implementors must provide
/// `run_scenario(&self, scenario: ContractScenario) -> Result<ContractScenarioResult, String>`.
pub trait HostDispatchProbe: private::Sealed {
    /// Executes one shared contract scenario through the real host path.
    fn run_scenario(&self, scenario: ContractScenario) -> Result<ContractScenarioResult, String>;
}

/// Function-backed implementation of [`HostDispatchProbe`].
pub struct FnHostDispatchProbe<F> {
    run: F,
}

impl<F> FnHostDispatchProbe<F> {
    /// Wraps a function or closure as a host dispatch probe.
    pub fn new(run: F) -> Self {
        Self { run }
    }
}

impl<F> private::Sealed for FnHostDispatchProbe<F> {}

impl<F> HostDispatchProbe for FnHostDispatchProbe<F>
where
    F: Fn(ContractScenario) -> Result<ContractScenarioResult, String>,
{
    fn run_scenario(&self, scenario: ContractScenario) -> Result<ContractScenarioResult, String> {
        (self.run)(scenario)
    }
}

/// Runs the basic manifest/protocol compliance suite against one plugin executable.
pub fn run_compliance(plugin_path: &Path) -> ComplianceReport {
    let mut checks = Vec::new();
    let plugin_path_str = plugin_path.display().to_string();

    checks.push(ComplianceCheck {
        name: "executable exists".to_string(),
        passed: plugin_path.exists(),
        detail: Some(plugin_path_str.clone()),
    });

    if !plugin_path.exists() {
        return ComplianceReport {
            plugin_path: plugin_path_str,
            checks,
        };
    }

    let manifest = match sc_hooks_sdk::manifest::load_manifest_from_executable(plugin_path) {
        Ok(manifest) => {
            checks.push(ComplianceCheck {
                name: "manifest valid".to_string(),
                passed: true,
                detail: None,
            });
            manifest
        }
        Err(err) => {
            checks.push(ComplianceCheck {
                name: "manifest valid".to_string(),
                passed: false,
                detail: Some(err.to_string()),
            });
            return ComplianceReport {
                plugin_path: plugin_path_str,
                checks,
            };
        }
    };

    checks.push(ComplianceCheck {
        name: "contract compatibility".to_string(),
        passed: sc_hooks_sdk::manifest::is_contract_compatible(
            sc_hooks_sdk::manifest::HOST_CONTRACT_VERSION,
            manifest.contract_version,
        ),
        detail: Some(format!("plugin={}", manifest.contract_version)),
    });

    checks.push(ComplianceCheck {
        name: "mode declared".to_string(),
        passed: matches!(
            manifest.mode,
            sc_hooks_core::dispatch::DispatchMode::Sync
                | sc_hooks_core::dispatch::DispatchMode::Async
        ),
        detail: Some(manifest.mode.as_str().to_string()),
    });

    checks.push(ComplianceCheck {
        name: "matcher non-empty".to_string(),
        passed: !manifest.matchers.is_empty(),
        detail: Some(format!("{} matcher(s)", manifest.matchers.len())),
    });

    checks.push(ComplianceCheck {
        name: "timeout valid".to_string(),
        passed: manifest.timeout_ms.is_none_or(|timeout| timeout > 0),
        detail: manifest.timeout_ms.map(|timeout| timeout.to_string()),
    });

    let protocol_result = invoke_plugin(
        plugin_path,
        serde_json::json!({"hook": {"type": "PreToolUse"}}),
    );
    checks.push(protocol_result);

    ComplianceReport {
        plugin_path: plugin_path_str,
        checks,
    }
}

/// Runs shared host-behavior compliance scenarios against a dispatch probe.
pub fn run_contract_behavior_suite(probe: &impl HostDispatchProbe) -> Vec<ComplianceCheck> {
    vec![
        check_absent_payload(probe),
        check_invalid_output(probe),
        check_multiple_json_objects(probe),
        check_async_block_misuse(probe),
        check_matcher_filtering(probe),
        check_timeout(probe),
    ]
}

fn invoke_plugin(plugin_path: &Path, input: serde_json::Value) -> ComplianceCheck {
    let output = Command::new(plugin_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let body = serde_json::to_vec(&input).map_err(std::io::Error::other)?;
                stdin.write_all(&body)?;
            }
            child.wait_with_output()
        });

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<sc_hooks_core::results::HookResult>(&stdout) {
                Ok(_) => ComplianceCheck {
                    name: "json protocol conformance".to_string(),
                    passed: true,
                    detail: None,
                },
                Err(err) => ComplianceCheck {
                    name: "json protocol conformance".to_string(),
                    passed: false,
                    detail: Some(format!("invalid JSON output: {err}")),
                },
            }
        }
        Ok(output) => ComplianceCheck {
            name: "json protocol conformance".to_string(),
            passed: false,
            detail: Some(format!(
                "non-zero status {:?}, stderr={} ",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            )),
        },
        Err(err) => ComplianceCheck {
            name: "json protocol conformance".to_string(),
            passed: false,
            detail: Some(err.to_string()),
        },
    }
}

fn check_absent_payload(probe: &impl HostDispatchProbe) -> ComplianceCheck {
    match probe.run_scenario(ContractScenario::AbsentPayload) {
        Ok(result) => ComplianceCheck {
            name: "host dispatch handles absent payload".to_string(),
            passed: result.exit_code == sc_hooks_core::exit_codes::SUCCESS,
            detail: Some(format!(
                "exit={}, stderr={}",
                result.exit_code,
                result.stderr.trim()
            )),
        },
        Err(err) => ComplianceCheck {
            name: "host dispatch handles absent payload".to_string(),
            passed: false,
            detail: Some(err),
        },
    }
}

fn check_invalid_output(probe: &impl HostDispatchProbe) -> ComplianceCheck {
    match probe.run_scenario(ContractScenario::InvalidOutput) {
        Ok(result) => {
            let disabled = session_disables_plugin_for_reason(
                result.session_state.as_deref(),
                "probe-plugin",
                "runtime-error",
            );
            ComplianceCheck {
                name: "host dispatch rejects invalid stdout".to_string(),
                passed: result.exit_code == sc_hooks_core::exit_codes::PLUGIN_ERROR
                    && result.stderr.contains("invalid JSON")
                    && disabled,
                detail: Some(format!(
                    "exit={}, stderr={}, session_state_present={}",
                    result.exit_code,
                    result.stderr.trim(),
                    result.session_state.is_some()
                )),
            }
        }
        Err(err) => ComplianceCheck {
            name: "host dispatch rejects invalid stdout".to_string(),
            passed: false,
            detail: Some(err),
        },
    }
}

fn check_multiple_json_objects(probe: &impl HostDispatchProbe) -> ComplianceCheck {
    match probe.run_scenario(ContractScenario::MultipleJsonObjects) {
        Ok(result) => {
            let warning_seen = result
                .last_log_line
                .as_deref()
                .and_then(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .and_then(|line| line["fields"]["results"].as_array().cloned())
                .is_some_and(|results| {
                    results.iter().any(|entry| {
                        entry["warning"]
                            .as_str()
                            .is_some_and(|warning| warning.contains("multiple JSON objects"))
                    })
                });

            ComplianceCheck {
                name: "host dispatch warns on multiple JSON objects".to_string(),
                passed: result.exit_code == sc_hooks_core::exit_codes::SUCCESS && warning_seen,
                detail: Some(format!(
                    "exit={}, log_warning_seen={warning_seen}",
                    result.exit_code
                )),
            }
        }
        Err(err) => ComplianceCheck {
            name: "host dispatch warns on multiple JSON objects".to_string(),
            passed: false,
            detail: Some(err),
        },
    }
}

fn check_async_block_misuse(probe: &impl HostDispatchProbe) -> ComplianceCheck {
    match probe.run_scenario(ContractScenario::AsyncBlockMisuse) {
        Ok(result) => {
            let disabled = session_disables_plugin_for_reason(
                result.session_state.as_deref(),
                "probe-plugin",
                "runtime-error",
            );
            let system_message_seen = stdout_field_present(&result.stdout, "systemMessage");
            ComplianceCheck {
                name: "host dispatch rejects async block misuse".to_string(),
                passed: result.exit_code == sc_hooks_core::exit_codes::SUCCESS
                    && disabled
                    && system_message_seen,
                detail: Some(format!(
                    "exit={}, stdout={}, session_state_present={}",
                    result.exit_code,
                    result.stdout.trim(),
                    result.session_state.is_some()
                )),
            }
        }
        Err(err) => ComplianceCheck {
            name: "host dispatch rejects async block misuse".to_string(),
            passed: false,
            detail: Some(err),
        },
    }
}

fn check_matcher_filtering(probe: &impl HostDispatchProbe) -> ComplianceCheck {
    match probe.run_scenario(ContractScenario::MatcherFiltering) {
        Ok(result) => ComplianceCheck {
            name: "host dispatch enforces matcher filtering".to_string(),
            passed: result.exit_code == sc_hooks_core::exit_codes::SUCCESS && !result.marker_exists,
            detail: Some(format!(
                "exit={}, marker_exists={}",
                result.exit_code, result.marker_exists
            )),
        },
        Err(err) => ComplianceCheck {
            name: "host dispatch enforces matcher filtering".to_string(),
            passed: false,
            detail: Some(err),
        },
    }
}

fn check_timeout(probe: &impl HostDispatchProbe) -> ComplianceCheck {
    match probe.run_scenario(ContractScenario::Timeout) {
        Ok(result) => {
            let disabled = session_disables_plugin_for_reason(
                result.session_state.as_deref(),
                "probe-plugin",
                "runtime-error",
            );
            let timeout_logged =
                last_log_reports_error_type(result.last_log_line.as_deref(), "timeout");
            ComplianceCheck {
                name: "host dispatch enforces timeout".to_string(),
                passed: result.exit_code == sc_hooks_core::exit_codes::TIMEOUT
                    && timeout_logged
                    && disabled,
                detail: Some(format!(
                    "exit={}, timeout_logged={}, stderr={}",
                    result.exit_code,
                    timeout_logged,
                    result.stderr.trim()
                )),
            }
        }
        Err(err) => ComplianceCheck {
            name: "host dispatch enforces timeout".to_string(),
            passed: false,
            detail: Some(err),
        },
    }
}

fn stdout_field_present(stdout: &str, field: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(stdout)
        .ok()
        .and_then(|value| value.get(field).cloned())
        .is_some_and(|value| !value.is_null())
}

fn last_log_reports_error_type(last_log_line: Option<&str>, error_type: &str) -> bool {
    last_log_line
        .and_then(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .and_then(|line| line["fields"]["results"].as_array().cloned())
        .is_some_and(|results| {
            results
                .iter()
                .any(|entry| entry["error_type"].as_str() == Some(error_type))
        })
}

fn session_disables_plugin_for_reason(
    session_state: Option<&str>,
    plugin: &str,
    reason: &str,
) -> bool {
    session_state
        .and_then(|state| serde_json::from_str::<serde_json::Value>(state).ok())
        .and_then(|state| state["sessions"].as_object().cloned())
        .is_some_and(|sessions| {
            sessions.values().any(|session| {
                session["disabled_plugins"][plugin]["reason"].as_str() == Some(reason)
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn reports_passing_shell_fixture() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let plugin = fixtures::plugin_path(temp.path(), "guard-paths");
        fixtures::create_shell_plugin(
            &plugin,
            r#"{"contract_version":1,"name":"guard-paths","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
            r#"{"action":"proceed"}"#,
        );

        let report = run_compliance(&plugin);
        assert!(report.passed());
    }
}
