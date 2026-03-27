use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use atm_session_lifecycle::{SessionRecord, load_session_record};
use regex::Regex;
use sc_hooks_core::dispatch::DispatchMode;
use sc_hooks_core::manifest::Manifest;
use sc_hooks_sdk::manifest::ManifestBuilder;
use sc_hooks_sdk::result::{HookResult, block, proceed};
use sc_hooks_sdk::traits::{ManifestProvider, SyncHandler};
use serde::Deserialize;
use serde_json::Value;

const ENV_ATM_HOME: &str = "ATM_HOME";
const ENV_ATM_IDENTITY: &str = "ATM_IDENTITY";

const SPAWN_POLICY_NAMED_REQUIRED: &str = "named_teammate_required";
const SPAWN_POLICY_LEADERS_ONLY: &str = "leaders-only";
const SPAWN_POLICY_ANY_MEMBER: &str = "any-member";

pub struct GateAgentSpawns;

impl ManifestProvider for GateAgentSpawns {
    fn manifest(&self) -> Manifest {
        ManifestBuilder::new("gate-agent-spawns", DispatchMode::Sync)
            .hooks(["PreToolUse"])
            .matchers(["Task"])
            .build()
            .expect("manifest should be valid")
    }
}

impl SyncHandler for GateAgentSpawns {
    fn handle(&self, input: Value) -> Result<HookResult, String> {
        Ok(match evaluate(&input) {
            Ok(outcome) => outcome,
            Err(err) => {
                eprintln!("[atm-hook] {err}");
                proceed()
            }
        })
    }
}

fn evaluate(input: &Value) -> Result<HookResult, String> {
    let parsed = ParsedInput::from_json(input);
    let session = parsed
        .session_id
        .as_deref()
        .map(load_session_record)
        .transpose()?
        .flatten();
    let policy = load_spawn_policy()?;

    if policy.required_team.is_none() && session.is_none() && env_identity().is_none() {
        return Ok(proceed());
    }

    if requires_named_teammate(&parsed.subagent_type)? && parsed.teammate_name.is_none() {
        return Ok(block(format!(
            "'{}' requires named teammate spawn policy",
            parsed
                .subagent_type
                .unwrap_or_else(|| "unknown-agent".to_string())
        )));
    }

    if let (Some(team_name), Some(required_team)) =
        (parsed.team_name.as_deref(), policy.required_team.as_deref())
        && team_name != required_team
    {
        return Ok(block(format!(
            "team_name must match .atm.toml core.default_team (required '{required_team}', got '{team_name}')"
        )));
    }

    let spawn_capable = parsed.team_name.is_some() || parsed.teammate_name.is_some();
    if spawn_capable
        && policy.spawn_policy != SPAWN_POLICY_ANY_MEMBER
        && let Some(required_team) = policy.required_team.as_deref()
        && let Some(team_config) = load_team_config(required_team)?
        && team_config.lead_session_id.as_deref().is_some()
    {
        let caller = resolve_caller_identity(&parsed, session.as_ref(), &team_config);
        let mut allowed = policy.co_leaders.clone();
        allowed.push("team-lead".to_string());
        if caller
            .as_deref()
            .is_none_or(|identity| !allowed.iter().any(|allowed_id| allowed_id == identity))
        {
            return Ok(block(format!(
                "leaders-only spawn policy violation for team '{required_team}'"
            )));
        }
    }

    Ok(proceed())
}

#[derive(Debug, Clone, Default)]
struct ParsedInput {
    session_id: Option<String>,
    subagent_type: Option<String>,
    teammate_name: Option<String>,
    team_name: Option<String>,
}

impl ParsedInput {
    fn from_json(input: &Value) -> Self {
        let payload = input.get("payload").unwrap_or(&Value::Null);
        let tool_input = payload.get("tool_input").unwrap_or(&Value::Null);

        Self {
            session_id: payload
                .get("session_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            subagent_type: tool_input
                .get("subagent_type")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            teammate_name: tool_input
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            team_name: tool_input
                .get("team_name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        }
    }
}

#[derive(Debug, Clone)]
struct SpawnPolicy {
    required_team: Option<String>,
    spawn_policy: String,
    co_leaders: Vec<String>,
}

fn load_spawn_policy() -> Result<SpawnPolicy, String> {
    let toml = read_atm_toml();
    let core = toml
        .as_ref()
        .and_then(|value| value.get("core"))
        .and_then(toml::Value::as_table);
    let required_team = core
        .and_then(|table| table.get("default_team"))
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let (spawn_policy, co_leaders) =
        if let (Some(toml), Some(required_team)) = (toml.as_ref(), required_team.as_deref()) {
            let team_cfg = toml.get("team").and_then(toml::Value::as_table);
            let team_entry = team_cfg.and_then(|table| table.get(required_team));
            let team_entry = team_entry.and_then(toml::Value::as_table);

            let spawn_policy = team_entry
                .and_then(|table| table.get("spawn_policy"))
                .and_then(toml::Value::as_str)
                .filter(|value| {
                    *value == SPAWN_POLICY_LEADERS_ONLY || *value == SPAWN_POLICY_ANY_MEMBER
                })
                .unwrap_or(SPAWN_POLICY_LEADERS_ONLY)
                .to_string();
            let co_leaders = team_entry
                .and_then(|table| table.get("co_leaders"))
                .and_then(toml::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(toml::Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            (spawn_policy, co_leaders)
        } else {
            (SPAWN_POLICY_LEADERS_ONLY.to_string(), Vec::new())
        };

    Ok(SpawnPolicy {
        required_team,
        spawn_policy,
        co_leaders,
    })
}

#[derive(Debug, Deserialize)]
struct TeamConfig {
    #[serde(rename = "leadSessionId")]
    lead_session_id: Option<String>,
    #[serde(default)]
    members: Vec<MemberRecord>,
}

#[derive(Debug, Deserialize)]
struct MemberRecord {
    name: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

fn load_team_config(team_name: &str) -> Result<Option<TeamConfig>, String> {
    let path = atm_home()
        .join(".claude")
        .join("teams")
        .join(team_name)
        .join("config.json");
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read team config {}: {err}", path.display()))?;
    let config = serde_json::from_str::<TeamConfig>(&content)
        .map_err(|err| format!("failed to parse team config {}: {err}", path.display()))?;
    Ok(Some(config))
}

fn resolve_caller_identity(
    parsed: &ParsedInput,
    session: Option<&SessionRecord>,
    team_config: &TeamConfig,
) -> Option<String> {
    if let Some(identity) = env_identity() {
        return Some(identity);
    }
    if let Some(identity) = session.and_then(|record| record.identity.clone()) {
        return Some(identity);
    }

    let session_id = parsed.session_id.as_deref()?;
    if team_config.lead_session_id.as_deref() == Some(session_id) {
        return Some("team-lead".to_string());
    }

    team_config
        .members
        .iter()
        .find(|member| member.session_id.as_deref() == Some(session_id))
        .and_then(|member| member.name.clone())
}

fn requires_named_teammate(subagent_type: &Option<String>) -> Result<bool, String> {
    let Some(subagent_type) = subagent_type.as_deref() else {
        return Ok(false);
    };
    let Some(agent_file) = agent_file_for(subagent_type) else {
        return Ok(false);
    };
    if !agent_file.exists() {
        return Ok(false);
    }

    let body = fs::read_to_string(&agent_file)
        .map_err(|err| format!("failed to read agent file {}: {err}", agent_file.display()))?;
    let Some(frontmatter) = extract_frontmatter(&body) else {
        return Ok(false);
    };

    Ok(frontmatter_requires_named_teammate(frontmatter))
}

fn agent_file_for(subagent_type: &str) -> Option<PathBuf> {
    let base = env::var_os("CLAUDE_PROJECT_DIR")
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())?;
    Some(
        base.join(".claude")
            .join("agents")
            .join(format!("{subagent_type}.md")),
    )
}

fn extract_frontmatter(text: &str) -> Option<&str> {
    if !text.starts_with("---\n") {
        return None;
    }
    let end = text[4..].find("\n---")?;
    Some(&text[4..4 + end])
}

fn frontmatter_requires_named_teammate(frontmatter: &str) -> bool {
    let direct = Regex::new(r"(?m)^metadata:\n(?:^[ \t].*\n)*?^[ \t]+spawn_policy:\s*([^\n#]+)")
        .expect("regex should compile");
    if direct
        .captures(frontmatter)
        .and_then(|captures| captures.get(1))
        .is_some_and(|value| {
            value
                .as_str()
                .trim()
                .trim_matches(|ch| ch == '"' || ch == '\'')
                == SPAWN_POLICY_NAMED_REQUIRED
        })
    {
        return true;
    }

    let nested = Regex::new(
        r"(?m)^metadata:\n(?:^[ \t].*\n)*?^[ \t]+atm:\n(?:^[ \t]{4,}.*\n)*?^[ \t]{4,}spawn_policy:\s*([^\n#]+)",
    )
    .expect("regex should compile");
    nested
        .captures(frontmatter)
        .and_then(|captures| captures.get(1))
        .is_some_and(|value| {
            value
                .as_str()
                .trim()
                .trim_matches(|ch| ch == '"' || ch == '\'')
                == SPAWN_POLICY_NAMED_REQUIRED
        })
}

fn read_atm_toml() -> Option<toml::Value> {
    let path = Path::new(".atm.toml");
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

fn atm_home() -> PathBuf {
    env::var_os(ENV_ATM_HOME)
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn env_identity() -> Option<String> {
    env::var(ENV_ATM_IDENTITY)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_detects_direct_spawn_policy() {
        let frontmatter = "metadata:\n  spawn_policy: named_teammate_required\n";
        assert!(frontmatter_requires_named_teammate(frontmatter));
    }

    #[test]
    fn frontmatter_detects_nested_spawn_policy() {
        let frontmatter = "metadata:\n  atm:\n    spawn_policy: named_teammate_required\n";
        assert!(frontmatter_requires_named_teammate(frontmatter));
    }
}
