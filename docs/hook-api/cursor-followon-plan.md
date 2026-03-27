# Cursor Follow-On Plan

## Purpose

This document defines the post-Claude follow-on plan for Cursor Agent support.

It is planning-only. It does not authorize implementation work.

## Current Verified Baseline

Current verified Cursor inputs for this plan are:

- local `cursor-agent --help` CLI behavior
- [cursor-agent-hook-api.md](./cursor-agent-hook-api.md)
- current `schook` hook-planning rules in `docs/plugin-plan-s9.md`

Current verified local CLI facts:

- `cursor-agent` is installed on this machine
- headless mode exists via `--print`
- output format control exists via `--output-format text | json | stream-json`
- session continuation exists via `--resume` and `--continue`
- workspace isolation exists via `--workspace` and `--worktree`

Current verified planning facts from the existing Cursor API doc:

- current public hook names relevant to `schook` planning include:
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
- no `schook`-owned captured Cursor payload fixtures exist yet
- current Cursor work remains deferred until a dedicated later harness pass

## Entry Gate

No Cursor-targeting hook implementation may start until all of these are true:

- the Claude-first hook track is accepted and stable
- a dedicated Cursor harness pass is explicitly approved
- actual `cursor-agent` hook payload fixtures are captured in `test-harness/hooks/cursor-agent/`
- Cursor validation models and schema artifacts exist
- this plan is revised from captured Cursor evidence

## Required Harness Work

The first Cursor-specific pass must produce:

- provider launch adapter under `test-harness/hooks/cursor-agent/`
- canned prompt scenarios for repeatable Cursor runs
- local Cursor hook configuration used only by the harness run
- raw captured payload fixtures
- Cursor-specific validation models
- formal validation and drift reports

Minimum first-pass capture targets:

- `beforeShellExecution`
- `beforeMCPExecution`
- `beforeReadFile`
- `afterFileEdit`
- `stop`

Optional later capture targets only after the first pass succeeds:

- `sessionStart`
- `sessionEnd`
- `preCompact`
- `subagentStart`
- `subagentStop`

## Current Blockers

Cursor work is still blocked on:

- no captured `cursor-agent` hook stdin schema in this repo
- no local harness-owned hook config checked into this repo yet
- no `schook`-owned Cursor validation models exist yet
- no proof yet that the public Cursor docs match the installed CLI runtime payloads exactly

## Planned Sequence

1. configure a dedicated Cursor harness run
2. capture raw Cursor payload fixtures for the controllable and informational first-pass hooks
3. validate them with Cursor-specific models
4. revise `docs/hook-api/cursor-agent-hook-api.md` and this plan from captured evidence
5. split the follow-on implementation targets into:
   - gate-style hooks
   - relay-style hooks
6. only then create implementation tasks

## Initial Design Boundaries

Until capture proves otherwise:

- do not rely on public Cursor doc field names as implementation inputs
- do not assume `sessionStart` / `sessionEnd` payloads map cleanly to the Claude ATM session model
- do not build Cursor runtime crates before the controllable-hook request/response contract is captured

## Deliverable For A Later Approved Cursor Sprint

A later Cursor implementation sprint may be opened only after this plan has
been revised from captured fixtures and the resulting Cursor hook split is
explicitly approved.
