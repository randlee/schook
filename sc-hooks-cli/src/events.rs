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

pub fn validate_matchers_for_hook(hook: &str, matchers: &[String]) -> MatcherValidation {
    let mut outcome = MatcherValidation::default();

    for matcher in matchers {
        if is_wildcard_only_hook(hook) && matcher != "*" {
            outcome.errors.push(format!(
                "hook `{hook}` only supports wildcard matcher `*`, found `{matcher}`"
            ));
            continue;
        }

        if hook == "Notification" && !NOTIFICATION_EVENTS.contains(&matcher.as_str()) {
            outcome.warnings.push(format!(
                "hook `{hook}` matcher `{matcher}` is unrecognized (forward-compatible warning)"
            ));
            continue;
        }

        if is_tool_hook(hook) && !TOOL_EVENTS.contains(&matcher.as_str()) {
            outcome.warnings.push(format!(
                "hook `{hook}` matcher `{matcher}` is unrecognized (forward-compatible warning)"
            ));
        }
    }

    outcome
}

pub fn is_tool_hook(hook: &str) -> bool {
    matches!(hook, "PreToolUse" | "PostToolUse")
}

pub fn is_wildcard_only_hook(hook: &str) -> bool {
    matches!(
        hook,
        "PreCompact"
            | "PostCompact"
            | "SessionStart"
            | "SessionEnd"
            | "TeammateIdle"
            | "PermissionRequest"
            | "Stop"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_tool_hook_matchers_with_warning_for_unknown() {
        let result = validate_matchers_for_hook(
            "PreToolUse",
            &["Write".to_string(), "FutureEvent".to_string()],
        );

        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("FutureEvent"));
    }

    #[test]
    fn lifecycle_hooks_require_wildcard_only() {
        let result =
            validate_matchers_for_hook("SessionEnd", &["Write".to_string(), "*".to_string()]);

        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("only supports wildcard"));
    }

    #[test]
    fn notification_allows_idle_prompt_and_warns_unknown() {
        let result = validate_matchers_for_hook(
            "Notification",
            &["idle_prompt".to_string(), "heartbeat".to_string()],
        );

        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("heartbeat"));
    }
}
