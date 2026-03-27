# Codex Follow-On Plan

## Purpose

This document defines the post-Claude follow-on plan for Codex support.

It is planning-only. It does not authorize implementation work.

## Current Verified Baseline

Current verified Codex inputs for this plan are:

- local `codex --help` CLI behavior
- [codex-hook-api.md](./codex-hook-api.md)
- current `schook` hook-planning rules in `docs/plugin-plan-s9.md`

Current verified local CLI facts:

- `codex` is installed on this machine
- interactive and non-interactive surfaces exist
- session continuation surfaces exist via `resume` and `fork`
- working-root control exists via `--cd`

Current verified `schook` planning facts:

- Codex is not part of the first Claude implementation path
- no standalone Codex hook stdin schema is captured in this repo yet
- current Codex evidence is relay/event-model oriented rather than full hook-payload capture

## Entry Gate

No Codex-targeting hook implementation may start until all of these are true:

- the Claude-first hook track is accepted and stable
- a dedicated Codex harness pass is explicitly approved
- real Codex hook payload fixtures are captured in `test-harness/hooks/codex/`
- Codex validation models and schema artifacts exist
- this plan is revised from captured Codex evidence

## Required Harness Work

The first Codex-specific pass must produce:

- provider launch adapter under `test-harness/hooks/codex/`
- canned prompt scenarios for repeatable Codex runs
- raw captured payload fixtures
- Codex-specific validation models
- formal validation and drift reports

Minimum first-pass capture targets:

- the first executable pre-tool hook surface actually available in the installed Codex runtime
- any verified session or turn identity surface exposed during the same run
- at least one relay-style completion or lifecycle signal if the runtime emits one

## Current Blockers

Codex work is still blocked on:

- no captured Codex hook stdin schema in this repo
- no verified Codex `SessionStart`-equivalent capture path in this repo
- no `schook`-owned Codex fixtures or models
- no evidence yet that the Claude ATM session-correlation approach maps directly onto Codex

## Planned Sequence

1. build the Codex harness adapter and canned prompts
2. capture raw Codex payload fixtures
3. validate them with Codex-specific models
4. revise `docs/hook-api/codex-hook-api.md` and this plan from captured evidence
5. decide whether Codex needs:
   - a session-lifecycle follow-on crate, or
   - only relay/gate follow-on crates
6. only then create implementation tasks

## Initial Design Boundaries

Until capture proves otherwise:

- do not claim Claude-equivalent `SessionStart` semantics for Codex
- do not reuse Claude field names as Codex contract assumptions
- do not implement Codex runtime crates on top of relay-event guesses alone

## Deliverable For A Later Approved Codex Sprint

A later Codex implementation sprint may be opened only after this plan has been
revised from captured fixtures and the resulting crate inventory is explicitly
approved.
