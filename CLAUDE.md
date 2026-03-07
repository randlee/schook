# Claude Instructions for schook

## ⚠️ CRITICAL: Branch Management Rules

**NEVER switch the main repository branch on disk from `develop`.**

- The main repo MUST remain on `develop` at all times
- **ALWAYS use `sc-git-worktree` skill** to create worktrees for all development work
- **ALWAYS create worktrees FROM `develop` branch** (not from `main`)
- Do NOT use `git checkout` or `git switch` in the main repository
- All sprint work happens in worktrees at `../schook-worktrees/<branch-name>`
- **All PRs target `develop` branch** (integration branch, not `main`)

**Why**: Switching branches in the main repo breaks worktree references and destabilizes the development environment.

**Worktree Creation Pattern**:
```bash
# ✅ CORRECT: Create worktree from develop
/sc-git-worktree --create feature/s0-workspace-scaffold develop

# ❌ WRONG: Creating from main
/sc-git-worktree --create feature/s0-workspace-scaffold main
```

---

## Project Overview

**sc-hooks** is a Rust CLI hook dispatcher for AI-assisted development workflows:
- Compiled host binary (`sc-hooks`) — config parsing, metadata assembly, plugin resolution, input validation, dispatch, structured logging, audit
- Config-driven routing via `.sc-hooks/config.toml` — maps hook names to ordered handler chains
- Plugin protocol — any executable responding to `--manifest` (JSON stdin/stdout), language-agnostic
- Builtin handlers (e.g., `log`) — run in-process for performance
- Sync/async chain splitting — sync handlers gate tool use; async handlers run in background and return context on next turn
- `sc-hooks install` — generates `.claude/settings.json` hook entries from config + manifests

**Goal**: Replace a fragile Python-based hook dispatcher with a compiled, testable, auditable alternative that treats plugins as standalone processes communicating via JSON over stdin/stdout, and supports any AI tool (Claude Code, Codex, Gemini) via shims.

---

## Project Plan

**Docs**: [`docs/architecture.md`](./docs/architecture.md) · [`docs/requirements.md`](./docs/requirements.md)

**Sprint plan** (tracked via task list `schook`):

| Sprint | Focus |
|--------|-------|
| S0 | Workspace scaffold, config parser (TOML), CLI skeleton, error types |
| S1 | Plugin protocol: manifest loading, validation, stdin/stdout dispatch |
| S2 | Metadata assembly, env var injection, builtin log handler |
| S3 | Sync/async chain splitting, `sc-hooks install` settings generation |
| S4 | `sc-hooks audit` command — manifest validation, chain correctness, CI gate |
| S5 | `sc-hooks fire` diagnostic mode, structured JSONL dispatch logging |
| S6 | Hardening, cross-platform CI (Linux + macOS), integration tests, E2E acceptance |

---

## Key Documentation

- [`docs/architecture.md`](./docs/architecture.md) — System design, execution flow, component overview, plugin protocol, metadata structure
- [`docs/requirements.md`](./docs/requirements.md) — Full functional/non-functional requirements and acceptance scenarios

---

## Workflow

### Sprint Execution Pattern

Every sprint follows this pattern:

1. **Create worktree** using `sc-git-worktree` skill from `develop`
2. **Implementation** by assigned agent(s)
3. **Tests pass** — `cargo check --workspace` clean, unit/integration tests green
4. **Commit/Push/PR** targeting `develop`
5. **Review and merge**

### Branch Flow

- Sprint PRs → `develop` (integration branch)
- Release PR → `main` (after user review/approval)

### Worktree Cleanup Policy

Do NOT clean up worktrees until the user has reviewed them. Cleanup only when explicitly requested.

---

## Agent Team Communication

### Team Configuration

- **Team**: `schook`
- **team-lead** (you, Claude Code) — manages task list, reviews work, coordinates sprints
- **arch-schook** is a Codex agent — communicates via ATM CLI messages

### Identity

`.atm.toml` at repo root sets `default_team = "schook"`.

### Communicating with arch-schook (Codex)

arch-schook does **not** monitor Claude Code messages. Use ATM CLI:

```bash
# Send a message
atm send arch-schook "your message here"

# Check inbox for replies
atm read

# Inbox summary
atm inbox
```

**Nudge arch-schook** (if no reply after 2 minutes):
```bash
# Find arch-schook's pane
tmux list-panes -a -F '#{session_name}:#{window_index}.#{pane_index} #{pane_title} #{pane_current_command}'

# Send nudge
tmux send-keys -t <pane-id> -l "You have unread ATM messages. Run: atm read --team schook" && sleep 0.5 && tmux send-keys -t <pane-id> Enter
```

### Communication Rules

1. No broadcast messages — all communications are direct
2. Poll for replies — after sending to arch-schook, wait 30–60s then `atm read`
3. arch-schook is async — do not block; continue other work and check back

---

## Design Rules (Enforce Always)

1. **Only the host binary validates plugin manifests** — plugins never validate each other
2. **Plugins are processes, not libraries** — no dlopen, no C FFI, no unsafe blocks for plugin loading
3. **JSON is the universal contract** — metadata, payloads, and results all flow as JSON; no C ABI, no shared memory
4. **Builtins always win name resolution** — a builtin name cannot be shadowed by a plugin executable
5. **Sync chain only: proceed/block/error** — async plugins may never return `action=block`
6. **Config contains no per-handler config** — handler-specific settings are the handler's responsibility
7. **At most three env vars set by host** — `SC_HOOK_TYPE`, `SC_HOOK_EVENT`, `SC_HOOK_METADATA`; all structured data flows via JSON

---

## Initialization Process

1. Run: `atm teams resume schook` (or `TeamCreate` if needed)
2. Run: `atm teams cleanup schook`
3. Check task list (`TaskList`) for current sprint status
4. Check current branches and worktrees
5. Output concise status summary
6. Identify next sprint ready to execute
