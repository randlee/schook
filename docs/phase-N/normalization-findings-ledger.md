# Normalization Findings Ledger

Status:
- `COMPLETE`

Purpose:
- authoritative findings ledger for `N.3` cross-provider normalization review

| id | provider | field_or_surface | finding | mapping_candidate | disposition | notes |
| --- | --- | --- | --- | --- | --- | --- |
| `NRM-001` | `claude,codex,gemini` | `session_id` | all three providers expose a semantic session identifier in approved fixtures | `canonical.session_id` | `canonical candidate` | future Rust runtime type should use a newtype such as `SessionId` rather than bare `String` |
| `NRM-002` | `claude,codex,gemini` | `cwd` | all three providers expose current working directory snapshots on approved surfaces | `canonical.ai_current_dir` | `canonical candidate` | `cwd` does not establish immutable root by itself |
| `NRM-003` | `claude,codex,gemini` | `transcript_path` | all three providers expose provider transcript paths | `canonical.transcript_path` | `canonical candidate` | path-like context only, not an identity field |
| `NRM-004` | `claude,codex,gemini` | `tool_input.command` | shell-command payloads share a compatible nested command string across approved shell-like tool surfaces | `canonical.shell_command` | `canonical candidate` | only valid after provider-specific shell-tool surfaces are normalized |
| `NRM-005` | `claude,codex,gemini` | `hook_event_name` / `notify.type` | direct hook names diverge because Codex `notify` uses `type = "agent-turn-complete"` instead of `hook_event_name` | none | `provider-local` | keep raw event names provider-side until a later lifecycle enum is proved |
| `NRM-006` | `claude,codex,gemini` | lifecycle `source` | `source` values overlap partially (`startup`, `resume`) but Claude also uses `compact` and `clear` while Codex/Gemini do not prove equivalent semantics | none | `unresolved` | treat `source` as provider-local enum candidate only |
| `NRM-007` | `claude,codex,gemini` | raw `tool_name` | approved raw tool names are not cross-provider compatible: Claude/Codex use `Bash`, Gemini uses `run_shell_command` | none | `provider-local` | normalize surface family first, then consider a canonical shell-tool enum later |
| `NRM-008` | `codex` | `thread-id`, `turn-id`, `tool_use_id` | Codex carries extra correlation identifiers with no approved Gemini or Claude equivalent in this sprint | none | `provider-local` | semantic identifiers within Codex only; keep out of canonical contract for now |
| `NRM-009` | `claude,codex,gemini` | root env and project-dir signals | `CLAUDE_PROJECT_DIR`, `GEMINI_PROJECT_DIR`, `GEMINI_CWD`, and `CODEX_THREAD_ID` do not share compatible semantics | none | `provider-local` | provider adapters must continue to own root/session recovery rules |
| `NRM-010` | `claude,codex,gemini` | post-response / turn-complete family | Claude `Stop`, Codex `notify`, and Gemini `AfterAgent` all sit in the same lifecycle family but have incompatible payload shapes and different blocking posture | none | `unresolved` | runtime idle/turn-complete abstraction remains deferred to a later provider-adapter phase |
| `NRM-011` | `gemini` | `tool_response.returnDisplay` | Gemini emits provider-rendered structured display output with no compatible Claude or Codex equivalent | none | `provider-local` | do not map into canonical tool-output fields |
| `NRM-012` | `claude,codex` | `permission_mode` | Claude and Codex both expose `permission_mode`, but Gemini does not and semantics were not proved equivalent cross-provider | none | `provider-local enum candidate` | retain in provider docs/ledgers only |
| `NRM-013` | `claude,codex,gemini` | enum/newtype planning | semantic identifiers and small closed value sets are visible enough to plan later Rust typing without promoting them into the current runtime contract | `future Rust typing` | `carry forward` | newtypes: `SessionId`, provider-local `ThreadId`/`TurnId`; enums: provider-local `source`, `tool_name`, `permission_mode` |
