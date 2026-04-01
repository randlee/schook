---
name: codex-orchestration
description: Orchestrate multi-sprint phases where chook (Codex) is the sole developer, with pipelined QA via quality-mgr teammate. Team-lead tracks findings and schedules fix passes.
---

# Codex Orchestration

This skill defines how the team-lead (ARCH-ATM) orchestrates phases where **chook (Codex)** is the sole developer, executing sprints sequentially while QA runs in parallel via a dedicated **quality-mgr** teammate.

**Audience**: Team-lead only.

**When to use**: When a phase's implementation is done entirely by chook (a Codex agent communicating via ATM CLI), not by Claude Code scrum-masters. This pattern was proven in Phase M (8 sprints) and Phase O.

## Prerequisites

Before starting a phase:
1. Phase plan document exists with sprint specs and dependencies
2. Integration branch `integrate/phase-{P}` created off `develop`
3. ATM team is active with team-lead and chook as members
4. chook is running and reachable via ATM CLI (`atm send chook "ping"`)

## Architecture

```
team-lead (ARCH-ATM)
  ├── chook (Codex) ──── sole developer, sequential sprints
  │     communicates via ATM CLI only
  └── quality-mgr (Claude Code) ──── QA coordinator teammate
        spawns rust-qa-agent + atm-qa-agent as background agents
```

Key principle: **chook does NOT wait for QA**. He proceeds to the next sprint as soon as he completes one, unless there are outstanding fix requests from earlier sprints.

## Phase Setup

### 1. Create Integration Branch

```bash
git fetch origin develop
git branch integrate/phase-{P} origin/develop
git push -u origin integrate/phase-{P}
```

### 2. Create First Sprint Worktree

```bash
# Use sc-git-worktree skill
/sc-git-worktree --create feature/p{P}-s1-{slug} integrate/phase-{P}
```

### 3. Spawn Quality Manager

Spawn once per phase. The quality-mgr persists across all sprints.

Use the Task tool with `name` parameter to spawn as a tmux teammate:

```json
{
  "subagent_type": "general-purpose",
  "name": "qa-hook",
  "team_name": "<team-name>",
  "model": "sonnet",
  "prompt": "You are qa-hook, the QA coordinator for Phase {P}. You will receive QA assignments from team-lead for each sprint as they complete. Stand by for first assignment. Integration branch: integrate/phase-{P}. Phase docs: docs/project-plan.md, docs/atm-agent-mcp/requirements.md."
}
```

**Tmux teammate launch troubleshooting**: If the pane opens but the Claude process doesn't start, manually launch in the pane with all three required flags:

```bash
CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1 /Users/randlee/.local/share/claude/versions/<VERSION> \
  --agent-id qa-hook \
  --agent-name qa-hook \
  --team-name <team-name>
```

All three flags (`--agent-id`, `--agent-name`, `--team-name`) are required together — omitting any one causes an error.

### 4. Send O.1 Assignment to chook

Render via `sc-compose`, then pipe to `atm send`:

```bash
sc-compose render .claude/skills/codex-orchestration/dev-template.xml.j2 \
  --var task_id=SC-P{P}-1 \
  --var sprint="P{P}.1" \
  --var assignee=chook \
  --var description="{title}" \
  --var worktree_path=/path/to/worktree \
  --var branch=feature/p{P}-s1-{slug} \
  --var pr_target=integrate/phase-{P} \
  --var $'deliverables=- {deliverable 1}\n- {deliverable 2}' \
  --var $'acceptance_criteria=- cargo test --workspace PASS\n- cargo clippy -- -D warnings PASS' \
  --var $'references=- docs/requirements.md\n- docs/architecture.md\n- docs/project-plan.md'

# Then send the rendered output:
atm send chook "$(sc-compose render ...)"
```

## Sprint Pipeline

### Steady-State Flow

```
Timeline:
  chook:     [── S.1 ──]──fixes──[── S.2 ──]──fixes──[── S.3 ──]
  quality-mgr:         [── QA S.1 ──]      [── QA S.2 ──]     [── QA S.3 ──]
  team-lead:    assign S.1 → track → assign S.2 → track → assign S.3 → track
```

### When chook Completes Sprint S

1. **chook sends completion message** via ATM CLI with PR number
2. **Team-lead creates worktree for S+1** based on sprint S branch:
   ```
   /sc-git-worktree --create feature/p{P}-s{N+1}-{slug} feature/p{P}-s{N}-{slug}
   ```
   All worktrees chain: S+1 bases on S, so later sprints include earlier work.
3. **Team-lead assigns QA to quality-mgr** via SendMessage:
   ```
   "Run QA on Sprint {P}.{S}. Worktree: {path}. Sprint deliverables: {summary}.
    Design docs: {list}. PR: #{N}."
   ```
4. **Team-lead checks for outstanding findings** from earlier sprints:
   - If findings exist for S-2 or S-1: send fix assignment to chook BEFORE S+1 assignment
   - If no findings: send S+1 assignment immediately
5. **chook addresses fixes first, then starts S+1**

### When chook Has Outstanding Findings

Priority order for chook:
1. Fix findings on oldest sprint first (S-2 before S-1)
2. Merge fixes forward into later sprint worktrees
3. Then proceed to next sprint

Fix workflow:
```bash
# chook fixes on the sprint's original worktree
# chook pushes fix commits to same PR branch
# team-lead asks quality-mgr to re-run QA on the fixed worktree
# If QA passes, team-lead merges PR to integration branch
```

### Merge Forward Protocol

After fixes merge to `integrate/phase-{P}`:
- chook must merge integration branch into any active sprint worktree before continuing:
  ```bash
  git fetch origin
  git merge origin/integrate/phase-{P}
  ```
- This ensures later sprints include all fixes from earlier sprints

## QA Coordination

### Team-lead → quality-mgr Messages

Assignment format:
```
Run QA on Sprint {P}.{S}: {title}
Worktree: {absolute path}
Sprint deliverables: {bullet list}
Design docs: {list of relevant doc paths}
PR: #{number}
```

Re-run after fixes:
```
Re-run QA on Sprint {P}.{S} (post-fix).
Worktree: {path}
Fixed findings: {list of QA IDs addressed}
```

### quality-mgr → team-lead Reports

quality-mgr reports PASS/FAIL with finding IDs. Team-lead tracks:

| Sprint | QA Run | Verdict | Blocking Findings | Status |
|--------|--------|---------|-------------------|--------|
| O.1    | 1      | FAIL    | QA-001, QA-002    | Fixes assigned |
| O.1    | 2      | PASS    | —                 | Merged |
| O.2    | 1      | PASS    | —                 | Merged |

### Finding Lifecycle

```
OPEN → assigned to chook → FIXED (chook pushes) → re-QA → VERIFIED (QA passes)
                             → WONTFIX (team-lead approves deviation)
```

## PR and Merge Strategy

- **All PRs target `integrate/phase-{P}`** (never develop directly)
- **Merge order**: Sprint PRs merge in order (S.1 before S.2)
- **Merge gate**: QA pass + CI green
- **Team-lead merges** (not chook)
- After all sprints merge: one final PR `integrate/phase-{P} → develop`

### Pre-PR Merge Check (REQUIRED before opening any PR)

Before opening a PR, verify the branch includes all prior sprint merges:

```bash
git log origin/integrate/phase-{P}..origin/{branch} --oneline   # commits unique to branch (expected)
git log origin/{branch}..origin/integrate/phase-{P} --oneline   # commits missing from branch (must be empty)
```

If the second command shows commits, have chook merge forward before opening the PR:

```bash
git fetch origin && git merge origin/integrate/phase-{P}
```

Missing merges cause pre-existing test failures that block CI and cause QA agents to file false root-cause reports.

## Task Templates

Two Jinja2 templates live alongside this skill:

- **`dev-template.xml.j2`** — task assignment to chook
- **`qa-template.xml.j2`** — QA assignment to quality-mgr

Every task message to chook MUST embed the 5-step workflow from `dev-template.xml.j2`. Do not rely on chook remembering instructions from prior messages — include them every time.

Every QA assignment to quality-mgr MUST embed the 5-step workflow from `qa-template.xml.j2`. This ensures quality-mgr always spawns `rust-qa-agent` and `atm-qa-agent` as background agents instead of running checks himself.

### Render Contract

`sc-compose --var-file` currently accepts scalar values. For these templates, `deliverables`, `acceptance_criteria`, and `references` must be passed as scalar strings, not JSON arrays.

Use one of these two patterns:
- `--var-file` with single-line scalar summaries
- `--var` with shell-quoted multiline text when you need line breaks

Do not change the templates to loop over arrays unless `sc-compose` gains typed array support and the examples below are updated and re-tested.

### Tested `sc-compose` Examples

Dev assignment example:

```json
{
  "task_id": "SC-EXAMPLE-DEV-1",
  "sprint": "S9-BC.1",
  "assignee": "chook",
  "description": "Implement session foundation hook runtime.",
  "worktree_path": "/Users/example/schook-worktrees/feature-s9-bc1-session-foundation",
  "branch": "feature/s9-bc1-session-foundation",
  "pr_target": "integrate/s9-hook-runtime",
  "deliverables": "Create canonical session.json lifecycle handling; persist project_root_dir from CLAUDE_PROJECT_DIR; add atomic write coverage tests.",
  "acceptance_criteria": "cargo test --workspace PASS; cargo clippy --all-targets --all-features -- -D warnings PASS; session foundation hooks update state only on material change.",
  "references": "docs/requirements.md; docs/architecture.md; docs/project-plan.md; docs/phase-bc-hook-runtime-design.md"
}
```

```bash
sc-compose render .claude/skills/codex-orchestration/dev-template.xml.j2 \
  --var-file .claude/skills/codex-orchestration/examples/dev-template-vars.json
```

Multiline dev example:

```bash
sc-compose render .claude/skills/codex-orchestration/dev-template.xml.j2 \
  --var task_id=SC-EXAMPLE-DEV-2 \
  --var sprint=S9-BC.1 \
  --var assignee=chook \
  --var description='Implement session foundation hook runtime.' \
  --var worktree_path=/Users/example/schook-worktrees/feature-s9-bc1-session-foundation \
  --var branch=feature/s9-bc1-session-foundation \
  --var pr_target=integrate/s9-hook-runtime \
  --var $'deliverables=- Create canonical session.json lifecycle handling\n- Persist project_root_dir from CLAUDE_PROJECT_DIR\n- Add atomic write coverage tests' \
  --var $'acceptance_criteria=- cargo test --workspace PASS\n- cargo clippy --all-targets --all-features -- -D warnings PASS\n- session foundation hooks update state only on material change' \
  --var $'references=- docs/requirements.md\n- docs/architecture.md\n- docs/project-plan.md\n- docs/phase-bc-hook-runtime-design.md'
```

QA assignment example:

```json
{
  "task_id": "SC-EXAMPLE-QA-1",
  "sprint": "S9-BC.1",
  "description": "Run QA on BC.1 session foundation implementation.",
  "pr_number": "57",
  "branch": "feature/s9-bc1-session-foundation",
  "worktree_path": "/Users/example/schook-worktrees/feature-s9-bc1-session-foundation",
  "commits": "abc1234",
  "deliverables": "Canonical session-state file ownership; atomic write semantics; mandatory hook logging for lifecycle hooks.",
  "references": "docs/requirements.md; docs/architecture.md; docs/project-plan.md; docs/phase-bc-hook-runtime-design.md"
}
```

```bash
sc-compose render .claude/skills/codex-orchestration/qa-template.xml.j2 \
  --var-file .claude/skills/codex-orchestration/examples/qa-template-vars.json
```

Multiline QA example:

```bash
sc-compose render .claude/skills/codex-orchestration/qa-template.xml.j2 \
  --var task_id=SC-EXAMPLE-QA-2 \
  --var sprint=S9-BC.1 \
  --var description='Run QA on BC.1 session foundation implementation.' \
  --var pr_number=57 \
  --var branch=feature/s9-bc1-session-foundation \
  --var worktree_path=/Users/example/schook-worktrees/feature-s9-bc1-session-foundation \
  --var commits=abc1234 \
  --var $'deliverables=- Canonical session-state file ownership\n- Atomic write semantics\n- Mandatory hook logging for lifecycle hooks' \
  --var $'references=- docs/requirements.md\n- docs/architecture.md\n- docs/project-plan.md\n- docs/phase-bc-hook-runtime-design.md'
```

These examples are the working baseline for this skill. If the templates change, re-run both commands and update the checked-in examples in the same change.

## Task List Tracking

Use TaskList to track each sprint's sub-tasks. Each sprint assignment creates 4 tasks:

| Task | Description | Completes when |
|------|-------------|----------------|
| `{sprint}: chook ack` | chook acknowledges task | chook sends ack message |
| `{sprint}: dev + push` | dev complete, commit pushed | chook reports commit hash |
| `{sprint}: cargo test` | tests pass, QA handoff | chook reports PASS |
| `{sprint}: QA pass` | QA agents report PASS | quality-mgr sends PASS verdict |
| `{sprint}: merge` | PR merged to integration branch | GitHub confirms merge |

On QA FAIL: create new `a/b/c` tasks for the fix pass. Do not reuse completed tasks.

## ATM Communication Protocol

All chook communication is via ATM CLI. Follow the dogfooding protocol (ACK → work → complete → ACK).

### Sending assignments

Render via `sc-compose`, then send the output:

```bash
MSG=$(sc-compose render .claude/skills/codex-orchestration/dev-template.xml.j2 --var-file vars.json)
atm send chook "$MSG"
```

NEVER hand-write task prose. Always use `dev-template.xml.j2`.

### Sending QA assignments

Use `SendMessage` to reach `qa-hook` (Claude Code teammate, not ATM):

```
SendMessage(to="qa-hook", message="<rendered qa-template content>")
```

Use the `qa-template.xml.j2` structure rendered via `sc-compose`. Always name both agents explicitly with `run_in_background=true`.

### Checking for replies
```bash
atm read
```

### Nudging (if no ack within 2 minutes)
```bash
# Find chook's pane
tmux list-panes -a -F '#{session_name}:#{window_index}.#{pane_index} #{pane_title} #{pane_current_command}'
# Send nudge
tmux send-keys -t <pane-id> -l "You have unread ATM messages. Run: atm read --team <team-name>" && sleep 0.5 && tmux send-keys -t <pane-id> Enter
```

**Do NOT assume chook received a task without an ack.** If no ack within ~2 minutes, nudge immediately.

### Advise chook to poll with timeout
When chook is waiting for assignments, tell him:
```
"Standing by? Use: atm read --timeout 60"
```
This keeps him responsive without busy-polling.

## Phase Completion

After all sprints pass QA and merge to integration branch:
1. Run final integration QA (quality-mgr validates full integration branch)
2. Create PR: `integrate/phase-{P} → develop`
3. Wait for CI green
4. Merge after user approval
5. Shutdown quality-mgr teammate
6. Do NOT clean up worktrees until user reviews

## Anti-Patterns

- Do NOT tell chook to wait for QA before starting the next sprint
- Do NOT skip QA on any sprint — quality-mgr runs both agents every time
- Do NOT merge PRs without QA pass + CI green
- Do NOT let findings accumulate — schedule fixes before assigning new sprints
- Do NOT create worktrees off `develop` — chain from previous sprint or integration branch
- Do NOT communicate with chook via SendMessage — use ATM CLI only
- Do NOT reuse quality-mgr across phases — spawn fresh per phase
- Do NOT clean up worktrees without user approval
- Do NOT assume chook received a task — wait for ack, nudge if none within 2 minutes
- Do NOT have chook open PRs — team-lead opens all PRs after chook reports commit hash
- Do NOT have quality-mgr run tests himself — he must always spawn rust-qa-agent + atm-qa-agent
- Do NOT omit the workflow steps from task messages — embed them every time, chook does not remember prior instructions
- Do NOT pre-load next task before current ack is received — confirm handoff before queuing next
- Do NOT let chook push a PR without first merging `origin/integrate/phase-{P}` into his branch — ensures all prior sprint fixes are included
