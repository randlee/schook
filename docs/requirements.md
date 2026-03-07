# sc-hooks — Requirements Document

> Version 0.2.1 — March 2026 — DRAFT

## 1. Purpose

This document specifies the functional and non-functional requirements for sc-hooks, a Rust-based hook dispatcher for AI-assisted development workflows. sc-hooks replaces an existing Python-based hook system that is fragile, untestable, and opaque.

The primary consumers are Claude Code hook configurations, with secondary support for Codex, Gemini, and other AI tools via shims.

## 2. Scope

### 2.1 In Scope

- CLI binary for hook dispatch, audit, and diagnostics
- Config-driven routing of hooks to handler chains
- Plugin protocol: manifest declaration, JSON stdin/stdout, validation
- Event matchers: plugins declare what events they handle; installation generates precise Claude Code matcher entries
- Timeout enforcement with long-running plugin support
- Plugin compliance testing (test harness)
- Builtin handlers (log at minimum)
- Structured dispatch logging with error reporting to AI session
- Sync/async chain splitting with time-bucketed aggregation
- Automatic Claude Code settings generation via `sc-hooks install`
- AI-agnostic shim pattern
- SDK with traits for sync, async, long-running, and context-source plugins
- Pre-made plugins for common patterns (guard-paths, conditional-source, template-source, notify, save-context)
- Sandbox compliance and override mechanism
- Defined exit codes with CLI help

### 2.2 Out of Scope

- Plugin marketplace / distribution (tracked separately with Synaptic Canvas)
- GUI or TUI for hook management
- Claude Code hook registration (handled by `sc-hooks install`)
- Diagnostic mode detailed design (deferred, noted as future requirement)
- Non-destructive merge with existing settings.json hooks (post-MVP)
- Plugin signature verification (post-MVP)

## 3. Functional Requirements

### 3.1 Configuration

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| CFG-001 | The system shall read configuration from a TOML file at `.sc-hooks/config.toml` relative to the repository root. | Must | Given a repo with `.sc-hooks/config.toml`, running `sc-hooks config` prints the resolved configuration. |
| CFG-002 | The config file shall recognize exactly five sections: `[meta]`, `[context]`, `[hooks]`, `[logging]`, and `[sandbox]`. Only `[meta]` and `[hooks]` are required; the rest are optional. | Must | A config with any other top-level section produces a parse error with a clear message. A config with only `[meta]` and `[hooks]` is valid. |
| CFG-003 | The `[hooks]` section shall map hook type names to ordered arrays of handler names. | Must | Given `PreToolUse = ["a", "b"]`, handlers a and b are invoked in order. |
| CFG-004 | The `[context]` section shall provide static key-value pairs merged into the metadata JSON under the team subsection. | Must | Given `team = "cal"`, the metadata JSON contains `{"team": {"name": "cal"}}`. |
| CFG-005 | The `[meta]` section shall contain a `version` field (integer) for config format versioning. | Must | Missing version field produces a parse error. Non-integer value produces a parse error. |
| CFG-006 | The `[logging]` section shall configure the dispatch log path (`hook_log`) and level (`level`). | Must | Changing `hook_log` path in config changes where dispatch logs are written. Valid levels: debug, info, warn, error. |
| CFG-007 | The `[logging]` section shall be optional with sensible defaults (`hook_log = ".sc-hooks/logs/hooks.jsonl"`, `level = "info"`). | Should | A config without `[logging]` uses defaults. |
| CFG-008 | The `[sandbox]` section shall be optional. When present, it provides explicit sandbox overrides per plugin (see SEC-003). | Should | A config without `[sandbox]` means no overrides; all plugins run under default sandbox restrictions. |
| CFG-009 | The `[context]` section key `team` shall map to `team.name` in the metadata JSON. All other keys map to top-level metadata fields. Dot-notation keys are treated as literal strings, not expanded. | Must | Given `[context] team = "cal"`, `metadata.team.name` is `"cal"`. Given `[context] project = "p3"`, `metadata.project` is `"p3"`. |

### 3.2 Handler Resolution

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| RES-001 | The system shall resolve handler names by checking builtins first, then plugin executables in the plugins directory. | Must | A name matching a builtin always resolves to the builtin, even if a same-named executable exists. |
| RES-002 | The default plugins directory shall be `.sc-hooks/plugins/` relative to the config file. | Must | An executable at `.sc-hooks/plugins/notify` resolves for handler name "notify". |
| RES-003 | An unresolvable handler name shall produce an error during both audit and runtime (exit code 4). | Must | Running `sc-hooks audit` with an unresolvable handler reports the error. Running `sc-hooks run` with the same config exits with code 4. |

### 3.3 Plugin Protocol

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| PLG-001 | A plugin shall respond to the `--manifest` flag by writing a JSON manifest to stdout and exiting 0. | Must | Running `./plugin --manifest` outputs valid JSON with name, contract_version, mode, hooks, matchers, and requires fields. |
| PLG-002 | The manifest shall declare required and optional metadata fields with dot-path notation (e.g., `repo.path`, `team.name`). | Must | The host can extract the field from the metadata JSON using the dot path. |
| PLG-003 | The manifest shall support validation rules on required fields: `non_empty`, `dir_exists`, `file_exists`, `path_resolves`, `one_of:<values>`, `positive_int`. | Must | For each rule type, validation fails on invalid input and passes on valid input. |
| PLG-004 | The host shall validate all required fields against manifest rules before invoking the plugin. | Must | A plugin with `requires repo.path: dir_exists` is never called if the path does not exist. The host returns an error result. |
| PLG-005 | The host shall pipe a JSON object to the plugin's stdin containing: (a) metadata fields filtered to only those declared in `requires` and `optional`, (b) the `hook` field (always included), and (c) the `payload` field as a verbatim passthrough if present. `payload` and `hook` are not subject to requires/optional filtering. | Must | A plugin declaring `requires repo.path` and `optional team.name` receives JSON with those fields plus `hook` and `payload` (if present). |
| PLG-006 | Sync-mode plugins shall write a JSON result to stdout with an `action` field of `proceed`, `block`, or `error`. | Must | The host correctly interprets each action type. Block short-circuits the chain. |
| PLG-007 | A plugin may be implemented in any language. The protocol is the contract, not the implementation language. | Must | A Python script and a Rust binary with identical manifests are interchangeable. |
| PLG-008 | The manifest shall declare a `mode` field of `sync` or `async`. | Must | A plugin declaring `mode=sync` is placed in the sync chain. A plugin declaring `mode=async` is placed in the async chain. |
| PLG-009 | Async-mode plugins shall not return `action=block`. They may return `additionalContext` and `systemMessage` fields for delivery to the AI tool on the next turn. | Must | An async plugin returning block is treated as a protocol error. Plugin is disabled. Audit flags this as a violation. |
| PLG-010 | The manifest shall declare a `matchers` array specifying which events the plugin handles. `["*"]` matches all events. | Must | A plugin with `matchers: ["Write", "Bash"]` is only invoked for those events. `sc-hooks install` generates matching Claude Code `matcher` entries. |
| PLG-011 | The manifest shall declare a `contract_version` field (integer) indicating which version of the JSON contract the plugin speaks. | Must | A host at contract version 2 can invoke contract version 1 plugins. A v1 host encountering a v2-only plugin reports an incompatibility error at audit time. |
| PLG-012 | The `payload` field in plugin input shall be passed verbatim from the AI tool. If no payload is provided, the field shall be omitted (not null, not empty object). | Must | Plugins handle absent payload gracefully. The test harness verifies this. |

### 3.4 Event Matching & Installation

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| MTR-001 | `sc-hooks install` shall generate Claude Code `matcher` entries based on plugin-declared matchers, not blanket wildcards. | Must | A plugin matching `["Write", "Bash"]` produces entries with those specific matchers. Claude Code does not invoke sc-hooks for non-matching events. |
| MTR-002 | When multiple plugins with different matchers exist for the same hook type, `install` shall generate the minimal set of matcher entries that covers all plugins. | Must | Plugins matching `["Write"]` and `["Write", "Bash"]` produce entries for `Write` (both plugins) and `Bash` (second plugin only). |
| MTR-003 | A plugin with `matchers: ["*"]` shall be included in all matcher entries for its hook type. | Must | A `log` plugin matching `*` appears in every generated entry for its hook type. |
| MTR-004 | `sc-hooks install` shall not generate a hook entry for a matcher+mode combination with zero applicable plugins. | Must | If no async plugins match event `Read`, no async entry is generated for `Read`. |

### 3.5 Timeout & Long-Running Plugins

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| TMO-001 | The host shall enforce a default timeout of 5000ms for sync plugins and 30000ms for async plugins. | Must | A sync plugin that runs for 6s is killed. |
| TMO-002 | Plugins may declare a custom `timeout_ms` in their manifest to override the default. | Must | A plugin with `timeout_ms: 10000` is allowed 10s before kill. |
| TMO-003 | When a plugin exceeds its timeout, the host shall send SIGTERM, wait 1s, then SIGKILL. The plugin is disabled for the session. Exit code 6. | Must | A hanging plugin is killed, disabled, and the error is logged and reported to the AI session. |
| TMO-004 | Plugins declaring `long_running: true` shall have extended or no timeout, as declared by their `timeout_ms`. | Must | A long-running plugin with `timeout_ms: 300000` is allowed 5 minutes. A long-running plugin with no `timeout_ms` has no timeout. |
| TMO-005 | Audit shall warn on long-running plugins. Manifest must include `description` justifying why long-running is needed. | Must | A long-running plugin without `description` fails audit. With description, audit logs a warning. |
| TMO-006 | For async invocations, timeout shall not produce a blocking exit code. The async chain exits 0. The timed-out plugin is disabled and an `ai_notification` is included in the async result. | Must | Async timeout degrades gracefully — tool use is not blocked, but the AI is informed that context from the failed plugin is unavailable. |

### 3.6a Session State & Plugin Disable Persistence

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| SES-001 | The host shall persist plugin disable state in `.sc-hooks/state/session.json`, keyed by the AI tool's session ID. | Must | A plugin disabled in one hook invocation remains disabled in subsequent invocations within the same session. |
| SES-002 | The session state file shall be cleaned up when a `SessionEnd` hook fires or when `sc-hooks audit --reset` is run. | Must | After session end, all plugins are re-enabled for the next session. |
| SES-003 | If the session state file is missing or unreadable, all plugins shall be considered enabled (fail-open for state). | Must | Deleting the state file re-enables all plugins. |

### 3.6b Contract Version Compatibility

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| CTV-001 | The host shall adapt downward to a plugin's declared `contract_version`. A vN host can invoke any plugin with contract_version ≤ N. | Must | A v2 host invokes a v1 plugin by sending v1-format input and accepting v1-format output. |
| CTV-002 | The host shall reject plugins with `contract_version` greater than its own at audit time and runtime. | Must | Audit reports incompatibility error. Runtime skips the plugin with exit code 5 if no other handlers remain. |
| CTV-003 | For MVP, only contract version 1 exists. The compatibility machinery is designed but not exercised. | Must | All plugins and the host declare contract_version 1. |

### 3.6c Event Taxonomy

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| EVT-001 | The host shall maintain a canonical taxonomy of valid events per hook type. | Must | `sc-hooks handlers --events` lists all valid events per hook type. |
| EVT-002 | Valid PreToolUse/PostToolUse events shall include: `Bash`, `Read`, `Write`, `Edit`, `Glob`, `Grep`, `WebFetch`, `WebSearch`, `Agent`, `NotebookEdit`, `TodoWrite`, `AskFollowup`, `SendMessage`, `Task`, and `*`. | Must | Audit validates plugin matchers against this list. |
| EVT-003 | Lifecycle and agent-team hooks (`PreCompact`, `PostCompact`, `SessionStart`, `SessionEnd`, `TeammateIdle`, `PermissionRequest`, `Stop`) use `*` matchers only. `Notification` supports `idle_prompt` and `*`. | Must | Audit flags non-`*` matchers on lifecycle hooks as errors (except `Notification` which allows named matchers). |
| EVT-005 | The host shall recognize all Claude Code hook types including agent-teams events: `TeammateIdle`, `PermissionRequest`, `Stop`. | Must | Plugins can register for these hook types in config and manifests. |
| EVT-004 | Audit shall warn (not fail) on unrecognized event names, to allow forward compatibility as Claude Code adds new events. | Should | An event name not in the taxonomy produces a warning, not an error. |

### 3.6 Error Handling & Plugin Validation

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| ERR-001 | If a plugin writes invalid JSON to stdout, the host shall disable the plugin for the session, log the error, and report to the AI session. | Must | Log entry includes raw stdout and error type. AI sees: `"hook <name> returned invalid JSON — disabled. Please notify user!"` |
| ERR-002 | If a plugin exits non-zero without writing valid output, the host shall disable the plugin, log the exit code and stderr, and report to the AI session. | Must | Exit code and stderr are captured in the dispatch log. |
| ERR-003 | If a plugin writes to stderr during normal execution, the host shall capture stderr and include it in the dispatch log entry. | Must | Stderr is logged but not forwarded to AI unless the plugin also failed. |
| ERR-004 | If a plugin writes multiple JSON objects to stdout, the host shall use the first valid object and log a warning. | Should | Warning includes plugin name and note about protocol compliance. |
| ERR-005 | An async plugin that returns `action=block` shall be treated as a protocol error: plugin is disabled and the error is reported. | Must | Same behavior as ERR-001. |
| ERR-006 | Disabled plugins shall not be re-invoked for the remainder of the host process session. The dispatch log records the disable event. | Must | After disable, subsequent hook fires skip the plugin. |
| ERR-007 | All error reports to the AI session shall include the plugin name, error type, and remediation guidance. | Must | Messages follow the pattern: `"hook <name> <error-type> — disabled. <guidance>"`. |

### 3.7 Dispatch

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| DSP-001 | The system shall execute sync handlers in the order specified in the config array. | Must | Given `["a", "b", "c"]` where all are sync, handler a runs first, then b, then c. |
| DSP-002 | If a sync handler returns `action=block`, the sync chain shall short-circuit and return the block reason to the caller. Exit code 1. | Must | Given `["a", "b"]` where a returns block, handler b is never invoked. Exit code is 1. |
| DSP-003 | If a handler returns `action=error`, the chain shall short-circuit, log the error, and exit with code 2. | Must | Error results are logged with the error message. |
| DSP-004 | If all sync handlers return `action=proceed`, the system shall exit 0. | Must | Claude Code receives exit 0 and proceeds normally. |
| DSP-005 | The system shall pass hook payload from stdin through to plugins in the `payload` field of the input JSON. If the AI tool sends no payload, the `payload` field is omitted. | Must | Claude Code's hook payload JSON appears in the plugin's input under the payload key. |
| DSP-006 | When invoked with `--sync`, the system shall run only sync-mode handlers. When invoked with `--async`, only async-mode handlers. | Must | A single hook event with both sync and async plugins results in two separate sc-hooks invocations. |
| DSP-007 | The async chain shall group plugins by declared `response_time` ranges into time buckets. Within a bucket, `additionalContext` is concatenated with `\n---\n` separator. `systemMessage` is concatenated with `\n`. | Must | Two plugins with similar response times have their context aggregated. Two plugins with very different response times produce separate async hook entries via `install`. |
| DSP-008 | When the event does not match any plugin's declared matchers, the host shall exit 0 immediately without invoking any handlers. | Must | No dispatch log entry, no handler invocation, no overhead. |

### 3.8 Metadata Assembly

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| MTA-001 | The system shall auto-discover: agent PID, repo path, current branch, working directory. | Must | In a git repository, `repo.path` and `repo.branch` are populated without config. |
| MTA-002 | The system shall read agent type and session ID from environment variables set by the calling AI tool or shim. | Must | Given `SC_HOOK_AGENT_TYPE=codex`, `metadata.agent.type` is "codex". |
| MTA-003 | The system shall merge `[context]` values from config into the metadata JSON. | Must | Given `[context] team = "cal"`, `metadata.team.name` is "cal". |
| MTA-004 | The system shall write the assembled metadata to a temp file and set `SC_HOOK_METADATA` env var for external executables. Temp files are cleaned up after the chain completes (both sync and async). | Must | A bash plugin handler can read `$SC_HOOK_METADATA` to access full context. Temp files do not accumulate. |
| MTA-005 | The system shall set minimal env vars: `SC_HOOK_TYPE`, `SC_HOOK_EVENT`, `SC_HOOK_METADATA`. | Must | Exactly these three env vars are set. No duplication of JSON fields. |

### 3.9 CLI Commands

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| CLI-001 | `sc-hooks run <hook> [event]` shall execute the handler chain for the given hook type. `--sync` runs only sync-mode handlers, `--async` runs only async-mode handlers. Default is `--sync`. | Must | `sc-hooks run PreToolUse Write --sync` invokes only sync plugins matching `Write` for PreToolUse. |
| CLI-002 | `sc-hooks audit` shall validate all handlers, manifests, matchers, data flow, timeout declarations, sandbox requirements, and sync/async correctness using **static analysis only** (manifest inspection, config validation, filesystem checks). It shall not execute any hook logic or send input to plugins. | Must | Audit reports per-handler status, matcher coverage, sync/async chain splits, time buckets, sandbox status, and identifies violations. Runtime-only violations (e.g., async plugin returning block) are caught by the dispatch error handler at invocation time. |
| CLI-003 | `sc-hooks fire <hook> [event]` shall trigger a hook in diagnostic mode for testing. | Should | `fire` invokes the handler chain and reports detailed results including timing. |
| CLI-004 | `sc-hooks config` shall display the resolved configuration. | Must | Output shows the parsed TOML with resolved paths. |
| CLI-005 | `sc-hooks handlers` shall list all available builtins and discovered plugin executables with their mode and matchers. | Must | Output distinguishes builtins from plugins, shows mode (sync/async), matchers, and timeout for each. |
| CLI-006 | `sc-hooks install` shall generate `.claude/settings.json` hook entries from the current config and plugin manifests, using plugin-declared matchers for precise event routing. | Must | For a hook event with both sync and async plugins matching `Write`, install generates two handler entries under `matcher: "Write"`. Events with no matching plugins get no entries. |
| CLI-007 | `sc-hooks test <plugin>` shall run the compliance test harness against a plugin and report pass/fail. | Must | Tests verify: valid manifest, contract version compatibility, correct mode behavior, matcher validity, timeout compliance, protocol conformance (valid JSON, correct action fields, absent payload handling). |
| CLI-008 | `sc-hooks exit-codes` shall display the full exit code reference with descriptions and remediation guidance. | Must | Output lists all exit codes (0–10) with names, meanings, and suggested fixes. |

### 3.10 Exit Codes

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| EXC-001 | Exit code 0 (`SUCCESS`): all handlers returned `proceed`. | Must | Claude Code receives 0 and proceeds. |
| EXC-002 | Exit code 1 (`BLOCKED`): a sync handler returned `block`. Block reason on stderr. | Must | Claude Code receives 1 and shows the block reason. |
| EXC-003 | Exit code 2 (`PLUGIN_ERROR`): a handler returned `error` or produced invalid output. | Must | Dispatch log contains error details. |
| EXC-004 | Exit code 3 (`CONFIG_ERROR`): config file missing, malformed, or invalid. | Must | `sc-hooks config` can be used to diagnose. |
| EXC-005 | Exit code 4 (`RESOLUTION_ERROR`): handler(s) could not be resolved. | Must | Error message names the unresolvable handler(s). |
| EXC-006 | Exit code 5 (`VALIDATION_ERROR`): metadata validation failed for a handler's requirements. | Must | Error message names the handler and unsatisfied field. |
| EXC-007 | Exit code 6 (`TIMEOUT`): a handler exceeded its timeout. | Must | Plugin was killed and disabled. |
| EXC-008 | Exit code 7 (`AUDIT_FAILURE`): `sc-hooks audit` found errors. | Must | Audit output details all findings. |
| EXC-009 | Exit code 10 (`INTERNAL_ERROR`): unexpected host error (panic, I/O failure). | Must | Host catches panics and exits cleanly with code 10. |

### 3.11 Security & Sandbox

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| SEC-001 | The host shall respect the sandboxing rules of the calling AI tool. Plugins inherit sandbox restrictions. | Must | A plugin cannot access paths outside the sandbox unless explicitly overridden. |
| SEC-002 | Plugins may declare filesystem paths and network access needs in a `sandbox` manifest field. | Must | `audit` verifies declared paths exist and are accessible. |
| SEC-003 | The config shall support explicit sandbox overrides: `[sandbox] allow_network = ["notify"]`, `allow_paths = { "guard-paths" = [".sc-hooks/guard-paths.toml"] }`. | Must | Only plugins listed in overrides receive expanded access. |
| SEC-004 | `audit` shall warn on sandbox requirements that exceed defaults. With `--strict`, these become errors. | Must | Default audit logs warnings. `audit --strict` exits non-zero on sandbox issues. |
| SEC-005 | `audit` shall warn if plugin executables are world-writable or not owned by the current user. | Should | Warning includes the file path and current permissions. |
| SEC-006 | `audit` shall warn if `.sc-hooks/plugins/` directory has overly permissive permissions. | Should | Warning includes the directory path and suggested fix. |

## 4. Non-Functional Requirements

### 4.1 Auditability

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| AUD-001 | The audit command shall verify that every handler in every hook chain is resolvable. | Must | A missing executable is reported as an error. |
| AUD-002 | The audit command shall call `--manifest` on every plugin and validate the response against the manifest schema. | Must | A plugin with invalid manifest JSON is reported as an error. |
| AUD-003 | The audit command shall validate that all required metadata fields declared by plugins can be satisfied by the current config + runtime environment. | Must | A plugin requiring `atm.inbox` when no such field exists is reported. |
| AUD-004 | The audit command shall validate `dir_exists` and `file_exists` rules against the current filesystem. | Must | A required path that does not exist is reported as an error. |
| AUD-005 | The audit command shall exit 0 on success and exit 7 on any error, suitable for CI integration. | Must | `sc-hooks audit` can be used as a CI gate. |
| AUD-006 | The audit command shall verify that async-mode plugins do not declare blocking behavior and that sync-mode plugins support the hooks they are assigned to. | Must | An async plugin that would need to block is flagged. |
| AUD-007 | The audit command shall display the install plan showing matcher entries, sync/async splits, and async time buckets. | Should | Output shows: `PreToolUse/Write → 2 entries (sync + async, bucket 10-100ms)`. |
| AUD-008 | The audit command shall verify that all plugin-declared matchers are valid event names for their declared hook types. | Must | A plugin declaring `matchers: ["InvalidEvent"]` is flagged. |
| AUD-009 | The audit command shall verify that long-running plugins include a `description` field justifying the declaration. | Must | A long-running plugin without description fails audit. |

### 4.2 Observability

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| OBS-001 | Every hook invocation that executes at least one handler shall produce a structured JSONL log entry with: timestamp, hook type, event, matcher, handler chain, per-handler results with timing, total duration, exit code. Zero-match invocations (no plugins match the event) produce no log entry and exit immediately per DSP-008/PRF-005. | Must | The log file contains one JSON object per line per handler-executing invocation. No entries for zero-match fast path. |
| OBS-002 | Logging configuration shall be in the config.toml under a `[logging]` section with a `hook_log` path and level. | Must | Changing `hook_log` path in config changes where dispatch logs are written. |
| OBS-003 | Log level shall be configurable: debug, info, warn, error. | Should | Setting `level = "debug"` produces more verbose dispatch log entries. |
| OBS-004 | Plugin-level logging is the plugin's responsibility. The host shall not provide logging infrastructure to plugins. | Must | No host config controls plugin log output. |
| OBS-005 | Error events (plugin disable, timeout, invalid JSON) shall include the error type, plugin stderr if available, and the AI notification message in the log entry. | Must | Log entries for errors contain enough information to diagnose without additional tools. |
| OBS-006 | The host shall report plugin errors to the AI session with actionable messages that include the plugin name, error type, and remediation steps. | Must | AI session sees messages like: `"hook guard-paths timed out after 5000ms — disabled. Run 'sc-hooks test guard-paths' to diagnose."` |

### 4.3 Performance

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| PRF-001 | The host binary shall start and parse config in under 5ms. | Must | Measured from process start to config loaded (benchmarked). |
| PRF-002 | A builtin-only handler chain shall complete in under 10ms total. | Must | Measured end-to-end for a typical PreToolUse with log builtin. |
| PRF-003 | Plugin manifest loading shall be cached per invocation to avoid redundant `--manifest` calls. | Should | A handler appearing in multiple hook chains calls `--manifest` once. |
| PRF-004 | Handlers performing long-running work shall either declare `long_running: true` or fork a detached child and return immediately. | Must | A notify handler that sends an ATM message returns in under 10ms; the actual send happens in a forked process. A Slack-approval handler declares `long_running: true` and blocks until response. |
| PRF-005 | When no plugins match the incoming event, the host shall exit 0 in under 2ms with no dispatch log entry. | Must | Zero-match fast path adds negligible overhead to Claude Code's hook system. |

### 4.4 Testability

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| TST-001 | Config parsing shall be unit-testable with in-memory TOML strings. | Must | Tests create configs from strings, not files. |
| TST-002 | Handler resolution shall be unit-testable with temporary directories. | Must | Tests create temp dirs with executables, assert resolution. |
| TST-003 | Dispatch shall be testable with mock handlers that return predetermined results. | Must | Tests verify chain ordering, short-circuit on block, error handling, timeout enforcement. |
| TST-004 | Metadata assembly shall be a pure function testable without filesystem or git. | Must | Tests provide inputs, assert JSON output. |
| TST-005 | The plugin protocol shall be integration-testable: create a minimal plugin, invoke it, assert results. | Must | An integration test creates a temp plugin script, runs the full dispatch loop, verifies log output. |
| TST-006 | The audit command shall be integration-testable with fixture directories. | Must | Tests set up `.sc-hooks/` structures and assert audit output. |
| TST-007 | The sc-hooks-test crate shall provide a compliance test suite that plugin authors use in their own test suites. | Must | `cargo test` in a plugin project runs protocol compliance checks via sc-hooks-test. |
| TST-008 | The compliance test suite shall verify: valid manifest, contract version, mode behavior, matcher validity, timeout compliance, JSON protocol conformance, absent payload handling. | Must | Each check reports pass/fail with specific error messages. |

### 4.5 Portability

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| PRT-001 | The host binary shall compile and run on Linux and macOS. | Must | CI builds and tests on both platforms. |
| PRT-002 | Plugin resolution shall handle platform-specific executable conventions. | Must | A plugin at `.sc-hooks/plugins/notify` (no `.exe`) resolves on both platforms. |

## 5. Constraints

- **Config simplicity:** The config file shall not contain per-handler configuration. Handler-specific settings are the handler's responsibility.
- **No plugin versioning system:** Plugins are not versioned by the host. If a plugin has a bug, replace the executable. The `contract_version` field handles breaking changes to the JSON contract.
- **Minimal env vars:** The host sets at most three environment variables. All structured data flows via JSON.
- **No unsafe code in host:** The host binary shall not use dlopen, C FFI, or unsafe blocks for plugin loading. Plugins are processes.
- **Fail safe:** Protocol violations disable the offending plugin and report to both the dispatch log and AI session. A broken plugin never silently corrupts the chain.
- **Sandbox compliance:** The host respects the calling AI tool's sandbox rules. Overrides are explicit and auditable.

## 6. Key Acceptance Scenarios

### 6.1 Happy Path: PreToolUse Guard (Sync)

Given a config with `PreToolUse = ["guard-paths", "log"]`, both sync-mode, guard-paths matches `["Write", "Bash"]`, log matches `["*"]`, and guard-paths allows the write: the system shall resolve both handlers, validate metadata, pipe JSON to guard-paths, receive proceed, run log builtin, log a JSONL entry, exit 0.

### 6.2 Block Path: Denied Write

Given guard-paths returns block with reason "path in deny list", the system shall: not invoke the log handler, log a JSONL entry showing the block, exit with code 1, and stderr contains the reason.

### 6.3 Mixed Sync/Async Chain

Given `PreToolUse = ["guard-paths", "collect-context"]` where guard-paths is sync and collect-context is async, `sc-hooks install` generates two Claude Code hook entries for matching events. When PreToolUse fires for Write: the sync invocation runs guard-paths and returns proceed/block; the async invocation runs collect-context in the background and its `additionalContext` is delivered on the next Claude turn.

### 6.4 Install Generates Precise Matcher Entries

Given guard-paths matching `["Write", "Bash"]` and log matching `["*"]` for PreToolUse: `sc-hooks install` generates matcher entries for `Write` (guard-paths + log), `Bash` (guard-paths + log). For events not matching any specific plugin (e.g., Read), only `log` is invoked (via its `*` matcher). No hook entry is generated for events where zero plugins match.

### 6.5 Audit Catches Async Plugin Trying to Block

Given an async-mode plugin whose manifest says `mode=async` but whose implementation returns `action=block`, `sc-hooks audit` flags this as a violation: async plugins cannot return block decisions.

### 6.6 Audit Catches Missing Handler

Given a config referencing handler "notify" with no corresponding builtin or executable, `sc-hooks audit` shall report the missing handler, list which hook chain is affected, and exit with code 7.

### 6.7 Audit Catches Unsatisfied Requirement

Given a plugin manifest requiring `atm.inbox` with `validate=non_empty`, and no such field in config context or runtime, `sc-hooks audit` shall report the unsatisfied requirement with the field path and validation rule.

### 6.8 Python Plugin Swap

Given a Rust plugin guard-paths with a bug, the user replaces it with a Python script `guard-paths` (with shebang) that implements the same manifest. `sc-hooks audit` passes. `sc-hooks run PreToolUse Write --sync` uses the Python plugin and produces correct results.

### 6.9 AI-Agnostic Shim

Given a Codex shim that sets `SC_HOOK_AGENT_TYPE=codex` and calls `sc-hooks run`, the handler chain executes identically to a Claude Code invocation. The log entry shows `agent.type` as "codex".

### 6.10 Plugin Timeout

Given a sync plugin that hangs (infinite loop), the host kills it after 5000ms (default), disables the plugin, logs the timeout, and reports to the AI session: `"hook <name> timed out after 5000ms — disabled. Run 'sc-hooks test <name>' to diagnose."` Exit code 6.

### 6.11 Invalid Plugin Output

Given a plugin that writes `not json` to stdout, the host disables the plugin, logs the raw output and error, and reports to the AI session: `"hook <name> returned invalid JSON — disabled. Please notify user!"` Exit code 2.

### 6.12 Long-Running Plugin

Given a sync plugin with `long_running: true` and `timeout_ms: 300000`, the host allows it to run for up to 5 minutes. Audit logs a warning with the plugin's justification. The plugin sends a Slack message and waits for a user response. After receiving the response, it returns `action=proceed` and the chain continues.

### 6.13 Async Time Buckets

Given two async plugins for PostToolUse: `collect-context` with `response_time: {min_ms: 10, max_ms: 100}` and `notify` with `response_time: {min_ms: 1000, max_ms: 5000}`. `sc-hooks install` generates two separate async hook entries. `collect-context` returns additionalContext within 50ms on its own timeline. `notify` returns additionalContext within 3s on its own timeline. Neither waits for the other.

### 6.14 Plugin Compliance Testing

Given a new plugin `my-plugin`, running `sc-hooks test my-plugin` executes the compliance suite: manifest validation, contract version check, mode behavior, matcher validity, timeout compliance, protocol conformance. The test report shows 11 passed, 0 failed. The plugin is marked COMPLIANT and can be used in production.
