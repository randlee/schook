# Cursor Agent Hook API

## Purpose

This document records the currently verified Cursor-facing hook surfaces that
matter to `sc-hooks` planning. It is intentionally separate from the Claude and
Codex documents because the current evidence comes from a different
combination of local CLI behavior and public Cursor hook documentation.

## Current Source Of Truth

- local CLI help from `cursor-agent --help`
- local Cursor CLI state under `$HOME/.cursor/`
- public Cursor docs page `https://cursor.com/docs/hooks`

This document only promotes facts that are directly visible from those sources.

## Platform Rules

- `cursor-agent` is a real installed CLI on this machine
- current planning treats Cursor Agent as a provider-specific compatibility
  target, not as part of the initial Claude ATM implementation baseline
- public Cursor hook names may be documented here before implementation, but
  they do not become implementation inputs until the harness captures them
- Cursor-targeting runtime work remains deferred until a later explicitly
  approved follow-on pass

## Path And Environment Rules

- local Cursor CLI state currently lives under `$HOME/.cursor/`
- `$HOME/.cursor/cli-config.json` is a verified local configuration
  input
- `$HOME/.cursor/hooks.json` is not currently present on this machine
- current working directory must not be treated as a stable hook identity or
  provider-contract signal
- provider-specific path and environment assumptions must be captured by the
  harness before implementation relies on them

## Current Local Runtime Baseline

Current locally verified CLI behavior:

- `cursor-agent` is installed and runnable
- headless/CLI usage supports:
  - `--print`
  - `--output-format text | json | stream-json`
  - `--mode plan | ask`
  - `--resume`
  - `--continue`
  - `--workspace`
  - `--worktree`

Current locally verified config state:

- `$HOME/.cursor/cli-config.json` exists
- `$HOME/.cursor/hooks.json` does not currently exist on this machine

That means `sc-hooks` can treat Cursor Agent as an installed provider with a
current CLI/runtime surface, but not as a provider whose local hook config and
stdin payloads have already been captured in this repo.

## Current Public Hook Baseline

Current publicly documented hook/event names visible on the Cursor hooks page
include:

- `beforeShellExecution`
- `beforeMCPExecution`
- `beforeReadFile`
- `afterFileEdit`
- `stop`
- `sessionStart`
- `sessionEnd`
- `preCompact`
- `subagentStart`
- `subagentStop`
- `beforeSubmitPrompt`
- `afterAgentResponse`
- `afterAgentThought`

For the current S9 follow-on planning scope, the relevant Cursor hook set is:

- controllable hooks:
  - `beforeShellExecution`
  - `beforeMCPExecution`
  - `beforeReadFile`
- informational hooks:
  - `afterFileEdit`
  - `stop`

## Verified Public Schema Fragments

The current Cursor hooks page also shows these currently documented field or
config names:

- common configuration keys:
  - `failClosed`
  - `matcher`
- example request/response fields:
  - `command`
  - `permission`
- documented hook payload fields in the current page content:
  - `transcript_path`
  - `user_email`
  - `is_parallel_worker`
  - `git_branch`
  - `duration_ms`
  - `message_count`
  - `tool_call_count`
  - `loop_count`
  - `modified_files`
  - `agent_transcript_path`
  - `is_first_compaction`

These are verified as names currently present in Cursor's public hook docs.
They are not yet promoted here as guaranteed `cursor-agent` CLI stdin fields
for the specific S9 hook set until the live harness captures them.

## Planning Rules For `sc-hooks`

- do not assume the full Cursor IDE hook schema is identical to the
  `cursor-agent` CLI runtime without live capture evidence
- do not write `sc-hooks` code against Cursor field names that have only been
  seen in public docs and not yet captured by the harness
- use the current public hook names as planning inputs only
- require live fixture capture before any Cursor-targeting hook crate is
  implemented

## Current Platform Gaps

- no captured `cursor-agent` hook payload fixtures exist in this repo yet
- no current local `hooks.json` is configured on this machine
- no `sc-hooks`-owned Cursor validation models exist yet
- no verified provider-specific stdin schema has been captured yet for:
  - `beforeShellExecution`
  - `beforeMCPExecution`
  - `beforeReadFile`
  - `afterFileEdit`
  - `stop`

## Design Implications For `sc-hooks`

- treat Cursor hook support as a documented follow-on provider target, not as
  part of the Claude implementation baseline
- use the schema-capture harness to prove the actual `cursor-agent` hook
  payloads before any hook crate depends on them
- separate controllable hooks from informational hooks during planning because
  they have different risk profiles and likely different response contracts
