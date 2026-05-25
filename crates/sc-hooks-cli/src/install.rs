use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::async_bucket::AsyncBucketRange;
use crate::config::{MetaConfig, ObservabilityConfig, SandboxConfig, ScHooksConfig};
use crate::errors::CliError;
use crate::events;
use sc_hooks_core::events::HookType;
use sc_hooks_core::manifest::ManifestMatcher;

const DEFAULT_SETTINGS_PATH: &str = ".claude/settings.json";
const LOCAL_RUNTIME_RELATIVE_ROOT: &str = ".local/share/sc-hooks/runtime-layout";
const LOCAL_RUNTIME_CONFIG_PATH: &str = ".sc-hooks/config.toml";
const LOCAL_RUNTIME_PLUGIN_DIR: &str = ".sc-hooks/plugins";
const LOCAL_STATE_RELATIVE_ROOT: &str = ".sc-hooks/state";
const LOCAL_CLAUDE_SETTINGS_PATH: &str = ".claude/settings.json";
const LOCAL_CODEX_SETTINGS_PATH: &str = ".codex/hooks.json";
const LOCAL_GEMINI_SETTINGS_PATH: &str = ".gemini/settings.json";
const LOCAL_BIN_RELATIVE_ROOT: &str = ".local/bin";
const BACKUP_SUFFIX: &str = ".sc-hooks.bak";
const HOOKS_ALIAS_NAME: &str = "hooks";
const DEFAULT_ATM_TEAM: &str = "schook";
const DEFAULT_ATM_IDENTITY: &str = "chook";
const LOCAL_RUNTIME_CONFIG: &str = r#"[meta]
version = 1

[hooks]
SessionStart = ["agent-session-foundation"]
SessionEnd = ["agent-session-foundation"]
PreToolUse = ["agent-spawn-gates", "atm-extension"]
PostToolUse = ["tool-output-gates"]
"#;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct InstallSettings {
    pub hooks: BTreeMap<String, Vec<MatcherEntry>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct MatcherEntry {
    pub matcher: String,
    pub hooks: Vec<CommandHook>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CommandHook {
    #[serde(rename = "type")]
    pub hook_type: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#async: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstallPlan {
    pub settings: InstallSettings,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetProvider {
    Claude,
    Codex,
    Gemini,
}

impl TargetProvider {
    pub(crate) const fn all() -> [Self; 3] {
        [Self::Claude, Self::Codex, Self::Gemini]
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum InstallError {
    #[allow(
        dead_code,
        reason = "Phase O sprint O.7 requires the internal unsupported-provider contract even though target selection is enum-backed today."
    )]
    #[error(
        "unsupported provider `{provider}`; supported providers: {supported:?}",
        provider = provider.as_str()
    )]
    UnsupportedProvider {
        provider: TargetProvider,
        supported: &'static [&'static str],
    },
    #[error(
        "missing provider config for `{provider}` at {path}",
        provider = provider.as_str(),
        path = path.display()
    )]
    MissingProviderConfig {
        provider: TargetProvider,
        path: PathBuf,
    },
    #[error("write failed at {path}: {reason}", path = path.display())]
    WriteFailed { path: PathBuf, reason: String },
    #[error(
        "rollback plan failed for `{provider}`{path_suffix}: {reason}",
        provider = provider.as_str(),
        path_suffix = rollback_path_suffix(path.as_ref())
    )]
    RollbackPlanFailed {
        provider: TargetProvider,
        path: Option<PathBuf>,
        reason: String,
    },
}

#[derive(Debug, Clone)]
struct HandlerInstallSpec {
    mode: sc_hooks_core::dispatch::DispatchMode,
    matchers: Vec<ManifestMatcher>,
    async_range: AsyncBucketRange,
}

pub(crate) fn write_default_settings(config: &ScHooksConfig) -> Result<InstallPlan, CliError> {
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

pub(crate) fn write_local_provider_cutover(
    provider: TargetProvider,
) -> Result<InstallPlan, InstallError> {
    let home = std::env::var_os("ATM_HOME")
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .ok_or_else(|| InstallError::WriteFailed {
            path: PathBuf::from("~"),
            reason: "unable to resolve home directory".to_string(),
        })?;
    let runtime_root = home.join(LOCAL_RUNTIME_RELATIVE_ROOT);
    let runtime_plugin_root = runtime_root.join(LOCAL_RUNTIME_PLUGIN_DIR);
    let runtime_config_path = runtime_root.join(LOCAL_RUNTIME_CONFIG_PATH);
    let state_root = home.join(LOCAL_STATE_RELATIVE_ROOT);
    let bin_root = home.join(LOCAL_BIN_RELATIVE_ROOT);
    let cli_binary = resolve_required_binary(&bin_root, "sc-hooks")?;
    ensure_runtime_layout(
        &runtime_root,
        &runtime_plugin_root,
        &runtime_config_path,
        &bin_root,
    )?;
    ensure_cli_alias(&bin_root, &cli_binary)?;

    match provider {
        TargetProvider::Claude => write_local_claude_cutover(
            &home,
            &runtime_root,
            &runtime_plugin_root,
            &state_root,
            &cli_binary,
        ),
        TargetProvider::Codex => {
            write_local_codex_cutover(&home, &runtime_root, &state_root, &cli_binary)
        }
        TargetProvider::Gemini => {
            write_local_gemini_cutover(&home, &runtime_root, &state_root, &cli_binary)
        }
    }
}

pub(crate) fn build_settings(config: &ScHooksConfig) -> Result<InstallPlan, CliError> {
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

fn plugin_path_in(root: &Path, handler_name: &str) -> PathBuf {
    root.join(handler_name)
}

fn rollback_path_suffix(path: Option<&PathBuf>) -> String {
    match path {
        Some(path) => format!(" at {}", path.display()),
        None => String::new(),
    }
}

fn resolve_required_binary(bin_root: &Path, binary_name: &str) -> Result<PathBuf, InstallError> {
    let preferred = bin_root.join(binary_name);
    if preferred.is_file() {
        return Ok(preferred);
    }

    if let Some(path) = std::env::var_os("PATH") {
        for entry in std::env::split_paths(&path) {
            let candidate = entry.join(binary_name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    Err(InstallError::WriteFailed {
        path: preferred,
        reason: format!(
            "required installed binary `{binary_name}` was not found; install it before local cutover"
        ),
    })
}

fn ensure_runtime_layout(
    runtime_root: &Path,
    runtime_plugin_root: &Path,
    runtime_config_path: &Path,
    bin_root: &Path,
) -> Result<(), InstallError> {
    fs::create_dir_all(runtime_plugin_root).map_err(|err| InstallError::WriteFailed {
        path: runtime_plugin_root.to_path_buf(),
        reason: err.to_string(),
    })?;

    write_file_atomic(runtime_config_path, LOCAL_RUNTIME_CONFIG.as_bytes()).map_err(|err| {
        InstallError::WriteFailed {
            path: runtime_config_path.to_path_buf(),
            reason: err.to_string(),
        }
    })?;

    for plugin in [
        "agent-session-foundation",
        "agent-spawn-gates",
        "atm-extension",
        "tool-output-gates",
    ] {
        let installed_binary = resolve_required_binary(bin_root, plugin)?;
        let wrapper_path = plugin_path_in(runtime_plugin_root, plugin);
        let wrapper = format!(
            "#!/bin/sh\nexec {} \"$@\"\n",
            shell_quote(&installed_binary.display().to_string())
        );
        write_executable_atomic(&wrapper_path, wrapper.as_bytes()).map_err(|err| {
            InstallError::WriteFailed {
                path: wrapper_path.clone(),
                reason: err.to_string(),
            }
        })?;
    }

    fs::create_dir_all(runtime_root.join(".sc-hooks/state")).map_err(|err| {
        InstallError::WriteFailed {
            path: runtime_root.join(".sc-hooks/state"),
            reason: err.to_string(),
        }
    })?;

    Ok(())
}

fn ensure_cli_alias(bin_root: &Path, cli_binary: &Path) -> Result<(), InstallError> {
    #[cfg(unix)]
    {
        let alias_path = bin_root.join(HOOKS_ALIAS_NAME);
        let wrapper = format!(
            "#!/bin/sh\nexec {} \"$@\"\n",
            shell_quote(&cli_binary.display().to_string())
        );
        write_executable_atomic(&alias_path, wrapper.as_bytes()).map_err(|err| {
            InstallError::WriteFailed {
                path: alias_path,
                reason: err.to_string(),
            }
        })?;
    }

    #[cfg(windows)]
    {
        let alias_path = bin_root.join(format!("{HOOKS_ALIAS_NAME}.cmd"));
        let wrapper = format!("@echo off\r\n\"{}\" %*\r\n", cli_binary.display());
        write_file_atomic(&alias_path, wrapper.as_bytes()).map_err(|err| {
            InstallError::WriteFailed {
                path: alias_path,
                reason: err.to_string(),
            }
        })?;
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = (bin_root, cli_binary);
    }

    Ok(())
}

fn write_local_claude_cutover(
    home: &Path,
    runtime_root: &Path,
    runtime_plugin_root: &Path,
    state_root: &Path,
    cli_binary: &Path,
) -> Result<InstallPlan, InstallError> {
    let target_path = home.join(LOCAL_CLAUDE_SETTINGS_PATH);
    let existing = read_existing_config(TargetProvider::Claude, &target_path)?;
    backup_existing_config(TargetProvider::Claude, &target_path)?;

    let runtime_config = local_runtime_config();
    let mut plan =
        build_settings_for_plugin_root(&runtime_config, runtime_plugin_root).map_err(|err| {
            InstallError::WriteFailed {
                path: target_path.clone(),
                reason: err.to_string(),
            }
        })?;
    for entries in plan.settings.hooks.values_mut() {
        for entry in entries {
            for hook in &mut entry.hooks {
                hook.command =
                    wrap_claude_command(&hook.command, runtime_root, state_root, cli_binary)?;
            }
        }
    }

    let mut root = ensure_object(existing);
    root.insert(
        "hooks".to_string(),
        serde_json::to_value(&plan.settings.hooks).map_err(|err| InstallError::WriteFailed {
            path: target_path.clone(),
            reason: err.to_string(),
        })?,
    );
    write_json_atomic(&target_path, &Value::Object(root)).map_err(|err| {
        InstallError::WriteFailed {
            path: target_path.clone(),
            reason: err.to_string(),
        }
    })?;

    plan.warnings.push(format!(
        "rollback backup: {}",
        backup_path(&target_path).display()
    ));
    Ok(plan)
}

fn write_local_codex_cutover(
    home: &Path,
    runtime_root: &Path,
    state_root: &Path,
    cli_binary: &Path,
) -> Result<InstallPlan, InstallError> {
    let target_path = home.join(LOCAL_CODEX_SETTINGS_PATH);
    let existing = read_existing_config(TargetProvider::Codex, &target_path)?;
    backup_existing_config(TargetProvider::Codex, &target_path)?;

    let session_start = provider_shell_command(
        TargetProvider::Codex,
        runtime_root,
        state_root,
        cli_binary,
        Some("codex"),
        Some("CODEX_SESSION_ID"),
        &["run", "SessionStart", "--sync"],
    )?;
    let pre_tool_use = provider_shell_command(
        TargetProvider::Codex,
        runtime_root,
        state_root,
        cli_binary,
        Some("codex"),
        Some("CODEX_SESSION_ID"),
        &["run", "PreToolUse", "Bash", "--sync"],
    )?;

    let mut root = ensure_object(existing);
    root.insert(
        "hooks".to_string(),
        serde_json::json!({
            "SessionStart": [{
                "hooks": [{
                    "type": "command",
                    "command": session_start,
                }]
            }],
            "PreToolUse": [{
                "hooks": [{
                    "type": "command",
                    "command": pre_tool_use,
                }]
            }],
        }),
    );
    write_json_atomic(&target_path, &Value::Object(root)).map_err(|err| {
        InstallError::WriteFailed {
            path: target_path.clone(),
            reason: err.to_string(),
        }
    })?;

    Ok(InstallPlan {
        settings: InstallSettings {
            hooks: BTreeMap::new(),
        },
        warnings: vec![format!(
            "rollback backup: {}",
            backup_path(&target_path).display()
        )],
    })
}

fn write_local_gemini_cutover(
    home: &Path,
    runtime_root: &Path,
    state_root: &Path,
    cli_binary: &Path,
) -> Result<InstallPlan, InstallError> {
    let target_path = home.join(LOCAL_GEMINI_SETTINGS_PATH);
    let existing = read_existing_config(TargetProvider::Gemini, &target_path)?;
    backup_existing_config(TargetProvider::Gemini, &target_path)?;

    let session_start = provider_shell_command(
        TargetProvider::Gemini,
        runtime_root,
        state_root,
        cli_binary,
        Some("gemini"),
        Some("GEMINI_SESSION_ID"),
        &["run", "SessionStart", "--sync"],
    )?;
    let session_end = provider_shell_command(
        TargetProvider::Gemini,
        runtime_root,
        state_root,
        cli_binary,
        Some("gemini"),
        Some("GEMINI_SESSION_ID"),
        &["run", "SessionEnd", "--sync"],
    )?;
    let before_agent = provider_shell_command(
        TargetProvider::Gemini,
        runtime_root,
        state_root,
        cli_binary,
        Some("gemini"),
        Some("GEMINI_SESSION_ID"),
        &["run", "PreToolUse", "Agent", "--sync"],
    )?;
    let before_tool = provider_shell_command(
        TargetProvider::Gemini,
        runtime_root,
        state_root,
        cli_binary,
        Some("gemini"),
        Some("GEMINI_SESSION_ID"),
        &["run", "PreToolUse", "Bash", "--sync"],
    )?;
    let after_tool = provider_shell_command(
        TargetProvider::Gemini,
        runtime_root,
        state_root,
        cli_binary,
        Some("gemini"),
        Some("GEMINI_SESSION_ID"),
        &["run", "PostToolUse", "Bash", "--sync"],
    )?;

    let mut root = ensure_object(existing);
    root.insert(
        "hooks".to_string(),
        serde_json::json!({
            "SessionStart": [{
                "hooks": [{
                    "type": "command",
                    "name": "sc-hooks-session-start",
                    "command": session_start,
                }]
            }],
            "SessionEnd": [{
                "hooks": [{
                    "type": "command",
                    "name": "sc-hooks-session-end",
                    "command": session_end,
                }]
            }],
            "BeforeAgent": [{
                "hooks": [{
                    "type": "command",
                    "name": "sc-hooks-before-agent",
                    "command": before_agent,
                }]
            }],
            "BeforeTool": [{
                "hooks": [{
                    "type": "command",
                    "name": "sc-hooks-before-tool",
                    "command": before_tool,
                }]
            }],
            "AfterTool": [{
                "hooks": [{
                    "type": "command",
                    "name": "sc-hooks-after-tool",
                    "command": after_tool,
                }]
            }],
        }),
    );
    write_json_atomic(&target_path, &Value::Object(root)).map_err(|err| {
        InstallError::WriteFailed {
            path: target_path.clone(),
            reason: err.to_string(),
        }
    })?;

    Ok(InstallPlan {
        settings: InstallSettings {
            hooks: BTreeMap::new(),
        },
        warnings: vec![format!(
            "rollback backup: {}",
            backup_path(&target_path).display()
        )],
    })
}

fn build_settings_for_plugin_root(
    config: &ScHooksConfig,
    plugin_root: &Path,
) -> Result<InstallPlan, CliError> {
    let mut hooks_output = BTreeMap::new();
    let mut warnings = Vec::new();

    for (hook_name, chain) in &config.hooks {
        let specs = collect_specs_for_hook_in_root(hook_name, chain, &mut warnings, plugin_root)?;
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

fn local_runtime_config() -> ScHooksConfig {
    let mut hooks = BTreeMap::new();
    hooks.insert(
        "SessionStart".to_string(),
        vec!["agent-session-foundation".to_string()],
    );
    hooks.insert(
        "SessionEnd".to_string(),
        vec!["agent-session-foundation".to_string()],
    );
    hooks.insert(
        "PreToolUse".to_string(),
        vec!["agent-spawn-gates".to_string(), "atm-extension".to_string()],
    );
    hooks.insert(
        "PostToolUse".to_string(),
        vec!["tool-output-gates".to_string()],
    );

    ScHooksConfig {
        meta: MetaConfig { version: 1 },
        context: BTreeMap::new(),
        hooks,
        sandbox: SandboxConfig::default(),
        observability: ObservabilityConfig::default(),
    }
}

fn collect_specs_for_hook_in_root(
    hook_name: &str,
    chain: &[String],
    warnings: &mut Vec<String>,
    plugin_root: &Path,
) -> Result<Vec<HandlerInstallSpec>, CliError> {
    let hook = HookType::from_str(hook_name).map_err(|_| {
        CliError::internal(format!(
            "unknown hook type `{hook_name}` in install settings build"
        ))
    })?;
    let mut specs = Vec::new();
    let mut manifest_cache: BTreeMap<PathBuf, sc_hooks_core::manifest::Manifest> = BTreeMap::new();

    for handler_name in chain {
        let path = plugin_path_in(plugin_root, handler_name);
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

fn read_existing_config(provider: TargetProvider, path: &Path) -> Result<Value, InstallError> {
    let rendered = fs::read_to_string(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            InstallError::MissingProviderConfig {
                provider,
                path: path.to_path_buf(),
            }
        } else {
            InstallError::WriteFailed {
                path: path.to_path_buf(),
                reason: err.to_string(),
            }
        }
    })?;
    serde_json::from_str(&rendered).map_err(|err| InstallError::WriteFailed {
        path: path.to_path_buf(),
        reason: format!("invalid JSON: {err}"),
    })
}

fn backup_existing_config(provider: TargetProvider, path: &Path) -> Result<(), InstallError> {
    if !path.exists() {
        return Err(InstallError::MissingProviderConfig {
            provider,
            path: path.to_path_buf(),
        });
    }

    let backup = backup_path(path);
    fs::copy(path, &backup).map_err(|err| InstallError::RollbackPlanFailed {
        provider,
        path: Some(backup),
        reason: err.to_string(),
    })?;
    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config".to_string());
    path.with_file_name(format!("{file_name}{BACKUP_SUFFIX}"))
}

fn ensure_object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}

fn wrap_claude_command(
    command: &str,
    runtime_root: &Path,
    state_root: &Path,
    cli_binary: &Path,
) -> Result<String, InstallError> {
    let rewritten = command.replacen(
        "sc-hooks",
        &shell_quote(&cli_binary.display().to_string()),
        1,
    );
    let script = format!(
        "export SC_HOOKS_STATE_DIR={state}; \
export SC_HOOK_AGENT_PID=\"$$\"; \
export ATM_TEAM=\"${{ATM_TEAM:-{team}}}\"; \
export ATM_IDENTITY=\"${{ATM_IDENTITY:-{identity}}}\"; \
cd {runtime_root} && exec {rewritten}",
        state = shell_quote(&state_root.display().to_string()),
        team = DEFAULT_ATM_TEAM,
        identity = DEFAULT_ATM_IDENTITY,
        runtime_root = shell_quote(&runtime_root.display().to_string()),
        rewritten = rewritten,
    );
    shell_wrap_command(TargetProvider::Claude, &script)
}

fn provider_shell_command(
    provider_target: TargetProvider,
    runtime_root: &Path,
    state_root: &Path,
    cli_binary: &Path,
    provider: Option<&str>,
    session_var: Option<&str>,
    args: &[&str],
) -> Result<String, InstallError> {
    let mut script = String::new();
    script.push_str(&format!(
        "export SC_HOOKS_STATE_DIR={}; ",
        shell_quote(&state_root.display().to_string())
    ));
    if let Some(provider) = provider {
        script.push_str(&format!(
            "export SC_HOOK_AGENT_TYPE={}; ",
            shell_quote(provider)
        ));
    }
    if let Some(session_var) = session_var {
        script.push_str(&format!(
            "export SC_HOOK_SESSION_ID=\"${{{session_var}:-unknown}}\"; "
        ));
        script.push_str("export SC_HOOK_AGENT_PID=\"$$\"; ");
    }
    script.push_str(&format!(
        "export ATM_TEAM=\"${{ATM_TEAM:-{}}}\"; export ATM_IDENTITY=\"${{ATM_IDENTITY:-{}}}\"; ",
        DEFAULT_ATM_TEAM, DEFAULT_ATM_IDENTITY
    ));
    script.push_str(&format!(
        "cd {} && exec {}",
        shell_quote(&runtime_root.display().to_string()),
        shell_quote(&cli_binary.display().to_string())
    ));
    for arg in args {
        script.push(' ');
        script.push_str(&shell_quote(arg));
    }
    shell_wrap_command(provider_target, &script)
}

#[cfg(unix)]
fn shell_wrap_command(_provider: TargetProvider, script: &str) -> Result<String, InstallError> {
    Ok(format!("/bin/sh -lc {}", shell_quote(script)))
}

#[cfg(not(unix))]
fn shell_wrap_command(provider: TargetProvider, _script: &str) -> Result<String, InstallError> {
    Err(InstallError::UnsupportedProvider {
        provider,
        supported: &["claude", "codex", "gemini (unix-only local cutover)"],
    })
}

fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), std::io::Error> {
    let rendered = serde_json::to_string_pretty(value)
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    write_file_atomic(path, rendered.as_bytes())
}

fn write_executable_atomic(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    write_file_atomic(path, bytes)?;
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(path)?.permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn write_file_atomic(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        use std::io::Write;
        temp.write_all(bytes)?;
        temp.as_file().sync_all()?;
        temp.persist(path)
            .map_err(|err| std::io::Error::other(err.to_string()))?;
    } else {
        fs::write(path, bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::test_support;
    use serial_test::serial;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

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
            let mut perms = fs::metadata(path)
                .expect("plugin metadata should be available")
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
            fs::set_permissions(path, perms).expect("plugin should be executable");
        }
    }

    fn make_executable(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("executable parent directory should be creatable");
        }
        fs::write(path, body).expect("executable should be writable");
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path)
                .expect("executable metadata should be available")
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
            fs::set_permissions(path, perms).expect("executable should be executable");
        }
    }

    fn with_atm_home<T>(value: &Path, body: impl FnOnce() -> T) -> T {
        let _lock = env_lock().lock().unwrap_or_else(|err| err.into_inner());
        temp_env::with_var("ATM_HOME", Some(value), body)
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

    #[test]
    #[serial]
    fn local_provider_cutover_writes_provider_configs_and_runtime_root() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let home = temp.path().join("home");
        with_atm_home(&home, || {
            fs::create_dir_all(home.join(".claude")).expect(".claude should create");
            fs::create_dir_all(home.join(".codex")).expect(".codex should create");
            fs::create_dir_all(home.join(".gemini")).expect(".gemini should create");

            fs::write(
                home.join(".claude/settings.json"),
                serde_json::json!({
                    "model": "sonnet",
                    "hooks": {}
                })
                .to_string(),
            )
            .expect("claude settings should write");
            fs::write(
                home.join(".codex/hooks.json"),
                serde_json::json!({
                    "hooks": {}
                })
                .to_string(),
            )
            .expect("codex settings should write");
            fs::write(
                home.join(".gemini/settings.json"),
                serde_json::json!({
                    "general": {
                        "sessionRetention": {
                            "enabled": true
                        }
                    },
                    "hooks": {}
                })
                .to_string(),
            )
            .expect("gemini settings should write");

            let bin_root = home.join(".local/bin");
            for binary in [
                "sc-hooks",
                "agent-session-foundation",
                "agent-spawn-gates",
                "atm-extension",
                "tool-output-gates",
            ] {
                make_executable(
                    &bin_root.join(binary),
                    &format!(
                        "#!/bin/sh\nif [ \"$1\" = \"--manifest\" ]; then\ncat <<'JSON'\n{}\nJSON\nexit 0\nfi\nexit 0\n",
                        match binary {
                            "agent-session-foundation" =>
                                r#"{"contract_version":1,"name":"agent-session-foundation","mode":"sync","hooks":["SessionStart","SessionEnd"],"matchers":["*"],"requires":{}}"#,
                            "agent-spawn-gates" =>
                                r#"{"contract_version":1,"name":"agent-spawn-gates","mode":"sync","hooks":["PreToolUse"],"matchers":["Agent"],"requires":{}}"#,
                            "atm-extension" =>
                                r#"{"contract_version":1,"name":"atm-extension","mode":"sync","hooks":["PreToolUse"],"matchers":["Bash"],"requires":{}}"#,
                            "tool-output-gates" =>
                                r#"{"contract_version":1,"name":"tool-output-gates","mode":"sync","hooks":["PostToolUse"],"matchers":["Bash"],"requires":{}}"#,
                            _ => "{}",
                        }
                    ),
                );
            }

            let claude_plan = write_local_provider_cutover(TargetProvider::Claude)
                .expect("claude cutover should succeed");
            write_local_provider_cutover(TargetProvider::Codex)
                .expect("codex cutover should succeed");
            write_local_provider_cutover(TargetProvider::Gemini)
                .expect("gemini cutover should succeed");

            assert!(
                claude_plan
                    .warnings
                    .iter()
                    .any(|warning| warning.contains("rollback backup"))
            );

            let runtime_root = home.join(LOCAL_RUNTIME_RELATIVE_ROOT);
            assert!(runtime_root.join(LOCAL_RUNTIME_CONFIG_PATH).exists());
            assert!(
                runtime_root
                    .join(LOCAL_RUNTIME_PLUGIN_DIR)
                    .join("atm-extension")
                    .exists()
            );
            #[cfg(unix)]
            {
                let alias_path = home.join(".local/bin").join(HOOKS_ALIAS_NAME);
                let alias_body =
                    fs::read_to_string(&alias_path).expect("hooks alias wrapper should exist");
                assert!(alias_body.contains("sc-hooks"));
            }
            #[cfg(windows)]
            {
                let alias_path = home
                    .join(".local/bin")
                    .join(format!("{HOOKS_ALIAS_NAME}.cmd"));
                let alias_body =
                    fs::read_to_string(&alias_path).expect("hooks alias wrapper should exist");
                assert!(alias_body.contains("sc-hooks"));
            }

            let claude: Value = serde_json::from_str(
                &fs::read_to_string(home.join(LOCAL_CLAUDE_SETTINGS_PATH))
                    .expect("claude settings should read"),
            )
            .expect("claude settings should parse");
            assert_eq!(claude["model"], "sonnet");
            let claude_pre_tool = &claude["hooks"]["PreToolUse"][0]["hooks"][0]["command"];
            assert!(
                claude_pre_tool
                    .as_str()
                    .unwrap_or_default()
                    .contains("sc-hooks")
            );
            assert!(
                claude_pre_tool
                    .as_str()
                    .unwrap_or_default()
                    .contains(".local/share/sc-hooks/runtime-layout")
            );

            let codex: Value = serde_json::from_str(
                &fs::read_to_string(home.join(LOCAL_CODEX_SETTINGS_PATH))
                    .expect("codex settings should read"),
            )
            .expect("codex settings should parse");
            let codex_command = codex["hooks"]["PreToolUse"][0]["hooks"][0]["command"]
                .as_str()
                .unwrap_or_default();
            assert!(codex_command.contains("SC_HOOK_AGENT_TYPE"));
            assert!(codex_command.contains("codex"));

            let gemini: Value = serde_json::from_str(
                &fs::read_to_string(home.join(LOCAL_GEMINI_SETTINGS_PATH))
                    .expect("gemini settings should read"),
            )
            .expect("gemini settings should parse");
            assert_eq!(gemini["general"]["sessionRetention"]["enabled"], true);
            let gemini_command = gemini["hooks"]["BeforeTool"][0]["hooks"][0]["command"]
                .as_str()
                .unwrap_or_default();
            assert!(gemini_command.contains("SC_HOOK_AGENT_TYPE"));
            assert!(gemini_command.contains("gemini"));
            assert!(home.join(".claude/settings.json.sc-hooks.bak").exists());
            assert!(home.join(".codex/hooks.json.sc-hooks.bak").exists());
            assert!(home.join(".gemini/settings.json.sc-hooks.bak").exists());
        });
    }

    #[test]
    #[serial]
    fn local_provider_cutover_errors_when_provider_config_is_missing() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let home = temp.path().join("home");
        with_atm_home(&home, || {
            fs::create_dir_all(home.join(".local/bin")).expect("bin root should create");
            for binary in [
                "sc-hooks",
                "agent-session-foundation",
                "agent-spawn-gates",
                "atm-extension",
                "tool-output-gates",
            ] {
                make_executable(&home.join(".local/bin").join(binary), "#!/bin/sh\nexit 0\n");
            }

            let err = write_local_provider_cutover(TargetProvider::Codex)
                .expect_err("missing codex config should fail");
            assert!(matches!(
                err,
                InstallError::MissingProviderConfig {
                    provider: TargetProvider::Codex,
                    ..
                }
            ));
        });
    }
}
