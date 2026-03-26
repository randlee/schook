# Runtime Layout Example

This directory is the checked example `.sc-hooks/` runtime layout for `sc-hooks`.

It is intended to be a minimal contributor-proof setup:

- `.sc-hooks/config.toml` is the runtime config root.
- `.sc-hooks/plugins/guard-paths` is a runnable example plugin executable.
- `sc-hooks audit` succeeds from this directory without reading source code.
- `sc-hooks run PreToolUse Write --sync` succeeds from this directory with the example plugin.

Quick verification:

```bash
sc-hooks audit
printf '%s\n' '{"tool_input":{"command":"echo hi"}}' | sc-hooks run PreToolUse Write --sync
```

This example is intentionally small. It proves the expected runtime shape, not release-grade plugin behavior.
