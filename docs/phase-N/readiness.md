# Phase N Readiness

## Purpose

Planning and promotion-gate record for `Phase N`.

## Record Schema

Each sprint row must record:

- `sprint`
- `accepted_commit`
- `verdict`
- `current_status`
- `notes`

Sprint planning status convention:

- sprint docs remain `status: planned` until execution closes the sprint on the
  implementation line

## Final Verdict Record

The final section of this document must record:

- `integrate_phase_n_candidate`
- `release_checklist_result`
- `release_verdict`
- `authorized_by`
- `notes`

The final release verdict must remain `PENDING` until:

- `docs/phase-N/release-checklist.md` records a final checklist result
- every row in `docs/phase-N/codex-findings-ledger.md` records a final
  disposition
- every row in `docs/phase-N/gemini-findings-ledger.md` records a final
  disposition
- every row in `docs/phase-N/normalization-findings-ledger.md` records a final
  disposition

## Initial State

| Sprint | Accepted Commit | Verdict | Current Status | Notes |
| --- | --- | --- | --- | --- |
| N.1 | `PENDING` | `PENDING` | `not started` | Codex harness checklist and findings ledger not yet frozen |
| N.2 | `PENDING` | `PENDING` | `not started` | Gemini harness checklist and findings ledger not yet frozen |
| N.3 | `PENDING` | `PENDING` | `not started` | awaits Codex and Gemini approved fixture baselines |
| N.4 | `PENDING` | `PENDING` | `not started` | awaits N.1–N.3 closure |

Final release verdict:

- integrate/phase-N candidate: `PENDING`
- release checklist result: `PENDING`
- release verdict: `PENDING`
- authorized by: `PENDING`
- notes: Phase N promotion verdict not yet recorded
