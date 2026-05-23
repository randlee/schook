set windows-shell := ["pwsh", "-NoLogo", "-Command"]

python_cmd := if os_family() == "windows" { "python" } else { "python3" }

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

# Build the full workspace.
build:
    cargo build --workspace

# Run workspace or harness tests.
test target='workspace' provider='':
    {{python_cmd}} .just/run_test.py {{target}} {{provider}}

# Remove workspace build artifacts.
clean:
    cargo clean

# Run the local CI-equivalent command set for the current repo surface.
ci:
    @just fmt check
    @just test workspace
    @just test hooks claude
    @just test hooks codex
    @just test hooks gemini
