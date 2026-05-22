# Phase N Release Checklist

Status:
- `PENDING`

Purpose:
- final promotion-gate checklist for `N.4`

Planned checks:
- Codex fixtures/models/tests complete
  - every locally exercisable Codex surface has a final disposition
  - no Codex surface remains in a `to be captured` state
- Gemini fixtures/models/tests complete
  - every locally exercisable Gemini surface has a final disposition
  - no Gemini surface remains in a `to be captured` state
- normalization inventory complete
  - zero open `BLOCKING` or `IMPORTANT` normalization findings for `GO`
  - exact approved/deferred surfaces named per provider for `PARTIAL_GO`
- provider API docs reconciled to captured evidence
- project-level control docs reconciled to the final promotion verdict
- remaining open issues explicitly recorded
- promotion verdict recorded in `docs/phase-N/readiness.md`
