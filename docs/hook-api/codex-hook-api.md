# Codex Hook API

## Purpose

This document records the currently verified Codex-facing hook surfaces that
matter to `schook` planning. It is intentionally separate from the Claude
document because the execution model is materially different.

## Platform Rules

- Codex `ai_cli` supports agent frontmatter hooks
- Codex runs frontmatter `PreToolUse` command hooks before agent execution
- hook exit `0` allows execution
- hook exit `2` blocks execution
- Codex resolves agent files directly or via `.claude/agents/` lookup instead
  of assuming Claude built-ins

This is different from Claude Code, where `settings.json` hooks are the stable
surface and frontmatter hooks should not be relied on.

## Path And Environment Rules

- `CODEX_PROJECT_DIR` is the intended project-root anchor when available
- frontmatter hooks should still avoid relative paths for the same reason as
  Claude hooks: current working directory is not a stable execution anchor
- any Codex hook plan should treat path resolution as an explicit part of the
  contract instead of leaving it to shell cwd behavior

## Current Verified Codex Relay

| Behavior | Current script | Invocation model | Fields consumed | Side effects |
| --- | --- | --- | --- | --- |
| Turn-complete ATM relay | `atm-hook-relay.py` | command receives one JSON payload argument plus optional `--agent`/`--team` | payload `type`, `thread-id`, `turn-id`, env or args for team/agent | appends event JSONL into `${ATM_HOME}/.atm/daemon/hooks/events.jsonl` |

The current relay is not a full session lifecycle hook set. It is a narrow
turn-complete integration point.

## Session Correlation Model

Codex currently does not have the same verified SessionStart capture path that
Claude uses in this repo. Until that exists, treat Codex identity as a planned
gap rather than pretending it matches Claude.

Current practical correlation inputs:

1. explicit `session_id` if the runner injects one in the future
2. `thread-id` or equivalent turn/thread metadata for relay correlation
3. `ATM_TEAM` + `ATM_IDENTITY` as routing labels

Design rule:

- do not claim Claude-equivalent session continuity for Codex until there is a
  verified hook or runner surface that emits a stable session identifier

## Design Implications For `schook`

- Codex should be documented as a separate compatibility target, not squeezed
  into the Claude hook assumptions
- frontmatter support makes Codex a better target for agent-local guard hooks
  than Claude, but the session-identity story is currently weaker
- the existing `atm-hook-relay.py` pattern is event-relay plumbing, not enough
  by itself to define a general Codex session context contract

## Current Platform Gaps

- no verified Codex SessionStart equivalent is documented in this repo yet
- no verified Codex-specific permission or lifecycle relay set is documented
- no verified upstream schema for all Codex hook payloads is captured here yet
- any future Codex planning should cite the runner or bundle source used to
  verify payload fields before those fields are promoted into `schook` docs
