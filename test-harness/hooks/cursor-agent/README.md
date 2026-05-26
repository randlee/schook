# Cursor Agent Hook Harness

Cursor Agent is now a maintained harness-only provider.

Current status:

- maintained harness provider
- approved-reference fixtures and non-empty manifest landed in `Q.6`
- runtime normalization and machine cutover remain deferred

Current planning rule:

- keep `docs/hook-api/cursor-agent-hook-api.md` as the provider reference
- keep all Cursor work under the existing `cursor-agent` / `cursor_agent`
  naming boundary
- do not treat this harness tree as runtime-authorization proof by itself

This directory now owns:

- Cursor prompts
- local Cursor capture hooks or capture scripts
- Cursor fixtures
- Cursor reports
- Cursor schema placeholders
- Cursor `pytest` tests

Follow-on in `Q.7`:

- provider-local payload models
- final API-doc/model reconciliation for the retained manifest surfaces
