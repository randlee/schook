# Claude Instructions for schook

## Critical Branch Rules

Never switch the main repository off `develop`.

- The main repo stays on `develop`.
- Create development worktrees from `develop`.
- Put sprint and feature work under `../schook-worktrees/<branch-name>`.
- Target `develop` for normal PRs.
- Do not clean up worktrees until the user asks.

## Project Summary

`sc-hooks` is a Rust hook dispatcher for AI-assisted development workflows.

Current scope:
- compiled host CLI in `sc-hooks-cli`
- shared contract/data types in `sc-hooks-core`
- Rust convenience helpers in `sc-hooks-sdk`
- reusable compliance harness in `sc-hooks-test`
- reference plugin source crates in `plugins/`

Do not describe scaffold plugins as shipped product behavior.

## Quality Bar

Before handing work off:

```bash
cargo check --workspace
cargo fmt --check --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --verbose
```

Use the smallest relevant subset when a full run is unnecessary, but do not skip validation silently.

## Documentation Source Of Truth

Read these first for product behavior:

- `docs/requirements.md`
- `docs/architecture.md`
- `docs/protocol-contract.md`
- `docs/observability-contract.md`
- `docs/logging-contract.md`
- `docs/archive/implementation-gaps.md`
- `docs/traceability.md`

Rules:
- behavior belongs in `docs/requirements.md`
- crate boundaries and execution flow belong in `docs/architecture.md`
- wire contracts belong in the contract docs
- missing or overstated behavior belongs in `docs/archive/implementation-gaps.md`

## ATM Coordination

Current repo coordination happens on the default ATM team in this environment.

Known roles for this repo:
- `arch-schook` is the repo architecture lead

Use direct ATM messages. Do not assume broadcast.

Common commands:

```bash
atm send arch-schook "status update or question"
atm read --timeout 120 --limit 20
```

Working rules:
- acknowledge tasking promptly
- complete the assigned work end to end
- commit and push from the designated worktree
- send a completion message when done
- finish with a blocking `atm read --timeout ...` waiting for the next task

## Design Rules

- Handler names resolve only through `.sc-hooks/plugins/`; there are no builtin handlers in the current runtime.
- Plugins are standalone executables, not linked libraries.
- JSON is the host/plugin contract.
- Rust SDK helpers are convenience surfaces, not the release-defining public contract.
- Config contains routing and shared context, not handler-specific config.
- If behavior is not implemented, mark it deferred or track it as a gap.
