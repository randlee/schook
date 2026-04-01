# schook

`schook` is the workspace for `sc-hooks`, a compiled Rust hook dispatcher for
AI-assisted development workflows.

Today’s release baseline is the dispatcher foundation:
- parse `.sc-hooks/config.toml`
- resolve runtime plugins from `.sc-hooks/plugins/`
- filter metadata and payloads against plugin manifests
- dispatch sync and async plugin chains
- persist per-session disable state
- emit `sc-observability` JSONL events
- audit config, handlers, and metadata requirements
- generate Claude hook settings entries from manifest matchers

The public contract is JSON, environment variables, and documented exit codes.
Internal Rust enums and typestates are implementation details.

## Quick Install

Build the workspace:

```bash
cargo check --workspace
cargo fmt --check --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

Install the CLI from this repo:

Unix-like shells (`bash`, `zsh`, etc. on macOS/Linux):

```bash
cargo install --path sc-hooks-cli --root ~/.local
export PATH="$HOME/.local/bin:$PATH"
```

If you do not want to install yet, run the CLI directly from the workspace:

```bash
cargo run -p sc-hooks-cli -- --help
```

## Quick Start

The runtime shape is:
- config: `.sc-hooks/config.toml`
- runtime plugins: `.sc-hooks/plugins/<name>`
- observability log: `.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl`

The checked example layout lives at:
- [examples/runtime-layout/README.md](examples/runtime-layout/README.md)

Minimal verification from the example runtime layout:

```bash
cd examples/runtime-layout
sc-hooks-cli audit
printf '%s\n' '{"tool_input":{"command":"echo hi"}}' | sc-hooks-cli run PreToolUse Write --sync
```

For a step-by-step operator guide, see [USAGE.md](USAGE.md).

Naming note:
- [docs/requirements.md](docs/requirements.md) uses `sc-hooks` as the product command label in acceptance scenarios.
- The current Cargo package and binary artifact in this repo is `sc-hooks-cli`, so the executable examples below use `sc-hooks-cli`.

## CLI Surface

Current top-level commands:

```text
sc-hooks-cli run
sc-hooks-cli audit
sc-hooks-cli fire
sc-hooks-cli install
sc-hooks-cli config
sc-hooks-cli handlers
sc-hooks-cli test
sc-hooks-cli exit-codes
```

Common invocations:

```bash
sc-hooks-cli audit
sc-hooks-cli config
sc-hooks-cli handlers
sc-hooks-cli handlers --events
printf '%s\n' '{"tool_input":{"command":"git status"}}' | sc-hooks-cli run PreToolUse Bash --sync
sc-hooks-cli fire PreToolUse Write
sc-hooks-cli test .sc-hooks/plugins/guard-paths
```

## Workspace Map

| Path | Role |
| --- | --- |
| `sc-hooks-cli/` | Host binary: config loading, resolution, dispatch, audit, install-plan generation, observability, exit behavior |
| `sc-hooks-core/` | Shared protocol/data types such as manifests, hook results, events, validation rules, and exit codes |
| `sc-hooks-sdk/` | Rust authoring conveniences for manifests, runner helpers, conditions, and results; not the release-defining contract |
| `sc-hooks-test/` | Reusable compliance harness and shell-based test fixtures |
| `plugins/` | Source crates only; all current crates remain scaffold/reference only in the release docs and are not described as shipped runtime plugins |
| `docs/` | Product requirements, architecture, protocol contracts, planning, and traceability |
| `examples/` | Checked runtime layout example |
| `shims/` | Thin adapters for Codex and Gemini |

Current source plugin inventory in `plugins/`:

| Crate | Release posture |
| --- | --- |
| `audit-logger` | scaffold/reference only; not a shipped runtime plugin |
| `conditional-source` | scaffold/reference only; not a shipped runtime plugin |
| `event-relay` | scaffold/reference only; not a shipped runtime plugin |
| `guard-paths` | scaffold/reference only; not a shipped runtime plugin |
| `identity-state` | scaffold/reference only; not a shipped runtime plugin |
| `notify` | scaffold/reference only; not a shipped runtime plugin |
| `policy-enforcer` | scaffold/reference only; not a shipped runtime plugin |
| `save-context` | scaffold/reference only; not a shipped runtime plugin |
| `template-source` | scaffold/reference only; not a shipped runtime plugin |

## Documentation

Start with:
- [docs/requirements.md](docs/requirements.md)
- [docs/architecture.md](docs/architecture.md)
- [docs/protocol-contract.md](docs/protocol-contract.md)
- [docs/observability-contract.md](docs/observability-contract.md)
- [docs/logging-contract.md](docs/logging-contract.md)
- [docs/traceability.md](docs/traceability.md)

Historical planning and gap context lives under:
- [docs/archive/](docs/archive/)

Important rule:
- use the top-level requirements and architecture docs as the source of truth
- do not infer shipped behavior from scaffold crates or old planning notes

## Notes

- The runtime config file is `.sc-hooks/config.toml`, not YAML.
- The dispatcher resolves only external plugins under `.sc-hooks/plugins/`; there are no builtin handler names in the current runtime.
- SDK helpers are conveniences for Rust plugin authors; they do not override the executable/JSON contract.
- Observability sink routing is not config-driven in the current release baseline.

## Development Workflow

- Keep the main repo on `develop`.
- Create feature work in `../schook-worktrees/<branch-name>`.
- Target `develop` for normal PRs unless tasking says otherwise.
- Do not describe behavior as shipped unless code and tests back it.
