use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::CliError;
use crate::events;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinHandlerInfo {
    pub name: String,
    pub mode: String,
    pub hooks: Vec<String>,
    pub matchers: Vec<String>,
    pub timeout: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginHandlerInfo {
    pub name: String,
    pub path: PathBuf,
    pub mode: Option<String>,
    pub matchers: Vec<String>,
    pub timeout: String,
    pub manifest_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HandlersReport {
    pub builtins: Vec<BuiltinHandlerInfo>,
    pub plugins: Vec<PluginHandlerInfo>,
}

pub fn discover() -> Result<HandlersReport, CliError> {
    let mut report = HandlersReport {
        builtins: builtin_handlers(),
        plugins: discover_plugins()?,
    };
    report.plugins.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(report)
}

pub fn render(report: &HandlersReport) -> String {
    let mut lines = Vec::new();
    lines.push("Handlers".to_string());
    lines.push("builtins:".to_string());
    for builtin in &report.builtins {
        lines.push(format!(
            "- {name} mode={mode} hooks={hooks} matchers={matchers} timeout={timeout}",
            name = builtin.name,
            mode = builtin.mode,
            hooks = builtin.hooks.join(","),
            matchers = builtin.matchers.join(","),
            timeout = builtin.timeout,
        ));
    }

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
            line.push_str(&format!(" manifest_error={err}"));
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

fn builtin_handlers() -> Vec<BuiltinHandlerInfo> {
    vec![BuiltinHandlerInfo {
        name: "log".to_string(),
        mode: "sync".to_string(),
        hooks: vec!["*".to_string()],
        matchers: vec!["*".to_string()],
        timeout: "n/a".to_string(),
    }]
}

fn discover_plugins() -> Result<Vec<PluginHandlerInfo>, CliError> {
    let plugin_dir = Path::new(".sc-hooks/plugins");
    if !plugin_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(plugin_dir).map_err(|err| {
        CliError::internal(format!(
            "failed reading plugins directory {}: {err}",
            plugin_dir.display()
        ))
    })?;

    let mut plugins = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| {
            CliError::internal(format!(
                "failed reading plugin directory entry in {}: {err}",
                plugin_dir.display()
            ))
        })?;
        let path = entry.path();
        if !is_plugin_executable(&path) {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        match sc_hooks_sdk::manifest::load_manifest_from_executable(&path) {
            Ok(manifest) => {
                let timeout = if manifest.long_running && manifest.timeout_ms.is_none() {
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
                    manifest_error: Some(err.to_string()),
                });
            }
        }
    }

    Ok(plugins)
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
    fn discovers_builtin_and_plugin_handlers() {
        let _guard = test_support::cwd_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch to temp");

        make_plugin(
            Path::new(".sc-hooks/plugins/guard-paths"),
            r#"{"contract_version":1,"name":"guard-paths","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
        );

        let report = discover().expect("handler discovery should succeed");
        let rendered = render(&report);
        assert!(rendered.contains("builtins:"));
        assert!(rendered.contains("- log mode=sync"));
        assert!(rendered.contains("plugins:"));
        assert!(rendered.contains("guard-paths"));
        assert!(rendered.contains("matchers=Write"));

        std::env::set_current_dir(original).expect("cwd should restore");
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
