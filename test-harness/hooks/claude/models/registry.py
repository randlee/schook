"""Placeholder model registry for Phase 1 scaffold validation."""

EXPECTED_HOOKS = {
    "SessionStart": "session-start",
    "SessionEnd": "session-end",
    "PreToolUse(Bash)": "pretooluse-bash",
    "PostToolUse(Bash)": "posttooluse-bash",
    "PreToolUse(Task)": "pretooluse-task",
    "PermissionRequest": "permission-request",
    "Notification(idle_prompt)": "notification",
    "Stop": "stop",
}
