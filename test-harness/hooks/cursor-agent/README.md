# Cursor Agent Hook Harness

Cursor Agent is documented as a provider reference in the current planning set,
but it is deferred from the first harness pass.

This directory exists now so the harness layout and ownership are clear without
forcing Cursor capture into the first implementation gate.

Current status:

- documented
- deferred from first harness implementation
- no provider-specific capture or runtime implementation is required yet

Current planning rule:

- keep `docs/hook-api/cursor-agent-hook-api.md` as the provider reference
- do not make Cursor harness work block the Claude-first path

When Cursor work starts later, this directory should own:

- Cursor prompts
- local Cursor capture hooks or capture scripts
- Cursor models and schema
- Cursor fixtures
- Cursor reports
- Cursor `pytest` tests
