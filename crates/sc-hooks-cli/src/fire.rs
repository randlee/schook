use serde_json::Value;
use std::str::FromStr;

use crate::config::ScHooksConfig;
use crate::dispatch;
use crate::errors::CliError;
use crate::metadata;
use crate::resolution;
use crate::session;
use sc_hooks_core::events::HookType;

pub fn run_fire(
    config: &ScHooksConfig,
    hook: &str,
    event: Option<&str>,
    payload: Option<&Value>,
) -> Result<String, CliError> {
    let hook_type = HookType::from_str(hook)
        .map_err(|_| CliError::internal(format!("unknown hook type `{hook}`")))?;
    let session_id = metadata::current_session_id();
    let disabled_plugins = session::load_disabled_plugins(
        session_id
            .as_ref()
            .map(sc_hooks_core::session::SessionId::as_str),
    )?;
    let handlers = resolution::resolve_chain(
        config,
        hook_type,
        event,
        sc_hooks_core::dispatch::DispatchMode::Sync,
        payload,
        None,
        &disabled_plugins,
    )?;

    if handlers.is_empty() {
        return Ok("no handlers matched".to_string());
    }

    match dispatch::execute_chain(
        &handlers,
        config,
        hook_type,
        event,
        sc_hooks_core::dispatch::DispatchMode::Sync,
        payload,
    )? {
        dispatch::DispatchOutcome::Proceed => Ok(format!(
            "fire completed: hook={} event={} handlers={} result=proceed",
            hook,
            event.unwrap_or("<none>"),
            handlers.len()
        )),
        dispatch::DispatchOutcome::Blocked { reason } => Ok(format!(
            "fire completed: hook={} event={} handlers={} result=blocked reason={}",
            hook,
            event.unwrap_or("<none>"),
            handlers.len(),
            reason
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::test_support;
    use std::path::Path;
    use std::time::{Duration, Instant};

    #[test]
    fn fire_returns_no_handlers_when_chain_missing() {
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

        let response =
            run_fire(&cfg, "PostToolUse", Some("Write"), None).expect("fire should execute");
        assert_eq!(response, "no handlers matched");
    }

    #[test]
    fn zero_match_fast_path_is_under_two_ms_and_writes_no_log() {
        let temp = tempfile::tempdir().expect("tempdir should create");
        let _cwd = test_support::scoped_current_dir(temp.path());

        let cfg = config::parse_config_str(
            r#"
[meta]
version = 1

[hooks]
PostToolUse = ["guard-paths"]
"#,
            "in-memory",
        )
        .expect("config should parse");

        let started = Instant::now();
        let response =
            run_fire(&cfg, "PreToolUse", Some("Write"), None).expect("fire should execute");
        let elapsed = started.elapsed();
        assert_eq!(response, "no handlers matched");
        assert!(
            elapsed < Duration::from_millis(2),
            "zero-match fire elapsed {elapsed:?} exceeded 2ms target"
        );
        assert!(
            !Path::new(".sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl").exists(),
            "zero-match path should not write observability logs"
        );
    }
}
