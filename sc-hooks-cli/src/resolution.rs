use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::ScHooksConfig;
use crate::errors::ResolutionError;
use crate::events;

#[derive(Debug)]
pub struct ResolvedHandler {
    pub name: String,
    pub executable_path: PathBuf,
    pub manifest: sc_hooks_core::manifest::Manifest,
}

pub fn resolve_chain(
    config: &ScHooksConfig,
    hook: &str,
    event: Option<&str>,
    mode: sc_hooks_core::dispatch::DispatchMode,
    payload: Option<&Value>,
    async_bucket: Option<&str>,
    disabled_plugins: &BTreeSet<String>,
) -> Result<Vec<ResolvedHandler>, ResolutionError> {
    let Some(chain) = config.hooks.get(hook) else {
        return Ok(Vec::new());
    };

    let mut manifest_cache: HashMap<PathBuf, sc_hooks_core::manifest::Manifest> = HashMap::new();
    let mut resolved = Vec::new();

    for handler_name in chain {
        if disabled_plugins.contains(handler_name) {
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
                .map_err(|source| ResolutionError::ManifestLoad {
                    plugin: handler_name.clone(),
                    source,
                })?;
            manifest_cache.insert(executable.clone(), loaded.clone());
            loaded
        };

        if manifest.mode != mode {
            continue;
        }

        if mode == sc_hooks_core::dispatch::DispatchMode::Async
            && let Some(bucket) = async_bucket
            && !bucket_matches_manifest(bucket, manifest.response_time.as_ref())
        {
            continue;
        }

        if !manifest.hooks.iter().any(|declared| declared == hook) {
            continue;
        }

        let taxonomy = events::validate_matchers_for_hook(hook, &manifest.matchers);
        if !taxonomy.errors.is_empty() {
            return Err(ResolutionError::HandlerRejected {
                plugin: handler_name.clone(),
                reason: taxonomy.errors.join("; "),
            });
        }

        if !matches_event(&manifest.matchers, event) {
            continue;
        }

        let payload_matches = sc_hooks_sdk::conditions::evaluate_payload_conditions(
            &manifest.payload_conditions,
            payload,
        )
        .map_err(|err| ResolutionError::HandlerRejected {
            plugin: handler_name.clone(),
            reason: err.to_string(),
        })?;

        if !payload_matches {
            continue;
        }

        resolved.push(ResolvedHandler {
            name: handler_name.clone(),
            executable_path: executable,
            manifest,
        });
    }

    Ok(resolved)
}

fn response_time_range(
    response_time: Option<&sc_hooks_core::manifest::ResponseTimeRange>,
) -> (u64, u64) {
    match response_time {
        Some(range) => (range.min_ms, range.max_ms),
        None => (0, 30_000),
    }
}

fn bucket_matches_manifest(
    requested_bucket: &str,
    response_time: Option<&sc_hooks_core::manifest::ResponseTimeRange>,
) -> bool {
    let Some((bucket_min, bucket_max)) = parse_bucket_range(requested_bucket) else {
        return false;
    };
    let (plugin_min, plugin_max) = response_time_range(response_time);
    bucket_min <= plugin_max && plugin_min <= bucket_max.saturating_add(1)
}

fn parse_bucket_range(bucket: &str) -> Option<(u64, u64)> {
    let (min, max) = bucket.split_once('-')?;
    let min = min.parse::<u64>().ok()?;
    let max = max.parse::<u64>().ok()?;
    Some((min, max))
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

    fn make_counting_manifest_plugin(path: &Path, manifest: &str, counter_path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("plugin parent directory should be creatable");
        }
        if let Some(parent) = counter_path.parent() {
            fs::create_dir_all(parent).expect("counter parent directory should be creatable");
        }

        let script = format!(
            "#!/bin/sh\nCOUNT_FILE=\"{counter}\"\nif [ \"$1\" = \"--manifest\" ]; then\n  count=0\n  if [ -f \"$COUNT_FILE\" ]; then\n    count=$(cat \"$COUNT_FILE\")\n  fi\n  count=$((count + 1))\n  echo \"$count\" > \"$COUNT_FILE\"\n  cat <<'JSON'\n{manifest}\nJSON\n  exit 0\nfi\ncat >/dev/null\ncat <<'JSON'\n{{\"action\":\"proceed\"}}\nJSON\n",
            counter = counter_path.display()
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
    fn resolves_external_plugin_for_matching_event() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
            None,
            &BTreeSet::new(),
        )
        .expect("resolution should succeed");

        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].name, "guard-paths");
    }

    #[test]
    fn async_bucket_filter_selects_matching_plugins() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        make_plugin(
            Path::new(".sc-hooks/plugins/notify"),
            r#"{
"contract_version":1,
"name":"notify",
"mode":"async",
"hooks":["PreToolUse"],
"matchers":["Write"],
"response_time":{"min_ms":1000,"max_ms":5000},
"requires":{}
}"#,
        );

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["notify"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let mismatched = resolve_chain(
            &cfg,
            "PreToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Async,
            None,
            Some("10-100"),
            &BTreeSet::new(),
        )
        .expect("resolution should succeed");
        assert!(mismatched.is_empty());

        let matched = resolve_chain(
            &cfg,
            "PreToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Async,
            None,
            Some("1000-5000"),
            &BTreeSet::new(),
        )
        .expect("resolution should succeed");
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn merged_async_bucket_matches_overlapping_plugin_range() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        make_plugin(
            Path::new(".sc-hooks/plugins/context-a"),
            r#"{
"contract_version":1,
"name":"context-a",
"mode":"async",
"hooks":["PreToolUse"],
"matchers":["Write"],
"response_time":{"min_ms":50,"max_ms":200},
"requires":{}
}"#,
        );

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["context-a"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let handlers = resolve_chain(
            &cfg,
            "PreToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Async,
            None,
            Some("10-200"),
            &BTreeSet::new(),
        )
        .expect("resolution should succeed");
        assert_eq!(handlers.len(), 1);
    }

    #[test]
    fn disabled_plugins_are_skipped() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        make_plugin(
            Path::new(".sc-hooks/plugins/guard-paths"),
            r#"{
"contract_version":1,
"name":"guard-paths",
"mode":"sync",
"hooks":["PreToolUse"],
"matchers":["Write"],
"requires":{}
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

        let mut disabled = BTreeSet::new();
        disabled.insert("guard-paths".to_string());
        let handlers = resolve_chain(
            &cfg,
            "PreToolUse",
            Some("Write"),
            sc_hooks_core::dispatch::DispatchMode::Sync,
            None,
            None,
            &disabled,
        )
        .expect("resolution should succeed");
        assert!(handlers.is_empty());
    }

    #[test]
    fn caches_manifest_loads_per_invocation() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        let counter = Path::new(".sc-hooks/state/manifest-count.txt");
        make_counting_manifest_plugin(
            Path::new(".sc-hooks/plugins/cached"),
            r#"{"contract_version":1,"name":"cached","mode":"sync","hooks":["PreToolUse"],"matchers":["Write"],"requires":{}}"#,
            counter,
        );

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["cached", "cached"]
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
            None,
            &BTreeSet::new(),
        )
        .expect("resolution should succeed");
        assert_eq!(handlers.len(), 2);

        let counter_value =
            fs::read_to_string(counter).expect("manifest counter file should be created");
        assert_eq!(counter_value.trim(), "1");
    }
}
