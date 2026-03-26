# schook

`schook` is a Rust workspace for `sc-hooks`, a compiled hook dispatcher for AI-assisted development workflows.

The project provides:
- a host CLI that parses hook config, resolves handlers, assembles metadata, dispatches builtins and plugins, enforces timeouts, writes JSONL logs, and audits configuration
- a shared core crate for protocol/data types
- an SDK crate for manifest helpers and Rust plugin ergonomics
- a reusable compliance-test crate for plugin authors

## Workspace

| Path | Role |
| --- | --- |
| `sc-hooks-cli/` | Host binary: `run`, `audit`, `fire`, `install`, `config`, `handlers`, `test`, `exit-codes` |
| `sc-hooks-core/` | Shared protocol/data types such as manifests, hook results, events, validation rules, and exit codes |
| `sc-hooks-sdk/` | Rust convenience layer for manifest generation, runner helpers, and result helpers |
| `sc-hooks-test/` | Reusable plugin compliance harness |
| `plugins/` | Reference/scaffold plugin source crates, not the runtime plugin install directory |
| `docs/` | Requirements, architecture, contracts, gap ledger, traceability, and governance |
| `shims/` | Thin adapters for Codex and Gemini |

Current source plugin inventory in `plugins/`:
- `audit-logger`
- `conditional-source`
- `event-relay`
- `guard-paths`
- `identity-state`
- `notify`
- `policy-enforcer`
- `save-context`
- `template-source`

## Current Shape

`sc-hooks` is a solid host/dispatcher foundation, but this repo is still a documentation and product-hardening project as much as a code project.

Important current realities:
- The runtime expects config at `.sc-hooks/config.toml`.
- The runtime resolves plugin executables from `.sc-hooks/plugins/`.
- This repo does not currently check in an example `.sc-hooks/` runtime layout.
- The source crates under `plugins/` are reference implementations and scaffolds; they are not production-ready bundled handlers.
- The docs in `docs/` are the source of truth for release scope and known implementation gaps.
- Internal Rust enums and error types are implementation details; the public contract is JSON, environment variables, and documented exit codes.

## Core Commands

```bash
cargo check --workspace
cargo fmt --check --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --verbose
```

CLI commands:

```bash
sc-hooks config
sc-hooks handlers
sc-hooks handlers --events
sc-hooks audit
sc-hooks fire PreToolUse Write
sc-hooks install
sc-hooks test <plugin>
sc-hooks exit-codes
```

## Documentation Map

| File | Purpose |
| --- | --- |
| `docs/requirements.md` | Normative release-facing behavior and status |
| `docs/architecture.md` | Current crate boundaries, execution model, and deferred areas |
| `docs/protocol-contract.md` | Host/plugin JSON contract |
| `docs/logging-contract.md` | JSONL logging contract, including mixed record shapes |
| `docs/implementation-gaps.md` | Current reality vs required release work |
| `docs/traceability.md` | Requirement-to-code/test/gap mapping |
| `docs/doc-governance.md` | Rules for keeping docs and code aligned |

## Workflow

- Keep the main repo on `develop`.
- Create feature work in `../schook-worktrees/<branch-name>`.
- Target `develop` for feature PRs.
- Do not describe behavior as shipped unless code and tests back it.
