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

## Current State

| Sprint | Accepted Commit | Verdict | Current Status | Notes |
| --- | --- | --- | --- | --- |
| N.1 | `007612e` | `ACCEPTED` | `merged to integrate/phase-N` | Codex harness schema capture is frozen on the integration baseline. Approved direct evidence exists for `SessionStart`, `PreToolUse`, `notify`, and `--cd` drift scenarios; `Stop`, `resume`, and `fork` remain `confirmed-not-exercisable`. |
| N.2 | `88ebc36` | `ACCEPTED` | `merged to integrate/phase-N` | Gemini harness schema capture is frozen on the integration baseline. All locally exercisable Gemini hook surfaces were captured; workspace `.gemini/settings.json` activation remains outside the approved registration contract. |
| N.3 | `acb2527` | `ACCEPTED` | `merged to integrate/phase-N` | Cross-provider normalization inventory is frozen on the integration baseline. Canonical candidates are limited to `session_id`, `cwd`, `transcript_path`, and `tool_input.command`; unresolved lifecycle families remain deferred. |
| N.4 | `PENDING` | `PENDING` | `complete on promotion-gate branch; awaiting integration-author fill` | Proposed `PARTIAL_GO` details are documented in `docs/phase-N/release-checklist.md`, but the authoritative readiness row and final verdict remain reserved for merge-time update by the integration author per `ADR-SHK-007`. |

Final release verdict:

- `integrate_phase_n_candidate`: `PENDING`
- `release_checklist_result`: `PENDING`
- `release_verdict`: `PENDING`
- `provider_verdicts`: `PENDING`
- `open_blocking_findings`: `PENDING`
- `open_important_findings`: `PENDING`
- `next_action`: `PENDING`
- `authorized_by`: `TBD — integration author updates at merge time per ADR-SHK-007`
- `notes`: `SC-PN-4` supplies a proposed `PARTIAL_GO` analysis in `docs/phase-N/release-checklist.md`, including approved surfaces, deferred surfaces, reasons, and follow-on sprint references. The authoritative readiness verdict remains reserved for merge-time update by the integration author.
