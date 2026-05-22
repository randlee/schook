# Phase N Release Checklist

Status:
- `COMPLETE`

Purpose:
- final promotion-gate checklist for `N.4`

Artifact inventory reviewed in this sprint:
- `N.1` integrated baseline
  - accepted commit: `007612e`
  - Codex approved fixtures under `test-harness/hooks/codex/fixtures/approved/`
  - Codex models/tests under `test-harness/hooks/codex/models/` and
    `test-harness/hooks/codex/tests/`
  - Codex evidence docs under `docs/hook-api/codex-hook-api.md`,
    `docs/phase-N/codex-capture-checklist.md`, and
    `docs/phase-N/codex-findings-ledger.md`
- `N.2` integrated baseline
  - accepted commit: `88ebc36`
  - Gemini approved fixtures/models/tests under `test-harness/hooks/gemini/`
  - Gemini evidence docs under `docs/hook-api/gemini-hook-api.md`,
    `docs/phase-N/gemini-capture-checklist.md`, and
    `docs/phase-N/gemini-findings-ledger.md`
- `N.3` integrated baseline
  - accepted commit: `acb2527`
  - normalization checklist/ledger and control-doc reconciliation are merged to
    `integrate/phase-N`
  - normalization evidence docs under
    `docs/phase-N/normalization-checklist.md` and
    `docs/phase-N/normalization-findings-ledger.md`

Checklist results:

| Check | Result | Evidence | Notes |
| --- | --- | --- | --- |
| Codex fixtures/models/tests complete on integration baseline | `PASS` | `007612e`, `docs/phase-N/codex-capture-checklist.md`, `docs/phase-N/codex-findings-ledger.md` | `SessionStart`, `notify`, `PreToolUse`, `--cd`, and non-exercisable dispositions are present on `integrate/phase-N`. |
| Gemini fixtures/models/tests complete on integration baseline | `PASS` | `88ebc36`, `docs/phase-N/gemini-capture-checklist.md`, `docs/phase-N/gemini-findings-ledger.md` | `SessionStart`, `SessionEnd`, `BeforeAgent`, `BeforeTool`, `AfterTool`, `AfterAgent`, resume continuity, and output-format probes are present on `integrate/phase-N`. |
| Normalization inventory complete on integration baseline | `PASS` | `acb2527`, `docs/phase-N/normalization-checklist.md`, `docs/phase-N/normalization-findings-ledger.md` | The merged `N.3` baseline freezes four canonical candidates and records the remaining provider-local and unresolved classifications explicitly. |
| Hook trait seal prerequisite recorded | `DEFERRED-ACKNOWLEDGED` | `docs/architecture.md` section `9.3`, `docs/implementation-gaps.md` `SEAL-001`, `docs/phase-N/release-checklist.md` (`SC-PN-4` audit) | The promotion-gate audit confirms the prerequisite was reviewed. `SEAL-001` remains the active governing note, so any follow-on runtime phase must preserve that documented boundary rather than claiming a new sealed-trait closure. |
| Provider API docs reconciled to merged captured evidence | `PASS` | `docs/hook-api/codex-hook-api.md`, `docs/hook-api/gemini-hook-api.md` | Both provider docs reflect the approved captured baseline on `integrate/phase-N`. |
| Remaining open issues explicitly recorded | `PASS` | `docs/phase-N/readiness.md` | The final readiness record names deferred surfaces and next action. |
| Promotion verdict recorded in authoritative ledger | `PENDING` | `docs/phase-N/readiness.md` | This branch leaves the authoritative readiness verdict for merge-time update by the integration author per `ADR-SHK-007`. The proposed verdict for review is `PARTIAL_GO`. |

Open deferred items carried past promotion:
- Codex `notify` remains deferred because the cross-provider turn-complete
  family is still unresolved in `NRM-010`
- Gemini `AfterAgent` remains deferred for the same reason
- Codex `Stop`, `resume`, and `fork` remain `confirmed-not-exercisable` and
  stay outside approved runtime assumptions
- provider-local fields such as raw `tool_name`, `source`, and
  `tool_response.returnDisplay` remain out of the canonical contract

Proposed merge-time result:
- release checklist result: `PARTIAL_GO`
- provider runtime adapter work: `authorize approved surfaces only`
- next action: start the next approved runtime phase using only the proposed
  approved Codex and Gemini surfaces plus the canonical `N.3` field set; plan a
  dedicated follow-on sprint for deferred lifecycle families and
  non-exercisable Codex surfaces

Follow-on sprint references for deferred surfaces:
- Codex `notify`: `TBD — deferred lifecycle-family normalization sprint`
- Codex `Stop`: `TBD — deferred Codex interactive-surface capture sprint`
- Codex `resume`: `TBD — deferred Codex interactive-surface capture sprint`
- Codex `fork`: `TBD — deferred Codex interactive-surface capture sprint`
- Gemini `AfterAgent`: `TBD — deferred lifecycle-family normalization sprint`

## QA Checklist Answers

1. `ADR-SHK-007` ownership:
   - `docs/phase-N/readiness.md` now leaves the `N.4` row and final verdict
     `PENDING`
   - `authorized_by` is now `TBD — integration author updates at merge time per ADR-SHK-007`
   - the branch carries the proposed `PARTIAL_GO` analysis in this checklist,
     not as the authoritative readiness verdict
2. Deferred-surface follow-on references:
   - every deferred Codex and Gemini surface now has an explicit `TBD` follow-on
     sprint reference above
3. Hook-trait seal status:
   - the checklist now records this as `RECORDED`, not `PASS`, because the
     governing prerequisite is still the active `SEAL-001` documented boundary
4. Traceability alignment:
   - `HKR-014` and `HKR-015` remain implemented for the harness/planning work
     closed by `Phase N`, with deferred runtime-surface follow-on scope noted
     in `docs/traceability.md`
