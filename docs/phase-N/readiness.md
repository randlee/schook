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
| N.4 | `7855476` | `ACCEPTED` | `merged to integrate/phase-N` | Promotion gate audit complete. PARTIAL_GO verdict finalized. Approved surfaces per provider documented with deferred-surface follow-on sprint references. Follow-up QA fixes applied in PR #113. |
| N.5 | `9d6dd57` | `ACCEPTED` | `merged to integrate/phase-N` | Claude harness and baseline verification complete. All locally exercisable Claude hook surfaces confirmed against repo-owned fixtures. PR #119. |
| N.6 | `e26175d` | `ACCEPTED` | `merged to integrate/phase-N` | Codex provider harness verification complete. API doc and model fixtures confirmed against approved Codex surfaces. PR #120. |
| N.7 | `d0528d8` | `ACCEPTED` | `merged to integrate/phase-N` | Gemini provider harness verification complete. API doc and model fixtures confirmed against approved Gemini surfaces. PR #121. |
| N.8 | `2b7ebf5` | `ACCEPTED` | `merged to integrate/phase-N` | Codex API document and models verification complete. codex-hook-api.md and test_harness/hooks/codex/models/payloads.py aligned to approved fixtures. All QA follow-ups closed. PR #126. |
| N.9 | `e725a37` | `ACCEPTED` | `merged to integrate/phase-N` | Gemini API document and models verification complete. gemini-hook-api.md and test_harness/hooks/gemini/models/payloads.py aligned to approved fixtures. |
| N.10 | `43b5290` | `ACCEPTED` | `merged to integrate/phase-N` | Harness just integration complete. just test hooks claude/codex/gemini entrypoints live and documented. release-checklist.md finalized. |

Final release verdict:

- `integrate_phase_n_candidate`: `approved`
- `release_checklist_result`: `PARTIAL_GO`
- `release_verdict`: `PARTIAL_GO`
- `provider_verdicts`:
  - Codex: `SessionStart`, `PreToolUse` approved (notify, Stop, resume, fork deferred)
  - Gemini: `SessionStart`, `SessionEnd`, `BeforeAgent`, `BeforeTool`, `AfterTool` approved (AfterAgent deferred)
- `open_blocking_findings`: `none`
- `open_important_findings`: `3 carry-forward minors signed off (SCHOOK-QA-N4-009, SCHOOK-QA-N4-NEW-002, QA-001-RESIDUAL)`
- `next_action`: `create PR integrate/phase-N → develop for final integration QA + phase-ending review`
- `authorized_by`: `team-lead` (2026-05-23T00:32:00Z)
- `notes`: PARTIAL_GO verdict reflects approved hook surfaces per provider with explicit deferred-surface follow-on sprint mapping per `docs/phase-N/release-checklist.md`. Runtime adapter work authorized for approved surfaces only. All N.1-N.4 QA findings resolved or signed-off carry-forward.
