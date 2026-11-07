# AGENTS Instructions for schook

## Must Read

Before working in this repo, read:
- `CLAUDE.md`
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/project-plan.md`
- `docs/archive/plugin-plan-s9.md`

## Project Overview

`schook` (`sc-hooks`) is a Rust hook dispatcher and test harness for AI-assisted development workflows. The current design authority lives in this repo; do not treat external repos as the source of truth for hook runtime planning.

## Key Conventions

- Never switch the main repository off `develop`.
- Create worktrees under `../schook-worktrees/<branch-name>`.
- Target `develop` for normal PRs unless tasking says otherwise.
- Treat `docs/requirements.md`, `docs/architecture.md`, and `docs/project-plan.md` as the control documents.
- Treat `docs/archive/plugin-plan-s9.md` and `docs/hook-api/*` as supporting detail and provider evidence.
- Do not describe scaffold plugins or planned crates as shipped behavior.

## Rust Guidance

Read before writing or reviewing Rust:
- `$HOME/.claude/skills/rust-best-practices/SKILL.md`

Use it for crate boundaries, error design, state machines, newtypes, sealed traits, and review timing. It complements style/lint guidance; it does not replace `cargo fmt`, `clippy`, or repo-specific design docs.

## Build and Test

Before handoff, run the smallest relevant subset and do not skip validation silently:

```bash
cargo check --workspace
cargo fmt --check --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

## ATM Coordination

- Acknowledge tasking promptly.
- Do the assigned work end to end in the designated worktree.
- Commit and push before reporting completion.
- Send completion status directly to the requesting ATM role.
- Finish task loops with a blocking `atm read --team schook --timeout ...` unless redirected.

## Design Rules Summary

- `project_root_dir` chains from `CLAUDE_PROJECT_DIR`; do not substitute cwd.
- Canonical session state is persisted and updated only on material change.
- Session-state writes must use atomic write semantics; no in-place overwrite.
- Hook logging is mandatory for every invocation, even when state does not change.
- Spawn/tool gating rules must return exact retryable failures; no vague blocking text.
- ATM behavior extends the generic hook runtime; it does not redefine it.

## Honest Work, No Ceremony (binding on agents and humans)

Goal: working, deployable capability. Process serves it and never becomes the product.

- **Process artifacts** (certificates, ledgers, dashboards, matrices, meta-reports, speculative checks) need four things to exist: a named consumer, the feature it gates, the observed defect class that justifies it, and a deletion condition. Missing any of these → don't create it. Test: if running code branches on it, it's product. If only humans or reports read it, it's process and needs the four things. Writing code just to flip that answer is itself the pathology. Exception: a minimal integrity/recovery control (crash-recovery state, provenance snapshot) that prevents a named evidence-loss or corruption mode.
- **Real code + real tests in the same unit of work.** Forbidden: faked tests, mocks/fixtures presented as live proof, weakened assertions, regenerating goldens to force green, hard-coded success paths, placeholder macros in commits, editing the spec instead of implementing it, narrowing scope while claiming full success.
- **No self-certification.** An independent verifier closes the work, citing evidence at an exact revision. In solo sessions, re-verify by re-running, and state what wasn't independently verified.
- **Typed refusal beats fabrication but is worth less than real capability.** Work that ends only in a refusal stays open and says so.
- **Null results are successes** ("checked X, found no material increment"). Unsupported claims are worse than silence.
- **Metrics** predeclare their denominator and countermetric. Agreement between agents can raise confidence but is never independent evidence. Never silence stderr in commands that produce evidence.
- **Name pathologies when they occur:** gate self-weakening, proof-class inflation, golden regeneration, tolerance widening, suppression-pragma laundering, refusal farming, follow-up laundering. The full catalog and countermeasures are in the `just-say-no-to-process-porn-and-ceremony` skill. Ask the operator for it if you can't find it.
