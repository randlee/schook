# Phase P Readiness

## Purpose

Planning and execution record for `Phase P`.

This document is the single authoritative go/no-go record for final `Phase P`
promotion status.

## Record Schema

Each sprint row must record:

- `sprint`
- `accepted_commit`
- `verdict`
- `current_status`
- `notes`

Execution ownership rule:

- `docs/phase-P/readiness.md` is `read-only` in sprint execution branches per
  `ADR-SHK-007`
- the integration author updates the accepted sprint row only after the sprint
  is accepted and merged

Sprint planning status convention:

- sprint docs remain `status: planned` until execution closes the sprint on the
  implementation line

## Pre-Sprint Kickoff Checklist

Before any `Phase P` sprint starts, the handoff or working notes must record:

- accepted `Phase O` baseline commit: `da6e450`
- confirmation that `just test hooks claude`, `just test hooks codex`, and
  `just test hooks gemini` were green on that accepted `Phase O` baseline
- the single owning implementation path for each behavior being extended, with
  no provider-specific bypass around `ProviderHookNormalizer`

## Final Verdict Record

The final section of this document must record:

- `integration_author`
- `integrate_phase_p_candidate`
- `release_verdict`
- `provider_verdicts`
- `open_blocking_findings`
- `open_important_findings`
- `next_action`
- `authorized_by`
- `notes`

## Initial State

| Sprint | Accepted Commit | Verdict | Current Status | Notes |
| --- | --- | --- | --- | --- |
| P.1 | `PENDING` | `PENDING` | `not started` | adds missing Codex hook surfaces to the permanent harness baseline |
| P.2 | `PENDING` | `PENDING` | `not started` | adds missing Gemini hook surfaces to the permanent harness baseline |
| P.3 | `PENDING` | `PENDING` | `not started` | adopts the available `sc-lint` suite and adds a hard cross-platform gate before new runtime work lands |
| P.4 | `PENDING` | `PENDING` | `not started` | extends canonical lifecycle normalization for the newly approved surfaces |
| P.5 | `PENDING` | `PENDING` | `not started` | closes Codex runtime parity for the missing lifecycle surfaces |
| P.6 | `PENDING` | `PENDING` | `not started` | closes Gemini runtime parity for `AfterAgent` |
| P.7 | `PENDING` | `PENDING` | `not started` | resolves the remaining active error/boundary/portability rulings |
| P.8 | `PENDING` | `PENDING` | `not started` | closes CLI alias and exhausted-retry coverage follow-on items |

Final release verdict:

- `integration_author`: `team-lead`
- `integrate_phase_p_candidate`: `PENDING`
- `release_verdict`: `PENDING`
- `provider_verdicts`: `PENDING`
- `open_blocking_findings`: `PENDING`
- `open_important_findings`: `PENDING`
- `next_action`: `PENDING`
- `authorized_by`: `PENDING`
- `notes`: `Phase P` parity-expansion verdict not yet recorded
