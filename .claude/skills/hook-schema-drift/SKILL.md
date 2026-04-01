---
name: hook-schema-drift
description: Run the project-local Phase 3 schema drift workflow for approved hook fixtures and generate validated drift artifacts.
---

# Hook Schema Drift

Use this skill to run the project-local Phase 3 drift tooling for schook.

## Workflow

1. Run:
   - `python3 test-harness/hooks/run-schema-drift.py claude`
2. Review the generated artifacts:
   - `test-harness/hooks/claude/drift-history/<timestamp>-drift.json`
   - `test-harness/hooks/claude/reports/<timestamp>/schema-drift-report.html`
   - `test-harness/hooks/claude/reports/<timestamp>/schema-drift-report.json`
3. If the tool returns exit code:
   - `0`: PASS
   - `1`: DRIFT
   - `2`: ERROR

## Reporting Contract

- the HTML report must be self-contained
- the report must pass `html-validate`
- generated XHTML fragments must pass `xmllint --noout`
- the HTML reporting stack is global:
  - `$HOME/.claude/skills/html-report/SKILL.md`
  - `~/.claude/agents/html-report-generator.md`
