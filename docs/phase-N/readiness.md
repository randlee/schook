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
| N.1 | `007612e` | `accepted` | `merged to integrate/phase-N` | Codex harness schema capture complete; all findings frozen; ready for N.2 baseline |
| N.2 | `88ebc36` | `accepted` | `merged to integrate/phase-N` | Gemini harness schema capture complete; all findings frozen; ready for N.3 baseline |
| N.3 | `acb2527` | `accepted` | `merged to integrate/phase-N` | Cross-provider normalization inventory complete; ready for N.4 promotion gate |
| N.4 | `7855476` | `accepted` | `merged to integrate/phase-N` | Promotion gate audit complete; PARTIAL_GO verdict recorded in release-checklist.md; follow-up QA fixes applied in PR #113 |

Final release verdict:

- `integrate/phase-N` candidate: `approved`
- release checklist result: `PARTIAL_GO`
- release verdict: `PARTIAL_GO`
- provider verdicts:
  - Codex: approved surfaces `SessionStart`, `PreToolUse` (notify/Stop/resume/fork deferred)
  - Gemini: approved surfaces `SessionStart`, `SessionEnd`, `BeforeAgent`, `BeforeTool`, `AfterTool` (AfterAgent deferred)
- open blocking findings: `none`
- open important findings: `3 carry-forward minor findings signed off (SCHOOK-QA-N4-009, SCHOOK-QA-N4-NEW-002, QA-001-RESIDUAL)`
- next action: create PR `integrate/phase-N` → `develop` and merge after final integration QA approval
- authorized by: `team-lead` (2026-05-23T00:16:00Z)
- notes: PARTIAL_GO verdict reflects approved hook surfaces per provider with explicit deferred-surface follow-on sprint mapping. Runtime adapter work authorized for approved surfaces only per `docs/phase-N/release-checklist.md`. All N.1-N.4 QA findings resolved or signed-off carry-forward. See `docs/phase-N/release-checklist.md` for detailed PARTIAL_GO analysis and approved/deferred surface mapping.
