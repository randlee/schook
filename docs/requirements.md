# sc-hooks — Requirements Document

> Version 0.1.0 — March 2026 — DRAFT

## 1. Purpose

This document specifies the functional and non-functional requirements for sc-hooks, a Rust-based hook dispatcher for AI-assisted development workflows. sc-hooks replaces an existing Python-based hook system that is fragile, untestable, and opaque.

The primary consumers are Claude Code hook configurations, with secondary support for Codex, Gemini, and other AI tools via shims.

## 2. Scope

### 2.1 In Scope

- CLI binary for hook dispatch, audit, and diagnostics
- Config-driven routing of hooks to handler chains
- Plugin protocol: manifest declaration, JSON stdin/stdout, validation
- Builtin handlers (log at minimum)
- Structured dispatch logging
- Sync/async chain splitting with automatic Claude Code settings generation
- AI-agnostic shim pattern

### 2.2 Out of Scope

- Plugin marketplace / distribution (tracked separately with Synaptic Canvas)
- GUI or TUI for hook management
- Claude Code hook registration (handled by `sc-hooks install`)
- Diagnostic mode detailed design (deferred, noted as future requirement)

## 3. Functional Requirements

### 3.1 Configuration

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| CFG-001 | The system shall read configuration from a TOML file at `.sc-hooks/config.toml` relative to the repository root. | Must | Given a repo with `.sc-hooks/config.toml`, running `sc-hooks config` prints the resolved configuration. |
| CFG-002 | The config file shall contain three sections only: `[meta]`, `[context]`, and `[hooks]`. | Must | A config with any other top-level section produces a parse error with a clear message. |
| CFG-003 | The `[hooks]` section shall map hook type names to ordered arrays of handler names. | Must | Given `PreToolUse = ["a", "b"]`, handlers a and b are invoked in order. |
| CFG-004 | The `[context]` section shall provide static key-value pairs merged into the metadata JSON under the team subsection. | Must | Given `team = "cal"`, the metadata JSON contains `{"team": {"name": "cal"}}`. |
| CFG-005 | The `[meta]` section shall contain a version field for config format versioning. | Must | Missing version field produces a parse error. |

### 3.2 Handler Resolution

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| RES-001 | The system shall resolve handler names by checking builtins first, then plugin executables in the plugins directory. | Must | A name matching a builtin always resolves to the builtin, even if a same-named executable exists. |
| RES-002 | The default plugins directory shall be `.sc-hooks/plugins/` relative to the config file. | Must | An executable at `.sc-hooks/plugins/notify` resolves for handler name "notify". |
| RES-003 | An unresolvable handler name shall produce an error during both audit and runtime. | Must | Running `sc-hooks audit` with an unresolvable handler reports the error. Running `sc-hooks run` with the same config exits non-zero. |

### 3.3 Plugin Protocol

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| PLG-001 | A plugin shall respond to the `--manifest` flag by writing a JSON manifest to stdout and exiting 0. | Must | Running `./plugin --manifest` outputs valid JSON with name, protocol, mode, hooks, and requires fields. |
| PLG-002 | The manifest shall declare required and optional metadata fields with dot-path notation (e.g., `repo.path`, `team.name`). | Must | The host can extract the field from the metadata JSON using the dot path. |
| PLG-003 | The manifest shall support validation rules on required fields: `non_empty`, `dir_exists`, `file_exists`, `path_resolves`, `one_of:<values>`, `positive_int`. | Must | For each rule type, validation fails on invalid input and passes on valid input. |
| PLG-004 | The host shall validate all required fields against manifest rules before invoking the plugin. | Must | A plugin with `requires repo.path: dir_exists` is never called if the path does not exist. The host returns an error result. |
| PLG-005 | The host shall pipe a JSON object to the plugin's stdin containing only the fields declared in requires and optional. | Must | A plugin declaring `requires repo.path` and `optional team.name` receives JSON with only those fields (plus payload if present). |
| PLG-006 | Sync-mode plugins shall write a JSON result to stdout with an `action` field of `proceed`, `block`, or `error`. | Must | The host correctly interprets each action type. Block short-circuits the chain. |
| PLG-007 | A plugin may be implemented in any language. The protocol is the contract, not the implementation language. | Must | A Python script and a Rust binary with identical manifests are interchangeable. |
| PLG-008 | The manifest shall declare a `mode` field of `sync` or `async`. | Must | A plugin declaring `mode=sync` is placed in the sync chain. A plugin declaring `mode=async` is placed in the async chain. |
| PLG-009 | Async-mode plugins shall not return `action=block`. They may return `additionalContext` and `systemMessage` fields for delivery to the AI tool on the next turn. | Must | An async plugin returning block is treated as an error by the host. Audit flags this as a violation. |

### 3.4 Dispatch

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| DSP-001 | The system shall execute sync handlers in the order specified in the config array. | Must | Given `["a", "b", "c"]` where all are sync, handler a runs first, then b, then c. |
| DSP-002 | If a sync handler returns `action=block`, the sync chain shall short-circuit and return the block reason to the caller. | Must | Given `["a", "b"]` where a returns block, handler b is never invoked. |
| DSP-003 | If a handler returns `action=error`, the chain shall short-circuit and log the error. | Must | Error results are logged with the error message and the exit code is non-zero. |
| DSP-004 | If all sync handlers return `action=proceed`, the system shall exit 0. | Must | Claude Code receives exit 0 and proceeds normally. |
| DSP-005 | The system shall pass hook payload from stdin through to plugins in the `payload` field of the input JSON. | Must | Claude Code's hook payload JSON appears in the plugin's input under the payload key. |
| DSP-006 | When invoked with `--sync`, the system shall run only sync-mode handlers. When invoked with `--async`, only async-mode handlers. | Must | A single hook event with both sync and async plugins results in two separate sc-hooks invocations. |
| DSP-007 | The async chain shall aggregate `additionalContext` from all async handlers and return it as combined JSON output. | Must | Claude Code delivers the aggregated context on the next conversation turn. |

### 3.5 Metadata Assembly

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| MTA-001 | The system shall auto-discover: agent PID, repo path, current branch, working directory. | Must | In a git repository, `repo.path` and `repo.branch` are populated without config. |
| MTA-002 | The system shall read agent type and session ID from environment variables set by the calling AI tool or shim. | Must | Given `SC_HOOK_AGENT_TYPE=codex`, `metadata.agent.type` is "codex". |
| MTA-003 | The system shall merge `[context]` values from config into the metadata JSON. | Must | Given `[context] team = "cal"`, `metadata.team.name` is "cal". |
| MTA-004 | The system shall write the assembled metadata to a temp file and set `SC_HOOK_METADATA` env var for external executables. | Must | A bash plugin handler can read `$SC_HOOK_METADATA` to access full context. |
| MTA-005 | The system shall set minimal env vars: `SC_HOOK_TYPE`, `SC_HOOK_EVENT`, `SC_HOOK_METADATA`. | Must | Exactly these three env vars are set. No duplication of JSON fields. |

### 3.6 CLI Commands

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| CLI-001 | `sc-hooks run <hook> [event]` shall execute the handler chain for the given hook type. `--sync` runs only sync-mode handlers, `--async` runs only async-mode handlers. Default is `--sync`. | Must | `sc-hooks run PreToolUse --sync` invokes only sync plugins for PreToolUse. |
| CLI-002 | `sc-hooks audit` shall validate all handlers, manifests, data flow, and sync/async correctness without executing any hook logic. | Must | Audit reports per-handler status, sync/async chain splits, and identifies violations. |
| CLI-003 | `sc-hooks fire <hook> [event]` shall trigger a hook in diagnostic mode for testing. | Should | `fire` invokes the handler chain and reports detailed results including timing. |
| CLI-004 | `sc-hooks config` shall display the resolved configuration. | Must | Output shows the parsed TOML with resolved paths. |
| CLI-005 | `sc-hooks handlers` shall list all available builtins and discovered plugin executables with their mode. | Must | Output distinguishes builtins from plugins and shows mode (sync/async) for each. |
| CLI-006 | `sc-hooks install` shall generate `.claude/settings.json` hook entries from the current config and plugin manifests. | Must | For a hook event with both sync and async plugins, install generates two handler entries (one sync, one with `async: true`). For events with only one mode, a single entry is generated. |

## 4. Non-Functional Requirements

### 4.1 Auditability

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| AUD-001 | The audit command shall verify that every handler in every hook chain is resolvable. | Must | A missing executable is reported as an error. |
| AUD-002 | The audit command shall call `--manifest` on every plugin and validate the response. | Must | A plugin with invalid manifest JSON is reported as an error. |
| AUD-003 | The audit command shall validate that all required metadata fields declared by plugins can be satisfied by the current config + runtime environment. | Must | A plugin requiring `atm.inbox` when no such field exists is reported. |
| AUD-004 | The audit command shall validate `dir_exists` and `file_exists` rules against the current filesystem. | Must | A required path that does not exist is reported as an error. |
| AUD-005 | The audit command shall exit 0 on success and non-zero on any error, suitable for CI integration. | Must | `sc-hooks audit` can be used as a CI gate. |
| AUD-006 | The audit command shall verify that async-mode plugins do not declare blocking behavior and that sync-mode plugins support the hooks they are assigned to. | Must | An async plugin that would need to block is flagged. |
| AUD-007 | The audit command shall display the install plan showing how many Claude Code hook entries each event will generate. | Should | Output shows: `PreToolUse → 2 entries (sync + async)`. |

### 4.2 Observability

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| OBS-001 | Every hook invocation shall produce a structured JSONL log entry with: timestamp, hook type, event, handler chain, per-handler results with timing, total duration, exit code. | Must | The log file contains one JSON object per line per invocation. |
| OBS-002 | Logging configuration shall be in the config.toml under a `[logging]` section with a `hook_log` path and level. | Must | Changing `hook_log` path in config changes where dispatch logs are written. |
| OBS-003 | Log level shall be configurable: debug, info, warn, error. | Should | Setting `level = "debug"` produces more verbose dispatch log entries. |
| OBS-004 | Plugin-level logging is the plugin's responsibility. The host shall not provide logging infrastructure to plugins. | Must | No host config controls plugin log output. |

### 4.3 Performance

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| PRF-001 | The host binary shall start and parse config in under 5ms. | Must | Measured from process start to config loaded (benchmarked). |
| PRF-002 | A builtin-only handler chain shall complete in under 10ms total. | Must | Measured end-to-end for a typical PreToolUse with log builtin. |
| PRF-003 | Plugin manifest loading shall be cached per invocation to avoid redundant `--manifest` calls. | Should | A handler appearing in multiple hook chains calls `--manifest` once. |
| PRF-004 | Handlers performing long-running work shall fork a detached child and return immediately. | Must | A notify handler that sends an ATM message returns in under 10ms; the actual send happens in a forked process. |

### 4.4 Testability

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| TST-001 | Config parsing shall be unit-testable with in-memory TOML strings. | Must | Tests create configs from strings, not files. |
| TST-002 | Handler resolution shall be unit-testable with temporary directories. | Must | Tests create temp dirs with executables, assert resolution. |
| TST-003 | Dispatch shall be testable with mock handlers that return predetermined results. | Must | Tests verify chain ordering, short-circuit on block, error handling. |
| TST-004 | Metadata assembly shall be a pure function testable without filesystem or git. | Must | Tests provide inputs, assert JSON output. |
| TST-005 | The plugin protocol shall be integration-testable: create a minimal plugin, invoke it, assert results. | Must | An integration test creates a temp plugin script, runs the full dispatch loop, verifies log output. |
| TST-006 | The audit command shall be integration-testable with fixture directories. | Must | Tests set up `.sc-hooks/` structures and assert audit output. |

### 4.5 Portability

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| PRT-001 | The host binary shall compile and run on Linux and macOS. | Must | CI builds and tests on both platforms. |
| PRT-002 | Plugin resolution shall handle platform-specific executable conventions. | Must | A plugin at `.sc-hooks/plugins/notify` (no `.exe`) resolves on both platforms. |

## 5. Constraints

- **Config simplicity:** The config file shall not contain per-handler configuration. Handler-specific settings are the handler's responsibility.
- **No versioning system:** Plugins are not versioned by the host. If a plugin has a bug, replace the executable. The manifest protocol field handles breaking changes to the JSON contract.
- **Minimal env vars:** The host sets at most three environment variables. All structured data flows via JSON.
- **No unsafe code in host:** The host binary shall not use dlopen, C FFI, or unsafe blocks for plugin loading. Plugins are processes.

## 6. Key Acceptance Scenarios

### 6.1 Happy Path: PreToolUse Guard (Sync)

Given a config with `PreToolUse = ["guard-paths", "log"]`, both sync-mode, and guard-paths allows the write, the system shall: resolve both handlers, validate metadata, pipe JSON to guard-paths, receive proceed, run log builtin, log a JSONL entry, exit 0.

### 6.2 Block Path: Denied Write

Given guard-paths returns block with reason "path in deny list", the system shall: not invoke the log handler, log a JSONL entry showing the block, exit with the block code, and stderr contains the reason.

### 6.3 Mixed Sync/Async Chain

Given `PreToolUse = ["guard-paths", "collect-context"]` where guard-paths is sync and collect-context is async, `sc-hooks install` generates two Claude Code hook entries. When PreToolUse fires: the sync invocation runs guard-paths and returns proceed/block; the async invocation runs collect-context in the background and its `additionalContext` is delivered on the next Claude turn.

### 6.4 Install Generates Correct Settings

Given a config with three hook events where one has only sync plugins, one has only async, and one has both: `sc-hooks install` generates 1, 1, and 2 Claude Code hook entries respectively. The generated `.claude/settings.json` is valid JSON and the async entries include `"async": true`.

### 6.5 Audit Catches Async Plugin Trying to Block

Given an async-mode plugin whose manifest says `mode=async` but whose implementation returns `action=block`, `sc-hooks audit` flags this as a violation: async plugins cannot return block decisions.

### 6.6 Audit Catches Missing Handler

Given a config referencing handler "notify" with no corresponding builtin or executable, `sc-hooks audit` shall report the missing handler, list which hook chain is affected, and exit non-zero.

### 6.7 Audit Catches Unsatisfied Requirement

Given a plugin manifest requiring `atm.inbox` with `validate=non_empty`, and no such field in config context or runtime, `sc-hooks audit` shall report the unsatisfied requirement with the field path and validation rule.

### 6.8 Python Plugin Swap

Given a Rust plugin guard-paths with a bug, the user replaces it with a Python script `guard-paths` (with shebang) that implements the same manifest. `sc-hooks audit` passes. `sc-hooks run PreToolUse --sync` uses the Python plugin and produces correct results.

### 6.9 AI-Agnostic Shim

Given a Codex shim that sets `SC_HOOK_AGENT_TYPE=codex` and calls `sc-hooks run`, the handler chain executes identically to a Claude Code invocation. The log entry shows `agent.type` as "codex".
