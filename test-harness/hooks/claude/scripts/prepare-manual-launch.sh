#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
HOOK_DIR="${CLAUDE_DIR}/hooks"
CAPTURE_ROOT="${SCHOOK_HOOK_CAPTURE_ROOT:-${CLAUDE_DIR}/captures/raw}"
MODEL="${CLAUDE_MODEL:-haiku}"

mkdir -p "${CAPTURE_ROOT}"

tmp_dir="${TMPDIR:-/tmp}"
temp_settings_base="$(mktemp "${tmp_dir%/}/schook-claude-settings.XXXXXX")"
temp_settings="${temp_settings_base}.json"
mv "${temp_settings_base}" "${temp_settings}"

cat > "${temp_settings}" <<JSON
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "*",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/session_start.py'" }]
      }
    ],
    "SessionEnd": [
      {
        "matcher": "*",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/session_end.py'" }]
      }
    ],
    "PreCompact": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/pre_compact.py'" }]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/pre_tool_use_bash.py'" }]
      },
      {
        "matcher": "Agent",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/pre_tool_use_agent.py'" }]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/post_tool_use_bash.py'" }]
      }
    ],
    "PermissionRequest": [
      {
        "matcher": "*",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/permission_request.py'" }]
      }
    ],
    "Notification": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/notification.py'" }]
      }
    ],
    "Stop": [
      {
        "matcher": "*",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/stop.py'" }]
      }
    ]
  }
}
JSON

cat <<EOF
Manual Claude harness launch is ready.

Worktree root:
  $(cd "${CLAUDE_DIR}/../../.." && pwd)

Capture root:
  ${CAPTURE_ROOT}

Settings file:
  ${temp_settings}

Run Claude manually with:
  cd $(cd "${CLAUDE_DIR}/../../.." && pwd)
  CLAUDE_MODEL=${MODEL} claude --model ${MODEL} --setting-sources local --permission-mode default --settings ${temp_settings}

When you are done, remove the settings file:
  rm -f ${temp_settings}
EOF
