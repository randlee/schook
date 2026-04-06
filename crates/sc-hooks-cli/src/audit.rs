use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::ScHooksConfig;
use crate::{events, install, metadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AuditOptions {
    pub strict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditDiagnostic {
    UnresolvedHandler {
        handler_name: String,
    },
    ManifestLoadFailed {
        handler_name: String,
        error: String,
    },
    MissingRequiredMetadataField {
        handler_name: String,
        field: String,
    },
    ValidationRuleFailed {
        handler_name: String,
        field: String,
        rule: ValidationAuditRule,
    },
    AsyncLongRunningUnsupported {
        handler_name: String,
    },
    HookNotDeclared {
        handler_name: String,
        hook_name: String,
    },
    MatcherWarning {
        message: String,
    },
    MatcherError {
        message: String,
    },
    InstallPlanWarning {
        message: String,
    },
    InstallPlanGenerationFailed {
        error: String,
    },
    InstallPlanEntry {
        hook: String,
        matcher: String,
        has_sync: bool,
        async_buckets: Vec<String>,
    },
    MissingLongRunningDescription {
        handler_name: String,
    },
    SandboxNeedsNetworkAck {
        handler_name: String,
    },
    SandboxPathMissing {
        handler_name: String,
        path: String,
    },
    SandboxPathUnacknowledged {
        handler_name: String,
        path: String,
    },
    PluginsDirPermissive {
        path: String,
        mode: u32,
    },
    PluginWorldWritable {
        handler_name: String,
        mode: u32,
        path: String,
    },
    PluginWrongOwner {
        handler_name: String,
        path: String,
    },
}

impl AuditDiagnostic {
    pub fn render(&self) -> String {
        match self {
            Self::UnresolvedHandler { handler_name } => {
                format!("AUD-001 unresolved handler `{handler_name}`")
            }
            Self::ManifestLoadFailed {
                handler_name,
                error,
            } => format!("AUD-002 manifest load failed for `{handler_name}`: {error}"),
            Self::MissingRequiredMetadataField {
                handler_name,
                field,
            } => format!("AUD-003 `{handler_name}` missing required metadata field `{field}`"),
            Self::ValidationRuleFailed {
                handler_name,
                field,
                rule,
            } => format!(
                "AUD-004 `{handler_name}` {} failed for `{field}`",
                rule.as_str()
            ),
            Self::AsyncLongRunningUnsupported { handler_name } => format!(
                "AUD-005 handler `{handler_name}` long_running=true is only supported for sync handlers"
            ),
            Self::HookNotDeclared {
                handler_name,
                hook_name,
            } => format!("AUD-006 handler `{handler_name}` does not declare hook `{hook_name}`"),
            Self::MatcherWarning { message } => format!("AUD-008W warning {message}"),
            Self::MatcherError { message } => format!("AUD-008 {message}"),
            Self::InstallPlanWarning { message } => format!("AUD-007 {message}"),
            Self::InstallPlanGenerationFailed { error } => {
                format!("AUD-007 install plan generation failed: {error}")
            }
            Self::InstallPlanEntry {
                hook,
                matcher,
                has_sync,
                async_buckets,
            } => {
                let bucket_summary = if async_buckets.is_empty() {
                    "none".to_string()
                } else {
                    async_buckets.join(",")
                };
                format!(
                    "AUD-007 {hook}/{matcher} -> sync={has_sync}, async_buckets={bucket_summary}"
                )
            }
            Self::MissingLongRunningDescription { handler_name } => format!(
                "AUD-009 handler `{handler_name}` long_running requires non-empty description"
            ),
            Self::SandboxNeedsNetworkAck { handler_name } => format!(
                "SEC-004 `{handler_name}` requires network but is not listed in [sandbox].allow_network"
            ),
            Self::SandboxPathMissing { handler_name, path } => format!(
                "SEC-002 `{handler_name}` declares sandbox path `{path}` that does not exist"
            ),
            Self::SandboxPathUnacknowledged { handler_name, path } => format!(
                "SEC-004 `{handler_name}` sandbox path `{path}` is not acknowledged in [sandbox].allow_paths"
            ),
            Self::PluginsDirPermissive { path, mode } => {
                format!("SEC-006 plugin directory `{path}` has permissive mode {mode:o}")
            }
            Self::PluginWorldWritable {
                handler_name,
                mode,
                path,
            } => format!("SEC-005 plugin `{handler_name}` is world-writable ({mode:o}) at {path}"),
            Self::PluginWrongOwner { handler_name, path } => {
                format!("SEC-005 plugin `{handler_name}` is not owned by current user at {path}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationAuditRule {
    DirExists,
    FileExists,
}

impl ValidationAuditRule {
    fn as_str(self) -> &'static str {
        match self {
            Self::DirExists => "dir_exists",
            Self::FileExists => "file_exists",
        }
    }
}

#[derive(Debug, Default)]
pub struct AuditReport {
    errors: Vec<AuditDiagnostic>,
    warnings: Vec<AuditDiagnostic>,
    install_summary: Vec<AuditDiagnostic>,
}

impl AuditReport {
    pub fn has_errors(&self) -> bool {
        !self.errors().is_empty()
    }

    pub fn errors(&self) -> &[AuditDiagnostic] {
        &self.errors
    }

    pub fn warnings(&self) -> &[AuditDiagnostic] {
        &self.warnings
    }

    pub fn install_summary(&self) -> &[AuditDiagnostic] {
        &self.install_summary
    }

    pub fn push_error(&mut self, diagnostic: AuditDiagnostic) {
        self.errors.push(diagnostic);
    }

    pub fn push_warning(&mut self, diagnostic: AuditDiagnostic) {
        self.warnings.push(diagnostic);
    }

    pub fn push_install_summary(&mut self, diagnostic: AuditDiagnostic) {
        self.install_summary.push(diagnostic);
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
                report.push_error(AuditDiagnostic::UnresolvedHandler {
                    handler_name: handler_name.to_string(),
                });
                continue;
            }

            warn_on_plugin_integrity(handler_name, &plugin_path, &mut report);

            let manifest = match sc_hooks_sdk::manifest::load_manifest_from_executable(&plugin_path)
            {
                Ok(manifest) => manifest,
                Err(sc_hooks_sdk::manifest::ManifestLoadError::Manifest(
                    sc_hooks_sdk::manifest::ManifestError::AsyncLongRunningUnsupported,
                )) => {
                    report.push_error(AuditDiagnostic::AsyncLongRunningUnsupported {
                        handler_name: handler_name.to_string(),
                    });
                    continue;
                }
                Err(sc_hooks_sdk::manifest::ManifestLoadError::Manifest(
                    sc_hooks_sdk::manifest::ManifestError::MissingLongRunningDescription,
                )) => {
                    report.push_error(AuditDiagnostic::MissingLongRunningDescription {
                        handler_name: handler_name.to_string(),
                    });
                    continue;
                }
                Err(err) => {
                    report.push_error(AuditDiagnostic::ManifestLoadFailed {
                        handler_name: handler_name.to_string(),
                        error: err.to_string(),
                    });
                    continue;
                }
            };

            if !manifest.hooks.iter().any(|hook| hook == hook_name) {
                report.push_error(AuditDiagnostic::HookNotDeclared {
                    handler_name: handler_name.to_string(),
                    hook_name: hook_name.to_string(),
                });
            }
            let taxonomy = events::validate_matchers_for_hook(hook_name, &manifest.matchers);
            report.warnings.extend(
                taxonomy
                    .warnings
                    .into_iter()
                    .map(|warning| AuditDiagnostic::MatcherWarning { message: warning }),
            );
            report.errors.extend(
                taxonomy
                    .errors
                    .into_iter()
                    .map(|error| AuditDiagnostic::MatcherError { message: error }),
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
                    report.push_error(AuditDiagnostic::MissingRequiredMetadataField {
                        handler_name: handler_name.to_string(),
                        field: field.to_string(),
                    });
                    continue;
                };

                if let Some(rule) = requirement.validate.as_ref()
                    && let Some((rule, _)) = sc_hooks_core::validation::parse_validation_rule(rule)
                {
                    match rule {
                        sc_hooks_core::validation::ValidationRule::DirExists => {
                            if value.as_str().is_none_or(|path| !Path::new(path).is_dir()) {
                                report.push_error(AuditDiagnostic::ValidationRuleFailed {
                                    handler_name: handler_name.to_string(),
                                    field: field.to_string(),
                                    rule: ValidationAuditRule::DirExists,
                                });
                            }
                        }
                        sc_hooks_core::validation::ValidationRule::FileExists => {
                            if value.as_str().is_none_or(|path| !Path::new(path).is_file()) {
                                report.push_error(AuditDiagnostic::ValidationRuleFailed {
                                    handler_name: handler_name.to_string(),
                                    field: field.to_string(),
                                    rule: ValidationAuditRule::FileExists,
                                });
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
                    .map(|warning| AuditDiagnostic::InstallPlanWarning { message: warning }),
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

                    report.push_install_summary(AuditDiagnostic::InstallPlanEntry {
                        hook: hook.to_string(),
                        matcher: entry.matcher.clone(),
                        has_sync,
                        async_buckets: async_buckets.into_iter().collect(),
                    });
                }
            }
        }
        Err(err) => {
            report.push_error(AuditDiagnostic::InstallPlanGenerationFailed {
                error: err.to_string(),
            });
        }
    }

    Ok(report)
}

pub fn render(report: &AuditReport) -> String {
    let mut lines = Vec::new();
    lines.push("Audit report".to_string());
    if report.errors().is_empty() {
        lines.push("errors: 0".to_string());
    } else {
        lines.push(format!("errors: {}", report.errors().len()));
        lines.extend(
            report
                .errors()
                .iter()
                .map(|error| format!("- {}", error.render())),
        );
    }

    if report.warnings().is_empty() {
        lines.push("warnings: 0".to_string());
    } else {
        lines.push(format!("warnings: {}", report.warnings().len()));
        lines.extend(
            report
                .warnings()
                .iter()
                .map(|warning| format!("- {}", warning.render())),
        );
    }

    if !report.install_summary().is_empty() {
        lines.push("install plan:".to_string());
        lines.extend(
            report
                .install_summary()
                .iter()
                .map(|entry| format!("- {}", entry.render())),
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
            AuditDiagnostic::SandboxNeedsNetworkAck {
                handler_name: handler_name.to_string(),
            },
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
            report.push_error(AuditDiagnostic::SandboxPathMissing {
                handler_name: handler_name.to_string(),
                path: path.clone(),
            });
        }

        if !allowed_paths.iter().any(|allowed| allowed == path) {
            push_sandbox_exceeded(
                report,
                strict,
                AuditDiagnostic::SandboxPathUnacknowledged {
                    handler_name: handler_name.to_string(),
                    path: path.clone(),
                },
            );
        }
    }
}

fn push_sandbox_exceeded(report: &mut AuditReport, strict: bool, diagnostic: AuditDiagnostic) {
    if strict {
        report.push_error(diagnostic);
    } else {
        report.push_warning(diagnostic);
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
                report.push_warning(AuditDiagnostic::PluginsDirPermissive {
                    path: plugin_dir.display().to_string(),
                    mode,
                });
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
                report.push_warning(AuditDiagnostic::PluginWorldWritable {
                    handler_name: handler_name.to_string(),
                    mode,
                    path: plugin_path.display().to_string(),
                });
            }

            // SAFETY: `geteuid` has no preconditions and returns the current effective UID.
            let effective_uid = unsafe { nix::libc::geteuid() };
            if metadata.uid() != effective_uid {
                report.push_warning(AuditDiagnostic::PluginWrongOwner {
                    handler_name: handler_name.to_string(),
                    path: plugin_path.display().to_string(),
                });
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
    use std::io::Write;

    fn make_plugin(path: &Path, manifest: &str) {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).expect("plugin parent directory should be creatable");

        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--manifest\" ]; then\n  cat <<'JSON'\n{manifest}\nJSON\n  exit 0\nfi\ncat >/dev/null\ncat <<'JSON'\n{{\"action\":\"proceed\"}}\nJSON\n"
        );
        let mut temp =
            tempfile::NamedTempFile::new_in(parent).expect("temporary plugin file should create");
        temp.write_all(script.as_bytes())
            .expect("plugin script should be writable");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = temp
                .as_file()
                .metadata()
                .expect("plugin metadata should be available")
                .permissions();
            perms.set_mode(0o755);
            temp.as_file()
                .set_permissions(perms)
                .expect("plugin should be made executable");
        }

        temp.as_file()
            .sync_all()
            .expect("plugin script should sync before persist");
        temp.into_temp_path()
            .persist(path)
            .expect("plugin script should persist atomically");
    }

    #[test]
    fn audit_reports_missing_handler() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
        assert!(
            report
                .errors()
                .iter()
                .any(|entry| matches!(entry, AuditDiagnostic::UnresolvedHandler { .. }))
        );
    }

    #[test]
    fn audit_accepts_valid_plugin_manifest() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
    }

    #[test]
    fn strict_mode_turns_unacknowledged_sandbox_needs_into_errors() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
        assert!(report.errors().iter().any(|entry| {
            matches!(
                entry,
                AuditDiagnostic::SandboxNeedsNetworkAck { .. }
                    | AuditDiagnostic::SandboxPathUnacknowledged { .. }
            )
        }));
    }

    #[test]
    fn audit_rejects_async_long_running_manifest() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
                .errors()
                .iter()
                .any(|entry| matches!(entry, AuditDiagnostic::AsyncLongRunningUnsupported { .. }))
        );
    }

    #[test]
    fn audit_rejects_long_running_without_description() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        make_plugin(
            Path::new(".sc-hooks/plugins/notify"),
            r#"{"contract_version":1,"name":"notify","mode":"sync","hooks":["PostToolUse"],"matchers":["*"],"long_running":true,"description":"   ","requires":{}}"#,
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
            report.errors().iter().any(|entry| matches!(
                entry,
                AuditDiagnostic::MissingLongRunningDescription { .. }
            ))
        );
    }
}
