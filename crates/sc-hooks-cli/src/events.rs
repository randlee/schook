use std::str::FromStr;

use sc_hooks_core::events::{EventTaxonomy, HookType};
use sc_hooks_core::manifest::ManifestMatcher;

pub const TOOL_EVENTS: [&str; 15] = [
    "Bash",
    "Read",
    "Write",
    "Edit",
    "Glob",
    "Grep",
    "WebFetch",
    "WebSearch",
    "Agent",
    "NotebookEdit",
    "TodoWrite",
    "AskFollowup",
    "SendMessage",
    "Task",
    "*",
];
pub const NOTIFICATION_EVENTS: [&str; 2] = ["idle_prompt", "*"];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MatcherValidation {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub fn validate_matchers_for_hook(
    hook: HookType,
    matchers: &[ManifestMatcher],
) -> MatcherValidation {
    let mut outcome = MatcherValidation::default();

    for matcher in matchers {
        let matcher_name = matcher.as_str();
        if is_wildcard_only_hook_type(hook) && matcher_name != "*" {
            outcome.errors.push(format!(
                "hook `{}` only supports wildcard matcher `*`, found `{matcher_name}`",
                hook.as_str()
            ));
            continue;
        }

        if matches!(hook, HookType::Notification) && !NOTIFICATION_EVENTS.contains(&matcher_name) {
            outcome.warnings.push(format!(
                "hook `{}` matcher `{matcher_name}` is unrecognized (forward-compatible warning)",
                hook.as_str()
            ));
            continue;
        }

        if is_tool_hook_type(hook) && !TOOL_EVENTS.contains(&matcher_name) {
            outcome.warnings.push(format!(
                "hook `{}` matcher `{matcher_name}` is unrecognized (forward-compatible warning)",
                hook.as_str()
            ));
        }
    }

    outcome
}

#[expect(
    dead_code,
    reason = "CLI matcher helpers remain exported even when only exercised by tests or downstream tooling"
)]
pub fn is_tool_hook(hook: &str) -> bool {
    HookType::from_str(hook).is_ok_and(is_tool_hook_type)
}

#[expect(
    dead_code,
    reason = "CLI matcher helpers remain exported even when only exercised by tests or downstream tooling"
)]
pub fn is_wildcard_only_hook(hook: &str) -> bool {
    HookType::from_str(hook).is_ok_and(is_wildcard_only_hook_type)
}

fn is_tool_hook_type(hook: HookType) -> bool {
    matches!(hook, HookType::PreToolUse | HookType::PostToolUse)
}

fn is_wildcard_only_hook_type(hook: HookType) -> bool {
    matches!(
        hook,
        HookType::PreCompact
            | HookType::PostCompact
            | HookType::SessionStart
            | HookType::SessionEnd
            | HookType::TeammateIdle
            | HookType::SubagentStop
            | HookType::PermissionRequest
            | HookType::WorktreeCreate
            | HookType::WorktreeRemove
            | HookType::Stop
    )
}

pub fn canonical_events_for_hook(hook: &str) -> Vec<&'static str> {
    match HookType::from_str(hook) {
        Ok(HookType::PreToolUse | HookType::PostToolUse) => TOOL_EVENTS.to_vec(),
        Ok(HookType::Notification) => NOTIFICATION_EVENTS.to_vec(),
        Ok(
            HookType::PreCompact
            | HookType::PostCompact
            | HookType::SessionStart
            | HookType::SessionEnd
            | HookType::TeammateIdle
            | HookType::SubagentStop
            | HookType::PermissionRequest
            | HookType::WorktreeCreate
            | HookType::WorktreeRemove
            | HookType::Stop,
        ) => vec![EventTaxonomy::Wildcard.as_str()],
        // Unknown future hooks intentionally degrade to wildcard-only guidance so
        // the CLI remains forward-compatible until the provider surface is reviewed.
        Ok(_) | Err(_) => vec![EventTaxonomy::Wildcard.as_str()],
    }
}

pub fn canonical_taxonomy() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("PreToolUse", canonical_events_for_hook("PreToolUse")),
        ("PostToolUse", canonical_events_for_hook("PostToolUse")),
        ("PreCompact", canonical_events_for_hook("PreCompact")),
        ("PostCompact", canonical_events_for_hook("PostCompact")),
        ("SessionStart", canonical_events_for_hook("SessionStart")),
        ("SessionEnd", canonical_events_for_hook("SessionEnd")),
        ("Notification", canonical_events_for_hook("Notification")),
        ("TeammateIdle", canonical_events_for_hook("TeammateIdle")),
        ("SubagentStop", canonical_events_for_hook("SubagentStop")),
        (
            "PermissionRequest",
            canonical_events_for_hook("PermissionRequest"),
        ),
        (
            "WorktreeCreate",
            canonical_events_for_hook("WorktreeCreate"),
        ),
        (
            "WorktreeRemove",
            canonical_events_for_hook("WorktreeRemove"),
        ),
        ("Stop", canonical_events_for_hook("Stop")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_tool_hook_matchers_with_warning_for_unknown() {
        let result = validate_matchers_for_hook(
            HookType::PreToolUse,
            &[
                ManifestMatcher::from("Write"),
                ManifestMatcher::from("FutureEvent"),
            ],
        );

        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("FutureEvent"));
    }

    #[test]
    fn lifecycle_hooks_require_wildcard_only() {
        let result = validate_matchers_for_hook(
            HookType::SessionEnd,
            &[ManifestMatcher::from("Write"), ManifestMatcher::from("*")],
        );

        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("only supports wildcard"));
    }

    #[test]
    fn notification_allows_idle_prompt_and_warns_unknown() {
        let result = validate_matchers_for_hook(
            HookType::Notification,
            &[
                ManifestMatcher::from("idle_prompt"),
                ManifestMatcher::from("heartbeat"),
            ],
        );

        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("heartbeat"));
    }

    #[test]
    fn taxonomy_exposes_known_hooks_and_events() {
        let taxonomy = canonical_taxonomy();
        assert!(taxonomy.iter().any(|(hook, _)| *hook == "PreToolUse"));
        let notification = taxonomy
            .iter()
            .find(|(hook, _)| *hook == "Notification")
            .expect("notification hook should exist");
        assert!(notification.1.contains(&"idle_prompt"));
    }
}
