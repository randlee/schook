# schook Usage

`schook` ships the `sc-hooks` CLI, which validates and executes hook/plugin
chains from a repository-local `.sc-hooks/` runtime layout.

This guide covers:
- install
- config layout
- running the dispatcher
- auditing config and plugin manifests
- where to look next if you are authoring plugins

## Install

Install the CLI from this repo:

Unix-like shells (`bash`, `zsh`, etc. on macOS/Linux):

```bash
cargo install --path sc-hooks-cli --root ~/.local
export PATH="$HOME/.local/bin:$PATH"
```

For local development without installing:

```bash
cargo run -p sc-hooks-cli -- --help
```

Naming note:
- [docs/requirements.md](docs/requirements.md) uses `sc-hooks` as the product command label in acceptance scenarios.
- The current Cargo package and binary artifact in this repo is `sc-hooks-cli`, so the executable examples below use `sc-hooks-cli`.

## Runtime Layout

The current runtime layout is:

```text
.sc-hooks/
  config.toml
  plugins/
    <plugin executable>
  state/
    session.json
  observability/
    sc-hooks/
      logs/
        sc-hooks.log.jsonl
```

Checked example:
- [examples/runtime-layout/README.md](examples/runtime-layout/README.md)

Important:
- the config file is `.sc-hooks/config.toml`
- runtime plugins are executable files under `.sc-hooks/plugins/`
- plugin source crates under `plugins/` in this repo are not the runtime plugin install directory

## Minimal Config

Smallest valid config:

```toml
[meta]
version = 1

[hooks]
PreToolUse = ["guard-paths"]
```

With optional context metadata:

```toml
[meta]
version = 1

[context]
team = "schook"

[hooks]
PreToolUse = ["guard-paths"]

[sandbox]
allow_network = false
allow_paths = []
```

Current config rules come from [docs/requirements.md](docs/requirements.md):
- recognized top-level sections are `[meta]`, `[context]`, `[hooks]`, and `[sandbox]`
- only `[meta]` and `[hooks]` are required

## Run The Dispatcher

Normal execution:

```bash
printf '%s\n' '{"tool_input":{"command":"git status"}}' | sc-hooks-cli run PreToolUse Bash --sync
```

Fire a diagnostic invocation:

```bash
sc-hooks-cli fire PreToolUse Write
```

Inspect the resolved configuration:

```bash
sc-hooks-cli config
```

List discovered plugins:

```bash
sc-hooks-cli handlers
sc-hooks-cli handlers --events
```

Show exit-code reference:

```bash
sc-hooks-cli exit-codes
```

## Audit

Audit validates config, manifests, matcher compatibility, metadata
requirements, and install-plan generation without executing live hook logic.

Run it from the repository root:

```bash
sc-hooks-cli audit
```

If you want to clear persisted session-disable state:

```bash
sc-hooks-cli audit --reset
```

## Generate Claude Hook Settings

Use install-plan generation to produce matcher-driven Claude settings entries:

```bash
sc-hooks-cli install
```

The current runtime generates settings from plugin-declared matchers rather
than blanket wildcard routing.

## Example Invocations

Example 1: validate a checked runtime layout

```bash
cd examples/runtime-layout
sc-hooks-cli audit
```

Expected result:
- exits successfully
- reports no config or manifest errors for the example layout

Example 2: run a sync hook with synthetic payload

```bash
printf '%s\n' '{"tool_input":{"command":"echo hi"}}' | sc-hooks-cli run PreToolUse Write --sync
```

Expected result:
- exits successfully when all sync handlers return `proceed`
- plugin stdout is interpreted through the `HookResult` contract described in [docs/protocol-contract.md](docs/protocol-contract.md)

Example 3: compliance-test a plugin executable

```bash
sc-hooks-cli test .sc-hooks/plugins/guard-paths
```

Expected result:
- runs the shared compliance harness against the specified plugin executable
- exits non-zero if manifest or runtime contract behavior fails the harness

## Plugin Authoring Pointer

If you are building a Rust plugin:
- shared contract/data types live in [sc-hooks-core](sc-hooks-core/)
- Rust authoring conveniences live in [sc-hooks-sdk](sc-hooks-sdk/)
- reusable compliance checks live in [sc-hooks-test](sc-hooks-test/)

If you are checking the wire contract:
- [docs/protocol-contract.md](docs/protocol-contract.md)
- [docs/requirements.md](docs/requirements.md)
- [docs/architecture.md](docs/architecture.md)

Remember:
- the public contract is JSON, env vars, and documented exit codes
- SDK conveniences do not redefine host behavior
