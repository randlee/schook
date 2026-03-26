use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::ScHooksConfig;
use crate::{events, install, metadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AuditOptions {
    pub strict: bool,
}

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

pub fn run(
    config: &ScHooksConfig,
    options: AuditOptions,
) -> Result<AuditReport, crate::errors::CliError> {
    let mut report = AuditReport::default();
    let runtime = metadata::RuntimeMetadata::discover()?;
    let context = config.mapped_context_metadata();

    warn_on_plugins_dir_permissions(&mut report);

    for (hook_name, chain) in &config.hooks {
        for handler_name in chain {
            let plugin_path = plugin_path(handler_name);
            if !plugin_path.exists() {
                report
                    .errors
                    .push(format!("AUD-001 unresolved handler `{handler_name}`"));
                continue;
            }

            warn_on_plugin_integrity(handler_name, &plugin_path, &mut report);

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
            if manifest.mode == sc_hooks_core::dispatch::DispatchMode::Async
                && manifest.long_running
            {
                report.errors.push(format!(
                    "AUD-006 handler `{handler_name}` declares blocking behavior (long_running=true) while mode=async"
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

            validate_sandbox_requirements(
                &mut report,
                config,
                handler_name,
                manifest.sandbox.as_ref(),
                options.strict,
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

                if let Some(rule) = requirement.validate.as_ref()
                    && let Some((rule, _)) = sc_hooks_core::validation::parse_validation_rule(rule)
                {
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

    match install::build_settings(config) {
        Ok(plan) => {
            report.warnings.extend(
                plan.warnings
                    .into_iter()
                    .map(|warning| format!("AUD-007 {warning}")),
            );

            for (hook, entries) in &plan.settings.hooks {
                for entry in entries {
                    let mut async_buckets: BTreeSet<String> = BTreeSet::new();
                    let mut has_sync = false;
                    for command in &entry.hooks {
                        match command.r#async {
                            Some(true) => {
                                if let Some(bucket) = extract_async_bucket(&command.command) {
                                    async_buckets.insert(bucket);
                                }
                            }
                            _ => has_sync = true,
                        }
                    }

                    let bucket_summary = if async_buckets.is_empty() {
                        "none".to_string()
                    } else {
                        async_buckets.into_iter().collect::<Vec<_>>().join(",")
                    };
                    report.install_summary.push(format!(
                        "AUD-007 {hook}/{matcher} -> sync={has_sync}, async_buckets={bucket_summary}",
                        matcher = entry.matcher
                    ));
                }
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

fn validate_sandbox_requirements(
    report: &mut AuditReport,
    config: &ScHooksConfig,
    handler_name: &str,
    sandbox: Option<&sc_hooks_core::manifest::SandboxSpec>,
    strict: bool,
) {
    let Some(sandbox) = sandbox else {
        return;
    };

    if sandbox.needs_network
        && !config
            .sandbox
            .allow_network
            .iter()
            .any(|name| name == handler_name)
    {
        push_sandbox_exceeded(
            report,
            strict,
            format!(
                "SEC-004 `{handler_name}` requires network but is not listed in [sandbox].allow_network"
            ),
        );
    }

    let allowed_paths = config
        .sandbox
        .allow_paths
        .get(handler_name)
        .cloned()
        .unwrap_or_default();

    for path in &sandbox.paths {
        if !Path::new(path).exists() {
            report.errors.push(format!(
                "SEC-002 `{handler_name}` declares sandbox path `{path}` that does not exist"
            ));
        }

        if !allowed_paths.iter().any(|allowed| allowed == path) {
            push_sandbox_exceeded(
                report,
                strict,
                format!(
                    "SEC-004 `{handler_name}` sandbox path `{path}` is not acknowledged in [sandbox].allow_paths"
                ),
            );
        }
    }
}

fn push_sandbox_exceeded(report: &mut AuditReport, strict: bool, message: String) {
    if strict {
        report.errors.push(message);
    } else {
        report.warnings.push(message);
    }
}

fn warn_on_plugins_dir_permissions(report: &mut AuditReport) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let plugin_dir = Path::new(".sc-hooks/plugins");
        if let Ok(metadata) = std::fs::metadata(plugin_dir) {
            let mode = metadata.permissions().mode();
            if mode & 0o022 != 0 {
                report.warnings.push(format!(
                    "SEC-006 plugin directory `{}` has permissive mode {:o}",
                    plugin_dir.display(),
                    mode
                ));
            }
        }
    }
}

fn warn_on_plugin_integrity(handler_name: &str, plugin_path: &Path, report: &mut AuditReport) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        if let Ok(metadata) = std::fs::metadata(plugin_path) {
            let mode = metadata.permissions().mode();
            if mode & 0o002 != 0 {
                report.warnings.push(format!(
                    "SEC-005 plugin `{handler_name}` is world-writable ({:o}) at {}",
                    mode,
                    plugin_path.display()
                ));
            }

            // SAFETY: `geteuid` has no preconditions and returns the current effective UID.
            let effective_uid = unsafe { nix::libc::geteuid() };
            if metadata.uid() != effective_uid {
                report.warnings.push(format!(
                    "SEC-005 plugin `{handler_name}` is not owned by current user ({})",
                    plugin_path.display()
                ));
            }
        }
    }
}

fn extract_async_bucket(command: &str) -> Option<String> {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    for (index, part) in parts.iter().enumerate() {
        if *part == "--async-bucket" {
            return parts.get(index + 1).map(|value| (*value).to_string());
        }
    }
    None
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
            .unwrap_or_else(|e| e.into_inner());
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

        let report = run(&cfg, AuditOptions::default()).expect("audit should execute");
        assert!(report.has_errors());
        assert!(report.errors.iter().any(|entry| entry.contains("AUD-001")));

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn audit_accepts_valid_plugin_manifest() {
        let _guard = test_support::cwd_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch to temp");

        make_plugin(
            Path::new(".sc-hooks/plugins/guard-paths"),
            r#"{"contract_version":1,"name":"guard-paths","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{},"sandbox":{"paths":[],"needs_network":false}}"#,
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

        let report = run(&cfg, AuditOptions::default()).expect("audit should execute");
        assert!(!report.has_errors());

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn strict_mode_turns_unacknowledged_sandbox_needs_into_errors() {
        let _guard = test_support::cwd_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch to temp");

        fs::create_dir_all(".sc-hooks/needs").expect("path should be creatable");
        make_plugin(
            Path::new(".sc-hooks/plugins/notify"),
            r#"{"contract_version":1,"name":"notify","mode":"async","hooks":["PostToolUse"],"matchers":["*"],"requires":{},"sandbox":{"paths":[".sc-hooks/needs"],"needs_network":true}}"#,
        );

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

        let report = run(&cfg, AuditOptions { strict: true }).expect("audit should execute");
        assert!(report.errors.iter().any(|entry| entry.contains("SEC-004")));

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn audit_rejects_async_handlers_declaring_blocking_behavior() {
        let _guard = test_support::cwd_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch to temp");

        make_plugin(
            Path::new(".sc-hooks/plugins/notify"),
            r#"{"contract_version":1,"name":"notify","mode":"async","hooks":["PostToolUse"],"matchers":["*"],"long_running":true,"description":"wait for remote ack","requires":{}}"#,
        );

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

        let report = run(&cfg, AuditOptions::default()).expect("audit should execute");
        assert!(
            report
                .errors
                .iter()
                .any(|entry| { entry.contains("AUD-006") && entry.contains("blocking behavior") })
        );

        std::env::set_current_dir(original).expect("cwd should restore");
    }
}
