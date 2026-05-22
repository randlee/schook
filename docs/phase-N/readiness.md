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

## Final State

| Sprint | Accepted Commit | Verdict | Current Status | Notes |
| --- | --- | --- | --- | --- |
| N.1 | `007612e` | `ACCEPTED` | `merged to integrate/phase-N` | Codex harness schema capture is frozen on the integration baseline. Approved direct evidence exists for `SessionStart`, `PreToolUse`, `notify`, and `--cd` drift scenarios; `Stop`, `resume`, and `fork` remain `confirmed-not-exercisable`. |
| N.2 | `88ebc36` | `ACCEPTED` | `merged to integrate/phase-N` | Gemini harness schema capture is frozen on the integration baseline. All locally exercisable Gemini hook surfaces were captured; workspace `.gemini/settings.json` activation remains outside the approved registration contract. |
| N.3 | `acb2527` | `ACCEPTED` | `merged to integrate/phase-N` | Cross-provider normalization inventory is frozen on the integration baseline. Canonical candidates are limited to `session_id`, `cwd`, `transcript_path`, and `tool_input.command`; unresolved lifecycle families remain deferred. |
| N.4 | `2faef08` | `PARTIAL_GO` | `complete on promotion-gate branch` | `Phase N` closes with enough evidence to authorize the next runtime phase for approved surfaces only. Turn-complete/post-response families and non-exercisable Codex surfaces remain deferred. |

Final release verdict:

- `integrate_phase_n_candidate`: `2faef08`
- `release_checklist_result`: `PARTIAL_GO`
- `release_verdict`: `PARTIAL_GO`
- `provider_verdicts`:
  - provider: `codex`
  - approved_surfaces:
    - `SessionStart`
    - `PreToolUse`
  - deferred_surfaces:
    - `notify`
    - `Stop`
    - `resume`
    - `fork`
  - open_blocking_findings: `0`
  - open_important_findings: `0`
  - notes: `SessionStart` and `PreToolUse` are approved fixture-backed adapter inputs. `notify` remains deferred because the cross-provider turn-complete family is unresolved in `NRM-010`. `Stop`, `resume`, and `fork` remain `confirmed-not-exercisable` and are not approved runtime assumptions.
  - provider: `gemini`
  - approved_surfaces:
    - `SessionStart`
    - `SessionEnd`
    - `BeforeAgent`
    - `BeforeTool`
    - `AfterTool`
  - deferred_surfaces:
    - `AfterAgent`
  - open_blocking_findings: `0`
  - open_important_findings: `0`
  - notes: All locally exercisable Gemini hook surfaces were captured, but `AfterAgent` remains deferred because the cross-provider turn-complete/post-response family is unresolved in `NRM-010`. Workspace-scope registration remains outside the approved contract, but that does not block the approved user-scope harness baseline.
- `open_blocking_findings`: `0`
- `open_important_findings`: `0`
- `next_action`: authorize the next runtime phase for approved Codex and Gemini surfaces only, while planning a follow-on sprint for deferred lifecycle families and non-exercisable Codex surfaces
- `authorized_by`: `chook via SC-PN-4 sprint execution`
- `notes`: `Phase N` is complete as a harness-planning and promotion-gate phase. This `PARTIAL_GO` authorizes follow-on runtime adapter work only for the approved provider surfaces and the canonical `N.3` field set. It does not promote unresolved lifecycle families, provider-local fields, or confirmed-not-exercisable Codex surfaces into the runtime baseline.
