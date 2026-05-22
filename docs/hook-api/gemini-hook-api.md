# Gemini Hook API

## Purpose

This document records the currently verified Gemini-facing hook surfaces that
matter to `sc-hooks` planning. It is intentionally separate from the Claude and
Codex documents because Gemini's hook activation path and control semantics are
different.

All verified facts below come from `N.2` repo-owned Gemini fixtures, env
snapshots, or direct local control-semantics probes.

## Registration Model

Current verified local activation path:

- user-scope `~/.gemini/settings.json`
- run under an isolated temporary `HOME` populated with copied auth files and
  harness-only hook definitions

Important caution:

- `gemini hooks --help` currently exposes only `migrate`
- that CLI subcommand surface is not the full runtime hook contract
- isolated local probes did not demonstrate clean workspace
  `.gemini/settings.json` activation, so workspace-scope registration remains
  unresolved and is not promoted into the approved capture workflow

## Verified Hook Surfaces

### `SessionStart`

Verified payload fields:

| Field | Type | Notes |
| --- | --- | --- |
| `session_id` | string UUID | stable per session |
| `transcript_path` | string | Gemini transcript path under the active `HOME` |
| `cwd` | string | current working directory at hook fire time |
| `hook_event_name` | string | `"SessionStart"` |
| `timestamp` | string | ISO-8601 timestamp |
| `source` | string | verified values: `"startup"`, `"resume"` |

Verified environment fields:

| Variable | Notes |
| --- | --- |
| `GEMINI_SESSION_ID` | matched `session_id` in every captured payload |
| `GEMINI_PROJECT_DIR` | matched repo root / working directory |
| `GEMINI_CWD` | matched current working directory |
| `GEMINI_PLANS_DIR` | Gemini per-session plans path |
| `GEMINI_CLI_TRUST_WORKSPACE` | `"true"` in captured runs |
| `GEMINI_CLI_NO_RELAUNCH` | `"true"` in captured runs |
| `GEMINI_API_KEY` | present in raw env; redacted in approved fixtures |

Control semantics:

- non-zero hook exit produced warnings on stderr
- the agent still completed and the CLI exited `0` in the local probe
- no stdout response contract was required for the captured path

### `SessionEnd`

Verified payload fields:

| Field | Type | Notes |
| --- | --- | --- |
| `session_id` | string UUID | matched `SessionStart.session_id` |
| `transcript_path` | string | same transcript path as session start |
| `cwd` | string | current working directory |
| `hook_event_name` | string | `"SessionEnd"` |
| `timestamp` | string | ISO-8601 timestamp |
| `reason` | string | verified value: `"exit"` |

Control semantics:

- post-exit surface
- failure warnings were surfaced on stderr
- the isolated non-zero exit path was partially confounded by Gemini quota
  exhaustion after the hook fired, so final shell-exit behavior is not treated
  as fully isolated evidence yet

### `BeforeAgent`

Verified payload fields:

| Field | Type | Notes |
| --- | --- | --- |
| `session_id` | string UUID | stable per session |
| `transcript_path` | string | transcript path under active `HOME` |
| `cwd` | string | current working directory |
| `hook_event_name` | string | `"BeforeAgent"` |
| `timestamp` | string | ISO-8601 timestamp |
| `prompt` | string | raw prompt text submitted to Gemini |

Control semantics:

- pre-agent blocking surface
- non-zero hook exit blocked the turn and surfaced the stderr text as the block
  reason
- the CLI still exited `0` in the local probe, so block signaling is not a
  shell-exit failure

### `BeforeTool`

Verified payload fields:

| Field | Type | Notes |
| --- | --- | --- |
| common session fields | same shape as other Gemini hooks | `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `timestamp` |
| `tool_name` | string | verified value: `"run_shell_command"` |
| `tool_input.command` | string | verified value: `"pwd"` |
| `tool_input.description` | string | natural-language tool description |

Control semantics:

- blocking surface for the tool call
- non-zero hook exit blocked tool execution and surfaced the stderr text in the
  tool error path
- the overall CLI still exited `0` in the local probe

### `AfterTool`

Verified payload fields:

| Field | Type | Notes |
| --- | --- | --- |
| common session fields | same shape as other Gemini hooks | `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `timestamp` |
| `tool_name` | string | verified value: `"run_shell_command"` |
| `tool_input` | object | same shape as `BeforeTool` |
| `tool_response.llmContent` | string | summarized tool output text |
| `tool_response.returnDisplay` | array | provider-specific rendered output structure |

Control semantics:

- post-tool but still gating surface
- non-zero hook exit marked the tool result blocked after the tool ran and
  surfaced stderr in the tool error path
- the overall CLI still exited `0` in the local probe

### `AfterAgent`

Verified payload fields:

| Field | Type | Notes |
| --- | --- | --- |
| common session fields | same shape as other Gemini hooks | `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `timestamp` |
| `prompt` | string | original prompt text |
| `prompt_response` | string | Gemini response text |
| `stop_hook_active` | boolean | verified value: `false` in captured runs |

Control semantics:

- post-response surface
- failure warnings were surfaced on stderr
- the isolated non-zero exit path was not re-run cleanly after quota exhaustion,
  so it remains only partially isolated evidence

## Resume Continuity

Direct local probes verified:

- `gemini --resume latest` fires a fresh `SessionStart`
- the resumed payload uses the same surface with `source = "resume"`

Current normalization caution:

- `source` belongs to Gemini until `N.3` proves compatible lifecycle semantics
  with another provider

## Output Format Result

Direct local probes compared `text`, `json`, and `stream-json` CLI output
modes.

Verified result:

- no hook-observable payload-key difference for `SessionStart` or `AfterAgent`
- no hook-observable env-key difference for `SessionStart` or `AfterAgent`
- CLI stdout format changes, but hook stdin/env schema did not change in the
  captured probes

## Approved Newtype Candidates

Fields that should be treated as semantic identifier candidates in later Rust
runtime planning:

- `session_id`
- `GEMINI_SESSION_ID`
- `hook_event_name`
- `tool_name`

Fields that remain provider-local or plain strings for now:

- `transcript_path`
- `cwd`
- `GEMINI_PROJECT_DIR`
- `GEMINI_CWD`
- `GEMINI_PLANS_DIR`
- `tool_response.returnDisplay`

## Design Implications For `sc-hooks`

- Gemini is now a first-pass harness provider, not only a planning placeholder
- hook registration cannot be inferred from `gemini hooks --help`; the settings
  runtime is the real contract surface
- `SessionStart.source`, `tool_name = "run_shell_command"`, and
  `tool_response.returnDisplay` remain provider-local until `N.3` proves a
  compatible cross-provider mapping
- output-format selection should not be modeled as a hook-schema variant based
  on current evidence
