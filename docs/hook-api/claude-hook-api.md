# Claude Hook API

## Purpose

This document records the currently verified Claude Code hook surfaces that
`schook` can target. It is a platform reference, not a generic hook contract.

The source-of-truth inputs for this document are:

- Claude Code hooks reference:
  - `https://code.claude.com/docs/en/hooks`
- installed Claude hook scripts
- Claude-specific notes in the `synaptic-canvas` repo:
  - `docs/agent-tool-use-best-practices.md`
  - `docs/agent-teams-best-practices.md`
- current ATM hook docs, scripts, tests, and session fallback code in the
  `agent-team-mail` repo
- real Claude Haiku captures under `test-harness/hooks/claude/captures/raw/`
  for claims marked captured in this document

## Platform Rules

- Claude Code uses `settings.json` hook registration for project and global
  hooks.
- Claude Code does not honor agent frontmatter hooks as a reliable execution
  surface.
- PreToolUse hooks in `settings.json` work for both in-process and tmux
  teammate modes.
- `WorktreeCreate` and `WorktreeRemove` are top-level Claude hook events, not
  `PreToolUse` matcher variants.
- Frontmatter hooks should not be treated as a Claude-compatible baseline.

## Path And Environment Rules

- the harness now captures both raw hook payloads and companion hook-process
  env snapshots under `test-harness/hooks/claude/captures/raw/`
- raw hook `cwd` means the literal `cwd` field in the hook payload
- `CLAUDE_PROJECT_DIR` means the project-root env var observed in a hook-process
  env snapshot
- in the current captured `SessionStart(source="startup")`,
  `SessionStart(source="compact")`, `SessionStart(source="resume")`, and
  `SessionStart(source="clear")` hooks, raw hook `cwd` and
  `CLAUDE_PROJECT_DIR` matched exactly
- in the current captured drift scenario, raw hook `cwd` changed after Claude
  `cd` while `CLAUDE_PROJECT_DIR` remained pinned to the session root
- this document uses `ai_root_dir` only for the runtime-normalized immutable
  session root; it is not a Claude payload field
- `CLAUDE_PLUGIN_ROOT` is not part of the current generic hook-runtime
  baseline; treat it as a likely plugin-context variable used for plugin-local
  path resolution when Claude executes an installed plugin
- do not set `CLAUDE_PLUGIN_ROOT` globally and do not treat it as a generic
  project-root signal
- if `schook` extensions are later distributed as Claude plugins,
  `CLAUDE_PLUGIN_ROOT` may become useful for locating plugin-local scripts and
  assets, but that usage should be documented from a dedicated plugin-context
  capture rather than assumption
- relative hook paths are not reliable because Claude may change the current
  directory during a session

## Current Schema Baseline

The live harness now verifies actual Claude Haiku payloads for these surfaces:

- `SessionStart`
- `SessionEnd`
- `PreCompact`
- `PreToolUse(Bash)`
- teammate/background spawn via `PreToolUse(Agent)`
- `PostToolUse(Bash)`
- `PermissionRequest`
- `Stop`

`Notification(idle_prompt)` remains DEFERRED: wired in the harness, but no
verified payload was captured locally.

Additional documented Claude provider surface outside the current `schook`
implementation baseline:

- `WorktreeCreate`
- `WorktreeRemove`

For `SessionStart`, the following is verified from live hook behavior:

- payload field names used by the live hook:
  - `session_id`
  - `source`
- verified observed `source` values:
  - `startup`
  - `compact`
  - `resume`
  - `clear`
- capture evidence:
  - `startup` and `compact` were captured in PR `#42`
  - `resume` and `clear` were captured in PR `#44`
- current script behavior:
  - `source == "compact"` -> compact-return message
  - any other value -> fresh-or-unknown start message

What is not verified today:

- a full upstream Claude JSON schema for all hook payloads
- cwd/root/agent metadata as guaranteed `SessionStart` payload fields across
  all launches
- parent/subagent/session lineage fields in Claude hook payloads
- a live `Notification` payload in this harness environment
- whether `CLAUDE_PLUGIN_ROOT` appears in a dedicated plugin-context capture
- whether a resumed Claude session launched from a different directory should
  establish a different immutable runtime root
- whether `CLAUDE_PROJECT_DIR` is present inside ordinary Bash tool subprocesses
  rather than hook-process env

## Verified Claude Worktree Hook Semantics

These two Claude hooks use a provider-specific I/O contract rather than the
normal `PreToolUse` / `PostToolUse` decision-JSON pattern.

### WorktreeCreate

Verified/provider-documented facts:

- `WorktreeCreate` is a top-level hook event
- input is JSON on stdin using the common hook fields plus:
  - `name`
- for command hooks, success output is:
  - absolute worktree path on stdout
- stderr carries rejection/failure detail
- non-zero exit fails worktree creation
- `HookResult` / decision-control JSON does not apply to command hooks on this
  surface

Current local evidence in this branch:

- live Claude `--worktree live-proof -p ...` capture at:
  - `test-harness/hooks/claude/captures/raw/20260331T180025.956819Z-worktree-create.json`
  - `test-harness/hooks/claude/captures/raw/20260331T180025.956819Z-worktree-create.env.json`
- captured payload fields:
  - `cwd`
  - `hook_event_name = "WorktreeCreate"`
  - `name`
  - `session_id`
  - `transcript_path`
- live block behavior:
  - Claude surfaced `WorktreeCreate hook rejected: use /sc-git-worktree instead of EnterWorktree directly`
  - non-zero hook exit blocked worktree creation

Local policy implication:
- a rejecting hook should write a redirect message to stderr and exit non-zero
- the current preferred redirect text is:
  - `use /sc-git-worktree instead of EnterWorktree directly`

### WorktreeRemove

Verified/provider-documented facts:

- `WorktreeRemove` is a top-level hook event
- input is JSON on stdin using the common hook fields plus:
  - `worktree_path`
- the hook is for cleanup side effects, using the path returned by
  `WorktreeCreate`
- it is not part of the `HookResult` / decision-control JSON model

Current local evidence in this branch:

- live Claude `--worktree live-remove-double-ctrl-d` capture at:
  - `test-harness/hooks/claude/captures/raw/20260331T182849.015758Z-worktree-create.json`
  - `test-harness/hooks/claude/captures/raw/20260331T182849.015758Z-worktree-create.env.json`
  - `test-harness/hooks/claude/captures/raw/20260331T182849.195636Z-worktree-remove.json`
  - `test-harness/hooks/claude/captures/raw/20260331T182849.195636Z-worktree-remove.env.json`
- captured `WorktreeRemove` payload fields:
  - `cwd`
  - `hook_event_name = "WorktreeRemove"`
  - `session_id`
  - `transcript_path`
  - `worktree_path`
- live exit behavior:
  - the worktree session exited through the REPL `Ctrl-D` flow
  - Claude surfaced `Removing worktree`
  - the `WorktreeRemove` hook fired with the provider-returned worktree path

Important local harness note:
- the capture-only `worktree_remove.py` hook records the payload but does not
  delete the directory; this proves hook firing and payload shape, not local
  cleanup policy

Current `schook` status:
- documented provider surface only
- not part of the current implemented eight-hook Claude ATM baseline

What is verified by the committed Sprint 9 Phase 3 schema/tooling:

- `SessionStart` also carries optional `model`
- `SessionEnd` may carry optional `reason`
- `PreToolUse(Bash)` and `PreToolUse(Agent)` carry optional
  `permission_mode` and `tool_use_id`
- `PreToolUse(Agent).tool_input` carries verified `description`, `name`, and
  `run_in_background`
- `PostToolUse(Bash).tool_response` is currently observed with
  `stdout`, `stderr`, `interrupted`, `isImage`, and `noOutputExpected`
- `PermissionRequest` may carry optional `permission_mode` and
  `permission_suggestions`
- `Stop` may carry optional `permission_mode` and `last_assistant_message`

Deferred in the Phase 3 schema because the model allows them for future drift
comparison but the current approved fixture set does not prove them yet:

- `PreToolUse(Agent).tool_input.subagent_type`
- `PreToolUse(Agent).tool_input.team_name`
- `PostToolUse(Bash).tool_response.output`
- `PostToolUse(Bash).tool_response.error`

## Session Correlation Model

Claude hook calls should treat identity and context as separate concerns.

Current verified anchor:

1. SessionStart-captured `session_id`
2. the root-establishing `SessionStart` launch directory for the runtime
   instance (`startup` in the current fresh-session captures)
3. `CLAUDE_PROJECT_DIR` in the current hook env snapshots, which matched the
   root-establishing `SessionStart` directory in every captured lifecycle
   source (`startup`, `compact`, `resume`, `clear`) and remained pinned to that
   value in the captured drift scenario
4. `ATM_TEAM` + `ATM_IDENTITY` only as routing labels when inherited, not as a
   unique instance key

Observed facts from the current harness:

- `SessionStart(source="startup")` captured `cwd ==
  CLAUDE_PROJECT_DIR == /Users/randlee/Documents/github/schook-worktrees/feature-s9-hook-env-capture`
- `PreCompact`, `SessionStart(source="compact")`, `SessionEnd(reason="clear")`,
  and `SessionStart(source="clear")` also captured the same
  `CLAUDE_PROJECT_DIR` value in hook env
- the current `resume` capture kept the same `session_id` and the same
  `cwd`/`CLAUDE_PROJECT_DIR` value as the immediately preceding startup run
- after a Claude `cd` into `test-harness/hooks/claude`, the captured
  `PostToolUse(Bash)`, `Stop`, and `SessionEnd` hooks reported the drifted raw
  hook `cwd`, while `CLAUDE_PROJECT_DIR` stayed pinned to the startup root

Runtime rule for `schook`:

- `ai_root_dir` is the immutable working directory for the runtime instance
- `ai_root_dir` must match `CLAUDE_PROJECT_DIR` whenever that env var is present
- if inbound `CLAUDE_PROJECT_DIR` diverges from `ai_root_dir`, the runtime must
  emit a prominent error of the form:
  - `divergence in CLAUDE_PROJECT_DIR from <ai-root-dir> to <current-claude-project-dir>`

Rules:

- directory changes do not change identity
- later raw hook `cwd` values are current-directory context only
- compaction does not change `session_id`
- a fresh Claude process creates a new `session_id`
- `/clear` ends the prior session and starts a new `session_id`
- later hooks should read persisted session state rather than trying to infer
  identity from current working directory or subprocess lineage
- `PPID` can be used as a local diagnostic cross-check, but it is not the
  persisted identity key in the verified Sprint 9 plan

Current verified ATM-backed persistent record fields:

- key by `session_id`
- store `session_id`, `team`, `identity`, `pid`, `created_at`, `updated_at`
- preserve `created_at` on re-fire for the same `session_id`
- refresh `updated_at` when the session file is touched again

This is a statement of the current ATM implementation, not a claim that the
future `schook` base record must stay identical.

Implementation-facing reading for `schook`:

- `session_id` is the verified Claude lifecycle anchor
- `ai_root_dir` is the immutable runtime root established from the
  root-establishing `SessionStart`
- later raw hook `cwd` values map to `ai_current_dir`, not root identity
- inbound `CLAUDE_PROJECT_DIR` is a required equality check against
  `ai_root_dir`, not a silent fallback
- downstream consumers should receive normalized project-root context even if
  Claude later omits or varies the raw env surface
- `active_pid` remains part of the planned runtime identity tuple, but is a
  runtime-managed field rather than a Claude payload contract claim
- `Notification(idle_prompt)` stays outside the verified identity/state model
  until a live payload is captured

## Verified Claude Hook Behaviors

| Behavior | Claude surface | Current script | Fields consumed | Current side effects | Planned `schook` mapping |
| --- | --- | --- | --- | --- | --- |
| Session start | `SessionStart` | `session-start.py` | `session_id`, `source`, ATM repo/env context | prints `SESSION_ID=...`, emits ATM `session_start`, writes session record | `SessionStart` sync plugin |
| Session end | `SessionEnd` | `session-end.py` | `session_id`, `.atm.toml` core routing | emits ATM `session_end`, removes session record | `SessionEnd` sync plugin |
| ATM identity write | `PreToolUse` on `Bash` | `atm-identity-write.py` | `tool_input.command`, `session_id`, ATM repo/env context | writes temp identity file for `atm` commands only | `PreToolUse/Bash` sync plugin |
| ATM identity cleanup | `PostToolUse` on `Bash` | `atm-identity-cleanup.py` | hook routing context only | deletes temp identity file written by the paired pre-hook | `PostToolUse/Bash` sync plugin |
| Agent spawn gate | `PreToolUse` on `Agent` | `gate-agent-spawns.py` | verified: `tool_input.prompt`, optional `description`, optional `run_in_background`; deferred: `subagent_type`, `name`, `team_name`; plus `session_id`, team config | blocks unsafe spawns or mismatched team usage | `PreToolUse/Agent` sync plugin |
| Idle notification relay | `Notification` on `idle_prompt` | `notification-idle-relay.py` | `session_id`, team/agent routing fields | emits ATM idle heartbeat | `Notification/idle_prompt` async-safe sync plugin |
| Permission relay | `PermissionRequest` | `permission-request-relay.py` | `session_id`, `tool_name`, `tool_input`, team/agent routing | emits ATM permission-request event | `PermissionRequest` sync plugin |
| Stop relay | `Stop` | `stop-relay.py` | `session_id`, optional `permission_mode`, optional `last_assistant_message`, team/agent routing | emits ATM stop/idle event | `Stop` sync plugin |

Adjacent but not part of the current eight-hook baseline:

- `atm-hook-relay.py` is a Codex notify relay, not a Claude Code hook
- `teammate-idle-relay.py` is separate team-state plumbing and should be planned
  only if the runtime elevates `TeammateIdle`

## Design Implications For `schook`

- `SessionStart` is the authoritative place to capture `session_id` for later
  hook calls
- the `source` field should be stored as raw payload evidence; any
  fresh/resume/compact classification must be documented as internal logic, not
  as a claimed Claude wire enum
- the runtime should preserve session records across directory changes
- `SessionStart(source="resume")` is now captured evidence, not just a
  documented provider claim
- `/clear` produces `SessionEnd(reason="clear")` followed by a new
  `SessionStart(source="clear")` and a new `session_id`
- Bash-specific hooks need command-sensitive behavior, not just hook-type
  matching
- Agent spawn gating is policy-heavy and should remain separate from generic ATM
  relays
- `PreToolUse(Agent)` currently provides a safe verified baseline of
  `prompt`, optional `description`, and optional `run_in_background`; other
  agent-tool fields stay deferred until captured again
- `PostToolUse(Bash)` should be implemented against `stdout`, `stderr`,
  `interrupted`, `isImage`, and `noOutputExpected`; legacy `output` / `error`
  fields stay deferred
- `Notification(idle_prompt)` stays part of the documented Claude surface, but
  remains DEFERRED until a local payload capture actually lands
- lifecycle and relay hooks are fail-open today; if `schook` changes that
  posture, the change must be explicit in requirements and protocol docs
- no `schook` code should be written against inferred Claude payload fields that
  are not backed by source-of-truth docs, scripts, tests, or captured harness
  fixtures

## Current Platform Gaps

- Claude hook payloads are only partially documented by the vendor, so some
  field names are verified from live scripts rather than a formal upstream
  schema
- `agent-team-mail` does not currently appear to use Pydantic models as the
  Claude hook source of truth; the current baseline is docs + scripts + tests +
  Rust fallback code
- `CLAUDE_SESSION_ID` is stable in the parent Claude process but is not
  directly available to bash subprocesses; `SessionStart` capture is required
- hook env var availability differs sharply between hook execution and ordinary
  Bash tool execution; this document records hook-process captures only and
  does not claim those same vars are present inside ordinary tool subprocesses
- `CLAUDE_PLUGIN_ROOT` remains unverified in the current harness capture set
  and should stay plugin-specific guidance, not generic runtime contract
- `Notification(idle_prompt)` remained uncaptured in the local harness even
  after a bounded idle probe with `matcher = ""`; the current local fact is
  "wired but not firing in this harness pass," not a verified vendor timing
  guarantee
