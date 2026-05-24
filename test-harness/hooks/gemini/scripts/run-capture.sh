#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <lifecycle|before-agent|tool|resume|format> [format]" >&2
  exit 2
fi

surface="$1"
shift || true

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
gemini_root="$repo_root/test-harness/hooks/gemini"
capture_root="$gemini_root/captures/raw"

scratch_root="$(mktemp -d "${TMPDIR:-/tmp}/schook-gemini-capture.XXXXXX")"
trap 'rm -rf "$scratch_root"' EXIT

mkdir -p "$scratch_root/home/.gemini"
cp "$HOME/.gemini/oauth_creds.json" "$scratch_root/home/.gemini/"
cp "$HOME/.gemini/google_accounts.json" "$scratch_root/home/.gemini/"
cp "$HOME/.gemini/projects.json" "$scratch_root/home/.gemini/" 2>/dev/null || true
cp "$HOME/.gemini/trustedFolders.json" "$scratch_root/home/.gemini/" 2>/dev/null || true

hook_command() {
  local script_name="$1"
  printf 'PYTHONPATH=%q SCHOOK_HOOK_CAPTURE_ROOT=%q python3 %q' \
    "$repo_root" \
    "$capture_root" \
    "$gemini_root/hooks/$script_name"
}

write_settings() {
  cat > "$scratch_root/home/.gemini/settings.json" <<JSON
{
  "hooks": {
    "SessionStart": [{"hooks": [{"type": "command", "name": "capture-session-start", "command": "$(hook_command session_start.py)"}]}],
    "SessionEnd": [{"hooks": [{"type": "command", "name": "capture-session-end", "command": "$(hook_command session_end.py)"}]}],
    "BeforeAgent": [{"hooks": [{"type": "command", "name": "capture-before-agent", "command": "$(hook_command before_agent.py)"}]}],
    "BeforeTool": [{"hooks": [{"type": "command", "name": "capture-before-tool", "command": "$(hook_command before_tool.py)"}]}],
    "AfterTool": [{"hooks": [{"type": "command", "name": "capture-after-tool", "command": "$(hook_command after_tool.py)"}]}],
    "AfterAgent": [{"hooks": [{"type": "command", "name": "capture-after-agent", "command": "$(hook_command after_agent.py)"}]}]
  }
}
JSON
}

write_settings
cd "$repo_root"

case "$surface" in
  lifecycle|before-agent)
    HOME="$scratch_root/home" gemini --skip-trust -p "Reply with exactly OK."
    ;;
  tool)
    HOME="$scratch_root/home" gemini --skip-trust --yolo -p "Run pwd using a shell command, then reply with exactly DONE."
    ;;
  resume)
    HOME="$scratch_root/home" gemini --skip-trust -p "Reply with exactly ONE." >/dev/null
    HOME="$scratch_root/home" gemini --skip-trust --resume latest -p "Reply with exactly TWO."
    ;;
  format)
    if [[ $# -ne 1 ]]; then
      echo "usage: $0 format <text|json|stream-json>" >&2
      exit 2
    fi
    HOME="$scratch_root/home" gemini --skip-trust --output-format "$1" -p "Reply with exactly OK."
    ;;
  *)
    echo "unknown surface: $surface" >&2
    exit 2
    ;;
esac
