use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::ScHooksConfig;
use crate::errors::ResolutionError;

#[derive(Debug)]
pub enum HandlerTarget {
    Builtin(BuiltinHandler),
    Plugin(Box<PluginHandler>),
}

#[derive(Debug)]
pub enum BuiltinHandler {
    Log,
}

#[derive(Debug)]
pub struct PluginHandler {
    pub executable_path: PathBuf,
    pub manifest: sc_hooks_core::manifest::Manifest,
}

#[derive(Debug)]
pub struct ResolvedHandler {
    pub name: String,
    pub target: HandlerTarget,
}

pub fn resolve_chain(
    config: &ScHooksConfig,
    hook: &str,
    event: Option<&str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
    payload: Option<&Value>,
) -> Result<Vec<ResolvedHandler>, ResolutionError> {
    let Some(chain) = config.hooks.get(hook) else {
        return Ok(Vec::new());
    };

    let mut manifest_cache: HashMap<PathBuf, sc_hooks_core::manifest::Manifest> = HashMap::new();
    let mut resolved = Vec::new();

    for handler_name in chain {
        if let Some(builtin) = resolve_builtin(handler_name, mode) {
            resolved.push(ResolvedHandler {
                name: handler_name.clone(),
                target: HandlerTarget::Builtin(builtin),
            });
            continue;
        }

        let executable = plugin_path(handler_name);
        if !executable.exists() {
            return Err(ResolutionError::UnresolvedHandler {
                handler: handler_name.clone(),
            });
        }

        let manifest = if let Some(cached) = manifest_cache.get(&executable) {
            cached.clone()
        } else {
            let loaded = sc_hooks_sdk::manifest::load_manifest_from_executable(&executable)
                .map_err(|err| ResolutionError::ManifestLoad {
                    plugin: handler_name.clone(),
                    reason: err.to_string(),
                })?;
            manifest_cache.insert(executable.clone(), loaded.clone());
            loaded
        };

        if manifest.mode != mode {
            continue;
        }

        if !manifest.hooks.iter().any(|declared| declared == hook) {
            continue;
        }

        if !matches_event(&manifest.matchers, event) {
            continue;
        }

        let payload_matches = sc_hooks_sdk::conditions::evaluate_payload_conditions(
            &manifest.payload_conditions,
            payload,
        )
        .map_err(|err| ResolutionError::ManifestLoad {
            plugin: handler_name.clone(),
            reason: err.to_string(),
        })?;

        if !payload_matches {
            continue;
        }

        resolved.push(ResolvedHandler {
            name: handler_name.clone(),
            target: HandlerTarget::Plugin(Box::new(PluginHandler {
                executable_path: executable,
                manifest,
            })),
        });
    }

    Ok(resolved)
}

fn resolve_builtin(
    handler_name: &str,
    mode: sc_hooks_core::dispatch::DispatchMode,
) -> Option<BuiltinHandler> {
    match (handler_name, mode) {
        ("log", sc_hooks_core::dispatch::DispatchMode::Sync) => Some(BuiltinHandler::Log),
        _ => None,
    }
}

fn plugin_path(handler_name: &str) -> PathBuf {
    Path::new(".sc-hooks").join("plugins").join(handler_name)
}

fn matches_event(matchers: &[String], event: Option<&str>) -> bool {
    if matchers.iter().any(|matcher| matcher == "*") {
        return true;
    }

    match event {
        Some(event) => matchers.iter().any(|matcher| matcher == event),
        None => false,
    }
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
    fn wildcard_matcher_matches_any_event() {
        assert!(matches_event(&["*".to_string()], Some("Write")));
        assert!(matches_event(&["*".to_string()], None));
    }

    #[test]
    fn explicit_matchers_require_event() {
        assert!(matches_event(&["Write".to_string()], Some("Write")));
        assert!(!matches_event(&["Write".to_string()], Some("Read")));
        assert!(!matches_event(&["Write".to_string()], None));
    }

    #[test]
    fn builtin_is_preferred_over_same_named_plugin() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("current_dir should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch to temp");

        make_plugin(
            Path::new(".sc-hooks/plugins/log"),
            r#"{"contract_version":1,"name":"log","mode":"sync","hooks":["PreToolUse"],"matchers":["*"],"requires":{}}"#,
        );

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["log"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let handlers = resolve_chain(
            &cfg,
            "PreToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Sync,
            None,
        )
        .expect("resolution should succeed");

        assert_eq!(handlers.len(), 1);
        assert!(matches!(
            handlers[0].target,
            HandlerTarget::Builtin(BuiltinHandler::Log)
        ));

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn resolves_external_plugin_for_matching_event() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("current_dir should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch to temp");

        make_plugin(
            Path::new(".sc-hooks/plugins/guard-paths"),
            r#"{
"contract_version":1,
"name":"guard-paths",
"mode":"sync",
"hooks":["PreToolUse"],
"matchers":["Write"],
"requires":{},
"payload_conditions":[{"path":"tool_input.command","op":"contains","value":"atm"}]
}"#,
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

        let payload = serde_json::json!({"tool_input":{"command":"atm send"}});
        let handlers = resolve_chain(
            &cfg,
            "PreToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Sync,
            Some(&payload),
        )
        .expect("resolution should succeed");

        assert_eq!(handlers.len(), 1);
        assert!(matches!(handlers[0].target, HandlerTarget::Plugin(_)));

        std::env::set_current_dir(original).expect("cwd should restore");
    }
}
