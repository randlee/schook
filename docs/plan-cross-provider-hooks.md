# Cross-Provider Hook Follow-On Plan

## Purpose

This document records the current planning baseline for Codex, Gemini, and
Cursor follow-on hook work. It is a planning-only artifact. It does not
authorize runtime implementation, hook-crate creation, or provider-specific
compatibility claims beyond the verified evidence already captured in `sc-hooks`.

## Current Source Of Truth

Use these sources in priority order:

1. `sc-hooks` control documents:
   - [docs/requirements.md](requirements.md)
   - [docs/architecture.md](architecture.md)
   - [docs/project-plan.md](project-plan.md)
2. current provider evidence documents:
   - [docs/hook-api/codex-hook-api.md](hook-api/codex-hook-api.md)
   - [docs/hook-api/cursor-agent-hook-api.md](hook-api/cursor-agent-hook-api.md)
3. future `sc-hooks`-owned harness captures, fixtures, validation models, and
   drift reports for each provider

This plan must not promote external provider docs, relay events, or local CLI
help into implementation assumptions unless `sc-hooks` captures them and stores
them as repo-owned evidence.

## Shared Entry Gates

Cross-provider implementation must not begin until all of these are true:

1. the Claude-first track is stable in `sc-hooks`
2. the provider has a `sc-hooks`-owned harness path and fixture set
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
- raw captured fixtures owned by `sc-hooks`
- provider-specific validation models based on those fixtures
- schema or drift reporting for future provider-version changes
- explicit documentation of hook control semantics, including what blocks,
  what allows, and what response shape the provider actually accepts

The goal is not “some example payloads.” The goal is a provider-owned proof of
the full observable hook contract that `schook` can normalize against.

If any one of those is missing, the provider remains in planning mode.

## Shared Schema-Proof Standard

For Codex, Gemini, and any later provider, the harness must prove all
hook-observable schema fields at every captured hook surface.

Minimum proof standard:

- capture raw stdin payload exactly as delivered to the hook
- capture relevant hook-process environment variables exactly as delivered
- record hook control semantics:
  - blocking vs non-blocking
  - exit-code behavior
  - stdout/stderr contract
- enumerate every observed field and variable into provider fixtures
- validate approved fixtures against provider-specific models
- produce a drift artifact that reports:
  - added fields
  - removed fields
  - changed types
  - changed required/optional status

Normalization rule:

- no field is eligible for mapping into `schooks` until it has appeared in
  repo-owned captured evidence and been promoted into a provider model
- a field is canonical only when approved fixtures from at least two providers
  show compatible semantics for that field
- provider-specific fields stay provider-local until a later phase proves
  broader compatibility
- unresolved fields must remain in the normalization findings ledger rather
  than being silently promoted
- mapping work must cite the provider fixture/model, not CLI help or memory

## Shared Design Boundaries

These rules apply to Codex, Gemini, and Cursor:

- do not assume Claude field names carry over unchanged
- do not assume Claude session identity or lifecycle semantics carry over
- do not assume Claude hook names, event ordering, or response contracts carry
  over
- do not treat current working directory as a stable provider identity signal
- do not write `sc-hooks` runtime code from provider marketing docs, CLI help,
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

- every real Codex hook surface that can be exercised locally, including:
  - `notify` turn-complete
  - `PreToolUse`
  - `SessionStart`
  - any other locally configurable hook surface that actually fires
- every payload field present at those surfaces
- every relevant hook-process environment variable present at those surfaces
- session/turn correlation behavior across:
  - startup
  - resume
  - compact or clear equivalents
  - `--cd`
  - `fork`, if still available and hook-visible
- root/current-dir behavior under directory drift after startup
- control semantics for each surface:
  - blocking vs async
  - exit-code handling
  - stdout/stderr contract

The first Codex pass should end with:

- approved raw fixtures for each captured surface
- provider-specific validation models
- automated pytest coverage proving fixture/model agreement
- a schema-drift report owned by `schook`
- a reconciled Codex API doc describing only verified fields and semantics

### Current Blockers

Codex remains blocked by missing `sc-hooks`-owned artifacts:

- no complete Codex fixture set covering all locally exercisable hook surfaces
- no provider-specific Codex validation models covering all captured surfaces
- no automated Codex schema-proof tests beyond the current debounce prototype;
  the existing five debounce tests are baseline evidence only and do not close
  the full Codex surface-capture scope
- no Codex schema-drift report owned by `schook`
- no reconciled provider-owned manifest of payload fields and hook env vars

### Design Boundaries

- do not claim Claude-equivalent `SessionStart` behavior for Codex
- do not reuse Claude field names without captured proof
- do not implement Codex runtime handling from relay-event guesses alone
- do not assume frontmatter behavior implies full parity with Claude hooks
- do not promote `agent-team-mail` relay-side fields into the approved Codex
  inventory unless the `schook` harness captures those fields directly from the
  hook process

## Gemini Follow-On Plan

### Current Verified Baseline

There is not yet a dedicated `sc-hooks` Gemini hook API document. Current useful
planning facts preserved from earlier evidence-gathering are:

- `gemini` is installed locally
- Gemini exposes hook-management commands through `gemini hooks ...`
- Gemini has resume-related surface via `--resume`
- Gemini exposes output controls:
  - `--output-format text`
  - `--output-format json`
  - `--output-format stream-json`

Those are planning inputs only. They are not yet a verified `sc-hooks` hook
contract.

### First-Pass Capture Targets

The first Gemini pass should capture:

- every real Gemini hook surface configurable through `gemini hooks` that can
  be exercised locally
- every payload field present at those surfaces
- every relevant hook-process environment variable present at those surfaces
- one tool-style payload
- one lifecycle/session-continuation payload if Gemini exposes one
- output-format interactions, if hook-observable:
  - `text`
  - `json`
  - `stream-json`
- control semantics for each surface:
  - blocking vs async
  - exit-code handling
  - stdout/stderr contract

The first Gemini pass should end with:

- approved raw fixtures for each captured surface
- provider-specific validation models
- automated pytest coverage proving fixture/model agreement
- a Gemini schema-drift report owned by `schook`
- a provider-owned Gemini hook API evidence document

### Current Blockers

Gemini remains blocked by missing `sc-hooks`-owned artifacts:

- no Gemini hook fixtures in this repo
- no Gemini validation models
- no automated Gemini schema-proof tests
- no Gemini schema-drift report
- no Gemini hook API evidence document owned by `schook`
- no verified provider-owned session/root identity model

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

Cursor remains blocked by missing `sc-hooks`-owned artifacts:

- no captured `cursor-agent` hook payload fixtures in this repo
- no `sc-hooks`-owned Cursor validation models
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
3. capture first-pass raw fixtures for every exercisable hook surface
4. capture hook-process env snapshots for the same surfaces
5. build provider-specific validation models
6. add pytest schema-proof tests that fail on fixture/model drift
7. publish a provider-owned hook API evidence document
8. re-evaluate whether implementation work is justified

If a provider fails at step 3, 4, 5, or 6, implementation stays deferred.

## Next Approved Phase

The next cross-provider execution phase should run Codex and Gemini in
parallel, both as schema-capture and normalization-prep tracks.

Codex track:

1. build a Codex live-capture harness parallel to the Claude harness
2. capture all locally exercisable Codex hook surfaces
3. freeze approved fixtures and env snapshots
4. build models and schema-drift reports
5. reconcile the Codex API doc to match only those fixtures

Gemini track:

1. build a Gemini live-capture harness parallel to the Claude harness
2. identify and capture all locally exercisable Gemini hook surfaces
3. freeze approved fixtures and env snapshots
4. build models and schema-drift reports
5. publish the first `schook`-owned Gemini hook API evidence doc

Parallel completion criteria:

- Codex and Gemini each have repo-owned fixtures for all locally tested hook
  points
- Codex and Gemini each have automated pytest schema-proof tests
- Codex and Gemini each have drift artifacts suitable for future version-bump
  checks
- mapping candidates into normalized `schooks` fields are explicitly listed and
  source-cited from fixtures

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
