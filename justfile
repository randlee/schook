set shell := ["zsh", "-cu"]

default:
  @just --list

test scope provider:
  test "{{scope}}" = "hooks" || { echo "unsupported test scope: {{scope}}" >&2; exit 1; }
  case "{{provider}}" in claude) pytest test-harness/hooks/claude/tests/ -q ;; codex) pytest test-harness/hooks/codex/tests/ -q ;; gemini) pytest test-harness/hooks/gemini/tests/ -q ;; *) echo "unsupported hooks provider: {{provider}}" >&2; exit 1 ;; esac
