set windows-shell := ["pwsh", "-NoLogo", "-Command"]

python_cmd := if os_family() == "windows" { "python" } else { "python3" }
clippy_cmd := if os_family() == "windows" { "cargo clippy --workspace --all-targets --all-features --target x86_64-pc-windows-msvc -- -D warnings" } else { "cargo clippy --workspace --all-targets --all-features -- -D warnings" }

# Show the curated repo task help.
default: help

# Show the curated repo task help.
help:
    {{python_cmd}} .just/print_help.py

[private]
_fmt-write:
    cargo fmt --all

[private]
_fmt-check:
    cargo fmt --all --check

# Format the Rust workspace or run the formatting gate.
fmt mode='check':
    {{python_cmd}} .just/run_fmt.py {{mode}}

[private]
_lint-fmt:
    @just fmt check

[private]
_lint-clippy:
    {{clippy_cmd}}

[private]
_lint-sc-boundary:
    {{python_cmd}} .just/lint_sc_boundary.py

# Build the full workspace.
build:
    cargo build --workspace

# Run the repo install/cutover helper surface.
install target='local-cutover':
    {{python_cmd}} .just/run_install.py {{target}}

# Run the full workspace test suite or the exact hook harness entrypoints.
test target='workspace' provider='':
    @if [ "{{target}}" = "workspace" ] && [ -z "{{provider}}" ]; then \
      {{python_cmd}} .just/run_test.py workspace; \
    elif [ "{{target}}" = "hooks" ] && [ "{{provider}}" = "claude" ]; then \
      {{python_cmd}} .just/run_hook_tests.py claude; \
    elif [ "{{target}}" = "hooks" ] && [ "{{provider}}" = "codex" ]; then \
      {{python_cmd}} .just/run_hook_tests.py codex; \
    elif [ "{{target}}" = "hooks" ] && [ "{{provider}}" = "gemini" ]; then \
      {{python_cmd}} .just/run_hook_tests.py gemini; \
    else \
      echo "error: unknown test target '{{target}}' provider '{{provider}}'" >&2; exit 1; \
    fi

# Remove workspace build artifacts.
clean:
    cargo clean

# Run the repo lint surface.
lint target='all':
    {{python_cmd}} .just/run_lint.py {{target}}

# Run the local CI-equivalent command set for the current repo surface.
ci:
    @just lint
    @just test
    @just test hooks claude
    @just test hooks codex
    @just test hooks gemini
