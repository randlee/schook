"""Placeholder model registry for Phase 1 scaffold validation."""

EXPECTED_HOOKS = {
    "SessionStart": "session-start",
    "SessionEnd": "session-end",
    "PreToolUse(Bash)": "pretooluse-bash",
    "PostToolUse(Bash)": "posttooluse-bash",
    "PreToolUse(Agent)": "pretooluse-agent",
    "PermissionRequest": "permission-request",
    "Stop": "stop",
}
