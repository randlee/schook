use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

pub const DEFAULT_CONFIG_PATH: &str = ".sc-hooks/config.toml";
pub const ENV_OBSERVABILITY_MODE: &str = "SC_HOOKS_OBSERVABILITY_MODE";
pub const ENV_AUDIT_PROFILE: &str = "SC_HOOKS_AUDIT_PROFILE";
pub const ENV_AUDIT_PATH: &str = "SC_HOOKS_AUDIT_PATH";
pub const ENV_AUDIT_MAX_RUNS: &str = "SC_HOOKS_AUDIT_MAX_RUNS";
pub const ENV_AUDIT_MAX_AGE_DAYS: &str = "SC_HOOKS_AUDIT_MAX_AGE_DAYS";
pub const ENV_AUDIT_REDACTION: &str = "SC_HOOKS_AUDIT_REDACTION";
pub const ENV_AUDIT_CAPTURE_PAYLOADS: &str = "SC_HOOKS_AUDIT_CAPTURE_PAYLOADS";
pub const ENV_AUDIT_CAPTURE_STDIO: &str = "SC_HOOKS_AUDIT_CAPTURE_STDIO";

const DEFAULT_GLOBAL_CONFIG_DIR: &str = ".sc-hooks";
const DEFAULT_GLOBAL_CONFIG_FILE: &str = "config.toml";
const REQUIRED_LOCAL_SECTIONS: [&str; 2] = ["meta", "hooks"];
const ALLOWED_LOCAL_SECTIONS: [&str; 5] = ["meta", "context", "hooks", "sandbox", "observability"];
const ALLOWED_GLOBAL_SECTIONS: [&str; 1] = ["observability"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScHooksConfig {
    pub meta: MetaConfig,
    #[serde(default)]
    pub context: BTreeMap<String, Value>,
    pub hooks: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub sandbox: SandboxConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

impl ScHooksConfig {
    pub fn to_pretty_toml(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self).map_err(|source| ConfigError::Format {
            source: Box::new(source),
        })
    }

    pub fn mapped_context_metadata(&self) -> BTreeMap<String, Value> {
        map_context_to_metadata(&self.context)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaConfig {
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SandboxConfig {
    #[serde(default)]
    pub allow_network: Vec<String>,
    #[serde(default)]
    pub allow_paths: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObservabilityMode {
    Off,
    #[default]
    Standard,
    Full,
}

impl ObservabilityMode {
    fn parse_token(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "standard" => Some(Self::Standard),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    fn expected_values() -> &'static str {
        "off, standard, full"
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FullAuditProfile {
    #[default]
    Lean,
    Debug,
}

impl FullAuditProfile {
    fn parse_token(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "lean" => Some(Self::Lean),
            "debug" => Some(Self::Debug),
            _ => None,
        }
    }

    fn expected_values() -> &'static str {
        "lean, debug"
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Lean => "lean",
            Self::Debug => "debug",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RedactionMode {
    #[default]
    Strict,
    Permissive,
}

impl RedactionMode {
    fn parse_token(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "strict" => Some(Self::Strict),
            "permissive" => Some(Self::Permissive),
            _ => None,
        }
    }

    fn expected_values() -> &'static str {
        "strict, permissive"
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptureStdio {
    None,
    #[default]
    Summary,
    Bounded,
}

impl CaptureStdio {
    fn parse_token(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "none" => Some(Self::None),
            "summary" => Some(Self::Summary),
            "bounded" => Some(Self::Bounded),
            _ => None,
        }
    }

    fn expected_values() -> &'static str {
        "none, summary, bounded"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub mode: ObservabilityMode,
    #[serde(default)]
    pub full_profile: FullAuditProfile,
    #[serde(default = "default_audit_path")]
    pub path: PathBuf,
    #[serde(default)]
    pub console_mirror: bool,
    #[serde(default = "default_retain_runs")]
    pub retain_runs: u32,
    #[serde(default = "default_retain_days")]
    pub retain_days: u32,
    #[serde(default)]
    pub redaction: RedactionMode,
    #[serde(default)]
    pub capture_payloads: bool,
    #[serde(default)]
    pub capture_stdio: CaptureStdio,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            mode: ObservabilityMode::Standard,
            full_profile: FullAuditProfile::Lean,
            path: default_audit_path(),
            console_mirror: false,
            retain_runs: default_retain_runs(),
            retain_days: default_retain_days(),
            redaction: RedactionMode::Strict,
            capture_payloads: false,
            capture_stdio: CaptureStdio::Summary,
        }
    }
}

impl ObservabilityConfig {
    fn apply_global(&mut self, layer: GlobalObservabilityConfigLayer) {
        if let Some(mode) = layer.mode {
            self.mode = mode;
        }
        if let Some(console_mirror) = layer.console_mirror {
            self.console_mirror = console_mirror;
        }
        if let Some(retain_runs) = layer.retain_runs {
            self.retain_runs = retain_runs;
        }
        if let Some(retain_days) = layer.retain_days {
            self.retain_days = retain_days;
        }
        if let Some(redaction) = layer.redaction {
            self.redaction = redaction;
        }
    }

    fn apply_local(&mut self, layer: LocalObservabilityConfigLayer) {
        if let Some(mode) = layer.mode {
            self.mode = mode;
        }
        if let Some(full_profile) = layer.full_profile {
            self.full_profile = full_profile;
        }
        if let Some(path) = layer.path {
            self.path = path;
        }
        if let Some(console_mirror) = layer.console_mirror {
            self.console_mirror = console_mirror;
        }
        if let Some(retain_runs) = layer.retain_runs {
            self.retain_runs = retain_runs;
        }
        if let Some(retain_days) = layer.retain_days {
            self.retain_days = retain_days;
        }
        if let Some(redaction) = layer.redaction {
            self.redaction = redaction;
        }
        if let Some(capture_payloads) = layer.capture_payloads {
            self.capture_payloads = capture_payloads;
        }
        if let Some(capture_stdio) = layer.capture_stdio {
            self.capture_stdio = capture_stdio;
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawLocalConfig {
    meta: MetaConfig,
    #[serde(default)]
    context: BTreeMap<String, Value>,
    hooks: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    sandbox: SandboxConfig,
    #[serde(default)]
    observability: LocalObservabilityConfigLayer,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGlobalConfig {
    #[serde(default)]
    observability: GlobalObservabilityConfigLayer,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct GlobalObservabilityConfigLayer {
    mode: Option<ObservabilityMode>,
    console_mirror: Option<bool>,
    retain_runs: Option<u32>,
    retain_days: Option<u32>,
    redaction: Option<RedactionMode>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalObservabilityConfigLayer {
    mode: Option<ObservabilityMode>,
    full_profile: Option<FullAuditProfile>,
    path: Option<PathBuf>,
    console_mirror: Option<bool>,
    retain_runs: Option<u32>,
    retain_days: Option<u32>,
    redaction: Option<RedactionMode>,
    capture_payloads: Option<bool>,
    capture_stdio: Option<CaptureStdio>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config at {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("invalid TOML in {location}: {source}")]
    TomlParse {
        location: String,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("config in {location} must be a TOML table at the top level")]
    RootNotTable { location: String },

    #[error("config in {location} is missing required section [{section}]")]
    MissingSection {
        location: String,
        section: &'static str,
    },

    #[error("config in {location} has unknown top-level section(s): {sections}")]
    UnknownSections { location: String, sections: String },

    #[error("config in {location} is missing required field [meta].version")]
    MissingVersion { location: String },

    #[error("config in {location} has non-integer [meta].version")]
    NonIntegerVersion { location: String },

    #[error("invalid config structure in {location}: {source}")]
    InvalidStructure {
        location: String,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("global observability config in {location} may not set mode = \"full\"")]
    GlobalFullMode { location: String },

    #[error("environment override {key} must be valid UTF-8")]
    NonUtf8EnvOverride { key: &'static str },

    #[error("invalid environment override {key}={value:?}; expected {expected}")]
    InvalidEnvOverride {
        key: &'static str,
        value: String,
        expected: &'static str,
    },

    #[error("failed to format resolved config: {source}")]
    Format {
        #[source]
        source: Box<toml::ser::Error>,
    },
}

pub fn load_default_config() -> Result<ScHooksConfig, ConfigError> {
    let global_path = default_global_config_path();
    load_layered_config(Path::new(DEFAULT_CONFIG_PATH), global_path.as_deref())
}

pub fn load_layered_config(
    local_path: &Path,
    global_path: Option<&Path>,
) -> Result<ScHooksConfig, ConfigError> {
    let local_contents = fs::read_to_string(local_path).map_err(|source| ConfigError::Read {
        path: local_path.to_path_buf(),
        source,
    })?;
    let local_location = local_path.display().to_string();
    let local = parse_local_raw_config(&local_contents, local_location.clone())?;

    let mut observability = ObservabilityConfig::default();
    if let Some(global_path) = global_path
        && let Some(global_contents) = read_optional_config(global_path)?
    {
        let global_location = global_path.display().to_string();
        let global = parse_global_raw_config(&global_contents, global_location.clone())?;
        if matches!(global.observability.mode, Some(ObservabilityMode::Full)) {
            return Err(ConfigError::GlobalFullMode {
                location: global_location,
            });
        }
        observability.apply_global(global.observability);
    }
    observability.apply_local(local.observability);
    apply_env_overrides(&mut observability)?;

    Ok(ScHooksConfig {
        meta: local.meta,
        context: local.context,
        hooks: local.hooks,
        sandbox: local.sandbox,
        observability,
    })
}

#[cfg(test)]
pub fn parse_config_str(
    input: &str,
    source_location: impl Into<String>,
) -> Result<ScHooksConfig, ConfigError> {
    let source_location = source_location.into();
    let raw = parse_local_raw_config(input, source_location)?;

    let mut observability = ObservabilityConfig::default();
    observability.apply_local(raw.observability);

    Ok(ScHooksConfig {
        meta: raw.meta,
        context: raw.context,
        hooks: raw.hooks,
        sandbox: raw.sandbox,
        observability,
    })
}

pub fn map_context_to_metadata(context: &BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let mut mapped = BTreeMap::new();

    if let Some(team_name) = context.get("team") {
        let mut team = toml::map::Map::new();
        team.insert("name".to_string(), team_name.clone());
        mapped.insert("team".to_string(), Value::Table(team));
    }

    for (key, value) in context {
        if key != "team" {
            mapped.insert(key.clone(), value.clone());
        }
    }

    mapped
}

fn parse_local_raw_config(
    input: &str,
    source_location: String,
) -> Result<RawLocalConfig, ConfigError> {
    let value = parse_root_value(input, &source_location)?;
    let root = value.as_table().ok_or_else(|| ConfigError::RootNotTable {
        location: source_location.clone(),
    })?;

    validate_sections(
        root,
        &source_location,
        &REQUIRED_LOCAL_SECTIONS,
        &ALLOWED_LOCAL_SECTIONS,
    )?;
    validate_version(root, &source_location)?;

    value
        .try_into()
        .map_err(|source| ConfigError::InvalidStructure {
            location: source_location,
            source: Box::new(source),
        })
}

fn parse_global_raw_config(
    input: &str,
    source_location: String,
) -> Result<RawGlobalConfig, ConfigError> {
    let value = parse_root_value(input, &source_location)?;
    let root = value.as_table().ok_or_else(|| ConfigError::RootNotTable {
        location: source_location.clone(),
    })?;

    validate_sections(root, &source_location, &[], &ALLOWED_GLOBAL_SECTIONS)?;

    value
        .try_into()
        .map_err(|source| ConfigError::InvalidStructure {
            location: source_location,
            source: Box::new(source),
        })
}

fn parse_root_value(input: &str, source_location: &str) -> Result<Value, ConfigError> {
    toml::from_str(input).map_err(|source| ConfigError::TomlParse {
        location: source_location.to_string(),
        source: Box::new(source),
    })
}

fn read_optional_config(path: &Path) -> Result<Option<String>, ConfigError> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ConfigError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn validate_sections(
    root: &toml::map::Map<String, Value>,
    source: &str,
    required_sections: &[&'static str],
    allowed_sections: &[&str],
) -> Result<(), ConfigError> {
    for section in required_sections {
        if !root.contains_key(*section) {
            return Err(ConfigError::MissingSection {
                location: source.to_string(),
                section,
            });
        }
    }

    let allowed: BTreeSet<&str> = allowed_sections.iter().copied().collect();
    let unknown: Vec<String> = root
        .keys()
        .filter(|name| !allowed.contains(name.as_str()))
        .cloned()
        .collect();

    if !unknown.is_empty() {
        return Err(ConfigError::UnknownSections {
            location: source.to_string(),
            sections: unknown.join(", "),
        });
    }

    Ok(())
}

fn validate_version(root: &toml::map::Map<String, Value>, source: &str) -> Result<(), ConfigError> {
    let Some(meta) = root.get("meta").and_then(Value::as_table) else {
        return Err(ConfigError::MissingSection {
            location: source.to_string(),
            section: "meta",
        });
    };

    match meta.get("version") {
        None => Err(ConfigError::MissingVersion {
            location: source.to_string(),
        }),
        Some(Value::Integer(value)) if *value >= 0 => Ok(()),
        Some(_) => Err(ConfigError::NonIntegerVersion {
            location: source.to_string(),
        }),
    }
}

fn default_global_config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(
        PathBuf::from(home)
            .join(DEFAULT_GLOBAL_CONFIG_DIR)
            .join(DEFAULT_GLOBAL_CONFIG_FILE),
    )
}

fn apply_env_overrides(observability: &mut ObservabilityConfig) -> Result<(), ConfigError> {
    if let Some(value) = env_override_value(ENV_OBSERVABILITY_MODE)? {
        observability.mode = ObservabilityMode::parse_token(&value).ok_or_else(|| {
            ConfigError::InvalidEnvOverride {
                key: ENV_OBSERVABILITY_MODE,
                value,
                expected: ObservabilityMode::expected_values(),
            }
        })?;
    }
    if let Some(value) = env_override_value(ENV_AUDIT_PROFILE)? {
        observability.full_profile = FullAuditProfile::parse_token(&value).ok_or_else(|| {
            ConfigError::InvalidEnvOverride {
                key: ENV_AUDIT_PROFILE,
                value,
                expected: FullAuditProfile::expected_values(),
            }
        })?;
    }
    if let Some(value) = std::env::var_os(ENV_AUDIT_PATH) {
        observability.path = PathBuf::from(value);
    }
    if let Some(value) = env_override_value(ENV_AUDIT_MAX_RUNS)? {
        observability.retain_runs =
            parse_env_u32(&value).ok_or(ConfigError::InvalidEnvOverride {
                key: ENV_AUDIT_MAX_RUNS,
                value,
                expected: "non-negative integer",
            })?;
    }
    if let Some(value) = env_override_value(ENV_AUDIT_MAX_AGE_DAYS)? {
        observability.retain_days =
            parse_env_u32(&value).ok_or(ConfigError::InvalidEnvOverride {
                key: ENV_AUDIT_MAX_AGE_DAYS,
                value,
                expected: "non-negative integer",
            })?;
    }
    if let Some(value) = env_override_value(ENV_AUDIT_REDACTION)? {
        observability.redaction =
            RedactionMode::parse_token(&value).ok_or_else(|| ConfigError::InvalidEnvOverride {
                key: ENV_AUDIT_REDACTION,
                value,
                expected: RedactionMode::expected_values(),
            })?;
    }
    if let Some(value) = env_override_value(ENV_AUDIT_CAPTURE_PAYLOADS)? {
        observability.capture_payloads = parse_env_bool(ENV_AUDIT_CAPTURE_PAYLOADS, &value).ok_or(
            ConfigError::InvalidEnvOverride {
                key: ENV_AUDIT_CAPTURE_PAYLOADS,
                value,
                expected: "1/true/yes/on or 0/false/no/off",
            },
        )?;
    }
    if let Some(value) = env_override_value(ENV_AUDIT_CAPTURE_STDIO)? {
        observability.capture_stdio =
            CaptureStdio::parse_token(&value).ok_or_else(|| ConfigError::InvalidEnvOverride {
                key: ENV_AUDIT_CAPTURE_STDIO,
                value,
                expected: CaptureStdio::expected_values(),
            })?;
    }

    Ok(())
}

fn env_override_value(key: &'static str) -> Result<Option<String>, ConfigError> {
    let Some(value) = std::env::var_os(key) else {
        return Ok(None);
    };
    let value = value
        .into_string()
        .map_err(|_| ConfigError::NonUtf8EnvOverride { key })?;
    Ok(Some(value))
}

fn parse_env_bool(_key: &'static str, value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_env_u32(value: &str) -> Option<u32> {
    value.trim().parse().ok()
}
fn default_audit_path() -> PathBuf {
    PathBuf::from(".sc-hooks/audit")
}

const fn default_retain_runs() -> u32 {
    10
}

const fn default_retain_days() -> u32 {
    14
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use std::time::{Duration, Instant};

    fn valid_base_config() -> &'static str {
        r#"
[meta]
version = 1

[context]
team = "calibration"
project = "p3"

[hooks]
PreToolUse = ["guard-paths"]
"#
    }

    fn minimal_required_config() -> &'static str {
        r#"
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths"]
"#
    }

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        _lock: MutexGuard<'static, ()>,
        saved: Vec<(&'static str, Option<OsString>)>,
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.saved.drain(..) {
                match value {
                    Some(value) => unsafe { std::env::set_var(key, value) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }
        }
    }

    fn scoped_env(overrides: &[(&'static str, Option<&str>)]) -> EnvGuard {
        let lock = env_lock().lock().unwrap_or_else(|err| err.into_inner());
        let saved = overrides
            .iter()
            .map(|(key, _)| (*key, std::env::var_os(key)))
            .collect::<Vec<_>>();

        for (key, value) in overrides {
            match value {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }

        EnvGuard { _lock: lock, saved }
    }

    fn write_config(path: &Path, contents: &str) {
        fs::create_dir_all(
            path.parent()
                .expect("config file should always have a parent directory"),
        )
        .expect("config directory should be creatable");
        fs::write(path, contents).expect("config file should be writable");
    }

    #[test]
    fn parses_from_in_memory_toml_with_sandbox_defaults() {
        let config =
            parse_config_str(valid_base_config(), "in-memory").expect("config should parse");

        assert_eq!(config.meta.version, 1);
        assert_eq!(config.sandbox, SandboxConfig::default());
        assert_eq!(config.observability, ObservabilityConfig::default());
        assert_eq!(
            config
                .hooks
                .get("PreToolUse")
                .expect("PreToolUse hook should be present"),
            &vec!["guard-paths".to_string()]
        );
    }

    #[test]
    fn parses_with_only_required_sections() {
        let config = parse_config_str(minimal_required_config(), "in-memory")
            .expect("minimal config should parse");

        assert!(config.context.is_empty());
        assert_eq!(config.sandbox, SandboxConfig::default());
        assert_eq!(config.observability, ObservabilityConfig::default());
    }

    #[test]
    fn parses_optional_sandbox_section() {
        let config = r#"
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths"]

[sandbox]
allow_network = ["notify"]
allow_paths = { "guard-paths" = [".sc-hooks/guard-paths.toml"] }
"#;
        let parsed = parse_config_str(config, "in-memory")
            .expect("config with sandbox overrides should parse");

        assert_eq!(parsed.sandbox.allow_network, vec!["notify".to_string()]);
        assert_eq!(
            parsed
                .sandbox
                .allow_paths
                .get("guard-paths")
                .expect("guard-paths override should exist"),
            &vec![".sc-hooks/guard-paths.toml".to_string()]
        );
    }

    #[test]
    fn parses_observability_section_with_local_only_fields() {
        let config = r#"
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths"]

[observability]
mode = "full"
full_profile = "debug"
path = ".sc-hooks/custom-audit"
console_mirror = true
retain_runs = 7
retain_days = 21
redaction = "permissive"
capture_payloads = true
capture_stdio = "bounded"
"#;
        let parsed =
            parse_config_str(config, "in-memory").expect("observability section should parse");

        assert_eq!(parsed.observability.mode, ObservabilityMode::Full);
        assert_eq!(parsed.observability.full_profile, FullAuditProfile::Debug);
        assert_eq!(
            parsed.observability.path,
            PathBuf::from(".sc-hooks/custom-audit")
        );
        assert!(parsed.observability.console_mirror);
        assert_eq!(parsed.observability.retain_runs, 7);
        assert_eq!(parsed.observability.retain_days, 21);
        assert_eq!(parsed.observability.redaction, RedactionMode::Permissive);
        assert!(parsed.observability.capture_payloads);
        assert_eq!(parsed.observability.capture_stdio, CaptureStdio::Bounded);
    }

    #[test]
    fn maps_context_team_key_to_team_name() {
        let config = parse_config_str(valid_base_config(), "in-memory")
            .expect("config should parse for context mapping");
        let mapped = config.mapped_context_metadata();

        assert_eq!(
            mapped.get("project"),
            Some(&Value::String("p3".to_string())),
            "non-team keys should map to top-level metadata fields"
        );

        let team = mapped
            .get("team")
            .and_then(Value::as_table)
            .expect("team key should map to a nested table");
        assert_eq!(
            team.get("name"),
            Some(&Value::String("calibration".to_string())),
            "team key should map to team.name"
        );
    }

    #[test]
    fn rejects_unknown_top_level_section() {
        let config = format!("{}\n[extra]\nfoo = \"bar\"\n", valid_base_config());

        let err = parse_config_str(&config, "in-memory").expect_err("unknown section must fail");
        assert!(
            matches!(err, ConfigError::UnknownSections { .. }),
            "expected unknown-section error, got {err:?}"
        );
    }

    #[test]
    fn rejects_unknown_observability_field() {
        let config = r#"
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths"]

[observability]
unknown_key = true
"#;

        let err =
            parse_config_str(config, "in-memory").expect_err("unknown observability key must fail");
        assert!(
            matches!(err, ConfigError::InvalidStructure { .. }),
            "expected invalid-structure error, got {err:?}"
        );
    }

    #[test]
    fn rejects_missing_meta_version() {
        let config = r#"
[meta]

[context]
team = "calibration"

[hooks]
PreToolUse = ["guard-paths"]
"#;

        let err = parse_config_str(config, "in-memory").expect_err("missing version must fail");
        assert!(
            matches!(err, ConfigError::MissingVersion { .. }),
            "expected missing-version error, got {err:?}"
        );
    }

    #[test]
    fn rejects_missing_hooks_section() {
        let config = r#"
[meta]
version = 1
"#;

        let err = parse_config_str(config, "in-memory").expect_err("missing [hooks] should fail");
        assert!(
            matches!(
                err,
                ConfigError::MissingSection {
                    section: "hooks",
                    ..
                }
            ),
            "expected missing hooks section error, got {err:?}"
        );
    }

    #[test]
    fn rejects_non_integer_meta_version() {
        let config = r#"
[meta]
version = "one"

[context]
team = "calibration"

[hooks]
PreToolUse = ["guard-paths"]
"#;

        let err = parse_config_str(config, "in-memory").expect_err("non-integer version must fail");
        assert!(
            matches!(err, ConfigError::NonIntegerVersion { .. }),
            "expected non-integer version error, got {err:?}"
        );
    }

    #[test]
    fn rejects_negative_meta_version() {
        let config = r#"
[meta]
version = -1

[hooks]
PreToolUse = ["guard-paths"]
"#;

        let err = parse_config_str(config, "in-memory").expect_err("negative version must fail");
        assert!(
            matches!(err, ConfigError::NonIntegerVersion { .. }),
            "expected non-integer version error for negative value, got {err:?}"
        );
    }

    #[test]
    fn rejects_full_mode_from_global_config_alone() {
        let temp = tempfile::tempdir().expect("tempdir should be creatable");
        let local_path = temp.path().join("repo/.sc-hooks/config.toml");
        let global_path = temp.path().join("home/.sc-hooks/config.toml");
        write_config(&local_path, minimal_required_config());
        write_config(
            &global_path,
            r#"
[observability]
mode = "full"
"#,
        );

        let err = load_layered_config(&local_path, Some(&global_path))
            .expect_err("global full mode must be rejected");
        assert!(
            matches!(err, ConfigError::GlobalFullMode { .. }),
            "expected global-full-mode error, got {err:?}"
        );
    }

    #[test]
    fn default_global_config_path_uses_userprofile_when_home_is_missing() {
        let userprofile = std::env::temp_dir().join("sc-hooks-home-fallback");
        let userprofile = userprofile
            .to_str()
            .expect("temp userprofile path should be valid utf-8")
            .to_string();
        let _env = scoped_env(&[("HOME", None), ("USERPROFILE", Some(userprofile.as_str()))]);

        assert_eq!(
            default_global_config_path(),
            Some(PathBuf::from(userprofile).join(".sc-hooks/config.toml"))
        );
    }

    #[test]
    fn layered_config_applies_built_in_global_local_and_env_precedence() {
        let audit_env_path = std::env::temp_dir().join("sc-hooks-audit-env");
        let audit_env_path = audit_env_path
            .to_str()
            .expect("temp audit path should be valid utf-8")
            .to_string();
        let _env = scoped_env(&[
            (ENV_OBSERVABILITY_MODE, Some("full")),
            (ENV_AUDIT_PROFILE, Some("lean")),
            (ENV_AUDIT_PATH, Some(audit_env_path.as_str())),
            (ENV_AUDIT_MAX_RUNS, Some("4")),
            (ENV_AUDIT_MAX_AGE_DAYS, Some("45")),
            (ENV_AUDIT_REDACTION, Some("strict")),
            (ENV_AUDIT_CAPTURE_PAYLOADS, Some("true")),
            (ENV_AUDIT_CAPTURE_STDIO, Some("summary")),
        ]);
        let temp = tempfile::tempdir().expect("tempdir should be creatable");
        let local_path = temp.path().join("repo/.sc-hooks/config.toml");
        let global_path = temp.path().join("home/.sc-hooks/config.toml");
        write_config(
            &local_path,
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths"]

[observability]
mode = "standard"
full_profile = "debug"
path = ".sc-hooks/local-audit"
console_mirror = true
retain_runs = 9
capture_stdio = "bounded"
"#,
        );
        write_config(
            &global_path,
            r#"
[observability]
mode = "off"
console_mirror = false
retain_runs = 5
retain_days = 30
redaction = "permissive"
"#,
        );

        let config = load_layered_config(&local_path, Some(&global_path))
            .expect("layered config should resolve");

        assert_eq!(config.observability.mode, ObservabilityMode::Full);
        assert_eq!(config.observability.full_profile, FullAuditProfile::Lean);
        assert_eq!(config.observability.path, PathBuf::from(&audit_env_path));
        assert!(config.observability.console_mirror);
        assert_eq!(config.observability.retain_runs, 4);
        assert_eq!(config.observability.retain_days, 45);
        assert_eq!(config.observability.redaction, RedactionMode::Strict);
        assert!(config.observability.capture_payloads);
        assert_eq!(config.observability.capture_stdio, CaptureStdio::Summary);
    }

    #[test]
    fn invalid_environment_override_is_rejected() {
        let _env = scoped_env(&[(ENV_OBSERVABILITY_MODE, Some("invalid"))]);
        let temp = tempfile::tempdir().expect("tempdir should be creatable");
        let local_path = temp.path().join("repo/.sc-hooks/config.toml");
        write_config(&local_path, minimal_required_config());

        let err = load_layered_config(&local_path, None)
            .expect_err("invalid env override must be rejected");
        assert!(
            matches!(
                err,
                ConfigError::InvalidEnvOverride {
                    key: ENV_OBSERVABILITY_MODE,
                    ..
                }
            ),
            "expected invalid-env-override error, got {err:?}"
        );
    }

    #[test]
    fn renders_resolved_config_for_config_subcommand() {
        let parsed = parse_config_str(valid_base_config(), "in-memory")
            .expect("config should parse for render");
        let rendered = parsed
            .to_pretty_toml()
            .expect("resolved config should render to TOML");

        assert!(rendered.contains("[meta]"));
        assert!(rendered.contains("[hooks]"));
        assert!(rendered.contains("[sandbox]"));
        assert!(rendered.contains("[observability]"));
    }

    #[test]
    fn parses_config_under_five_ms_median() {
        let mut samples = Vec::new();
        for _ in 0..21 {
            let started = Instant::now();
            let _ =
                parse_config_str(valid_base_config(), "in-memory").expect("config should parse");
            samples.push(started.elapsed());
        }

        samples.sort_unstable();
        let median = samples[samples.len() / 2];
        assert!(
            median < Duration::from_millis(5),
            "median config parse time {median:?} exceeded 5ms target"
        );
    }
}
