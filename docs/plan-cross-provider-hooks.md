# Cross-Provider Hook Follow-On Plan

## Purpose

This document records the current planning baseline for Codex, Gemini, and
Cursor follow-on hook work. It is a planning-only artifact. It does not
authorize runtime implementation, hook-crate creation, or provider-specific
compatibility claims beyond the verified evidence already captured in `schook`.

## Current Source Of Truth

Use these sources in priority order:

1. `schook` control documents:
   - [docs/requirements.md](requirements.md)
   - [docs/architecture.md](architecture.md)
   - [docs/project-plan.md](project-plan.md)
2. current provider evidence documents:
   - [docs/hook-api/codex-hook-api.md](hook-api/codex-hook-api.md)
   - [docs/hook-api/cursor-agent-hook-api.md](hook-api/cursor-agent-hook-api.md)
3. future `schook`-owned harness captures, fixtures, validation models, and
   drift reports for each provider

This plan must not promote external provider docs, relay events, or local CLI
help into implementation assumptions unless `schook` captures them and stores
them as repo-owned evidence.

## Shared Entry Gates

Cross-provider implementation must not begin until all of these are true:

1. the Claude-first track is stable in `schook`
2. the provider has a `schook`-owned harness path and fixture set
3. the provider has captured raw payloads for its first-pass hook surfaces
4. the provider has validation models derived from those captured payloads
5. the provider has a documented design boundary section stating what must not
   be inferred from Claude

Until those gates are satisfied, provider work remains planning-only.

## Shared Harness And Capture Requirements

Every non-Claude provider follow-on must add all of the following before
runtime work starts:

- a provider-specific harness directory under `test-harness/hooks/<provider>/`
- reproducible capture prompts/commands for the first-pass hook surfaces
- raw captured fixtures owned by `schook`
- provider-specific validation models based on those fixtures
- schema or drift reporting for future provider-version changes
- explicit documentation of hook control semantics, including what blocks,
  what allows, and what response shape the provider actually accepts

If any one of those is missing, the provider remains in planning mode.

## Shared Design Boundaries

These rules apply to Codex, Gemini, and Cursor:

- do not assume Claude field names carry over unchanged
- do not assume Claude session identity or lifecycle semantics carry over
- do not assume Claude hook names, event ordering, or response contracts carry
  over
- do not treat current working directory as a stable provider identity signal
- do not write `schook` runtime code from provider marketing docs, CLI help,
  or relay-side observations alone

The harness must capture the provider contract before implementation relies on
it.

## Codex Follow-On Plan

### Current Verified Baseline

Current Codex-facing evidence is summarized in
[docs/hook-api/codex-hook-api.md](hook-api/codex-hook-api.md).

Current useful planning facts:

- Codex is a separate compatibility target, not part of the Claude baseline
- Codex frontmatter `PreToolUse` behavior is materially different from Claude's
  stable `settings.json` hook surface
- local Codex runtime surfaces worth preserving in planning include:
  - `resume`
  - `fork`
  - `--cd`
- current repo evidence is stronger on relay/session event handling than on raw
  hook stdin payloads

### First-Pass Capture Targets

The first Codex pass should capture:

- the first executable pre-tool hook surface that Codex actually fires
- any verified session or turn identity surface exposed during real execution
- at least one lifecycle or relay-completion surface that can be correlated in
  `schook`
- any path/root signal used when `resume`, `fork`, or `--cd` changes execution
  context

### Current Blockers

Codex remains blocked by missing `schook`-owned artifacts:

- no standalone Codex hook stdin schema fixtures
- no provider-specific Codex validation models
- no Codex schema-drift report owned by `schook`
- no verified `schook` capture proving how Codex root/session identity behaves
  under `resume`, `fork`, and `--cd`

### Design Boundaries

- do not claim Claude-equivalent `SessionStart` behavior for Codex
- do not reuse Claude field names without captured proof
- do not implement Codex runtime handling from relay-event guesses alone
- do not assume frontmatter behavior implies full parity with Claude hooks

## Gemini Follow-On Plan

### Current Verified Baseline

There is not yet a dedicated `schook` Gemini hook API document. Current useful
planning facts preserved from earlier evidence-gathering are:

- `gemini` is installed locally
- Gemini exposes hook-management commands through `gemini hooks ...`
- Gemini has resume-related surface via `--resume`
- Gemini exposes output controls:
  - `--output-format text`
  - `--output-format json`
  - `--output-format stream-json`

Those are planning inputs only. They are not yet a verified `schook` hook
contract.

### First-Pass Capture Targets

The first Gemini pass should capture:

- the first real hook surfaces configurable through `gemini hooks`
- one tool-style hook payload
- one lifecycle or session-continuation payload if Gemini exposes one
- one capture proving how output-format choice affects hook-observable behavior,
  if it does at all

### Current Blockers

Gemini remains blocked by missing `schook`-owned artifacts:

- no Gemini hook fixtures in this repo
- no Gemini validation models
- no Gemini schema-drift report
- no Gemini hook API evidence document owned by `schook`
- no verified session/root identity model comparable to the Claude track

### Design Boundaries

- do not infer payload fields from `gemini hooks` command names
- do not assume Claude or Codex session-correlation fields exist in Gemini
- do not begin Gemini-targeting hook crates until hook-manager behavior and raw
  payload shape are captured together
- do not assume Gemini output-format controls define hook stdin/output contract

## Cursor Follow-On Plan

### Current Verified Baseline

Current Cursor-facing evidence is summarized in
[docs/hook-api/cursor-agent-hook-api.md](hook-api/cursor-agent-hook-api.md).

Current useful planning facts:

- Cursor Agent is a separate compatibility target, not part of the Claude
  baseline
- public Cursor docs and local CLI behavior are useful planning inputs, but not
  implementation proof
- the first-pass hook surfaces worth preserving are:
  - `beforeShellExecution`
  - `beforeMCPExecution`
  - `beforeReadFile`
  - `afterFileEdit`
  - `stop`
- later optional capture targets are:
  - `sessionStart`
  - `sessionEnd`
  - `preCompact`
  - `subagentStart`
  - `subagentStop`

### First-Pass Capture Targets

The first Cursor pass should capture:

- the three controllable hook surfaces:
  - `beforeShellExecution`
  - `beforeMCPExecution`
  - `beforeReadFile`
- at least two informational surfaces:
  - `afterFileEdit`
  - `stop`
- any provider root/session signals present in the real `cursor-agent` runtime

### Current Blockers

Cursor remains blocked by missing `schook`-owned artifacts:

- no captured `cursor-agent` hook payload fixtures in this repo
- no `schook`-owned Cursor validation models
- no current local `hooks.json` capture proving the actual configured runtime
  path
- no provider-specific schema-drift report

### Design Boundaries

- do not rely on public Cursor doc field names as runtime inputs before capture
- do not assume Cursor session hooks map cleanly to the Claude ATM/session
  model
- do not build Cursor-targeting runtime crates before controllable hook
  contracts are captured by the harness

## Planned Sequence

For each provider, the follow-on sequence should be:

1. document current verified baseline
2. wire provider-specific harness scaffolding
3. capture first-pass raw fixtures
4. build provider-specific validation models
5. publish a provider-owned hook API evidence document
6. re-evaluate whether implementation work is justified

If a provider fails at step 3 or 4, implementation stays deferred.

## Deliverable For A Later Approved Sprint

The later approved sprint for any one provider should produce:

- a provider-specific harness directory
- approved raw fixtures
- validation models
- schema-drift reporting
- a provider hook API evidence document
- updated control-doc references only if the new evidence justifies them

Until then, this document remains the cross-provider planning baseline and not
an implementation mandate.
