# Phase P Canonical Hook Mapping

This table is the authoritative retained lifecycle mapping for the Phase P
extension work.

Rules:

- Codex and Gemini retained lifecycle surfaces must enter the generic runtime
  only through `sc_hooks_core::normalization::normalize_runtime_dispatch`
- no provider-specific runtime bypass around `ProviderHookNormalizer` is
  allowed for these surfaces
- Codex surfaces that `P.1` proved non-exercisable remain disposition-only
  rows here and do not gain live canonical variants

| Provider | Provider Hook | Canonical Hook | Canonical Payload | Available Variables | Provider-Local Fields | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Claude | `Stop` | `Stop` | `StopLifecycle` | `session_id`, `cwd`, `transcript_path`, `stop_hook_active`, `permission_mode`, `last_assistant_message` | `hook_event_name` | Baseline direct runtime path; Claude does not pass through `ProviderHookNormalizer`. |
| Codex | `notify` (`type = "agent-turn-complete"`) | `Stop` | `StopLifecycle` | `session_id` (from `thread-id`), `cwd`, `stop_hook_active = false`, `last_assistant_message` | `client`, `input-messages`, `turn-id`, `type`, raw `last-assistant-message` key | Retained live surface; normalized as `CanonicalHook::Codex(CodexHook::Notify)`. |
| Codex | `Stop` | disposition only | n/a | none | n/a | `P.1` confirmed this surface is not exercisable in the retained Codex hook set. |
| Codex | `resume` | disposition only | n/a | none | n/a | `P.1` confirmed this surface is not exercisable in the retained Codex hook set. |
| Codex | `fork` | disposition only | n/a | none | n/a | `P.1` confirmed this surface is not exercisable in the retained Codex hook set. |
| Gemini | `AfterAgent` | `Stop` | `StopLifecycle` | `session_id`, `cwd`, `transcript_path`, `stop_hook_active`, `last_assistant_message` (trimmed from `prompt_response`) | `prompt`, raw `prompt_response`, `timestamp`, `hook_event_name` | Retained live surface; normalized as `CanonicalHook::Gemini(GeminiHook::AfterAgent)`. |
