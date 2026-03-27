#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
HOOK_DIR="${CLAUDE_DIR}/hooks"
PROMPT_DIR="${CLAUDE_DIR}/prompts"
CAPTURE_ROOT="${CLAUDE_DIR}/captures/raw"

surface="${1:-pretooluse-bash}"
prompt_file="${PROMPT_DIR}/${surface}.md"

if [[ ! -f "${prompt_file}" ]]; then
  echo "unknown prompt surface: ${surface}" >&2
  exit 1
fi

temp_settings="$(mktemp)"
trap 'rm -f "${temp_settings}"' EXIT

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
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/pre_tool_use_bash.py'" }]
      },
      {
        "matcher": "Task",
        "hooks": [{ "type": "command", "command": "SCHOOK_HOOK_CAPTURE_ROOT='${CAPTURE_ROOT}' python3 '${HOOK_DIR}/pre_tool_use_task.py'" }]
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
        "matcher": "idle_prompt",
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

mkdir -p "${CAPTURE_ROOT}"

claude --print --settings "${temp_settings}" "$(cat "${prompt_file}")"
