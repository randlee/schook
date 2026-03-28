#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
HOOK_DIR="${CLAUDE_DIR}/hooks"
PROMPT_DIR="${CLAUDE_DIR}/prompts"
CAPTURE_ROOT="${SCHOOK_HOOK_CAPTURE_ROOT:-${CLAUDE_DIR}/captures/raw}"
MODEL="${CLAUDE_MODEL:-haiku}"

surface="${1:-pretooluse-bash}"
prompt_file="${PROMPT_DIR}/${surface}.md"

if [[ ! -f "${prompt_file}" ]]; then
  echo "unknown prompt surface: ${surface}" >&2
  exit 1
fi

permission_mode="bypassPermissions"
if [[ "${surface}" == "permission-request" ]]; then
  exec "${SCRIPT_DIR}/run-interactive-capture.py" permission-request --model "${MODEL}" --capture-root "${CAPTURE_ROOT}"
fi

if [[ "${surface}" == "notification" ]]; then
  exec "${SCRIPT_DIR}/run-interactive-capture.py" notification --model "${MODEL}" --capture-root "${CAPTURE_ROOT}"
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

mkdir -p "${CAPTURE_ROOT}"

claude --model "${MODEL}" -p --setting-sources local --permission-mode "${permission_mode}" --settings "${temp_settings}" "$(cat "${prompt_file}")"
