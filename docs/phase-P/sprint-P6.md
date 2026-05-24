---
id: P.6
title: Gemini Missing-Hook Runtime Parity
status: planned
branch: feature/pP-s6-gemini-missing-hook-runtime
worktree: ../schook-worktrees/feature/pP-s6-gemini-missing-hook-runtime
target: integrate/phase-P
---

# Sprint P.6 — Gemini Missing-Hook Runtime Parity

## Goal

- close the remaining Gemini lifecycle parity gap by carrying `AfterAgent`
  through the shared runtime/plugin path

## Hard Dependencies

- accepted `P.4`

## Exact Targets

- `crates/sc-hooks-core/`
- `crates/sc-hooks-cli/`
- `plugins/agent-session-foundation/`
- `plugins/atm-extension/`
- provider runtime tests
- `docs/requirements.md`
- `docs/traceability.md`

## Deliverables

- Gemini runtime support for `AfterAgent`
- end-to-end Gemini runtime tests for `AfterAgent`
- requirements/traceability updates closing the Gemini parity delta
- explicit `docs/requirements.md` amendment note stating that `P.6` completes
  the retained Gemini lifecycle runtime closure for `HKR-006` and updates
  `HKR-010` only for the Gemini lifecycle surfaces proved in this sprint
- Gemini handler implementation available through the shared Rust runtime path,
  not through a side-channel provider-specific path
- explicit plugin-side behavioral contract for Gemini `AfterAgent`:
  - `plugins/agent-session-foundation/` owns persisted session-state updates
    and provider-specific lifecycle state transitions that follow normalized
    Gemini `AfterAgent` events
  - `plugins/atm-extension/` owns relay/identity/ATM behavior driven by
    normalized Gemini `AfterAgent` events where the plugin already participates
    in the shared runtime path
  - their `docs/architecture.md` section `3.2` classification remains
    `Runtime implementation source crate`; `P.6` changes behavior and tests,
    not plugin classification

## Acceptance Criteria

- `AfterAgent` runs through the shared runtime path
- the resulting behavior matches the retained Gemini harness/API evidence landed
  in `P.2`, including the fixture-backed hook/payload expectations documented
  in `docs/hook-api/gemini-hook-api.md`
- Gemini parity is no longer blocked on `AfterAgent`
- `plugins/agent-session-foundation/` is updated as needed for Gemini
  `AfterAgent` and its changed behavior is covered by runtime tests
- `plugins/atm-extension/` is updated as needed for Gemini `AfterAgent` and
  its changed behavior is covered by runtime tests
- any supported-platform limit is documented explicitly instead of being hidden
  behind an implicit Unix-only implementation
- there is no direct Gemini-specific runtime bypass around
  `ProviderHookNormalizer`; if one existed on branch entry, it is removed
- `P.6` explicitly records that its seam additions remain consistent with the
  current `RULING-NEEDED-ECR-001` deferral and that `P.7` cannot retroactively
  remove already-landed seam additions without a new breaking-change sprint

## Out Of Scope

- Codex runtime parity
- packaging/CLI alias work

## Required Validation

- `cargo test --workspace`
- `just lint sc-portability`
- `just test hooks gemini`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What follow-on work is blocked or unblocked by this sprint?
