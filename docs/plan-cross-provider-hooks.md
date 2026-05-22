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
   - [docs/hook-api/gemini-hook-api.md](hook-api/gemini-hook-api.md)
   - [docs/hook-api/cursor-agent-hook-api.md](hook-api/cursor-agent-hook-api.md)
3. future `sc-hooks`-owned harness captures, fixtures, validation models, and
   drift reports for each provider

This plan must not promote external provider docs, relay events, or local CLI
help into implementation assumptions unless `sc-hooks` captures them and stores
them as repo-owned evidence.

## Shared Entry Gates

Cross-provider runtime implementation must not begin until all of these are
true:

1. the Claude-first track is stable in `sc-hooks`
2. the provider has a `sc-hooks`-owned harness path, approved fixtures, and
   provider-specific validation models
3. `N.3` has classified provider fields into canonical candidates,
   provider-local fields, and unresolved differences per `ADR-SHK-006`
4. `N.4` has recorded a readiness verdict in `docs/phase-N/readiness.md`
5. the provider has a documented design boundary section stating what must not
   be inferred from Claude or from another provider

Until those gates are satisfied, provider work remains harness/documentation
planning only.

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

Current verified planning facts:

- Codex is a separate compatibility target, not part of the Claude baseline
- repo-owned approved fixtures now exist for direct `SessionStart`,
  `PreToolUse`, and `notify` (`agent-turn-complete`)
- repo-owned env snapshots exist for each approved direct hook surface
- `notify` and `PreToolUse` are sufficient to model the local debounce
  prototype, but that prototype remains planning input only
- `resume`, `fork`, and `Stop` remain relevant provider surfaces, but current
  harness evidence classifies them as `confirmed-not-exercisable` or
  unreliable rather than approved runtime baselines

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

### Current Deferred Items

Codex is no longer blocked on first-pass schema capture. Current post-`Phase N`
deferred items are:

- `notify` remains outside the approved runtime baseline because the
  turn-complete family (`notify` vs Claude `Stop` vs Gemini `AfterAgent`)
  remains unresolved in `docs/phase-N/normalization-findings-ledger.md`
- Codex-specific correlation fields such as `thread-id`, `turn-id`, and
  `tool_use_id` remain provider-local only
- `CODEX_THREAD_ID` remains explicitly non-canonical because approved env
  fixtures show it can stay stale across new `codex exec` runs
- `Stop`, `resume`, and `fork` remain `confirmed-not-exercisable` and are not
  approved runtime assumptions

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

Current Gemini-facing evidence is summarized in
[docs/hook-api/gemini-hook-api.md](hook-api/gemini-hook-api.md).

Current verified planning facts:

- repo-owned approved fixtures now exist for:
  - `SessionStart`
  - `SessionEnd`
  - `BeforeAgent`
  - `BeforeTool`
  - `AfterTool`
  - `AfterAgent`
- direct local probes verified `--resume latest`
- direct local probes verified no hook-observable payload or env-key change
  across `text`, `json`, and `stream-json`
- the verified local registration path is user-scope `~/.gemini/settings.json`
  under an isolated temporary `HOME`

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

### Current Deferred Items

Gemini is no longer blocked on first-pass schema capture. Current post-`Phase N`
deferred items are:

- workspace `.gemini/settings.json` activation remains unresolved and stays out
  of the approved registration contract
- no approved canonical mapping yet for Gemini-only fields such as
  `tool_response.returnDisplay`, `prompt_response`, and `stop_hook_active`
- raw `tool_name = "run_shell_command"` remains provider-local until later
  cross-provider tool-surface normalization proves a compatible canonical enum
- `AfterAgent` remains outside the approved runtime baseline because the
  shared turn-complete/post-response family remains unresolved in
  `docs/phase-N/normalization-findings-ledger.md`

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

The next cross-provider execution phase after `Phase N` should treat Codex and
Gemini as fixture-backed provider candidates rather than planning placeholders.
`docs/phase-N/readiness.md` records `PARTIAL_GO`, so the next runtime phase is
approved only for the exact surfaces named there.

Codex approved follow-on track:

1. consume the approved Codex fixture/model baseline from `N.1`
2. design the runtime adapter against the `N.3` normalization ledger only
3. prove provider-specific correlation and root-recovery behavior in adapter
   tests
4. limit the first runtime pass to approved surfaces:
   - `SessionStart`
   - `PreToolUse`
5. keep `notify`, `Stop`, `resume`, and `fork` deferred until a later sprint
   closes the unresolved lifecycle family or captures the non-exercisable
   surfaces directly

Gemini approved follow-on track:

1. consume the approved Gemini fixture/model baseline from `N.2`
2. design the runtime adapter against the `N.3` normalization ledger only
3. preserve Gemini registration-path and control-semantics differences as
   provider-local behavior
4. limit the first runtime pass to approved surfaces:
   - `SessionStart`
   - `SessionEnd`
   - `BeforeAgent`
   - `BeforeTool`
   - `AfterTool`
5. keep `AfterAgent` deferred until a later sprint closes the unresolved
   turn-complete/post-response family

Parallel completion criteria:

- Codex and Gemini each have repo-owned fixtures for all locally tested hook
  points
- Codex and Gemini each have automated pytest schema-proof tests
- Codex and Gemini each have drift artifacts suitable for future version-bump
  checks
- mapping candidates into normalized `schooks` fields are explicitly listed and
  source-cited from fixtures
- deferred lifecycle families and provider-local fields remain explicitly named
  in `docs/phase-N/readiness.md` and the `N.3` normalization ledger

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
