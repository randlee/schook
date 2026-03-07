use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Serialize;

use crate::errors::CliError;
use crate::events;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ComplianceCheck {
    pub name: String,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    let mut checks = Vec::new();

    checks.push(ComplianceCheck {
        name: "executable exists".to_string(),
        passed: path.exists(),
        detail: Some(path.display().to_string()),
    });
    if !path.exists() {
        return Ok(ComplianceReport {
            plugin: plugin.to_string(),
            checks,
        });
    }

    let manifest = match sc_hooks_sdk::manifest::load_manifest_from_executable(&path) {
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
            return Ok(ComplianceReport {
                plugin: plugin.to_string(),
                checks,
            });
        }
    };

    let contract_ok = sc_hooks_sdk::manifest::is_contract_compatible(
        sc_hooks_sdk::manifest::HOST_CONTRACT_VERSION,
        manifest.contract_version,
    );
    checks.push(ComplianceCheck {
        name: "contract compatibility".to_string(),
        passed: contract_ok,
        detail: Some(format!(
            "host={}, plugin={}",
            sc_hooks_sdk::manifest::HOST_CONTRACT_VERSION,
            manifest.contract_version
        )),
    });

    let mut matcher_errors = Vec::new();
    for hook in &manifest.hooks {
        let validation = events::validate_matchers_for_hook(hook, &manifest.matchers);
        matcher_errors.extend(validation.errors);
    }
    checks.push(ComplianceCheck {
        name: "matcher validity".to_string(),
        passed: matcher_errors.is_empty(),
        detail: if matcher_errors.is_empty() {
            None
        } else {
            Some(matcher_errors.join("; "))
        },
    });

    let minimal_input = serde_json::json!({"hook":{"type":"PreToolUse"}});
    let output = Command::new(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let body = serde_json::to_vec(&minimal_input).map_err(std::io::Error::other)?;
                stdin.write_all(&body)?;
            }
            child.wait_with_output()
        });

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parsed = serde_json::from_str::<sc_hooks_core::results::HookResult>(&stdout);
            checks.push(ComplianceCheck {
                name: "protocol JSON output".to_string(),
                passed: parsed.is_ok(),
                detail: parsed.err().map(|err| err.to_string()),
            });
        }
        Ok(output) => {
            checks.push(ComplianceCheck {
                name: "protocol JSON output".to_string(),
                passed: false,
                detail: Some(format!(
                    "non-zero exit {:?}; stderr={} ",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr)
                )),
            });
        }
        Err(err) => {
            checks.push(ComplianceCheck {
                name: "protocol JSON output".to_string(),
                passed: false,
                detail: Some(err.to_string()),
            });
        }
    }

    Ok(ComplianceReport {
        plugin: plugin.to_string(),
        checks,
    })
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
