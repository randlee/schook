set shell := ["zsh", "-cu"]

default:
  @just --list

test category provider:
  case "{{category}}:{{provider}}" in \
    hooks:claude) pytest test-harness/hooks/claude/tests/ -q ;; \
    hooks:codex) pytest test-harness/hooks/codex/tests/ -q ;; \
    hooks:gemini) pytest test-harness/hooks/gemini/tests/ -q ;; \
    *) echo "unsupported target: just test {{category}} {{provider}}" >&2; exit 1 ;; \
  esac
