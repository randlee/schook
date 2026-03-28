# Phase BC Hook Runtime Design

## Purpose

This document freezes the post-capture hook runtime design for `schook` before
runtime implementation begins.

It is subordinate to the three control docs:

- `docs/requirements.md`
- `docs/architecture.md`
- `docs/project-plan.md`

It exists to hold the detailed design that those three control docs reference,
not to compete with them.

## Captured Claude Baseline

The current Claude baseline is evidence-driven.

Verified local captures:

- `SessionStart.source = startup`
- `SessionStart.source = compact`
- `SessionStart.source = resume`
- `SessionStart.source = clear`
- `PreCompact`
- `PreToolUse(Bash)`
- `PostToolUse(Bash)`
- `PreToolUse(Agent)`
- `PermissionRequest` for `Write`
- `PermissionRequest` for `Bash`
- `Stop`
- `SessionEnd`

Important observed facts:

- Claude spawn requests arrive as `PreToolUse` with `tool_name = "Agent"`, not
  `Task`.
- `Stop` is the reliable observed transition back to normalized idle.
- `CLAUDE_PROJECT_DIR` is the authoritative project-root signal at
  `SessionStart`.
- `SessionStart` stdin carries `session_id` and `source`; it does not carry cwd
  or root-path fields.
- `Notification(idle_prompt)` remains part of the documented Claude surface, but
  it is currently wired-but-unresolved in local Haiku capture.

## Design Rules

1. Hook state has one source of truth.
2. `project_root_dir` is never guessed from cwd.
3. State and logging stay single-process; no daemon sits in the critical path.
4. Hook logging is mandatory for every invocation initially.
5. Raw hook events and normalized runtime state are separate concepts.
6. ATM behavior stays in a separate extension crate and does not become the
   generic runtime contract.

## Canonical Session-State File

### Storage Rules

- One JSON file per `session_id`
- Disk is the source of truth; in-memory state is a working copy only
- Every `session.json` update must use an atomic write in the same directory
  (`temp + rename`); in-place mutation is forbidden
- If the atomic rename step fails after the temp file is written:
  - the temp file must be explicitly removed
  - the previous canonical state file remains the source of truth
  - the failure must be logged through `sc-observability`
  - stderr and hook logs must include the temp-file path
  - the runtime must surface the error instead of silently reporting success
- No daemon cache is authoritative for hook-state correctness
- Session-state storage must not use `/tmp`
- If the canonical session record is unchanged after handler execution, the
  runtime must not rewrite the file
- `state_revision` increments only on persisted material change
- Hook logging remains mandatory even when no state write occurs

### Storage Root Resolution

- The runtime base-directory signal is `ATM_HOME`.
- Path resolution must follow the standard ATM home lookup:
  1. non-empty `ATM_HOME`
  2. platform home directory from the canonical ATM home resolver
- The canonical BC session-state directory is:
  - `<atm_home>/.atm/hooks/state/sessions/`
- The canonical BC hook-log directory remains owned by `sc-observability`; hook
  state must not invent a second log root.
- All paths must be constructed with path-join APIs, not string concatenation.
- Hardcoded absolute paths, `/tmp`, and Unix-only separators are forbidden.
- Cross-platform path behavior must follow
  [cross-platform-guidelines.md](cross-platform-guidelines.md).

### Canonical Schema

```json
{
  "schema_version": "v1",
  "provider": "claude",
  "session_id": "9e6e0d07-2f90-4b24-8f5a-5efcd4123456",
  "active_pid": 12345,
  "parent_session_id": null,
  "parent_active_pid": null,
  "project_root_dir": "/repo/root",
  "session_start_source": "startup",
  "agent_state": "starting",
  "state_revision": 1,
  "created_at": "2026-03-27T22:00:00Z",
  "updated_at": "2026-03-27T22:00:00Z",
  "ended_at": null,
  "last_hook_event": "SessionStart",
  "last_hook_event_at": "2026-03-27T22:00:00Z",
  "state_reason": "session_started",
  "extensions": {
    "atm": {
      "atm_team": "atm-dev",
      "atm_identity": "team-lead"
    }
  }
}
```

### Required Fields

- `session_id`
- `active_pid`
- `project_root_dir`
- `agent_state`
- `session_start_source`
- `state_revision`
- timestamps and last-event fields

The minimal stable association is:

- `session_id`
- `active_pid`
- `project_root_dir`

## Normalized Agent State

### Enum

- `starting`
- `busy`
- `awaiting_permission`
- `compacting`
- `idle`
- `ended`

This is a runtime enum, not typestate, because it must round-trip through JSON
across separate hook processes.

### Transition Model

| Raw Event | Condition | New State |
| --- | --- | --- |
| `SessionStart` | `source = startup` | `starting` |
| `SessionStart` | `source = resume` | `starting` |
| `SessionStart` | `source = clear` | `starting` |
| `SessionStart` | `source = compact` | `starting` |
| `PreToolUse(*)` | any tool | `busy` |
| `PermissionRequest` | approval needed | `awaiting_permission` |
| `PreCompact` | compaction begins | `compacting` |
| `Stop` | turn completed | `idle` |
| `SessionEnd` | any reason | `ended` |

`Notification(idle_prompt)` may be logged when present, but it is not the
primary idle transition.

## Hook Execution Path

Every hook invocation follows one path:

1. Parse raw JSON stdin and required env
2. Resolve canonical context:
   - `session_id`
   - `active_pid`
   - `project_root_dir`
3. Load the canonical session file if it exists, or create it on
   `SessionStart`
4. Build normalized context from raw event plus persisted state
5. Resolve handlers in deterministic order
6. Execute handlers
7. Collect handler results
8. Compute the normalized state transition
9. Perform an atomic write only if the canonical state materially changed
10. Emit the structured hook log record through `sc-observability`
11. Return final hook JSON to the runtime

## Logging Contract

Hook logging is mandatory for 100% of invocations in the initial BC
implementation.

Required per-invocation fields:

- `ts`
- `provider`
- `hook_event`
- `session_id`
- `active_pid`
- `project_root_dir`
- `agent_state_before`
- `agent_state_after`
- `matched_handlers`
- `handler_results`
- `host_result`
- `state_revision`

Optional ATM extension fields:

- `atm_team`
- `atm_identity`

## Error Types

`sc-hooks-core` owns the canonical hook-runtime error enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("invalid payload near {input_excerpt}")]
    InvalidPayload {
        input_excerpt: String,
        #[source]
        source: Option<serde_json::Error>,
    },
    #[error("invalid context: {message}")]
    InvalidContext { message: String },
    #[error("state I/O failed for {path}")]
    StateIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("validation failed for {field}")]
    Validation {
        field: String,
        message: String,
    },
    #[error("internal error in {component}")]
    Internal {
        component: &'static str,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}
```

Rules:

- library crates use canonical typed errors, not `anyhow` in their public
  contract
- `HookError` variants are stable across runtime crates
- parse failures are represented as `HookError::InvalidPayload` with a source
  `serde_json::Error` and an excerpt of the offending input
- no hook error may discard an originating source error through bare `?`
  propagation without wrapping it in a parent `HookError` that preserves the
  source chain
- fail-open versus fail-closed posture is determined by the handler/crate
  contract, not by ad hoc string matching

## Error Posture Matrix

| Crate | Primary responsibility | Error posture | Canonical error type |
| --- | --- | --- | --- |
| `sc-hooks-core` | shared types, state transitions, persistence contract | library surface only | `HookError` |
| `sc-hooks-sdk` | provider adapters, handler registry, observability bridge | fail-open adapter surface unless an upstream gate explicitly blocks | `HookError` |
| `sc-hooks-session-foundation` | canonical session-state ownership and lifecycle hooks | fail-open | `HookError` |
| `sc-hooks-agent-spawn-gates` | `PreToolUse(Agent)` policy checks | fail-closed | `HookError` |
| `sc-hooks-tool-output-gates` | fenced JSON validation for tools and agents | fail-closed | `HookError` |
| `sc-hooks-atm-extension` | ATM routing and identity enrichment | fail-open | `HookError` |

## Planned Crate Split

- `sc-hooks-core`
  - canonical types
  - state-transition engine
  - sealed trait and canonical error types
- `sc-hooks-sdk`
  - provider adapters
  - handler registration
  - logging bridge to `sc-observability`
- `sc-hooks-session-foundation`
  - `SessionStart`
  - `SessionEnd`
  - `PreCompact`
  - persisted session-state ownership
- `sc-hooks-agent-spawn-gates`
  - `PreToolUse(Agent)` policy checks
  - named-agent vs background-agent rules
- `sc-hooks-tool-output-gates`
  - fenced JSON validation for tool and agent payloads
- `sc-hooks-atm-extension`
  - ATM routing and identity enrichment only

Legacy prototype names such as `atm-session-lifecycle`,
`atm-bash-identity`, `gate-agent-spawns`, and `atm-state-relay` are retired
planning names, not the clean BC authority.

## Trait Boundary

The canonical trait lives in `sc-hooks-core` and is sealed:

```rust
mod private {
    pub trait Sealed {}
}

pub trait HookHandler: private::Sealed {
    fn id(&self) -> &'static str;
    fn handles(&self, event: &HookInvocation) -> bool;
    fn evaluate(&self, event: &HookInvocation) -> Result<HookEffect, HookError>;
}
```

External plugin executables do not implement the trait directly. They
communicate with the runtime through normalized JSON contracts.

## Identity Types

The identity triple uses mandatory newtypes:

- `SessionId(String)`
- `ActivePid(u32)`
- `ProjectRootDir(PathBuf)`

Rules:

- validate at construction time
- no `Deref` to inner types
- explicit conversion only

## Fenced JSON Policy

When an agent or tool requires structured input:

1. The schema must live in the agent prompt contract or a sibling schema file
   with the same stem.
2. Exactly one fenced `json` block is required.
3. The runtime validates the payload against the schema.
4. Invalid structured input blocks with exact, retryable validation errors.

## Phase Mapping

- Hook Phase 1: harness scaffold and live capture
- Hook Phase 2: captured-evidence revision and BC design freeze
- Hook Phase 3: session foundation implementation
- Hook Phase 4: agent spawn and tool gates
- Hook Phase 5: relay and ATM extension behavior

Implementation starts only after the captured baseline and this BC design are
accepted together.
