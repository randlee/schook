# Gemini Follow-On Plan

## Purpose

This document defines the post-Claude follow-on plan for Gemini CLI support.

It is planning-only. It does not authorize implementation work.

## Current Verified Baseline

Current verified Gemini inputs for this plan are:

- local `gemini --help` CLI behavior
- current `schook` hook-planning rules in `docs/plugin-plan-s9.md`

Current verified local CLI facts:

- `gemini` is installed on this machine
- interactive and non-interactive surfaces exist
- session continuation exists via `--resume`
- Gemini exposes explicit hook-management commands through `gemini hooks <command>`
- output format control exists via `--output-format text | json | stream-json`

Current verified `schook` planning facts:

- Gemini is not part of the first Claude implementation path
- no Gemini hook payload fixtures exist in this repo yet
- no `schook`-owned Gemini validation models exist yet

## Entry Gate

No Gemini-targeting hook implementation may start until all of these are true:

- the Claude-first hook track is accepted and stable
- a dedicated Gemini harness pass is explicitly approved
- actual Gemini hook payload fixtures are captured in `test-harness/hooks/gemini/`
- Gemini validation models and schema artifacts exist
- this plan is revised from captured Gemini evidence

## Required Harness Work

The first Gemini-specific pass must produce:

- provider launch adapter under `test-harness/hooks/gemini/`
- canned prompt scenarios for repeatable Gemini runs
- local capture hooks for the Gemini runtime
- raw captured payload fixtures
- Gemini-specific validation models
- formal validation and drift reports

Minimum first-pass capture targets:

- the first Gemini hook surfaces made configurable by `gemini hooks`
- one tool-style hook payload
- one lifecycle or session-continuation payload if the runtime exposes one

## Current Blockers

Gemini work is still blocked on:

- no captured Gemini hook stdin schema in this repo
- no verified Gemini hook event taxonomy in this repo
- no verified Gemini session-identity strategy for `schook`
- no current evidence yet for which Gemini hook surfaces are closest to the Claude ATM baseline

## Planned Sequence

1. inspect and configure the installed Gemini hook surfaces in a dedicated harness pass
2. capture raw Gemini payload fixtures
3. validate them with Gemini-specific models
4. revise this plan from captured evidence
5. decide whether Gemini maps onto:
   - lifecycle hooks,
   - tool hooks,
   - relay hooks,
   - or a different provider-specific split
6. only then create implementation tasks

## Initial Design Boundaries

Until capture proves otherwise:

- do not infer Gemini hook payload fields from the CLI command names alone
- do not assume Claude or Codex session-correlation fields exist in Gemini
- do not write Gemini-targeting crates before the hook manager and payload shape are captured together

## Deliverable For A Later Approved Gemini Sprint

A later Gemini implementation sprint may be opened only after this plan has
been revised from captured fixtures and the resulting Gemini hook taxonomy is
explicitly approved.
