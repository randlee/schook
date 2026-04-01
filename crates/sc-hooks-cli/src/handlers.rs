use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::CliError;
use crate::events;
use crate::timeout::resolve_timeout_ms;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginHandlerInfo {
    pub name: String,
    pub path: PathBuf,
    pub mode: Option<String>,
    pub matchers: Vec<String>,
    pub timeout: String,
    pub manifest_error_kind: Option<&'static str>,
    pub manifest_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HandlersReport {
    pub plugins: Vec<PluginHandlerInfo>,
}

pub fn discover() -> Result<HandlersReport, CliError> {
    let mut report = HandlersReport {
        plugins: discover_plugins()?,
    };
    report.plugins.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(report)
}

pub fn render(report: &HandlersReport) -> String {
    let mut lines = Vec::new();
    lines.push("Handlers".to_string());
    lines.push("plugins:".to_string());
    if report.plugins.is_empty() {
        lines.push("- none discovered".to_string());
        return lines.join("\n");
    }

    for plugin in &report.plugins {
        let mut line = format!(
            "- {name} path={path}",
            name = plugin.name,
            path = plugin.path.display()
        );
        if let Some(mode) = plugin.mode.as_ref() {
            line.push_str(&format!(
                " mode={mode} matchers={matchers} timeout={timeout}",
                matchers = plugin.matchers.join(","),
                timeout = plugin.timeout
            ));
        }
        if let Some(err) = plugin.manifest_error.as_ref() {
            let kind = plugin.manifest_error_kind.unwrap_or("unknown");
            line.push_str(&format!(" manifest_error_kind={kind} manifest_error={err}"));
        }
        lines.push(line);
    }

    lines.join("\n")
}

pub fn render_events() -> String {
    let mut lines = Vec::new();
    lines.push("Event taxonomy".to_string());
    for (hook, events) in events::canonical_taxonomy() {
        lines.push(format!("- {hook}: {}", events.join(",")));
    }
    lines.join("\n")
}

fn discover_plugins() -> Result<Vec<PluginHandlerInfo>, CliError> {
    let plugin_dir = Path::new(".sc-hooks/plugins");
    if !plugin_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(plugin_dir).map_err(|source| {
        CliError::internal_with_source(
            format!("failed reading plugins directory {}", plugin_dir.display()),
            source,
        )
    })?;

    let mut plugins = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| {
            CliError::internal_with_source(
                format!(
                    "failed reading plugin directory entry in {}",
                    plugin_dir.display()
                ),
                source,
            )
        })?;
        let path = entry.path();
        if !is_plugin_executable(&path) {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        match sc_hooks_sdk::manifest::load_manifest_from_executable(&path) {
            Ok(manifest) => {
                let resolved_timeout =
                    resolve_timeout_ms(manifest.mode, manifest.timeout_ms, manifest.long_running);
                let timeout = if resolved_timeout.is_none() {
                    "none(long-running)".to_string()
                } else if let Some(timeout_ms) = manifest.timeout_ms {
                    format!("{timeout_ms}ms")
                } else {
                    match manifest.mode {
                        sc_hooks_core::dispatch::DispatchMode::Sync => {
                            "5000ms(default)".to_string()
                        }
                        sc_hooks_core::dispatch::DispatchMode::Async => {
                            "30000ms(default)".to_string()
                        }
                    }
                };

                plugins.push(PluginHandlerInfo {
                    name,
                    path,
                    mode: Some(manifest.mode.as_str().to_string()),
                    matchers: manifest.matchers,
                    timeout,
                    manifest_error_kind: None,
                    manifest_error: None,
                });
            }
            Err(err) => {
                plugins.push(PluginHandlerInfo {
                    name,
                    path,
                    mode: None,
                    matchers: Vec::new(),
                    timeout: "unknown".to_string(),
                    manifest_error_kind: Some(manifest_error_kind(&err)),
                    manifest_error: Some(err.to_string()),
                });
            }
        }
    }

    Ok(plugins)
}

fn manifest_error_kind(err: &sc_hooks_sdk::manifest::ManifestLoadError) -> &'static str {
    match err {
        sc_hooks_sdk::manifest::ManifestLoadError::Spawn { .. } => "spawn",
        sc_hooks_sdk::manifest::ManifestLoadError::NonZeroExit { .. } => "non_zero",
        sc_hooks_sdk::manifest::ManifestLoadError::TerminatedBySignal { .. } => "signal",
        sc_hooks_sdk::manifest::ManifestLoadError::Terminated { .. } => "terminated",
        sc_hooks_sdk::manifest::ManifestLoadError::Manifest(_) => "manifest",
    }
}

fn is_plugin_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            return metadata.permissions().mode() & 0o111 != 0;
        }
        false
    }

    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            fs::set_permissions(path, perms).expect("plugin should be executable");
        }
    }

    #[test]
    fn discovers_plugins() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        make_plugin(
            Path::new(".sc-hooks/plugins/guard-paths"),
            r#"{"contract_version":1,"name":"guard-paths","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        );

        let report = discover().expect("handler discovery should succeed");
        let rendered = render(&report);
        assert!(rendered.contains("plugins:"));
        assert!(rendered.contains("guard-paths"));
        assert!(rendered.contains("matchers=Write"));
    }

    #[test]
    fn async_long_running_manifest_surfaces_as_manifest_error() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        make_plugin(
            Path::new(".sc-hooks/plugins/notify"),
            r#"{"contract_version":1,"name":"notify","mode":"async","hooks":["PostToolUse"],"matchers":["*"],"long_running":true,"description":"wait for remote ack","requires":{}}"#,
        );

        let report = discover().expect("handler discovery should succeed");
        let notify = report
            .plugins
            .iter()
            .find(|plugin| plugin.name == "notify")
            .expect("notify plugin should be present");
        assert!(notify.manifest_error.is_some());
        assert_eq!(notify.timeout, "unknown");
    }

    #[test]
    fn renders_event_taxonomy() {
        let rendered = render_events();
        assert!(rendered.contains("Event taxonomy"));
        assert!(rendered.contains("PreToolUse:"));
        assert!(rendered.contains("Notification:"));
        assert!(rendered.contains("idle_prompt"));
    }
}
