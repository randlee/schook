use serde_json::Value;

use crate::config::ScHooksConfig;
use crate::dispatch;
use crate::errors::CliError;
use crate::metadata;
use crate::resolution;
use crate::session;

pub fn run_fire(
    config: &ScHooksConfig,
    hook: &str,
    event: Option<&str>,
    payload: Option<&Value>,
) -> Result<String, CliError> {
    let session_id = metadata::current_session_id();
    let disabled_plugins = session::load_disabled_plugins(session_id.as_deref());
    let handlers = resolution::resolve_chain(
        config,
        hook,
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
        hook,
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

    #[test]
    fn fire_returns_no_handlers_when_chain_missing() {
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

        let response =
            run_fire(&cfg, "PostToolUse", Some("Write"), None).expect("fire should execute");
        assert_eq!(response, "no handlers matched");
    }
}
