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
- `docs/implementation-gaps.md`
- `docs/traceability.md`

Rules:
- behavior belongs in `docs/requirements.md`
- crate boundaries and execution flow belong in `docs/architecture.md`
- wire contracts belong in the contract docs
- missing or overstated behavior belongs in `docs/implementation-gaps.md`

## ATM Coordination

Current repo coordination happens on the default ATM team in this environment.

Known roles for this repo:
- `arch-hook` is the team-lead (Claude Code agent coordinating this repo)

Use direct ATM messages. Do not assume broadcast.

Common commands:

```bash
atm send arch-hook "status update or question"
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

## Honest Work, No Ceremony (binding on agents and humans)

Goal: working, deployable capability. Process serves it and never becomes the product.

- **Process artifacts** (certificates, ledgers, dashboards, matrices, meta-reports, speculative checks) need four things to exist: a named consumer, the feature it gates, the observed defect class that justifies it, and a deletion condition. Missing any of these → don't create it. Test: if running code branches on it, it's product. If only humans or reports read it, it's process and needs the four things. Writing code just to flip that answer is itself the pathology. Exception: a minimal integrity/recovery control (crash-recovery state, provenance snapshot) that prevents a named evidence-loss or corruption mode.
- **Real code + real tests in the same unit of work.** Forbidden: faked tests, mocks/fixtures presented as live proof, weakened assertions, regenerating goldens to force green, hard-coded success paths, placeholder macros in commits, editing the spec instead of implementing it, narrowing scope while claiming full success.
- **No self-certification.** An independent verifier closes the work, citing evidence at an exact revision. In solo sessions, re-verify by re-running, and state what wasn't independently verified.
- **Typed refusal beats fabrication but is worth less than real capability.** Work that ends only in a refusal stays open and says so.
- **Null results are successes** ("checked X, found no material increment"). Unsupported claims are worse than silence.
- **Metrics** predeclare their denominator and countermetric. Agreement between agents can raise confidence but is never independent evidence. Never silence stderr in commands that produce evidence.
- **Name pathologies when they occur:** gate self-weakening, proof-class inflation, golden regeneration, tolerance widening, suppression-pragma laundering, refusal farming, follow-up laundering. The full catalog and countermeasures are in the `just-say-no-to-process-porn-and-ceremony` skill. Ask the operator for it if you can't find it.
