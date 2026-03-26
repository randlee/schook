use std::path::{Path, PathBuf};

use sc_hooks_test::compliance;
use serde::Serialize;

use crate::errors::CliError;

pub type ComplianceCheck = compliance::ComplianceCheck;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ComplianceReport {
    pub plugin: String,
    pub checks: Vec<ComplianceCheck>,
}

impl ComplianceReport {
    fn from_shared(plugin: &str, report: compliance::ComplianceReport) -> Self {
        Self {
            plugin: plugin.to_string(),
            checks: report.checks,
        }
    }

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
    Ok(ComplianceReport::from_shared(
        plugin,
        compliance::run_compliance(&path),
    ))
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
