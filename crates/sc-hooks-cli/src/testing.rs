use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Serialize;

use crate::errors::CliError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ComplianceCheck {
    pub name: String,
    pub passed: bool,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ComplianceReport {
    pub plugin: String,
    pub checks: Vec<ComplianceCheck>,
}

impl ComplianceReport {
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|check| check.passed)
    }

    pub fn render_text(&self) -> String {
        let mut lines = vec![format!("Plugin: {}", self.plugin)];
        for check in &self.checks {
            let status = if check.passed { "✓" } else { "✗" };
            if let Some(detail) = check.detail.as_ref() {
                lines.push(format!("  {} {}: {}", status, check.name, detail));
            } else {
                lines.push(format!("  {} {}", status, check.name));
            }
        }
        lines.join("\n")
    }
}

pub fn run_plugin_compliance(plugin: &str) -> Result<ComplianceReport, CliError> {
    let path = plugin_path(plugin);
    Ok(run_compliance(&path, plugin))
}

fn run_compliance(plugin_path: &Path, plugin: &str) -> ComplianceReport {
    let mut checks = Vec::new();
    let plugin_path_str = plugin_path.display().to_string();

    checks.push(ComplianceCheck {
        name: "executable exists".to_string(),
        passed: plugin_path.exists(),
        detail: Some(plugin_path_str.clone()),
    });

    if !plugin_path.exists() {
        return ComplianceReport {
            plugin: plugin.to_string(),
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
                plugin: plugin.to_string(),
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

    checks.push(invoke_plugin(
        plugin_path,
        serde_json::json!({"hook": {"type": "PreToolUse"}}),
    ));

    ComplianceReport {
        plugin: plugin.to_string(),
        checks,
    }
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
                "non-zero status {:?}, stderr={}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            )),
        },
        Err(err) => ComplianceCheck {
            name: "json protocol conformance".to_string(),
            passed: false,
            detail: Some(err.to_string()),
        },
    }
}

fn plugin_path(plugin: &str) -> PathBuf {
    Path::new(".sc-hooks").join("plugins").join(plugin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compliance_reports_missing_plugin() {
        let report = run_plugin_compliance("missing-plugin").expect("compliance should run");
        assert!(!report.passed());
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.name == "executable exists" && !check.passed)
        );
    }
}
