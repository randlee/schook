use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::ScHooksConfig;
use crate::{events, install, metadata};

#[derive(Debug, Default)]
pub struct AuditReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub install_summary: Vec<String>,
}

impl AuditReport {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

pub fn run(config: &ScHooksConfig) -> Result<AuditReport, crate::errors::CliError> {
    let mut report = AuditReport::default();

    let runtime = metadata::RuntimeMetadata::discover()?;
    let context = config.mapped_context_metadata();

    for (hook_name, chain) in &config.hooks {
        for handler_name in chain {
            if handler_name == "log" {
                continue;
            }

            let plugin_path = plugin_path(handler_name);
            if !plugin_path.exists() {
                report
                    .errors
                    .push(format!("AUD-001 unresolved handler `{handler_name}`"));
                continue;
            }

            let manifest = match sc_hooks_sdk::manifest::load_manifest_from_executable(&plugin_path)
            {
                Ok(manifest) => manifest,
                Err(err) => {
                    report.errors.push(format!(
                        "AUD-002 manifest load failed for `{handler_name}`: {err}"
                    ));
                    continue;
                }
            };

            if !manifest.hooks.iter().any(|hook| hook == hook_name) {
                report.errors.push(format!(
                    "AUD-006 handler `{handler_name}` does not declare hook `{hook_name}`"
                ));
            }

            if manifest.long_running
                && manifest
                    .description
                    .as_ref()
                    .map(|text| text.trim().is_empty())
                    .unwrap_or(true)
            {
                report.errors.push(format!(
                    "AUD-009 handler `{handler_name}` long_running requires non-empty description"
                ));
            }

            let taxonomy = events::validate_matchers_for_hook(hook_name, &manifest.matchers);
            report.warnings.extend(
                taxonomy
                    .warnings
                    .into_iter()
                    .map(|warning| format!("AUD-008 warning {warning}")),
            );
            report.errors.extend(
                taxonomy
                    .errors
                    .into_iter()
                    .map(|error| format!("AUD-008 {error}")),
            );

            let metadata_value =
                metadata::assemble_metadata(&runtime, &context, hook_name, None, None)?;
            for (field, requirement) in &manifest.requires {
                let Some(value) = value_by_path(&metadata_value, field) else {
                    report.errors.push(format!(
                        "AUD-003 `{handler_name}` missing required metadata field `{field}`"
                    ));
                    continue;
                };

                if let Some(rule) = requirement.validate.as_ref() {
                    let parsed = sc_hooks_core::validation::parse_validation_rule(rule);
                    if let Some((rule, _)) = parsed {
                        match rule {
                            sc_hooks_core::validation::ValidationRule::DirExists => {
                                if value.as_str().is_none_or(|path| !Path::new(path).is_dir()) {
                                    report.errors.push(format!(
                                        "AUD-004 `{handler_name}` dir_exists failed for `{field}`"
                                    ));
                                }
                            }
                            sc_hooks_core::validation::ValidationRule::FileExists => {
                                if value.as_str().is_none_or(|path| !Path::new(path).is_file()) {
                                    report.errors.push(format!(
                                        "AUD-004 `{handler_name}` file_exists failed for `{field}`"
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    match install::build_settings(config) {
        Ok(plan) => {
            report.warnings.extend(
                plan.warnings
                    .into_iter()
                    .map(|warning| format!("AUD-007 {warning}")),
            );
            for (hook, entries) in &plan.settings.hooks {
                report.install_summary.push(format!(
                    "AUD-007 {hook} has {} matcher entries",
                    entries.len()
                ));
            }
        }
        Err(err) => {
            report
                .errors
                .push(format!("AUD-007 install plan generation failed: {err}"));
        }
    }

    Ok(report)
}

pub fn render(report: &AuditReport) -> String {
    let mut lines = Vec::new();
    lines.push("Audit report".to_string());
    if report.errors.is_empty() {
        lines.push("errors: 0".to_string());
    } else {
        lines.push(format!("errors: {}", report.errors.len()));
        lines.extend(report.errors.iter().map(|error| format!("- {error}")));
    }

    if report.warnings.is_empty() {
        lines.push("warnings: 0".to_string());
    } else {
        lines.push(format!("warnings: {}", report.warnings.len()));
        lines.extend(report.warnings.iter().map(|warning| format!("- {warning}")));
    }

    if !report.install_summary.is_empty() {
        lines.push("install plan:".to_string());
        lines.extend(
            report
                .install_summary
                .iter()
                .map(|entry| format!("- {entry}")),
        );
    }

    lines.join("\n")
}

fn value_by_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current.as_object()?.get(segment)?;
    }
    Some(current)
}

fn plugin_path(handler_name: &str) -> PathBuf {
    Path::new(".sc-hooks").join("plugins").join(handler_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::test_support;
    use std::fs;

    fn make_plugin(path: &Path, manifest: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("plugin parent directory should be creatable");
        }

        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--manifest\" ]; then\n  cat <<'JSON'\n{manifest}\nJSON\n  exit 0\nfi\ncat >/dev/null\ncat <<'JSON'\n{{\"action\":\"proceed\"}}\nJSON\n"
        );
        fs::write(path, script).expect("plugin script should be writable");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)
                .expect("plugin metadata should be available")
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).expect("plugin should be made executable");
        }
    }

    #[test]
    fn audit_reports_missing_handler() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch to temp");

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["missing-plugin"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let report = run(&cfg).expect("audit should execute");
        assert!(report.has_errors());
        assert!(report.errors.iter().any(|entry| entry.contains("AUD-001")));

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn audit_accepts_valid_plugin_manifest() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch to temp");

        make_plugin(
            Path::new(".sc-hooks/plugins/guard-paths"),
            r#"{"contract_version":1,"name":"guard-paths","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
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

        let report = run(&cfg).expect("audit should execute");
        assert!(!report.has_errors());

        std::env::set_current_dir(original).expect("cwd should restore");
    }
}
