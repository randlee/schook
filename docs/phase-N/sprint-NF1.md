---
id: NF1
title: Simple Harness And Doc Hygiene Batch
status: planned
branch: feature/pN-fix-s1-harness-hygiene-batch
worktree: ../schook-worktrees/feature/pN-fix-s1-harness-hygiene-batch
target: integrate/phase-N
---

# Sprint NF1 — Simple Harness And Doc Hygiene Batch

## Goal

- clear the simple Phase N blocker set that does not change runtime semantics
- clear the simple harness-fixture sanitization and hook-entry documentation items
- close the minor wording/cosmetic carry-forward items in the same pass

## Hard Dependencies

- `integrate/phase-N` merged baseline at `df2794f`
- `docs/phase-N/plan-remediation.md`

## Exact Targets

- `test-harness/hooks/tests/__init__.py`
- `test-harness/hooks/codex/fixtures/approved/`
- `test-harness/hooks/claude/fixtures/approved/permission-request-bash.json`
- `test-harness/hooks/gemini/fixtures/approved/after-tool.json`
- `test-harness/hooks/codex/hooks/`
- `test-harness/hooks/gemini/hooks/`
- `docs/phase-N/sprint-N1.md`
- `docs/phase-N/sprint-N2.md`

## Deliverables

Every listed deliverable is expected to land at a production-ready level for
the scope this sprint claims. If that cannot be done cleanly in one sprint, the
sprint must be split before implementation begins. No deliverable may be
silently dropped or partially deferred.

- `BP-HARNESS-NEW-002` fixed by adding `test-harness/hooks/tests/__init__.py`
- `BP-HARNESS-NEW-003` fixed by replacing Codex `$REPO_ROOT` placeholders with
  `/synthetic/test/codex-harness/`
- `BP-HARNESS-NEW-004` fixed by replacing the real GitHub owner in the Claude
  approved fixture
- `SCHOOK-QA-PN-001` fixed by setting `status: complete` in
  `docs/phase-N/sprint-N1.md` and `docs/phase-N/sprint-N2.md`
- `BP-CLI-002` fixed by documenting the `OnceLock` concurrency invariant
- `BP-HARNESS-NEW-006` fixed by adding idempotency comments to the Codex and
  Gemini harness entry points
- `BP-HARNESS-NEW-007` fixed by replacing the real PGID in the Gemini approved
  fixture
- minor carry-forward items closed if they remain wording/cosmetic only:
  `BP-CORE-006`, `BP-CLI-004`, `BP-CLI-005`, `BP-TEST-001`,
  `SCHOOK-QA-PN-003`, `BP-HARNESS-NEW-010`, `BP-HARNESS-NEW-011`

## Required Work

- replace all simple machine-local or operator-local tokens in the approved
  fixture set with the established synthetic values
- add the missing test-package initializer at the cross-provider harness root
- add one-line idempotency comments to each harness entry point that can be
  re-run by hook infrastructure
- update the old sprint-doc frontmatter statuses so the phase docs no longer
  claim `planned` after acceptance
- batch any remaining minor wording/cosmetic items only if they do not change
  runtime or harness semantics

## Explicit Code Samples

No protocol or trait shape change is expected in this sprint.

## This Sprint Does Not Close

- `BP-HARNESS-NEW-001`
- `GEMINI-MODULE-IMPORT`
- `BP-HARNESS-NEW-008`
- `SCHOOK-QA-PN-002`
- `PN-PRR-001`
- `BP-CORE-005`
- `BP-CLI-001`
- `BP-CLI-003`
- `BP-HARNESS-NEW-005`

## Acceptance Criteria

- every `NF1` fixture edit uses the existing synthetic-path conventions already
  used by Claude or Gemini fixtures
- no approved Phase N fixture under `codex/`, `gemini/`, or the cited Claude
  fixture contains the specific live values called out in the QA findings
- Codex and Gemini harness hook entry points each contain a concise idempotency
  note
- `docs/phase-N/sprint-N1.md` and `docs/phase-N/sprint-N2.md` both say
  `status: complete`
- every minor item claimed closed in this sprint remains wording/cosmetic only;
  if an item proves structural, it must be removed from `NF1` and planned in a
  separate sprint

## Required Validation

- `pytest test-harness/hooks/ -q`
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
