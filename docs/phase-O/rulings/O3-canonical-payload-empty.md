# O.3 CanonicalPayload Empty Variant

## Decision

`CanonicalPayload<'a>` intentionally omits the planned `Empty` variant.

## Why

None of the approved `O.3` normalization surfaces require an empty canonical
payload:

- Codex `SessionStart` and Gemini `SessionStart` / `SessionEnd` normalize as
  `SessionLifecycle`
- Codex `PreToolUse` and Gemini `BeforeTool` / `AfterTool` normalize as
  `ToolUse`
- Gemini `BeforeAgent` normalizes as `AgentLifecycle`

Because every approved surface already maps to a concrete payload family, an
`Empty` variant would be dead structure rather than active runtime behavior.

## Scope

This ruling applies only to `CanonicalPayload<'a>` in
`crates/sc-hooks-core/src/normalization.rs`.

If a later approved provider surface truly needs no payload body, that later
phase must add the variant together with fixture evidence and updated
compatibility rules.
