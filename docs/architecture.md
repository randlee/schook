# sc-hooks — Architecture Document

> Version 0.2.1 — March 2026 — DRAFT

## 1. Overview

sc-hooks is a Rust CLI that serves as a universal hook dispatcher for AI-assisted development. Claude Code (and other AI tools) fire hooks at defined lifecycle points. sc-hooks receives those hooks, routes them through a config-driven handler chain, validates inputs, and returns results—all with structured logging and full auditability.

The system replaces a fragile Python-based hook dispatcher with a compiled, testable, fast alternative that treats plugins as standalone processes communicating via JSON over stdin/stdout.

## 2. Design Principles

- **Simple config:** The TOML config maps hook names to handler names. That is the entire config surface. Handler-specific settings are the handler's concern.
- **JSON as the universal contract:** Metadata, payloads, and results all flow as JSON. No C ABI, no shared memory, no unsafe. Plugins are processes, not libraries.
- **Manifest-declared requirements:** Each plugin advertises what metadata fields it needs, their validation rules, what events it matches, and its execution characteristics. The host validates before invocation. Failures are caught at audit time, not runtime.
- **Language-agnostic plugins:** A plugin is any executable that responds to `--manifest` and reads JSON from stdin. Rust, Python, bash—whatever solves the problem. Swap a Rust plugin for a Python script to debug, swap back when fixed.
- **AI-agnostic dispatch:** The environment contract (minimal env vars + JSON metadata) allows thin shims to adapt non-Claude AI tools (Codex, Gemini) to the same hook system.
- **Fast by default:** Compiled Rust host. Builtins run in-process. External plugins are processes, but the host returns immediately for async follow-up work (the plugin forks its own background tasks).
- **Fail safe, fail loud:** Plugins that violate the protocol (invalid JSON, unexpected exits, timeout) are disabled and the error is reported to both the dispatch log and the AI session. A broken plugin must never silently corrupt the hook chain.
- **Sandbox-aware:** The host respects the sandboxing rules of the calling AI tool. Plugins declare what filesystem paths and resources they need; the host validates access before invocation and provides an override mechanism for authorized cases.

## 3. System Architecture

### 3.1 Component Overview

| Layer | Artifact | Responsibility |
|-------|----------|----------------|
| Host | sc-hooks-cli (binary) | Config parsing, metadata assembly, plugin resolution, input validation, dispatch, timeout enforcement, structured logging, audit, diagnostic fire, exit code management |
| SDK | sc-hooks-sdk (Rust crate) | Trait-based plugin framework: `SyncHandler`, `AsyncHandler`, `LongRunning`, `AsyncContextSource`. Manifest builder, stdin/stdout protocol handling, typed result types. Pre-made handler implementations for common patterns. |
| Test Harness | sc-hooks-test (Rust crate) | Plugin compliance testing: manifest validation, protocol conformance, timeout behavior, error handling, matcher verification. Used by plugin authors in their test suites. |
| Plugins | Standalone executables | Implement hook behavior. Advertise requirements and matchers via manifest. Receive validated JSON on stdin. Return action result on stdout. |
| Pre-made Plugins | Bundled executables | Common-case handlers shipped with sc-hooks: conditional file source, Jinja2 template source, log, notify. |

### 3.2 Execution Flow

```
Claude Code hook fires (event-specific via matcher)
  → sc-hooks run <hook-type> [event] --sync|--async
  → load .sc-hooks/config.toml
  → resolve handler chain for hook+event
  → filter chain by mode (sync or async)
  → for each handler in chain:
      → load manifest (cached after first call)
      → verify event matches plugin's declared matchers
      → validate metadata against manifest requirements
      → start timeout clock (default or plugin-declared)
      → pipe validated JSON subset to plugin stdin
      → read result JSON from plugin stdout
      → if timeout exceeded: kill plugin, log error, disable for session
      → if invalid JSON returned: log error, disable plugin, notify AI session
      → if action=block or action=error: short-circuit, return to caller
  → log structured dispatch entry to hook log
  → exit with defined exit code
```

### 3.3 Plugin Model

A plugin is any executable that implements two behaviors:

- **Manifest response:** When called with `--manifest`, returns a JSON object declaring its name, contract version, execution mode, supported hooks, **event matchers**, required metadata fields, validation rules, and execution characteristics (timeout expectations, long-running declaration).
- **Hook handling:** When called normally, reads a JSON object from stdin (containing only the fields it declared as required/optional), performs its work, and writes a result JSON to stdout.

This means a plugin can be a compiled Rust binary (using sc-hooks-sdk for convenience), a Python script with a shebang line, a bash script that uses jq for JSON processing, or any executable in any language that speaks the protocol.

**Critical design point:** there is no plugin versioning system. If a plugin has a bug, replace the executable with a working one (in any language). The manifest is the version—if the host can read it, the plugin is compatible. The `contract_version` field (integer) handles breaking changes to the JSON contract itself (see §5.1).

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
version = 1

[context]
team = "calibration"
project = "p3-platform"

[hooks]
PreToolUse = ["guard-paths", "log"]
PostToolUse = ["log", "notify"]
PreCompact = ["save-context", "log"]

[logging]
hook_log = ".sc-hooks/logs/hooks.jsonl"
level = "info"

[sandbox]
allow_network = ["notify"]
allow_paths = { "guard-paths" = [".sc-hooks/guard-paths.toml"] }
```

The config has exactly five recognized sections:

| Section | Required | Purpose |
|---------|----------|---------|
| `[meta]` | **yes** | Config format version (integer). |
| `[hooks]` | **yes** | Maps hook type names to ordered arrays of handler names. |
| `[context]` | no | Static key-value pairs merged into metadata JSON (see §4.1 for mapping rules). |
| `[logging]` | no | Dispatch log path and level. Defaults: `hook_log = ".sc-hooks/logs/hooks.jsonl"`, `level = "info"`. |
| `[sandbox]` | no | Explicit sandbox overrides for plugins that need expanded access (see §14). |

Any top-level section not in this list is a parse error. There are no per-handler configuration blocks in this file. If a handler needs its own settings, it reads its own configuration file (e.g., guard-paths reads `.sc-hooks/guard-paths.toml`). This keeps the dispatcher config auditable at a glance.

### 4.1 Context Mapping Rules

The `[context]` section provides flat key-value pairs that the host maps into nested JSON paths under a deterministic scheme:

```toml
[context]
team = "calibration"
project = "p3-platform"
```

Maps to:
```json
{ "team": { "name": "calibration" }, "project": "p3-platform" }
```

**Rules:**
- The key `team` is special: its value becomes `team.name` in the metadata JSON. This is the only implicit nesting.
- All other keys map to top-level fields in the metadata JSON: `project = "foo"` → `metadata.project = "foo"`.
- Dot-notation keys in context are literal (not expanded): `foo.bar = "x"` → `metadata["foo.bar"] = "x"`. Use this for custom fields that plugins reference via dot-path requires.
- Plugin `requires` fields like `team.name` are satisfied by `[context] team = "calibration"` via the special-case mapping.
- Plugin `requires` fields for paths not produced by context or runtime discovery will fail validation at audit time.

## 5. Plugin Protocol

### 5.1 Manifest

Every plugin must respond to the `--manifest` flag with a JSON declaration:

```json
{
  "name": "guard-paths",
  "contract_version": 1,
  "mode": "sync",
  "hooks": ["PreToolUse"],
  "matchers": ["Write", "Bash", "Edit"],
  "timeout_ms": 5000,
  "long_running": false,
  "requires": {
    "repo.path": { "type": "string", "validate": "dir_exists" },
    "repo.branch": { "type": "string", "validate": "non_empty" },
    "hook.event": { "type": "string" }
  },
  "optional": {
    "team.name": { "type": "string" }
  },
  "sandbox": {
    "paths": [".sc-hooks/guard-paths.toml"],
    "needs_network": false
  }
}
```

**Field reference:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Plugin identifier. Must match the executable name. |
| `contract_version` | integer | yes | JSON contract version this plugin speaks (see §5.7 for compatibility rules). |
| `mode` | string | yes | `"sync"` or `"async"`. Determines which chain the plugin runs in. |
| `hooks` | array | yes | Hook types this plugin handles (e.g., `["PreToolUse", "PostToolUse"]`). |
| `matchers` | array | yes | Event patterns this plugin matches. `["*"]` matches all events. `["Write", "Bash"]` matches only those events. Used by `sc-hooks install` to generate precise Claude Code `matcher` entries. |
| `timeout_ms` | integer | no | Expected maximum execution time in milliseconds. Default: 5000ms for sync, 30000ms for async. The host kills the plugin if it exceeds this. |
| `long_running` | boolean | no | If `true`, the host applies an extended timeout (or no timeout) for this plugin. Use sparingly—only for plugins that genuinely block on external input (e.g., waiting for a Slack response). Must be justified in the manifest `description` field. Audit warns on long-running plugins. |
| `response_time` | object | no | Async plugins only. Declares expected response time range: `{"min_ms": 10, "max_ms": 100}`. Used by the host to group async plugins into time-bucketed hook entries for efficient aggregation (see §10). |
| `requires` | object | yes | Metadata fields the plugin needs, with dot-path keys and validation rules. |
| `optional` | object | no | Metadata fields the plugin can use but doesn't require. |
| `sandbox` | object | no | Resource declarations for sandbox compliance (see §14). |
| `description` | string | no | Human-readable description. Required if `long_running` is true (must justify why). |

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

The host writes a JSON object to the plugin's stdin containing two categories of data:

1. **Metadata fields** — only the fields declared in `requires` and `optional`, filtered from the assembled metadata.
2. **Payload** — the `payload` field is a **passthrough**: it is always included verbatim from the AI tool's hook invocation if present, regardless of whether the plugin declares it in `requires` or `optional`. It is not subject to metadata filtering. If the AI tool sends no payload, the `payload` field is omitted entirely (not null, not empty object).

```json
{
  "repo": { "path": "/home/rand/src/p3", "branch": "feature/cal-v2" },
  "team": { "name": "calibration" },
  "hook": { "type": "PreToolUse", "event": "Write" },
  "payload": { "file_path": "src/calibration/sensor.rs" }
}
```

The `hook` field (`hook.type` and `hook.event`) is also always included as context, regardless of `requires` declarations. Plugins must handle the absent-payload case gracefully.

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

The host aggregates `additionalContext` from async plugins within the same time bucket and passes it through to the AI tool (see §10 for aggregation strategy). Async plugins must not return `block`—the audit command enforces this.

### 5.5 Error Handling & Plugin Validation

The host enforces strict protocol compliance. A plugin that violates the protocol is **disabled for the remainder of the session** and the error is reported to both the dispatch log and the AI session.

| Failure Mode | Host Behavior |
|-------------|---------------|
| Plugin writes invalid JSON to stdout | Disable plugin. Log error. Report to AI: `"hook <name> returned invalid JSON — disabled. Please notify user!"` |
| Plugin exits non-zero without output | Disable plugin. Log exit code and any stderr. Report to AI. |
| Plugin exceeds timeout | Kill process (SIGTERM, then SIGKILL after 1s). Disable plugin. Log timeout. Report to AI. |
| Plugin writes multiple JSON objects | First valid object is used. Warning logged. |
| Plugin writes to stderr | Captured and included in dispatch log entry. Not forwarded to AI unless the plugin also fails. |
| Manifest missing required fields | Rejected at audit time. Not loaded at runtime. |
| Manifest declares `mode=async` but returns `action=block` | Treated as protocol error. Disable plugin. Log violation. |

**Principle:** A broken plugin must never silently corrupt the hook chain or leave Claude Code in an undefined state. Every failure is visible and actionable.

#### Disable State Persistence

Since sc-hooks is invoked as a one-shot CLI process per hook fire, disable state must be persisted across invocations. The host maintains a **session state file** at `.sc-hooks/state/session.json` keyed by the AI tool's session ID (from `SC_HOOK_SESSION_ID` env var or discovery). This file records:

```json
{
  "session_id": "abc123",
  "disabled_plugins": {
    "broken-plugin": { "reason": "invalid_json", "disabled_at": "2026-03-06T10:23:46Z" }
  }
}
```

On each invocation, the host reads this file and skips disabled plugins. The file is cleaned up when the session ends (via `SessionEnd` hook) or when `sc-hooks audit --reset` is run. If the file is missing or unreadable, all plugins are considered enabled (fail-open for state, fail-safe for protocol).

### 5.6 Timeout & Long-Running Plugins

Every plugin invocation has a timeout enforced by the host:

| Mode | Default Timeout | Override |
|------|----------------|----------|
| Sync | 5,000ms | Plugin declares `timeout_ms` in manifest |
| Async | 30,000ms | Plugin declares `timeout_ms` in manifest |
| Long-running | No default limit | Plugin declares `long_running: true` + `timeout_ms` (optional hard cap) |

**Long-running plugins** implement the `LongRunning` trait (SDK) or declare `"long_running": true` in their manifest. This signals to the host that the plugin may block on external input (e.g., send a Slack message and wait for user response, prompt for MFA, wait for CI status). The host:

1. Applies the plugin's declared `timeout_ms` if present, otherwise no timeout
2. Logs a warning at audit time: "long-running plugin detected"
3. Requires the manifest to include a `description` explaining why long-running is needed
4. Monitors the plugin process and reports status to the dispatch log at intervals

**The `LongRunning` trait** (SDK):
```rust
pub trait LongRunning: SyncHandler {
    /// Human-readable justification for why this handler blocks.
    fn justification(&self) -> &str;

    /// Optional hard timeout. None = no limit (host will still log periodic status).
    fn hard_timeout(&self) -> Option<Duration>;

    /// Called periodically by the SDK runner to report progress.
    /// Return false to signal the host should cancel.
    fn progress(&self) -> bool { true }
}
```

This allows flexibility for corner cases (Slack approval flows, external CI waits) while making the decision explicit and auditable.

### 5.7 Contract Version Compatibility

The `contract_version` field in the manifest declares which version of the host↔plugin JSON contract the plugin implements. This is **not** a plugin version — it's the protocol envelope version.

**Compatibility matrix:**

| Host Version | Plugin Version | Behavior |
|-------------|---------------|----------|
| 1 | 1 | Full compatibility. |
| 2 | 1 | Host adapts: sends v1-format input, accepts v1-format output. Plugin works unchanged. |
| 2 | 2 | Full compatibility. |
| 1 | 2 | **Incompatible.** Audit reports error. Plugin not loaded at runtime. |
| N | M (M > N) | Incompatible. Audit error. |
| N | M (M ≤ N) | Host adapts to plugin's declared version. |

**Rules:**
- The host always adapts **downward** to the plugin's declared version. A v3 host can talk to v1, v2, or v3 plugins.
- The host never adapts **upward**. A v1 host cannot invoke a v2 plugin because it doesn't know the v2 schema.
- When the contract version increments, the changelog must document exactly which input/output fields changed and how the host adapts for older plugins.
- For MVP, only contract version 1 exists. The compatibility machinery is designed but not exercised until v2 is introduced.

### 5.8 Async Timeout & Exit Semantics

For **sync** invocations, timeout produces exit code 6 (`TIMEOUT`), which Claude Code interprets as a tool-use failure.

For **async** invocations, timeout is handled differently:
- The timed-out plugin is disabled (same as sync).
- The async chain's overall result is still `proceed` (exit 0) — async failures do not block tool use.
- The timeout is logged with `"async_timeout": true` in the dispatch log entry.
- An `ai_notification` is included so the AI session is informed, but it arrives as context on the next turn, not as a blocking error.

This means async plugin failures degrade gracefully: the AI tool continues working, and the user/AI is notified that context from the failed plugin is unavailable.

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
  test <plugin>            Run compliance tests against a plugin
  exit-codes               Show exit code reference
```

### 7.1 Exit Codes

The host uses structured exit codes so callers (Claude Code, CI, scripts) can programmatically determine what happened:

| Code | Name | Meaning |
|------|------|---------|
| 0 | `SUCCESS` | All handlers returned `proceed`. |
| 1 | `BLOCKED` | A sync handler returned `action=block`. Reason on stderr. |
| 2 | `PLUGIN_ERROR` | A handler returned `action=error` or produced invalid output. |
| 3 | `CONFIG_ERROR` | Config file missing, malformed, or invalid. |
| 4 | `RESOLUTION_ERROR` | One or more handlers could not be resolved. |
| 5 | `VALIDATION_ERROR` | Metadata validation failed for a handler's requirements. |
| 6 | `TIMEOUT` | A handler exceeded its timeout. |
| 7 | `AUDIT_FAILURE` | `sc-hooks audit` found errors. |
| 10 | `INTERNAL_ERROR` | Unexpected host error (panic, I/O failure). |

Run `sc-hooks exit-codes` for the full reference with descriptions and suggested remediation:

```
$ sc-hooks exit-codes
Exit Code Reference:
  0  SUCCESS           All handlers proceeded successfully.
  1  BLOCKED           A sync handler blocked the action.
                       → Check stderr for the block reason.
                       → Review the blocking plugin's deny rules.
  2  PLUGIN_ERROR      A handler returned an error or violated protocol.
                       → Check dispatch log for details.
                       → Run 'sc-hooks test <plugin>' to diagnose.
  3  CONFIG_ERROR      Configuration is invalid.
                       → Run 'sc-hooks config' to see parsed output.
                       → Run 'sc-hooks audit' for detailed validation.
  ...
```

### 7.2 Hook Installation

The `install` command reads the sc-hooks config and all plugin manifests, then generates the correct `.claude/settings.json` entries. **Plugins declare their event matchers** in their manifest, and `install` uses these to generate precise Claude Code `matcher` entries—not blanket `"*"` wildcards.

#### Claude Code Matcher Semantics

Claude Code evaluates hook entries in order and runs **all matching entries** (not first-match). Given multiple entries for the same hook type, every entry whose `matcher` matches the event will fire. This means:

- An entry with `"matcher": "Write"` fires only for Write events.
- An entry with `"matcher": "*"` fires for all events.
- Both can fire for the same event — Claude Code runs all matches.

`sc-hooks install` leverages this: specific matchers handle targeted plugins, and wildcard matchers handle plugins that apply to everything. There is no exclusion — a `*` entry always fires alongside specific entries.

#### Install Algorithm

The install algorithm is deterministic:

1. For each hook type in `[hooks]`, collect all handler names.
2. Resolve each handler and load its manifest (matchers, mode).
3. Compute the **union of all specific matchers** (non-`*`) across all plugins for this hook type.
4. For each specific matcher (e.g., `"Write"`):
   a. Collect all plugins whose matchers include this event OR include `"*"`.
   b. Split into sync and async groups.
   c. Generate one Claude Code hook entry per non-empty group.
5. If any plugins have **only** `"*"` matchers (no specific matchers at all), generate a `"*"` entry for them. This covers events not explicitly listed by any plugin.
6. If ALL plugins use `"*"`, generate a single `"*"` entry (no per-event entries needed).

#### Event Taxonomy

Valid events per hook type (based on Claude Code's hook system):

| Hook Type | Valid Events |
|-----------|-------------|
| `PreToolUse` | `Bash`, `Read`, `Write`, `Edit`, `Glob`, `Grep`, `WebFetch`, `WebSearch`, `Agent`, `NotebookEdit`, `TodoWrite`, `AskFollowup`, `SendMessage`, `Task`, `*` (any tool) |
| `PostToolUse` | Same as PreToolUse |
| `PreCompact` | *(no event sub-types — always fires)* |
| `PostCompact` | *(no event sub-types — always fires)* |
| `SessionStart` | *(no event sub-types — always fires)* |
| `SessionEnd` | *(no event sub-types — always fires)* |
| `Notification` | *(no event sub-types — always fires)* |

Lifecycle hooks (`PreCompact`, `PostCompact`, `SessionStart`, `SessionEnd`, `Notification`) have no event sub-types. Plugins for these hooks should declare `matchers: ["*"]`. The `audit` command validates matcher names against this taxonomy.

**Note:** This taxonomy reflects current Claude Code hook points. As Claude Code evolves, new hook types and events may be added. The `audit` command will warn on unrecognized hook types or events but not fail, to allow forward compatibility.

#### Example

```
$ sc-hooks install

Installing hooks for PreToolUse:
  guard-paths (sync) matches: Write, Bash, Edit
  log (sync) matches: *
  collect-context (async) matches: Write, Bash

  Generated entries:
    matcher "Write" → sync: [guard-paths, log] + async: [collect-context]  (2 hooks)
    matcher "Bash"  → sync: [guard-paths, log] + async: [collect-context]  (2 hooks)
    matcher "Edit"  → sync: [guard-paths, log]                            (1 hook)
    matcher "*"     → sync: [log]                                          (1 hook, for all other events)

Installing hooks for PostToolUse:
  log (sync) matches: *
  notify (async) matches: *
  → matcher "*" → 2 hooks (sync + async)
```

Generated settings.json:

```jsonc
// .claude/settings.json (generated by sc-hooks install)
"hooks": {
  "PreToolUse": [
    {
      "matcher": "Write",
      "hooks": [
        { "type": "command", "command": "sc-hooks run PreToolUse Write --sync" },
        { "type": "command", "command": "sc-hooks run PreToolUse Write --async", "async": true }
      ]
    },
    {
      "matcher": "Bash",
      "hooks": [
        { "type": "command", "command": "sc-hooks run PreToolUse Bash --sync" },
        { "type": "command", "command": "sc-hooks run PreToolUse Bash --async", "async": true }
      ]
    },
    {
      "matcher": "Edit",
      "hooks": [
        { "type": "command", "command": "sc-hooks run PreToolUse Edit --sync" }
      ]
    },
    {
      "matcher": "*",
      "hooks": [
        { "type": "command", "command": "sc-hooks run PreToolUse --sync" }
      ]
    }
  ]
}
```

**Key principle:** There is zero reason to invoke sc-hooks if the event doesn't match any plugin. The `install` command ensures Claude Code only calls sc-hooks when at least one plugin is interested in the event.

**Note (post-MVP):** Merging with existing manually-configured hooks in settings.json is a nice-to-have feature. For MVP, `install` manages a clearly delimited section. Future versions may support non-destructive merge.

### 7.3 Plugin Compliance Testing

The `test` command runs the test harness against a plugin to verify protocol compliance:

```
$ sc-hooks test guard-paths

Plugin: guard-paths
  ✓ Manifest: valid JSON, all required fields present
  ✓ Contract version: 1 (compatible with host)
  ✓ Mode: sync
  ✓ Matchers: ["Write", "Bash", "Edit"] (valid event names)
  ✓ Timeout: 5000ms (within bounds)
  ✓ Requires: all fields satisfiable
  ✓ Protocol: returns valid JSON on stdin input
  ✓ Protocol: returns action field (proceed/block/error)
  ✓ Protocol: handles missing optional fields gracefully
  ✓ Protocol: handles empty payload
  ✓ Protocol: exits 0 on success
  ✓ Timeout: completes within declared timeout_ms
  ✗ Protocol: stderr output on normal execution (warning: debug output detected)

11 passed, 0 failed, 1 warning
Plugin is COMPLIANT.
```

Plugins that fail compliance testing are rejected by `audit` and will not be loaded at runtime.

## 8. Audit System

The audit command validates the entire hook system using **static analysis only** — it reads manifests and config but never executes hook logic or sends input to plugins. The async-no-block constraint (§5.4, §10.2) is enforced by checking the manifest's `mode` declaration, not by running the plugin and observing its output. Runtime violations (e.g., an async plugin that ignores its own manifest and returns `block`) are caught by the dispatch error handler (§5.5) at invocation time.

```
$ sc-hooks audit

Config: .sc-hooks/config.toml
Contract version: 1

Plugins:
  ✓ guard-paths    (contract=1, mode=sync, matchers=[Write,Bash,Edit], timeout=5000ms)
  ✓ log            (builtin, mode=sync, matchers=[*])
  ✓ collect-context (contract=1, mode=async, matchers=[Write,Bash], response_time=10-100ms)
  ✓ notify         (contract=1, mode=async, matchers=[*], response_time=1000-5000ms)
  ⚠ slack-approval (contract=1, mode=sync, long_running=true, timeout=300000ms)
    → Warning: long-running plugin. Justification: "Waits for Slack thread reply."

Hook chains:
  PreToolUse:
    matchers: Write, Bash, Edit, *
    sync:  [guard-paths (Write,Bash,Edit), log (*)]
    async: [collect-context (Write,Bash)]
    guard-paths requires:
      ✓ repo.path     (dir_exists) → /home/rand/src/p3 exists
      ✓ repo.branch   (non_empty)  → "feature/cal-v2"
      ✓ hook.event    (string)
    collect-context requires:
      ✓ repo.path     (dir_exists)

  PostToolUse:
    sync:  [log (*)]
    async: [notify (*)]
    notify requires:
      ✓ team.name     (non_empty)  → "calibration"

Sandbox:
  ✓ guard-paths: paths [.sc-hooks/guard-paths.toml] — accessible
  ✓ notify: needs_network=true — allowed by sandbox override

Install plan:
  PreToolUse  → 3 matcher entries (Write: 2, Bash: 2, Edit: 1)
  PostToolUse → 1 matcher entry (* : 2)

Async time buckets:
  PreToolUse/async: [collect-context] bucket 10-100ms
  PostToolUse/async: [notify] bucket 1000-5000ms

0 errors, 1 warning
```

## 9. Observability

### 9.1 Host Dispatch Log

The host logs every dispatch that executes at least one handler: which hook fired, which handlers ran, their results, and timing. Zero-match invocations (no plugins match the event) produce no log entry and exit immediately (see §3.2 execution flow, DSP-008). This is the host's responsibility and the only log the host writes.

Log path and level are configured in `[logging]`:

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
  "matcher": "Write",
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

Error entries include additional detail:

```json
{
  "ts": "2026-03-06T10:23:46.456Z",
  "hook": "PreToolUse",
  "event": "Write",
  "mode": "sync",
  "handlers": ["broken-plugin"],
  "results": [
    {
      "handler": "broken-plugin",
      "action": "error",
      "error_type": "invalid_json",
      "stderr": "Traceback (most recent call last):\n  ...",
      "ms": 12,
      "disabled": true
    }
  ],
  "total_ms": 12,
  "exit": 2,
  "ai_notification": "hook broken-plugin returned invalid JSON — disabled. Please notify user!"
}
```

## 10. Sync/Async Execution Model

Each plugin declares its execution mode in its manifest. The host uses this to split handler chains and generate correct AI tool integration.

### 10.1 Sync Plugins

Sync plugins make blocking decisions. They run sequentially in a chain that short-circuits on block/error. Claude Code waits for the result. Use sync mode for: path guards, permission checks, input validation, pre-flight logging, rate limiting, workspace state assertions—anything that must complete before the tool action proceeds.

### 10.2 Async Plugins

Async plugins perform work that doesn't gate the tool action: context collection, notifications, analysis, telemetry, compaction triggers, audit trail writes, CI polling, and any other background processing. Claude Code does not wait for them—it starts the async chain and continues immediately. When the async chain completes, any `additionalContext` or `systemMessage` is delivered to the AI on the next conversation turn.

Async plugins cannot return `block`. The audit command enforces this constraint.

### 10.3 Async Time Buckets & Aggregation

Async plugins declare a `response_time` range in their manifest: `{"min_ms": 10, "max_ms": 100}`. The host uses this to group async plugins into **time buckets** during installation.

**Why time buckets matter:** If two async plugins have wildly different response times (e.g., 10–100ms vs 5–10s), aggregating their output into a single Claude Code async hook entry means the fast plugin's context is delayed until the slow one finishes. Instead, `sc-hooks install` generates separate async hook entries for different time buckets, so fast context arrives early and slow context arrives when ready.

**Bucketing rules:**
- Plugins whose `response_time` ranges overlap or are adjacent are grouped into the same bucket.
- Plugins with non-overlapping ranges (e.g., max of one < min of another) become separate hook entries.
- Plugins that omit `response_time` go into a default bucket (0–30000ms).

**Aggregation within a bucket:**
- `additionalContext` strings from all plugins in the bucket are concatenated with a `\n---\n` separator.
- `systemMessage` strings are concatenated with `\n`.
- If only one plugin is in the bucket, its output is passed through unmodified.

**SDK traits for async plugins:**

```rust
/// Implement this for async plugins that provide context to the AI.
pub trait AsyncContextSource: AsyncHandler {
    /// Expected response time range. Used for time-bucket grouping.
    fn response_time(&self) -> ResponseTimeRange;
}

pub struct ResponseTimeRange {
    pub min_ms: u64,
    pub max_ms: u64,
}
```

### 10.4 Chain Splitting & Matcher Integration

When a single hook event (e.g., PreToolUse/Write) has both sync and async plugins that match, `sc-hooks install` generates two Claude Code hook entries for that matcher: one sync (blocking), one async (background). If only sync plugins match, only the sync entry is generated. If only async match, only async. Combined with per-event matchers, this means Claude Code only invokes sc-hooks when there's actual work to do.

### 10.5 Handler-Internal Async

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

## 12. SDK & Pre-made Plugins

### 12.1 SDK Traits

The sc-hooks-sdk crate provides traits that plugin authors implement:

| Trait | Purpose |
|-------|---------|
| `SyncHandler` | Base trait for sync plugins. Requires `handle(&self, input: HookInput) -> HookResult`. |
| `AsyncHandler` | Base trait for async plugins. Returns `AsyncResult` with optional `additionalContext` and `systemMessage`. |
| `LongRunning` | Extension trait for sync plugins that may block on external input. Requires justification. |
| `AsyncContextSource` | Extension trait for async plugins. Declares response time range for bucketing. |
| `ManifestBuilder` | Fluent API for generating correct manifest JSON. |
| `PluginRunner` | Handles `--manifest` flag detection, stdin/stdout protocol, and error wrapping. |

The SDK is optional. Plugins can implement the JSON protocol directly in any language. The SDK simply makes it easier for Rust plugin authors and ensures protocol compliance.

### 12.2 Pre-made Plugins

sc-hooks ships with common-case handlers that cover frequent patterns:

| Plugin | Mode | Description |
|--------|------|-------------|
| `log` | sync (builtin) | Logs hook invocations. Always available. |
| `guard-paths` | sync | Blocks writes to denied paths. Reads deny/allow patterns from its own config. |
| `conditional-source` | async | Reads a file and returns its contents as `additionalContext`, optionally filtered by conditions on the incoming metadata JSON. |
| `template-source` | async | Reads a Jinja2 template file and renders it using the incoming metadata JSON as context. Returns the rendered output as `additionalContext`. Enables dynamic context injection without writing a custom plugin. |
| `notify` | async | Sends notifications (ATM messages, webhooks). Forks and returns immediately. |
| `save-context` | sync | Persists context before compaction events. |

These serve as both useful tools and reference implementations for plugin authors.

## 13. Project Structure

```
sc-hooks/
├── Cargo.toml                # workspace
├── sc-hooks-sdk/             # trait-based plugin framework
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── traits.rs         # SyncHandler, AsyncHandler, LongRunning, AsyncContextSource
│       ├── manifest.rs       # ManifestBuilder
│       ├── validate.rs       # Validation rule types
│       ├── runner.rs         # --manifest + stdin/stdout protocol + error wrapping
│       └── result.rs         # HookResult, AsyncResult enums
├── sc-hooks-cli/             # the host binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # clap: run, audit, fire, install, test, exit-codes
│       ├── config.rs         # TOML parsing (4 sections)
│       ├── metadata.rs       # assemble JSON from config + runtime
│       ├── resolve.rs        # name → builtin | plugin binary
│       ├── manifest.rs       # call --manifest, parse + cache
│       ├── validate.rs       # validate metadata against requirements
│       ├── dispatch.rs       # pipe JSON → plugin → read result → timeout enforcement
│       ├── install.rs        # generate .claude/settings.json with matcher routing
│       ├── builtins/
│       │   └── log.rs
│       ├── audit.rs
│       ├── fire.rs
│       ├── testing.rs        # plugin compliance test runner
│       ├── exit_codes.rs     # exit code definitions and help text
│       └── logging.rs
├── sc-hooks-test/            # test harness crate (used by plugin authors)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── compliance.rs     # manifest validation, protocol conformance checks
│       └── fixtures.rs       # test fixture builders
├── plugins/                  # pre-made plugins (each compiles independently)
│   ├── guard-paths/
│   ├── conditional-source/
│   ├── template-source/
│   ├── notify/
│   └── save-context/
└── shims/
    ├── codex-shim.sh
    └── gemini-shim.sh
```

## 14. Security Model

### 14.1 Sandbox Compliance

The host respects the sandboxing rules of the calling AI tool. Claude Code enforces a sandbox that restricts file access, network access, and process spawning. Plugins inherit these restrictions.

**Plugin resource declarations** allow the host to validate access before invocation:

```json
"sandbox": {
  "paths": [".sc-hooks/guard-paths.toml", ".sc-hooks/deny-patterns/"],
  "needs_network": false
}
```

The `audit` command verifies that declared paths exist and are accessible. At runtime, the host checks declared paths before invocation.

### 14.2 Sandbox Overrides

Some plugins legitimately need access beyond the default sandbox (e.g., a notify plugin needs network access to send a webhook). The override mechanism:

1. Plugin declares its needs in the `sandbox` manifest field.
2. `sc-hooks audit` reports all sandbox requirements and flags any that exceed defaults.
3. The config can explicitly allow overrides:

```toml
[sandbox]
allow_network = ["notify"]
allow_paths = { "guard-paths" = [".sc-hooks/guard-paths.toml"] }
```

4. Unacknowledged sandbox requirements cause `audit` to warn (not fail by default). A `--strict` flag makes them errors.

### 14.3 Plugin Integrity

- `audit` warns if plugin executables are world-writable or not owned by the current user.
- `audit` warns if the `.sc-hooks/plugins/` directory has overly permissive permissions.
- Future: optional plugin signature verification (post-MVP, tracked with Synaptic Canvas plugin marketplace).

## 15. Open Design Decisions

- **Config discovery:** Walk up from CWD to find `.sc-hooks/config.toml` (mirrors `.claude/` behavior), or explicit `--config` flag, or both?
- **Manifest caching:** Cache manifests in memory per invocation (baseline), or persist to disk with invalidation on plugin mtime change (optimization)?
- **Config inheritance:** Should a global `~/.config/sc-hooks/config.toml` provide defaults that repo-level configs extend? Useful for logging defaults and global plugins.
- **Diagnostic mode design:** The `fire` command needs design for payload generation. Synthetic payloads, recorded payloads, or both?
- **Plugin marketplace:** Integration with Synaptic Canvas package registry for plugin distribution. Separate concern but tracked.
- **Jinja2 runtime:** The `template-source` plugin needs a Jinja2-compatible template engine. Use a Rust crate (minijinja) or shell out to Python?
