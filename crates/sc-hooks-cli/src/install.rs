use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Serialize;

use crate::async_bucket::AsyncBucketRange;
use crate::config::ScHooksConfig;
use crate::errors::CliError;
use crate::events;
use sc_hooks_core::events::HookType;
use sc_hooks_core::manifest::ManifestMatcher;

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
    matchers: Vec<ManifestMatcher>,
    async_range: AsyncBucketRange,
}

pub fn write_default_settings(config: &ScHooksConfig) -> Result<InstallPlan, CliError> {
    let plan = build_settings(config)?;
    let path = Path::new(DEFAULT_SETTINGS_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CliError::internal_with_source(
                format!("failed to create settings directory {}", parent.display()),
                source,
            )
        })?;
    }

    let rendered = serde_json::to_string_pretty(&plan.settings).map_err(|source| {
        CliError::internal_with_source("failed serializing settings.json", source)
    })?;
    fs::write(path, rendered).map_err(|source| {
        CliError::internal_with_source(
            format!("failed writing settings file {}", path.display()),
            source,
        )
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
    let hook = HookType::from_str(hook_name).map_err(|_| {
        CliError::internal(format!(
            "unknown hook type `{hook_name}` in install settings build"
        ))
    })?;
    let mut specs = Vec::new();
    let mut manifest_cache: BTreeMap<PathBuf, sc_hooks_core::manifest::Manifest> = BTreeMap::new();

    for handler_name in chain {
        let path = plugin_path(handler_name);
        let manifest = if let Some(cached) = manifest_cache.get(&path) {
            cached.clone()
        } else {
            let loaded =
                sc_hooks_sdk::manifest::load_manifest_from_executable(&path).map_err(|source| {
                    CliError::internal_with_source(
                        format!("failed loading manifest for `{handler_name}`"),
                        source,
                    )
                })?;
            manifest_cache.insert(path.clone(), loaded.clone());
            loaded
        };

        if !manifest.hooks.contains(&hook) {
            continue;
        }

        let validated = events::validate_matchers_for_hook(hook, &manifest.matchers);
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
            async_range: AsyncBucketRange::from_response_time(manifest.response_time.as_ref()),
        });
    }

    Ok(specs)
}

fn build_matcher_entries(hook_name: &str, specs: &[HandlerInstallSpec]) -> Vec<MatcherEntry> {
    let mut explicit_matchers = BTreeSet::new();
    let has_wildcard_only = specs.iter().any(is_wildcard_only_spec);

    for spec in specs {
        for matcher in &spec.matchers {
            if matcher.as_str() != "*" {
                explicit_matchers.insert(matcher.as_str().to_string());
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

    spec.matchers
        .iter()
        .any(|declared| declared.as_str() == "*")
        || spec
            .matchers
            .iter()
            .any(|declared| declared.as_str() == matcher)
}

fn merged_async_buckets(ranges: &[AsyncBucketRange]) -> Vec<String> {
    if ranges.is_empty() {
        return Vec::new();
    }

    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|range| (range.min_ms, range.max_ms));

    let mut merged: Vec<AsyncBucketRange> = Vec::new();
    for range in sorted {
        if let Some(last) = merged.last_mut()
            && range.min_ms <= last.max_ms.saturating_add(1)
        {
            last.min_ms = last.min_ms.min(range.min_ms);
            last.max_ms = last.max_ms.max(range.max_ms);
            continue;
        }
        merged.push(range);
    }

    merged
        .into_iter()
        .map(AsyncBucketRange::as_bucket)
        .collect()
}

fn is_wildcard_only_spec(spec: &HandlerInstallSpec) -> bool {
    spec.matchers.len() == 1 && spec.matchers[0].as_str() == "*"
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
PreToolUse = ["guard-paths", "collect-context", "notify"]
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
    }

    #[test]
    fn wildcard_entry_only_includes_wildcard_only_handlers() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
    }

    #[test]
    fn overlaps_are_merged_into_single_async_bucket() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

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
    }
}
