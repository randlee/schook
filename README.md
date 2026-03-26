# schook

`schook` is a Rust workspace for `sc-hooks`, a compiled hook dispatcher for AI-assisted development workflows.

The project provides:
- a host CLI that parses hook config, resolves plugin handlers, assembles metadata, dispatches plugins, enforces timeouts, emits `sc-observability` JSONL events, and audits configuration
- a shared core crate for protocol/data types
- an SDK crate for Rust authoring conveniences that does not define the public host/plugin contract
- a reusable compliance-test crate for plugin authors

## Workspace

| Path | Role |
| --- | --- |
| `sc-hooks-cli/` | Host binary: `run`, `audit`, `fire`, `install`, `config`, `handlers`, `test`, `exit-codes` |
| `sc-hooks-core/` | Shared protocol/data types such as manifests, hook results, events, validation rules, and exit codes |
| `sc-hooks-sdk/` | Rust convenience layer for manifest generation, runner helpers, and result helpers; not the release-defining public contract |
| `sc-hooks-test/` | Reusable plugin compliance harness |
| `plugins/` | Reference/scaffold plugin source crates, not the runtime plugin install directory |
| `docs/` | Requirements, architecture, contracts, execution plan, gap ledger, traceability, and governance |
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
- The runtime emits service-scoped observability events at `.sc-hooks/observability/sc-hooks/logs/sc-hooks.log.jsonl`.
- A checked example runtime layout lives at `examples/runtime-layout/.sc-hooks/`.
- The source crates under `plugins/` are reference implementations and scaffolds; they are not production-ready bundled handlers.
- The docs in `docs/` are the source of truth for release scope and known implementation gaps.
- Internal Rust enums and error types are implementation details; the public contract is JSON, environment variables, and documented exit codes.
- SDK runner helpers are authoring conveniences; host behavior is defined by the executable/JSON contract, not by SDK fallback defaults.

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
| `docs/project-plan.md` | Derived sprint plan from requirements, traceability, and gaps; not a normative source-of-truth doc |
| `docs/protocol-contract.md` | Host/plugin JSON contract |
| `docs/observability-contract.md` | Current `sc-observability` event path and JSONL contract |
| `docs/logging-contract.md` | Current JSONL dispatch-log schema for downstream consumers |
| `docs/implementation-gaps.md` | Current reality vs required release work |
| `docs/traceability.md` | Requirement-to-code/test/gap mapping |
| `docs/doc-governance.md` | Rules for keeping docs and code aligned |

## Workflow

- Keep the main repo on `develop`.
- Create feature work in `../schook-worktrees/<branch-name>`.
- Target `develop` for feature PRs.
- Do not describe behavior as shipped unless code and tests back it.
