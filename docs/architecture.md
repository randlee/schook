# sc-hooks — Architecture Document

> Version 0.1.0 — March 2026 — DRAFT

## 1. Overview

sc-hooks is a Rust CLI that serves as a universal hook dispatcher for AI-assisted development. Claude Code (and other AI tools) fire hooks at defined lifecycle points. sc-hooks receives those hooks, routes them through a config-driven handler chain, validates inputs, and returns results—all with structured logging and full auditability.

The system replaces a fragile Python-based hook dispatcher with a compiled, testable, fast alternative that treats plugins as standalone processes communicating via JSON over stdin/stdout.

## 2. Design Principles

- **Simple config:** The TOML config maps hook names to handler names. That is the entire config surface. Handler-specific settings are the handler's concern.
- **JSON as the universal contract:** Metadata, payloads, and results all flow as JSON. No C ABI, no shared memory, no unsafe. Plugins are processes, not libraries.
- **Manifest-declared requirements:** Each plugin advertises what metadata fields it needs and their validation rules. The host validates before invocation. Failures are caught at audit time, not runtime.
- **Language-agnostic plugins:** A plugin is any executable that responds to `--manifest` and reads JSON from stdin. Rust, Python, bash—whatever solves the problem. Swap a Rust plugin for a Python script to debug, swap back when fixed.
- **AI-agnostic dispatch:** The environment contract (minimal env vars + JSON metadata) allows thin shims to adapt non-Claude AI tools (Codex, Gemini) to the same hook system.
- **Fast by default:** Compiled Rust host. Builtins run in-process. External plugins are processes, but the host returns immediately for async follow-up work (the plugin forks its own background tasks).

## 3. System Architecture

### 3.1 Component Overview

| Layer | Artifact | Responsibility |
|-------|----------|----------------|
| Host | sc-hooks-cli (binary) | Config parsing, metadata assembly, plugin resolution, input validation, dispatch, structured logging, audit, diagnostic fire |
| SDK | sc-hooks-sdk (Rust crate) | Optional helper library. Provides manifest builder, stdin/stdout protocol handling, typed result types. Not required—plugins can implement the protocol directly. |
| Plugins | Standalone executables | Implement hook behavior. Advertise requirements via manifest. Receive validated JSON on stdin. Return action result on stdout. |

### 3.2 Execution Flow

```
Claude Code hook fires
  → sc-hooks run <hook-type> [event] --sync|--async
  → load .sc-hooks/config.toml
  → resolve handler chain for hook+event
  → filter chain by mode (sync or async)
  → for each handler in chain:
      → load manifest (cached after first call)
      → validate metadata against manifest requirements
      → pipe validated JSON subset to plugin stdin
      → read result JSON from plugin stdout
      → if action=block or action=error: short-circuit, return to caller
  → log structured dispatch entry to hook log
  → exit code back to Claude Code
```

### 3.3 Plugin Model

A plugin is any executable that implements two behaviors:

- **Manifest response:** When called with `--manifest`, returns a JSON object declaring its name, protocol version, execution mode, supported hooks, required metadata fields, and validation rules.
- **Hook handling:** When called normally, reads a JSON object from stdin (containing only the fields it declared as required/optional), performs its work, and writes a result JSON to stdout.

This means a plugin can be a compiled Rust binary (using sc-hooks-sdk for convenience), a Python script with a shebang line, a bash script that uses jq for JSON processing, or any executable in any language that speaks the protocol.

**Critical design point:** there is no plugin versioning system. If a plugin has a bug, replace the executable with a working one (in any language). The manifest is the version—if the host can read it, the plugin is compatible. The `protocol` field (integer) handles breaking changes to the JSON contract itself.

### 3.4 Metadata Model

Hook context flows as a single JSON object with a stable core structure and extensible subsystem sections:

```json
{
  "agent": { "type": "claude-code", "session_id": "abc123", "pid": 48201, "role": "implementer" },
  "repo": { "path": "/home/rand/src/p3", "branch": "feature/cal-v2", "working_dir": "/home/rand/src/p3/src/calibration" },
  "team": { "name": "calibration", "project": "p3-platform" },
  "hook": { "type": "PreToolUse", "event": "Write" },
  "payload": { ... }
}
```

The host assembles this from three sources: runtime discovery (agent info, repo state), config.toml `[context]` section (team/project), and the hook payload from the AI tool. Plugins only receive the subset of fields they declared in their manifest.

**Environment variables are minimal and intentional**—only set for cases where they're genuinely needed (spawning new processes, shell ergonomics):

```
SC_HOOK_TYPE=PreToolUse
SC_HOOK_EVENT=Write
SC_HOOK_METADATA=/tmp/sc-hooks/meta-xxxx.json
```

Three env vars, not twenty. If a handler needs specific fields, it reads them from the metadata JSON.

## 4. Configuration

```toml
# .sc-hooks/config.toml

[meta]
version = "1"

[context]
team = "calibration"
project = "p3-platform"

[hooks]
PreToolUse = ["guard-paths", "log"]
PostToolUse = ["log", "notify"]
PreCompact = ["save-context", "log"]
```

There are no per-handler configuration blocks in this file. If a handler needs settings, it reads its own configuration (e.g., guard-paths reads `.sc-hooks/guard-paths.toml` or similar). This keeps the dispatcher config auditable at a glance.

## 5. Plugin Protocol

### 5.1 Manifest

Every plugin must respond to the `--manifest` flag with a JSON declaration:

```json
{
  "name": "guard-paths",
  "protocol": 1,
  "mode": "sync",
  "hooks": ["PreToolUse"],
  "requires": {
    "repo.path": { "type": "string", "validate": "dir_exists" },
    "repo.branch": { "type": "string", "validate": "non_empty" },
    "hook.event": { "type": "string" }
  },
  "optional": {
    "team.name": { "type": "string" }
  }
}
```

**The `mode` field** declares the plugin's execution expectation:

- **sync:** Plugin makes blocking decisions (proceed/block/error). Runs in the synchronous hook chain. Must return fast.
- **async:** Plugin performs context collection, logging, or long-running work. Runs in the async hook chain. Cannot block. May return `additionalContext` for the AI tool's next turn.

### 5.2 Validation Rules

| Rule | Meaning |
|------|---------|
| `non_empty` | String with length > 0 |
| `dir_exists` | String, path exists and is a directory |
| `file_exists` | String, path exists and is a file |
| `path_resolves` | String, path resolves (symlinks followed) |
| `one_of:a,b,c` | String, must be one of listed values |
| `positive_int` | Number, greater than 0 |
| *(none)* | Type check only, no additional validation |

### 5.3 Input (Host → Plugin)

The host writes a JSON object to the plugin's stdin containing only the fields declared in `requires` and `optional`:

```json
{
  "repo": { "path": "/home/rand/src/p3", "branch": "feature/cal-v2" },
  "team": { "name": "calibration" },
  "hook": { "type": "PreToolUse", "event": "Write" },
  "payload": { "file_path": "src/calibration/sensor.rs" }
}
```

### 5.4 Output (Plugin → Host)

Sync-mode plugins return an action:

```json
{ "action": "proceed", "log": "path allowed" }
{ "action": "block", "reason": "path matches deny pattern" }
{ "action": "error", "message": "unexpected state" }
```

The host interprets: `proceed` continues the chain, `block` short-circuits and reports the reason to the AI tool, `error` short-circuits and is logged as a failure.

Async-mode plugins return context instead of decisions:

```json
{
  "action": "proceed",
  "additionalContext": "guard-paths verified: src/calibration/sensor.rs",
  "systemMessage": "optional warning shown to user"
}
```

The host aggregates `additionalContext` from all async plugins and passes it through to the AI tool. Async plugins must not return `block`—the audit command enforces this.

## 6. Handler Resolution

Given a handler name from the config, the host resolves it in order:

- **Builtin:** Is the name a handler compiled into the host binary? (e.g., `log`). Run in-process.
- **Plugin executable:** Is there an executable at `.sc-hooks/plugins/{name}`? Call it via the JSON protocol.
- **Unresolved:** Error at audit time. Error at runtime.

The local root folder in config can be changed to point at a different plugins directory. This enables the swap-to-debug workflow: replace a Rust plugin binary with a Python script of the same name, debug, fix, swap back.

## 7. CLI Interface

```
sc-hooks <subcommand>

SUBCOMMANDS:
  run <hook> [event]       Normal execution (called by AI tool hooks)
    --sync                 Run only sync-mode handlers (default)
    --async                Run only async-mode handlers
  audit                    Validate config + all handlers + data flow
  fire <hook> [event]      Diagnostic trigger with synthetic/real payload
  install                  Generate .claude/settings.json hook entries
  config                   Show resolved configuration
  handlers                 List available builtins + discovered plugins
```

### 7.1 Hook Installation

The `install` command reads the sc-hooks config and all plugin manifests, then generates the correct `.claude/settings.json` entries. When a hook event has both sync and async plugins, it installs two handler entries:

```
$ sc-hooks install

Installing hooks for PreToolUse:
  sync chain:  guard-paths, log
  async chain: collect-context
  → 2 entries in .claude/settings.json

Installing hooks for PostToolUse:
  sync chain:  log
  async chain: notify
  → 2 entries

Installing hooks for PreCompact:
  sync chain:  save-context, log
  → 1 entry
```

The generated settings.json:

```jsonc
// .claude/settings.json (generated by sc-hooks install)
"hooks": {
  "PreToolUse": [{
    "matcher": "*",
    "hooks": [
      { "type": "command", "command": "sc-hooks run PreToolUse --sync" },
      { "type": "command", "command": "sc-hooks run PreToolUse --async", "async": true }
    ]
  }],
  "PostToolUse": [{
    "matcher": "*",
    "hooks": [
      { "type": "command", "command": "sc-hooks run PostToolUse --sync" },
      { "type": "command", "command": "sc-hooks run PostToolUse --async", "async": true }
    ]
  }],
  "PreCompact": [{
    "hooks": [
      { "type": "command", "command": "sc-hooks run PreCompact --sync" }
    ]
  }]
}
```

If a hook event has only sync plugins, only one entry is generated. If only async plugins, only the async entry. The user never manually writes these entries—`install` handles it.

## 8. Audit System

The audit command validates the entire hook system without executing any handlers:

```
$ sc-hooks audit

Config: .sc-hooks/config.toml
Protocol: 1

Plugins:
  ✓ guard-paths    (protocol=1, mode=sync)
  ✓ log            (builtin, mode=sync)
  ✓ collect-context (protocol=1, mode=async)
  ✓ notify         (protocol=1, mode=async)

Hook chains:
  PreToolUse:
    sync:  [guard-paths, log]
    async: [collect-context]
    guard-paths requires:
      ✓ repo.path     (dir_exists) → /home/rand/src/p3 exists
      ✓ repo.branch   (non_empty)  → "feature/cal-v2"
      ✓ hook.event    (string)
    collect-context requires:
      ✓ repo.path     (dir_exists)

  PostToolUse:
    sync:  [log]
    async: [notify]
    notify requires:
      ✓ team.name     (non_empty)  → "calibration"
      ✗ atm.inbox     (non_empty)  → missing from context

Install plan:
  PreToolUse  → 2 entries (sync + async)
  PostToolUse → 2 entries (sync + async)
  PreCompact  → 1 entry  (sync only)

1 error: notify requires 'atm.inbox' but it is not provided
```

## 9. Observability

### 9.1 Host Dispatch Log

The host logs every dispatch-level event: which hook fired, which handlers ran, their results, and timing. This is the host's responsibility and the only log the host writes.

```toml
[logging]
hook_log = ".sc-hooks/logs/hooks.jsonl"
level = "info"
```

If a plugin needs additional logging, that is the plugin's responsibility. The host does not provide logging infrastructure to plugins.

### 9.2 Log Entry Structure

```json
{
  "ts": "2026-03-06T10:23:45.123Z",
  "hook": "PreToolUse",
  "event": "Write",
  "mode": "sync",
  "handlers": ["guard-paths", "log"],
  "results": [
    { "handler": "guard-paths", "action": "proceed", "ms": 2 },
    { "handler": "log", "action": "proceed", "ms": 0 }
  ],
  "total_ms": 3,
  "exit": 0
}
```

## 10. Sync/Async Execution Model

Each plugin declares its execution mode in its manifest. The host uses this to split handler chains and generate correct AI tool integration.

### 10.1 Sync Plugins

Sync plugins make blocking decisions. They run sequentially in a chain that short-circuits on block/error. Claude Code waits for the result. Use sync mode for: path guards, permission decisions, input validation—anything that must complete before the tool action proceeds.

### 10.2 Async Plugins

Async plugins perform context collection, notifications, or long-running analysis. Claude Code does not wait for them—it starts the async chain and continues immediately. When the async chain completes, any `additionalContext` or `systemMessage` is delivered to the AI on the next conversation turn.

Async plugins cannot return `block`. The audit command enforces this constraint.

### 10.3 Chain Splitting

When a single hook event (e.g., PreToolUse) has both sync and async plugins, `sc-hooks install` generates two Claude Code hook entries for that event. The `--sync` invocation runs only sync-mode plugins; the `--async` invocation runs only async-mode plugins. This is invisible to the plugin author—each plugin only sees its own chain.

Example: `PreToolUse = ["guard-paths", "collect-context"]` where guard-paths is sync and collect-context is async:

- Claude Code fires PreToolUse
- Two sc-hooks processes start: one sync (blocking), one async (background)
- Sync chain: guard-paths runs, returns proceed/block
- Async chain: collect-context runs in background, returns additionalContext on next turn

### 10.4 Handler-Internal Async

Independent of the sync/async chain split, any plugin (sync or async) may internally fork a detached child process for follow-up work. A sync plugin that sends an ATM notification can fork the notification send and return proceed immediately. The dispatcher does not know about the fork. This is the plugin's concern.

## 11. AI-Agnostic Shims

Because the integration contract is env vars + JSON stdin, adapting non-Claude AI tools requires only a thin shim:

```bash
#!/bin/bash
# codex-hook-shim.sh
export SC_HOOK_AGENT_TYPE=codex
export SC_HOOK_SESSION_ID="${CODEX_SESSION_ID:-unknown}"
export SC_HOOK_AGENT_PID=$$

case "$1" in
  pre-edit)  exec sc-hooks run PreToolUse Write ;;
  post-edit) exec sc-hooks run PostToolUse Write ;;
  *)         exec sc-hooks run "$@" ;;
esac
```

Handlers do not care which AI tool invoked them. They read metadata JSON. Same handlers, same logging, same audit.

## 12. Project Structure

```
sc-hooks/
├── Cargo.toml                # workspace
├── sc-hooks-sdk/             # optional helper for Rust plugins
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── manifest.rs       # Manifest builder
│       ├── validate.rs       # Validation rule types
│       ├── runner.rs         # --manifest + stdin/stdout protocol
│       └── result.rs         # HookResult enum
├── sc-hooks-cli/             # the host binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # clap: run, audit, fire, install
│       ├── config.rs         # TOML parsing
│       ├── metadata.rs       # assemble JSON from config + runtime
│       ├── resolve.rs        # name → builtin | plugin binary
│       ├── manifest.rs       # call --manifest, parse + cache
│       ├── validate.rs       # validate metadata against requirements
│       ├── dispatch.rs       # pipe JSON → plugin → read result
│       ├── install.rs        # generate .claude/settings.json
│       ├── builtins/
│       │   └── log.rs
│       ├── audit.rs
│       ├── fire.rs
│       └── logging.rs
├── plugins/                  # each compiles independently
│   ├── guard-paths/
│   ├── notify/
│   └── save-context/
└── shims/
    ├── codex-shim.sh
    └── gemini-shim.sh
```

## 13. Open Design Decisions

- **Config discovery:** Walk up from CWD to find `.sc-hooks/config.toml` (mirrors `.claude/` behavior), or explicit `--config` flag, or both?
- **Manifest caching:** Cache manifests in memory per invocation, or persist to disk with invalidation on plugin mtime change?
- **Config inheritance:** Should a global `~/.config/sc-hooks/config.toml` provide defaults that repo-level configs extend? Useful for logging defaults.
- **Diagnostic mode design:** The `fire` command needs design for payload generation. Synthetic payloads, recorded payloads, or both?
- **Plugin marketplace:** Integration with Synaptic Canvas package registry for plugin distribution? Separate concern but worth tracking.
