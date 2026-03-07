use std::path::Path;
use std::process::{Command, Stdio};

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ComplianceCheck {
    pub name: String,
    pub passed: bool,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ComplianceReport {
    pub plugin_path: String,
    pub checks: Vec<ComplianceCheck>,
}

impl ComplianceReport {
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|check| check.passed)
    }
}

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

    let absent_payload_result = invoke_plugin(
        plugin_path,
        serde_json::json!({"hook": {"type": "PreToolUse"}}),
    );
    checks.push(ComplianceCheck {
        name: "absent payload handling".to_string(),
        passed: absent_payload_result.passed,
        detail: absent_payload_result.detail,
    });

    ComplianceReport {
        plugin_path: plugin_path_str,
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
