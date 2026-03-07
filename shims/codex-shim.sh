#!/usr/bin/env bash
set -eu

export SC_HOOK_AGENT_TYPE=codex
export SC_HOOK_SESSION_ID="${CODEX_SESSION_ID:-unknown}"
export SC_HOOK_AGENT_PID=$$

case "${1:-}" in
  pre-edit)
    exec sc-hooks run PreToolUse Write
    ;;
  post-edit)
    exec sc-hooks run PostToolUse Write
    ;;
  *)
    exec sc-hooks run "$@"
    ;;
esac
