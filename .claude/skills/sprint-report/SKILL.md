---
name: sprint-report
description: Generate a sprint status report for schook. Default is --table.
---

# Sprint Report Skill

Build fenced JSON and pipe to the Jinja2 template. `mode` controls table vs detailed.

## Usage

```
/sprint-report [--table | --detailed]
```

Default: `--table`

---

## Data Source

Use `gh pr list` to get current sprint PR state:

```bash
cd /Users/randlee/Documents/github/schook
gh pr list --state all --limit 20
```

For CI status on open PRs:

```bash
gh pr checks <PR_NUMBER>
```

Only drill into individual `gh run view` calls if you need failure details for a specific job.

## Render Command

The template path is relative — must run from the **schook repo root** (not a worktree).

```bash
cd /Users/randlee/Documents/github/schook
echo '<json>' > /tmp/sprint-report.json
sc-compose render skills/sprint-report/report.md.j2 --var-file /tmp/sprint-report.json
```

## --table (default)

```json
{
  "mode": "table",
  "sprint_rows": "| S1 Baseline alignment | ✅ | ✅ | 🏁 | #14 |\n| S2 Compliance hardening | ✅ | ✅ | 🌀 | #15 |",
  "integration_row": "| **integration/s7-baseline → develop** | | — | 🌀 | — |"
}
```

## --detailed

```json
{
  "mode": "detailed",
  "sprint_rows": "Sprint: S1  Baseline alignment\nPR: #14\nQA: PASS ✓\nCI: Merged to integration/s7-baseline ✓\n────────────────────────────────────────\nSprint: S2  Compliance hardening\nPR: #15\nQA: PASS ✓ (iter 3)\nCI: Running (1 pending)",
  "integration_row": "Integration: integration/s7-baseline → develop\nCI: Pending S6 merge"
}
```

## Icon Reference

| State | DEV | QA | CI |
|-------|-----|----|----|
| Assigned | 📥 | 📥 | |
| In progress | 🌀 | 🌀 | 🌀 |
| Done/Pass | ✅ | ✅ | ✅ |
| Findings | 🚩 | 🚩 | |
| Fixing | 🔨 | | |
| Blocked | | | 🚧 |
| Fail | | | ❌ |
| Merged | | | 🏁 |
| Ready to merge | | | 🚀 |
