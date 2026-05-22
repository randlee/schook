# Gemini Hook API

## Purpose

This document records the current `schook` planning baseline for Gemini hook
capture. Unlike the Codex and Claude documents, this file is currently a
planning-stage boundary document, not a captured-evidence ledger.

Until `N.2` closes, no field or behavior described here is approved for
canonical `schook` fixture inventory unless it is backed by repo-owned Gemini
fixtures and models.

## Current Planning Baseline

Current locally observed planning inputs:

- `gemini` is installed locally
- Gemini exposes hook-management commands through `gemini hooks ...`
- Gemini exposes resume-related invocation through `--resume`
- Gemini exposes output controls:
  - `--output-format text`
  - `--output-format json`
  - `--output-format stream-json`

These are planning inputs only. They are not yet approved `schook`
hook-contract evidence.

## N.2 Capture Boundary

`N.2` must capture and prove only `schook`-owned Gemini hook evidence:

- raw stdin payloads delivered directly to the hook
- hook-process environment variables delivered directly to the hook
- harness-owned control-semantic observations captured during local execution

Relay-side observations, CLI help, or provider memory may inform the plan, but
they must not be promoted into approved Gemini fixture inventory without direct
harness capture.

## First-Pass Capture Targets

The first Gemini pass should capture:

- every real Gemini hook surface configurable through `gemini hooks` that can
  be exercised locally
- every payload field present at those surfaces
- every relevant hook-process environment variable present at those surfaces
- one tool-style payload
- one lifecycle/session-continuation payload if Gemini exposes one
- output-format interactions, if hook-observable:
  - `text`
  - `json`
  - `stream-json`
- control semantics for each surface:
  - blocking vs async
  - exit-code handling
  - stdout/stderr contract

## Current Planning Placeholders

Until `N.2` captures real Gemini evidence, any named surfaces here remain
placeholders only:

| Surface | Current status | Notes |
| --- | --- | --- |
| `preTool` | planning-placeholder | named candidate direct hook surface for tool-pre interception |
| `sessionStart` | planning-placeholder | named candidate lifecycle surface if exposed by local Gemini hooks runtime |

`N.2` must replace placeholder-only surface descriptions with either:

- a fixture-backed named hook-surface entry
- or an explicit `confirmed-not-exercisable` note tied to the Gemini findings
  ledger

## Output-Format Verification Requirement

`N.2` must record whether Gemini output-format selection changes
hook-observable behavior.

The authoritative result belongs in:

- `docs/phase-N/gemini-findings-ledger.md`

Accepted outcomes:

- `no hook-observable difference across text/json/stream-json`
- or a per-format finding row describing the observed difference
- or an explicit blocking reason if output-format verification cannot be
  completed locally

## Promotion Rule

No Gemini field becomes an approved canonical `schook` mapping candidate until:

1. `N.2` captures it in repo-owned Gemini fixtures
2. Gemini provider models validate it
3. `N.3` proves compatible semantics with at least one other provider per
   `ADR-SHK-006`
