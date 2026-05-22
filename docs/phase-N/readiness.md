# Phase N Readiness

## Purpose

Planning and promotion-gate record for `Phase N`.

This document is the single authoritative go/no-go record for final `Phase N`
promotion status. Any later summary in `docs/project-plan.md` or
`docs/plan-cross-provider-hooks.md` must reference the verdict recorded here
rather than restating an independent decision.

## Record Schema

Each sprint row must record:

- `sprint`
- `accepted_commit`
- `verdict`
- `current_status`
- `notes`

Execution ownership rule:

- `docs/phase-N/readiness.md` is `read-only` in sprint execution branches per
  `ADR-SHK-007`
- the integration author updates the accepted sprint row only after the sprint
  is accepted and merged

Sprint planning status convention:

- sprint docs remain `status: planned` until execution closes the sprint on the
  implementation line

## Final Verdict Record

The final section of this document must record:

- `integrate_phase_n_candidate`
- `release_checklist_result`
- `release_verdict`
- `provider_verdicts`
- `open_blocking_findings`
- `open_important_findings`
- `next_action`
- `authorized_by`
- `notes`

Each provider verdict entry must record:

- `provider`
- `approved_surfaces`
- `deferred_surfaces`
- `open_blocking_findings`
- `open_important_findings`
- `notes`

The final release verdict must remain `PENDING` until:

- `docs/phase-N/release-checklist.md` records a final checklist result
- every row in `docs/phase-N/codex-findings-ledger.md` records a final
  disposition
- every row in `docs/phase-N/gemini-findings-ledger.md` records a final
  disposition
- every row in `docs/phase-N/normalization-findings-ledger.md` records a final
  disposition

Promotion criteria:

- `GO` requires approved fixtures for every locally exercisable Codex and
  Gemini hook surface and zero open `BLOCKING` or `IMPORTANT` normalization
  findings
- `PARTIAL_GO` requires the final verdict to name the exact approved and
  deferred surfaces per provider
- `NO_GO` blocks runtime adapter work and requires a new sprint before retry

## Initial State

| Sprint | Accepted Commit | Verdict | Current Status | Notes |
| --- | --- | --- | --- | --- |
| N.1 | `PENDING` | `PENDING` | `not started` | Codex harness checklist and findings ledger not yet frozen |
| N.2 | `PENDING` | `PENDING` | `not started` | Gemini harness checklist and findings ledger not yet frozen |
| N.3 | `PENDING` | `PENDING` | `not started` | awaits Codex and Gemini approved fixture baselines |
| N.4 | `PENDING` | `PENDING` | `not started` | awaits N.1–N.3 closure |

Final release verdict:

- `integrate/phase-N` candidate: `PENDING`
- release checklist result: `PENDING`
- release verdict: `PENDING`
- provider verdicts: `PENDING`
- open blocking findings: `PENDING`
- open important findings: `PENDING`
- next action: `PENDING`
- authorized by: `PENDING`
- notes: Phase N promotion verdict not yet recorded
