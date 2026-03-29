use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use serde_json::{Map, Value};
use tempfile::NamedTempFile;
use toml::Value as TomlValue;

use crate::config::ScHooksConfig;
use crate::errors::CliError;

pub const ENV_HOOK_TYPE: &str = "SC_HOOK_TYPE";
pub const ENV_HOOK_EVENT: &str = "SC_HOOK_EVENT";
pub const ENV_HOOK_METADATA: &str = "SC_HOOK_METADATA";
const ENV_AGENT_TYPE: &str = "SC_HOOK_AGENT_TYPE";
const ENV_SESSION_ID: &str = "SC_HOOK_SESSION_ID";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMetadata {
    pub agent_pid: u32,
    pub agent_type: Option<String>,
    pub session_id: Option<String>,
    pub repo_path: Option<String>,
    pub repo_branch: Option<String>,
    pub working_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookEnv {
    hook_type: String,
    hook_event: Option<String>,
    metadata_path: PathBuf,
}

impl HookEnv {
    fn new(hook_type: &str, event: Option<&str>, metadata_path: PathBuf) -> Self {
        Self {
            hook_type: hook_type.to_string(),
            hook_event: event.map(str::to_string),
            metadata_path,
        }
    }
}

#[derive(Debug)]
pub struct PreparedMetadata {
    pub metadata: Value,
    pub env: HookEnv,
    pub session_id: Option<String>,
    pub project_root: PathBuf,
    // Intentionally retained for drop-on-scope-exit cleanup of SC_HOOK_METADATA temp file.
    _metadata_file: MetadataFileGuard,
}

#[derive(Debug)]
struct MetadataFileGuard {
    _file: NamedTempFile,
    path: PathBuf,
}

impl RuntimeMetadata {
    pub fn discover() -> Result<Self, CliError> {
        let working_dir = std::env::current_dir()
            .map_err(|source| {
                CliError::internal_with_source("failed to resolve current dir", source)
            })?
            .display()
            .to_string();

        Ok(Self {
            agent_pid: std::process::id(),
            agent_type: std::env::var(ENV_AGENT_TYPE).ok(),
            session_id: std::env::var(ENV_SESSION_ID).ok(),
            repo_path: git_output(&["rev-parse", "--show-toplevel"]),
            repo_branch: git_output(&["rev-parse", "--abbrev-ref", "HEAD"]),
            working_dir,
        })
    }
}

pub fn prepare_for_dispatch(
    config: &ScHooksConfig,
    hook: &str,
    event: Option<&str>,
    payload: Option<&Value>,
) -> Result<PreparedMetadata, CliError> {
    let runtime = RuntimeMetadata::discover()?;
    let temp_root = std::env::temp_dir().join("sc-hooks");
    sweep_stale_metadata_files(&temp_root, Duration::from_secs(60 * 60 * 24));
    prepare_with_runtime(config, hook, event, payload, &runtime, &temp_root)
}

pub fn prepare_with_runtime(
    config: &ScHooksConfig,
    hook: &str,
    event: Option<&str>,
    payload: Option<&Value>,
    runtime: &RuntimeMetadata,
    temp_root: &Path,
) -> Result<PreparedMetadata, CliError> {
    let context = config.mapped_context_metadata();
    let metadata = assemble_metadata(runtime, &context, hook, event, payload)?;
    let metadata_file = write_metadata_file(&metadata, temp_root)?;
    let env = HookEnv::new(hook, event, metadata_file.path.clone());
    Ok(PreparedMetadata {
        metadata,
        env,
        session_id: runtime.session_id.clone(),
        project_root: runtime
            .repo_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&runtime.working_dir)),
        _metadata_file: metadata_file,
    })
}

pub fn current_session_id() -> Option<String> {
    std::env::var(ENV_SESSION_ID).ok()
}

pub fn assemble_metadata(
    runtime: &RuntimeMetadata,
    context: &BTreeMap<String, TomlValue>,
    hook: &str,
    event: Option<&str>,
    payload: Option<&Value>,
) -> Result<Value, CliError> {
    let mut root = Map::new();

    let mut agent = Map::new();
    agent.insert("pid".to_string(), Value::from(runtime.agent_pid));
    if let Some(agent_type) = runtime.agent_type.as_ref() {
        agent.insert("type".to_string(), Value::String(agent_type.clone()));
    }
    if let Some(session_id) = runtime.session_id.as_ref() {
        agent.insert("session_id".to_string(), Value::String(session_id.clone()));
    }
    root.insert("agent".to_string(), Value::Object(agent));

    let mut repo = Map::new();
    if let Some(path) = runtime.repo_path.as_ref() {
        repo.insert("path".to_string(), Value::String(path.clone()));
    }
    if let Some(branch) = runtime.repo_branch.as_ref() {
        repo.insert("branch".to_string(), Value::String(branch.clone()));
    }
    repo.insert(
        "working_dir".to_string(),
        Value::String(runtime.working_dir.clone()),
    );
    root.insert("repo".to_string(), Value::Object(repo));

    for (key, value) in context {
        root.insert(key.clone(), toml_value_to_json(value)?);
    }

    let mut hook_metadata = Map::new();
    hook_metadata.insert("type".to_string(), Value::String(hook.to_string()));
    if let Some(event) = event {
        hook_metadata.insert("event".to_string(), Value::String(event.to_string()));
    }
    root.insert("hook".to_string(), Value::Object(hook_metadata));

    if let Some(payload) = payload {
        root.insert("payload".to_string(), payload.clone());
    }

    Ok(Value::Object(root))
}

pub fn inject_env_vars(command: &mut Command, env: &HookEnv) {
    command
        .env(ENV_HOOK_TYPE, &env.hook_type)
        .env(ENV_HOOK_METADATA, &env.metadata_path);
    if let Some(event) = env.hook_event.as_ref() {
        command.env(ENV_HOOK_EVENT, event);
    }
}

fn write_metadata_file(metadata: &Value, temp_root: &Path) -> Result<MetadataFileGuard, CliError> {
    fs::create_dir_all(temp_root).map_err(|source| {
        CliError::internal_with_source(
            format!(
                "failed to create metadata temp directory {}",
                temp_root.display()
            ),
            source,
        )
    })?;

    let bytes = serde_json::to_vec(metadata).map_err(|source| {
        CliError::internal_with_source("failed to serialize metadata JSON", source)
    })?;
    let mut file = NamedTempFile::new_in(temp_root).map_err(|source| {
        CliError::internal_with_source(
            format!("failed to create metadata file in {}", temp_root.display()),
            source,
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        file.as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|source| {
                CliError::internal_with_source(
                    format!("failed to secure metadata file {}", file.path().display()),
                    source,
                )
            })?;
    }
    use std::io::Write;
    file.write_all(&bytes).map_err(|source| {
        CliError::internal_with_source(
            format!("failed to write metadata file {}", file.path().display()),
            source,
        )
    })?;
    file.flush().map_err(|source| {
        CliError::internal_with_source(
            format!("failed to flush metadata file {}", file.path().display()),
            source,
        )
    })?;

    Ok(MetadataFileGuard {
        path: file.path().to_path_buf(),
        _file: file,
    })
}

fn sweep_stale_metadata_files(temp_root: &Path, max_age: Duration) {
    let Ok(entries) = fs::read_dir(temp_root) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !name.starts_with(".tmp") && !name.starts_with("meta-") {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        let Ok(age) = now.duration_since(modified) else {
            continue;
        };
        if age >= max_age {
            let _ = fs::remove_file(path);
        }
    }
}

fn toml_value_to_json(value: &TomlValue) -> Result<Value, CliError> {
    serde_json::to_value(value).map_err(|source| {
        CliError::internal_with_source("failed converting TOML value to JSON", source)
    })
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::test_support;

    #[test]
    fn assembles_metadata_with_injected_runtime_and_context() {
        let mut context = BTreeMap::new();
        let mut team = toml::map::Map::new();
        team.insert(
            "name".to_string(),
            TomlValue::String("calibration".to_string()),
        );
        context.insert("team".to_string(), TomlValue::Table(team));
        context.insert(
            "project".to_string(),
            TomlValue::String("p3-platform".to_string()),
        );

        let runtime = RuntimeMetadata {
            agent_pid: 42,
            agent_type: Some("codex".to_string()),
            session_id: Some("abc123".to_string()),
            repo_path: Some("/repo".to_string()),
            repo_branch: Some("feature/s2".to_string()),
            working_dir: "/repo/subdir".to_string(),
        };
        let payload = serde_json::json!({"tool_input":{"command":"Write"}} );
        let metadata = assemble_metadata(
            &runtime,
            &context,
            "PreToolUse",
            Some("Write"),
            Some(&payload),
        )
        .expect("metadata should assemble");

        assert_eq!(metadata["agent"]["pid"], Value::from(42_u32));
        assert_eq!(
            metadata["agent"]["type"],
            Value::String("codex".to_string())
        );
        assert_eq!(
            metadata["agent"]["session_id"],
            Value::String("abc123".to_string())
        );
        assert_eq!(metadata["repo"]["path"], Value::String("/repo".to_string()));
        assert_eq!(
            metadata["repo"]["branch"],
            Value::String("feature/s2".to_string())
        );
        assert_eq!(
            metadata["repo"]["working_dir"],
            Value::String("/repo/subdir".to_string())
        );
        assert_eq!(
            metadata["team"]["name"],
            Value::String("calibration".to_string())
        );
        assert_eq!(
            metadata["project"],
            Value::String("p3-platform".to_string())
        );
        assert_eq!(
            metadata["hook"]["type"],
            Value::String("PreToolUse".to_string())
        );
        assert_eq!(
            metadata["hook"]["event"],
            Value::String("Write".to_string())
        );
        assert_eq!(metadata["payload"], payload);
    }

    #[test]
    fn prepares_env_and_cleans_metadata_file_after_drop() {
        let _guard = test_support::cwd_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp = tempfile::tempdir().expect("tempdir should create");
        let config = config::parse_config_str(
            r#"
[meta]
version = 1

[context]
team = "calibration"

[hooks]
PreToolUse = ["guard-paths"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let runtime = RuntimeMetadata {
            agent_pid: 11,
            agent_type: Some("codex".to_string()),
            session_id: Some("session-1".to_string()),
            repo_path: Some("/repo".to_string()),
            repo_branch: Some("feature/s2".to_string()),
            working_dir: "/repo".to_string(),
        };

        let prepared = prepare_with_runtime(
            &config,
            "PreToolUse",
            Some("Write"),
            None,
            &runtime,
            temp.path(),
        )
        .expect("metadata should prepare");

        assert!(prepared.env.metadata_path.exists());

        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg("printf '%s|%s|%s' \"$SC_HOOK_TYPE\" \"$SC_HOOK_EVENT\" \"$SC_HOOK_METADATA\"");
        inject_env_vars(&mut command, &prepared.env);
        let output = command.output().expect("command should execute");
        assert!(output.status.success());
        let rendered = String::from_utf8(output.stdout).expect("stdout should be utf8");
        let expected = format!("PreToolUse|Write|{}", prepared.env.metadata_path.display());
        assert_eq!(rendered, expected);

        let metadata_path = prepared.env.metadata_path.clone();
        drop(prepared);
        assert!(!metadata_path.exists());
    }

    #[test]
    fn omits_sc_hook_event_when_event_is_absent() {
        let _guard = test_support::cwd_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp = tempfile::tempdir().expect("tempdir should create");
        let config = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths"]
"#,
            "in-memory",
        )
        .expect("config should parse");
        let runtime = RuntimeMetadata {
            agent_pid: 11,
            agent_type: None,
            session_id: None,
            repo_path: None,
            repo_branch: None,
            working_dir: "/repo".to_string(),
        };

        let prepared =
            prepare_with_runtime(&config, "PreToolUse", None, None, &runtime, temp.path())
                .expect("metadata should prepare");

        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg("if [ -z \"${SC_HOOK_EVENT+x}\" ]; then printf 'unset'; else printf 'set'; fi");
        inject_env_vars(&mut command, &prepared.env);
        let output = command.output().expect("command should execute");
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).expect("stdout should be utf8"),
            "unset"
        );
    }

    #[cfg(unix)]
    #[test]
    fn metadata_file_is_owner_only_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = test_support::cwd_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp = tempfile::tempdir().expect("tempdir should create");
        let config = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths"]
"#,
            "in-memory",
        )
        .expect("config should parse");
        let runtime = RuntimeMetadata {
            agent_pid: 11,
            agent_type: None,
            session_id: None,
            repo_path: None,
            repo_branch: None,
            working_dir: "/repo".to_string(),
        };

        let prepared =
            prepare_with_runtime(&config, "PreToolUse", None, None, &runtime, temp.path())
                .expect("metadata should prepare");
        let mode = fs::metadata(&prepared.env.metadata_path)
            .expect("metadata should exist")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn stale_metadata_files_are_swept_before_dispatch() {
        let _guard = test_support::cwd_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp = tempfile::tempdir().expect("tempdir should create");
        let stale = temp.path().join("meta-stale.json");
        fs::write(&stale, "{}").expect("stale file should be writable");

        sweep_stale_metadata_files(temp.path(), Duration::ZERO);

        assert!(!stale.exists());
    }
}
