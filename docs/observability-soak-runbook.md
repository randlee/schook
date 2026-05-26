# sc-hooks Observability Soak Runbook

## 1. Purpose

This runbook defines the long-term QA path for observability concurrency and
degraded-path validation.

It is intentionally separate from the normal fast unit-test suite.

## 2. Fast Branch Gate

Every branch that claims `SC-LOG-S7` coverage must pass:

- `cargo +1.94.1 test --workspace`
- `cargo +1.94.1 test --workspace --release`

The required branch-level concurrency proof is:

- `full_mode_concurrent_agents_shard_runs_without_corruption`

What it proves:

- 64 concurrent host invocations can share one audit root
- each invocation gets its own run-scoped audit directory
- every run writes valid JSON records without corruption
- the design avoids a single shared hot audit file by sharding under
  `.sc-hooks/audit/runs/<run-id>/`

The required degraded-path release proof is:

- `standard_mode_logger_init_failure_is_non_blocking`
- `standard_mode_emit_failure_is_non_blocking`
- `full_mode_append_failure_is_non_blocking`
- `full_mode_prune_failure_is_non_blocking`

## 3. Long-Term Soak Procedure

Use this when validating production hardening beyond the branch gate.

1. Run `cargo +1.94.1 test --workspace --release full_mode_concurrent_agents_shard_runs_without_corruption -- --nocapture`.
2. Repeat the release concurrency test several times on the same machine.
3. Run the four degraded-path release tests:
   `standard_mode_logger_init_failure_is_non_blocking`,
   `standard_mode_emit_failure_is_non_blocking`,
   `full_mode_append_failure_is_non_blocking`,
   and `full_mode_prune_failure_is_non_blocking`.
4. Record the commit hash, platform, toolchain, and whether all runs stayed green.

## 4. Interpretation

Pass criteria:

- every concurrent run finishes successfully
- one valid run directory exists per concurrent invocation
- every `events.jsonl` file parses cleanly
- degraded-path release tests stay green

Escalate if any of these occur:

- missing run directories
- malformed JSON in any audit file
- intermittent failures across repeated release runs
- degraded-path tests changing hook outcomes

## 5. Scope Boundary

This runbook validates the committed local file-backed audit design only.

It does not validate:

- live structured streaming
- OTLP/exporter behavior
- remote sinks
