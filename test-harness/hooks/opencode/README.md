# opencode Hook Harness

opencode is now a maintained harness-only provider.

Current status:

- maintained harness provider
- approved-reference fixtures and non-empty manifest landed in `Q.8`
- runtime normalization and machine cutover remain deferred

Current planning rule:

- keep `docs/hook-api/opencode-agent-hook-api.md` as the provider reference
- keep all opencode work under the existing `opencode` naming boundary
- do not treat this harness tree as runtime-authorization proof by itself

This directory now owns:

- opencode prompts
- local opencode capture hooks or capture scripts
- opencode fixtures
- opencode reports
- opencode schema placeholders
- opencode `pytest` tests

Follow-on in `Q.9`:

- provider-local payload models
- final API-doc/model reconciliation for the retained manifest surfaces
