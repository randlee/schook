# AGENTS Instructions for schook

## Must Read

Before working in this repo, read:
- `CLAUDE.md`
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/project-plan.md`
- `docs/plugin-plan-s9.md`

## Project Overview

`schook` (`sc-hooks`) is a Rust hook dispatcher and test harness for AI-assisted development workflows. The current design authority lives in this repo; do not treat external repos as the source of truth for hook runtime planning.

## Key Conventions

- Never switch the main repository off `develop`.
- Create worktrees under `../schook-worktrees/<branch-name>`.
- Target `develop` for normal PRs unless tasking says otherwise.
- Treat `docs/requirements.md`, `docs/architecture.md`, and `docs/project-plan.md` as the control documents.
- Treat `docs/plugin-plan-s9.md` and `docs/hook-api/*` as supporting detail and provider evidence.
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
- Finish task loops with a blocking `atm read --team atm-dev --timeout ...` unless redirected.

## Design Rules Summary

- `project_root_dir` chains from `CLAUDE_PROJECT_DIR`; do not substitute cwd.
- Canonical session state is persisted and updated only on material change.
- Session-state writes must use atomic write semantics; no in-place overwrite.
- Hook logging is mandatory for every invocation, even when state does not change.
- Spawn/tool gating rules must return exact retryable failures; no vague blocking text.
- ATM behavior extends the generic hook runtime; it does not redefine it.
