"""Approved Codex hook-to-fixture mapping for harness validation."""

EXPECTED_HOOKS = {
    "SessionStart": "session-start-startup",
    "SessionStart (--cd)": "session-start-cwd-drift",
    "PreToolUse": "pretooluse-bash",
    "PreToolUse (--cd)": "pretooluse-bash-cwd-drift",
    "notify": "notify-agent-turn-complete",
    "notify (--cd)": "notify-agent-turn-complete-cwd-drift",
}
