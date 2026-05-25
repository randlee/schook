# Phase Q Readiness

## Purpose

Planning and execution record for `Phase Q`.

This document is the single authoritative go/no-go record for final `Phase Q`
promotion status.

## Record Schema

Each sprint row must record:

- `sprint`
- `accepted_commit`
- `verdict`
- `current_status`
- `notes`

Execution ownership rule:

- `docs/phase-Q/readiness.md` is `read-only` in sprint execution branches
- the integration author updates the accepted sprint row only after the sprint
  is accepted and merged

Sprint planning status convention:

- sprint docs remain `status: planned` until execution closes the sprint on the
  implementation line

## Pre-Sprint Kickoff Checklist

Before any `Phase Q` sprint starts, the handoff or working notes must record:

- accepted `Phase P` baseline commit
- confirmation that `just test hooks claude`, `just test hooks codex`, and
  `just test hooks gemini` were green on that accepted baseline
- the single owning smoke or harness path being extended
- confirmation that Cursor Agent and opencode remain harness-only in `Phase Q`
  unless a later phase explicitly authorizes runtime support

## Initial State

| Sprint | Accepted Commit | Verdict | Current Status | Notes |
| --- | --- | --- | --- | --- |
| Q.1 | `PENDING` | `PENDING` | `not started` | openshell evaluation and recommendation for smoke/harness execution |
| Q.2 | `PENDING` | `PENDING` | `not started` | curated `just smoke` surface, runner wiring, and CI gate |
| Q.3 | `PENDING` | `PENDING` | `not started` | Claude executable smoke coverage |
| Q.4 | `PENDING` | `PENDING` | `not started` | Codex executable smoke coverage |
| Q.5 | `PENDING` | `PENDING` | `not started` | Gemini executable smoke coverage |
| Q.6 | `PENDING` | `PENDING` | `not started` | Cursor Agent harness expansion |
| Q.7 | `PENDING` | `PENDING` | `not started` | Cursor Agent API doc and Pydantic model closure |
| Q.8 | `PENDING` | `PENDING` | `not started` | opencode harness expansion |
| Q.9 | `PENDING` | `PENDING` | `not started` | opencode API doc and Pydantic model closure |

Final release verdict:

- `integration_author`: `team-lead`
- `integrate_phase_q_candidate`: `PENDING`
- `release_verdict`: `PENDING`
- `provider_verdicts`: `PENDING`
- `open_blocking_findings`: `PENDING`
- `open_important_findings`: `PENDING`
- `next_action`: `PENDING`
- `authorized_by`: `PENDING`
- `notes`: `Phase Q` smoke/harness readiness verdict not yet recorded
