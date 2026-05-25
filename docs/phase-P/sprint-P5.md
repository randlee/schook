---
id: P.5
title: Codex Missing-Hook Runtime Parity
status: planned
branch: feature/pP-s5-codex-missing-hook-runtime
worktree: ../schook-worktrees/feature/pP-s5-codex-missing-hook-runtime
target: integrate/phase-P
---

# Sprint P.5 — Codex Missing-Hook Runtime Parity

## Goal

- close the remaining Codex runtime parity gap with the live Python-hook
  behavior used on this machine

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

- Codex runtime support for the retained missing lifecycle surfaces
- end-to-end Codex runtime tests for those surfaces
- requirements/traceability updates closing the Codex parity delta
- explicit `docs/requirements.md` amendment note stating that `P.5` advances
  `HKR-006` from retained lifecycle approval into active Codex runtime parity
  and updates `HKR-010` only for the Codex lifecycle surfaces proved in this
  sprint, with final cross-provider closure still reserved for later accepted
  work
- Codex handler implementation available through the shared Rust runtime path,
  not through a side-channel provider-specific path
- explicit plugin-side behavioral contract for the retained Codex lifecycle
  surfaces:
  - `plugins/agent-session-foundation/` owns persisted session-state updates
    and provider-specific lifecycle state transitions that follow normalized
    Codex hook events
  - `plugins/atm-extension/` owns relay/identity/ATM behavior driven by those
    normalized Codex hook events where the plugin already participates in the
    shared runtime path
  - their `docs/architecture.md` section `3.2` classification remains
    `Runtime implementation source crate`; `P.5` changes behavior and tests,
    not plugin classification

## Acceptance Criteria

- the retained Codex lifecycle surfaces run through the shared runtime path
- the resulting behavior matches the retained Codex harness/API evidence landed
  in `P.1`, including the fixture-backed hook/payload expectations documented
  in `docs/hook-api/codex-hook-api.md`
- Codex parity is no longer blocked on `notify`, `Stop`, or `resume`
- `plugins/agent-session-foundation/` is updated as needed for the retained
  Codex lifecycle surfaces and its changed behavior is covered by runtime tests
- `plugins/atm-extension/` is updated as needed for the retained Codex
  lifecycle surfaces and its changed behavior is covered by runtime tests
- any supported-platform limit is documented explicitly instead of being hidden
  behind an implicit Unix-only implementation
- there is no direct Codex-specific runtime bypass around
  `ProviderHookNormalizer`; if one existed on branch entry, it is removed
- `P.5` explicitly records that its seam additions remain consistent with the
  current `RULING-NEEDED-ECR-001` deferral and that `P.7` cannot retroactively
  remove already-landed seam additions without a new breaking-change sprint

## Out Of Scope

- Gemini runtime parity
- packaging/CLI alias work

## Required Validation

- `cargo test --workspace`
- `just lint sc-portability`
- `just test hooks codex`
- `git diff --check`

## Sprint QA Checklist

- Which requirement IDs or gap IDs changed status?
- What code was removed early rather than left in parallel?
- Which files/crates were the owned write scope for the sprint?
- What validation commands and direct tests proved the new contract?
- What follow-on work is blocked or unblocked by this sprint?
