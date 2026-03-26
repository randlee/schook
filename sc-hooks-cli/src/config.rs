use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

pub const DEFAULT_CONFIG_PATH: &str = ".sc-hooks/config.toml";

const REQUIRED_SECTIONS: [&str; 2] = ["meta", "hooks"];
const ALLOWED_SECTIONS: [&str; 4] = ["meta", "context", "hooks", "sandbox"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScHooksConfig {
    pub meta: MetaConfig,
    #[serde(default)]
    pub context: BTreeMap<String, Value>,
    pub hooks: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub sandbox: SandboxConfig,
}

impl ScHooksConfig {
    pub fn to_pretty_toml(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self).map_err(|source| ConfigError::Format {
            message: source.to_string(),
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

#[derive(Debug, Deserialize)]
struct RawConfig {
    meta: MetaConfig,
    #[serde(default)]
    context: BTreeMap<String, Value>,
    hooks: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    sandbox: SandboxConfig,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config at {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("invalid TOML in {location}: {message}")]
    TomlParse { location: String, message: String },

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

    #[error("invalid config structure in {location}: {message}")]
    InvalidStructure { location: String, message: String },

    #[error("failed to format resolved config: {message}")]
    Format { message: String },
}

pub fn load_default_config() -> Result<ScHooksConfig, ConfigError> {
    load_config(Path::new(DEFAULT_CONFIG_PATH))
}

pub fn load_config(path: &Path) -> Result<ScHooksConfig, ConfigError> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    parse_config_str(&contents, path.display().to_string())
}

pub fn parse_config_str(
    input: &str,
    source: impl Into<String>,
) -> Result<ScHooksConfig, ConfigError> {
    let source = source.into();
    let value: Value = toml::from_str(input).map_err(|err| ConfigError::TomlParse {
        location: source.clone(),
        message: err.to_string(),
    })?;

    let root = value.as_table().ok_or_else(|| ConfigError::RootNotTable {
        location: source.clone(),
    })?;

    validate_sections(root, &source)?;
    validate_version(root, &source)?;

    let raw: RawConfig = value
        .try_into()
        .map_err(|err| ConfigError::InvalidStructure {
            location: source.clone(),
            message: err.to_string(),
        })?;

    Ok(ScHooksConfig {
        meta: raw.meta,
        context: raw.context,
        hooks: raw.hooks,
        sandbox: raw.sandbox,
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

fn validate_sections(
    root: &toml::map::Map<String, Value>,
    source: &str,
) -> Result<(), ConfigError> {
    for section in REQUIRED_SECTIONS {
        if !root.contains_key(section) {
            return Err(ConfigError::MissingSection {
                location: source.to_string(),
                section,
            });
        }
    }

    let allowed: BTreeSet<&str> = ALLOWED_SECTIONS.into_iter().collect();
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

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn parses_from_in_memory_toml_with_sandbox_defaults() {
        let config =
            parse_config_str(valid_base_config(), "in-memory").expect("config should parse");

        assert_eq!(config.meta.version, 1);
        assert_eq!(config.sandbox, SandboxConfig::default());
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
    fn renders_resolved_config_for_config_subcommand() {
        let parsed = parse_config_str(valid_base_config(), "in-memory")
            .expect("config should parse for render");
        let rendered = parsed
            .to_pretty_toml()
            .expect("resolved config should render to TOML");

        assert!(rendered.contains("[meta]"));
        assert!(rendered.contains("[hooks]"));
        assert!(rendered.contains("[sandbox]"));
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
