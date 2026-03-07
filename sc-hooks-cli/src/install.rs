use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::ScHooksConfig;
use crate::errors::CliError;
use crate::events;

const DEFAULT_SETTINGS_PATH: &str = ".claude/settings.json";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InstallSettings {
    pub hooks: BTreeMap<String, Vec<MatcherEntry>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MatcherEntry {
    pub matcher: String,
    pub hooks: Vec<CommandHook>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CommandHook {
    #[serde(rename = "type")]
    pub hook_type: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#async: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    pub settings: InstallSettings,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct HandlerInstallSpec {
    mode: sc_hooks_core::dispatch::DispatchMode,
    matchers: Vec<String>,
    async_range: (u64, u64),
}

pub fn write_default_settings(config: &ScHooksConfig) -> Result<InstallPlan, CliError> {
    let plan = build_settings(config)?;
    let path = Path::new(DEFAULT_SETTINGS_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            CliError::internal(format!(
                "failed to create settings directory {}: {err}",
                parent.display()
            ))
        })?;
    }

    let rendered = serde_json::to_string_pretty(&plan.settings)
        .map_err(|err| CliError::internal(format!("failed serializing settings.json: {err}")))?;
    fs::write(path, rendered).map_err(|err| {
        CliError::internal(format!(
            "failed writing settings file {}: {err}",
            path.display()
        ))
    })?;

    Ok(plan)
}

pub fn build_settings(config: &ScHooksConfig) -> Result<InstallPlan, CliError> {
    let mut hooks_output = BTreeMap::new();
    let mut warnings = Vec::new();

    for (hook_name, chain) in &config.hooks {
        let specs = collect_specs_for_hook(hook_name, chain, &mut warnings)?;
        let entries = build_matcher_entries(hook_name, &specs);
        if !entries.is_empty() {
            hooks_output.insert(hook_name.clone(), entries);
        }
    }

    Ok(InstallPlan {
        settings: InstallSettings {
            hooks: hooks_output,
        },
        warnings,
    })
}

fn collect_specs_for_hook(
    hook_name: &str,
    chain: &[String],
    warnings: &mut Vec<String>,
) -> Result<Vec<HandlerInstallSpec>, CliError> {
    let mut specs = Vec::new();
    let mut manifest_cache: BTreeMap<PathBuf, sc_hooks_core::manifest::Manifest> = BTreeMap::new();

    for handler_name in chain {
        if let Some(spec) = builtin_spec(handler_name) {
            specs.push(spec);
            continue;
        }

        let path = plugin_path(handler_name);
        let manifest = if let Some(cached) = manifest_cache.get(&path) {
            cached.clone()
        } else {
            let loaded =
                sc_hooks_sdk::manifest::load_manifest_from_executable(&path).map_err(|err| {
                    CliError::internal(format!(
                        "failed loading manifest for `{handler_name}`: {err}"
                    ))
                })?;
            manifest_cache.insert(path.clone(), loaded.clone());
            loaded
        };

        if !manifest.hooks.iter().any(|declared| declared == hook_name) {
            continue;
        }

        let validated = events::validate_matchers_for_hook(hook_name, &manifest.matchers);
        warnings.extend(validated.warnings);
        if !validated.errors.is_empty() {
            return Err(CliError::Validation(
                crate::errors::ValidationError::InvalidField {
                    handler: handler_name.clone(),
                    field: "matchers".to_string(),
                    reason: validated.errors.join("; "),
                },
            ));
        }

        specs.push(HandlerInstallSpec {
            mode: manifest.mode,
            matchers: manifest.matchers,
            async_range: async_range_for_response_time(manifest.response_time.as_ref()),
        });
    }

    Ok(specs)
}

fn build_matcher_entries(hook_name: &str, specs: &[HandlerInstallSpec]) -> Vec<MatcherEntry> {
    let mut explicit_matchers = BTreeSet::new();
    let has_wildcard_only = specs.iter().any(is_wildcard_only_spec);

    for spec in specs {
        for matcher in &spec.matchers {
            if matcher != "*" {
                explicit_matchers.insert(matcher.clone());
            }
        }
    }

    let mut all_matchers: Vec<String> = explicit_matchers.into_iter().collect();
    if has_wildcard_only {
        all_matchers.push("*".to_string());
    }

    let mut entries = Vec::new();
    for matcher in all_matchers {
        let sync_count = specs
            .iter()
            .filter(|spec| {
                spec.mode == sc_hooks_core::dispatch::DispatchMode::Sync && applies(spec, &matcher)
            })
            .count();

        let mut async_ranges = Vec::new();
        for spec in specs {
            if spec.mode == sc_hooks_core::dispatch::DispatchMode::Async && applies(spec, &matcher)
            {
                async_ranges.push(spec.async_range);
            }
        }

        let mut hooks = Vec::new();
        if sync_count > 0 {
            hooks.push(CommandHook {
                hook_type: "command".to_string(),
                command: build_run_command(hook_name, &matcher, false, None),
                r#async: None,
            });
        }

        for bucket in merged_async_buckets(&async_ranges) {
            hooks.push(CommandHook {
                hook_type: "command".to_string(),
                command: build_run_command(hook_name, &matcher, true, Some(&bucket)),
                r#async: Some(true),
            });
        }

        if !hooks.is_empty() {
            entries.push(MatcherEntry { matcher, hooks });
        }
    }

    entries
}

fn build_run_command(hook: &str, matcher: &str, is_async: bool, bucket: Option<&str>) -> String {
    let mut command = if matcher == "*" {
        format!("sc-hooks run {hook}")
    } else {
        format!("sc-hooks run {hook} {matcher}")
    };

    if is_async {
        command.push_str(" --async");
        if let Some(bucket) = bucket {
            command.push_str(" --async-bucket ");
            command.push_str(bucket);
        }
    } else {
        command.push_str(" --sync");
    }

    command
}

fn applies(spec: &HandlerInstallSpec, matcher: &str) -> bool {
    if matcher == "*" {
        return is_wildcard_only_spec(spec);
    }

    spec.matchers.iter().any(|declared| declared == "*")
        || spec.matchers.iter().any(|declared| declared == matcher)
}

fn async_range_for_response_time(
    response_time: Option<&sc_hooks_core::manifest::ResponseTimeRange>,
) -> (u64, u64) {
    match response_time {
        Some(range) => (range.min_ms, range.max_ms),
        None => (0, 30_000),
    }
}

fn merged_async_buckets(ranges: &[(u64, u64)]) -> Vec<String> {
    if ranges.is_empty() {
        return Vec::new();
    }

    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|(min, max)| (*min, *max));

    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (min, max) in sorted {
        if let Some((last_min, last_max)) = merged.last_mut()
            && min <= last_max.saturating_add(1)
        {
            *last_min = (*last_min).min(min);
            *last_max = (*last_max).max(max);
            continue;
        }
        merged.push((min, max));
    }

    merged
        .into_iter()
        .map(|(min, max)| format!("{min}-{max}"))
        .collect()
}

fn is_wildcard_only_spec(spec: &HandlerInstallSpec) -> bool {
    spec.matchers.len() == 1 && spec.matchers[0] == "*"
}

fn builtin_spec(handler_name: &str) -> Option<HandlerInstallSpec> {
    match handler_name {
        "log" => Some(HandlerInstallSpec {
            mode: sc_hooks_core::dispatch::DispatchMode::Sync,
            matchers: vec!["*".to_string()],
            async_range: (0, 30_000),
        }),
        _ => None,
    }
}

fn plugin_path(handler_name: &str) -> PathBuf {
    Path::new(".sc-hooks").join("plugins").join(handler_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::test_support;
    use std::path::Path;

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
    fn build_settings_splits_sync_async_and_buckets() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch");

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
        make_plugin(
            Path::new(".sc-hooks/plugins/collect-context"),
            r#"{
"contract_version":1,
"name":"collect-context",
"mode":"async",
"hooks":["PreToolUse"],
"matchers":["Write","Bash"],
"response_time":{"min_ms":10,"max_ms":100},
"requires":{}
}"#,
        );
        make_plugin(
            Path::new(".sc-hooks/plugins/notify"),
            r#"{
"contract_version":1,
"name":"notify",
"mode":"async",
"hooks":["PreToolUse"],
"matchers":["Write","Bash"],
"response_time":{"min_ms":1000,"max_ms":5000},
"requires":{}
}"#,
        );

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths", "collect-context", "notify", "log"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let plan = build_settings(&cfg).expect("install plan should build");
        let entries = plan
            .settings
            .hooks
            .get("PreToolUse")
            .expect("PreToolUse should exist");

        let write = entries
            .iter()
            .find(|entry| entry.matcher == "Write")
            .expect("Write matcher should exist");
        assert!(
            write
                .hooks
                .iter()
                .any(|hook| hook.command.contains("--sync"))
        );
        assert_eq!(
            write
                .hooks
                .iter()
                .filter(|hook| hook.command.contains("--async"))
                .count(),
            2
        );
        assert!(
            write
                .hooks
                .iter()
                .any(|hook| hook.command.contains("--async-bucket 10-100"))
        );
        assert!(
            write
                .hooks
                .iter()
                .any(|hook| hook.command.contains("--async-bucket 1000-5000"))
        );

        let wildcard = entries
            .iter()
            .find(|entry| entry.matcher == "*")
            .expect("wildcard entry should exist for log builtin");
        assert_eq!(wildcard.hooks.len(), 1);
        assert!(wildcard.hooks[0].command.contains("--sync"));

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn wildcard_entry_only_includes_wildcard_only_handlers() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch");

        make_plugin(
            Path::new(".sc-hooks/plugins/mixed"),
            r#"{
"contract_version":1,
"name":"mixed",
"mode":"sync",
"hooks":["PreToolUse"],
"matchers":["Write","*"],
"requires":{}
}"#,
        );

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["mixed"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let plan = build_settings(&cfg).expect("install plan should build");
        let entries = plan
            .settings
            .hooks
            .get("PreToolUse")
            .expect("PreToolUse should exist");
        assert!(entries.iter().any(|entry| entry.matcher == "Write"));
        assert!(!entries.iter().any(|entry| entry.matcher == "*"));

        std::env::set_current_dir(original).expect("cwd should restore");
    }

    #[test]
    fn overlaps_are_merged_into_single_async_bucket() {
        let _guard = test_support::cwd_lock()
            .lock()
            .expect("cwd lock should acquire");
        let temp = tempfile::tempdir().expect("tempdir should create");
        let original = std::env::current_dir().expect("cwd should resolve");
        std::env::set_current_dir(temp.path()).expect("cwd should switch");

        make_plugin(
            Path::new(".sc-hooks/plugins/a"),
            r#"{
"contract_version":1,
"name":"a",
"mode":"async",
"hooks":["PreToolUse"],
"matchers":["Write"],
"response_time":{"min_ms":10,"max_ms":100},
"requires":{}
}"#,
        );
        make_plugin(
            Path::new(".sc-hooks/plugins/b"),
            r#"{
"contract_version":1,
"name":"b",
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
PreToolUse = ["a", "b"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let plan = build_settings(&cfg).expect("install plan should build");
        let write = plan
            .settings
            .hooks
            .get("PreToolUse")
            .expect("PreToolUse should exist")
            .iter()
            .find(|entry| entry.matcher == "Write")
            .expect("Write matcher should exist");

        let async_commands: Vec<&CommandHook> = write
            .hooks
            .iter()
            .filter(|hook| hook.r#async == Some(true))
            .collect();
        assert_eq!(async_commands.len(), 1);
        assert!(async_commands[0].command.contains("--async-bucket 10-200"));

        std::env::set_current_dir(original).expect("cwd should restore");
    }
}
