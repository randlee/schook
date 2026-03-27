"""Placeholder model registry for Phase 1 scaffold validation."""

EXPECTED_HOOKS = {
    "SessionStart": "session-start",
    "SessionEnd": "session-end",
    "PreCompact": "pre-compact",
    "PreToolUse(Bash)": "pretooluse-bash",
    "PreToolUse(Agent)": "pretooluse-agent",
    "PostToolUse(Bash)": "posttooluse-bash",
    "PermissionRequest": "permission-request",
    "Notification": "notification",
    "Stop": "stop",
}
