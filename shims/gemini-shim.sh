#!/usr/bin/env bash
set -eu

export SC_HOOK_AGENT_TYPE=gemini
export SC_HOOK_SESSION_ID="${GEMINI_SESSION_ID:-unknown}"
export SC_HOOK_AGENT_PID=$$

case "${1:-}" in
  pre-tool|pre-edit)
    exec sc-hooks run PreToolUse Write
    ;;
  post-tool|post-edit)
    exec sc-hooks run PostToolUse Write
    ;;
  *)
    exec sc-hooks run "$@"
    ;;
esac
