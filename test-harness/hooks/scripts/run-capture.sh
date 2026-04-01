#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOKS_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

provider="${1:-claude}"
surface="${2:-pretooluse-bash}"

case "${provider}" in
  claude)
    exec "${HOOKS_DIR}/claude/scripts/run-capture.sh" "${surface}"
    ;;
  *)
    echo "unsupported provider: ${provider}" >&2
    exit 1
    ;;
esac
