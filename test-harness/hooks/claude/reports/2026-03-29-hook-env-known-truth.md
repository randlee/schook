# Claude Hook Env Known Truth (2026-03-29)

## Scope

This note records the env-backed harness pass on
`feature-s9-hook-env-capture`. It exists to pin down root-resolution and hook
environment behavior from real captures before the next documentation rewrite.

## Evidence Set

Primary env-backed captures in this pass:

- `20260329T203144.187831Z-session-start.json`
- `20260329T203144.187831Z-session-start.env.json`
- `20260329T203144.767873Z-pretooluse-bash.json`
- `20260329T203144.767873Z-pretooluse-bash.env.json`
- `20260329T203144.881734Z-posttooluse-bash.json`
- `20260329T203144.881734Z-posttooluse-bash.env.json`
- `20260329T203357.612677Z-pretooluse-agent.json`
- `20260329T203357.612677Z-pretooluse-agent.env.json`
- `20260329T203149.133073Z-stop.json`
- `20260329T203149.133073Z-stop.env.json`
- `20260329T203149.289632Z-session-end.json`
- `20260329T203149.289632Z-session-end.env.json`
- `20260329T204830.666482Z-session-start.json`
- `20260329T204830.666482Z-session-start.env.json`
- `20260329T204833.204858Z-session-start.json`
- `20260329T204833.204858Z-session-start.env.json`
- `20260329T204834.283536Z-session-end.json`
- `20260329T204834.283536Z-session-end.env.json`
- `20260329T211418.974201Z-pre-compact.json`
- `20260329T211418.974201Z-pre-compact.env.json`
- `20260329T211435.352660Z-session-start.json`
- `20260329T211435.352660Z-session-start.env.json`
- `20260329T211532.933293Z-session-end.json`
- `20260329T211532.933293Z-session-end.env.json`
- `20260329T211532.964966Z-session-start.json`
- `20260329T211532.964966Z-session-start.env.json`

Additional baseline observation during this pass:

- the outer shell environment already included several `ATM_*` variables and
  non-root `CLAUDE_*` variables such as `CLAUDE_MCP_CONFIG`
- the outer shell environment did **not** include `CLAUDE_PROJECT_DIR`
- `CLAUDE_PROJECT_DIR` first appeared inside the hook process env snapshots

## `CLAUDE_*` Baseline Diff Summary

Baseline shell env before launching Claude for this pass:

- `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`
- `CLAUDE_CODE_TASK_LIST_ID=agent-team-mail`
- `CLAUDE_MCP_CONFIG=/Users/randlee/.config/claude/mcp.json`
- `CLAUDE_TEMPLATES_ROOT=/Users/randlee/Documents/p3-documentation/.templates/nuget-package`

Observed across the hook env snapshots in this pass:

- `CLAUDE_CODE_ENTRYPOINT`
- `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`
- `CLAUDE_CODE_TASK_LIST_ID`
- `CLAUDE_ENV_FILE`
- `CLAUDE_MCP_CONFIG`
- `CLAUDE_PROJECT_DIR`
- `CLAUDE_TEMPLATES_ROOT`

Classification:

| Variable | Baseline shell | Hook env | Classification | Notes |
| --- | --- | --- | --- | --- |
| `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` | present | present | preserved from baseline | same value observed in hooks |
| `CLAUDE_CODE_TASK_LIST_ID` | present | present | preserved from baseline | same value observed in hooks |
| `CLAUDE_MCP_CONFIG` | present | present | preserved from baseline | same value observed in hooks |
| `CLAUDE_TEMPLATES_ROOT` | present | present | preserved from baseline | same value observed in hooks |
| `CLAUDE_CODE_ENTRYPOINT` | absent | present | hook-only addition | injected by Claude during hook execution |
| `CLAUDE_ENV_FILE` | absent | present | hook-only addition | injected by Claude during hook execution |
| `CLAUDE_PROJECT_DIR` | absent | present | hook-only addition | the most important root signal observed in this pass |

Current implication:

- the harness now records the full hook-side `CLAUDE_*` set
- for this pass, `CLAUDE_PROJECT_DIR` is confirmed as hook-only relative to the
  launch shell
- `CLAUDE_CODE_ENTRYPOINT` and `CLAUDE_ENV_FILE` are additional hook-only
  signals worth keeping under regression so future design work can investigate
  whether they carry stable, useful semantics

## Known Truth Table

| Surface | Evidence | Raw `cwd` | `CLAUDE_PROJECT_DIR` | `ATM_IDENTITY` / `ATM_TEAM` | `CLAUDE_PLUGIN_ROOT` | Known Truth |
| --- | --- | --- | --- | --- | --- | --- |
| `SessionStart(source="startup")` | `20260329T203144.187831Z-session-start{,.env}.json` | project root | present and equal to startup `cwd` | present (`chook` / `atm-dev`) in ATM-driven run | not observed | startup `cwd` and `CLAUDE_PROJECT_DIR` align exactly |
| `PreToolUse(Bash)` before `cd` | `20260329T203144.767873Z-pretooluse-bash{,.env}.json` | project root | present and still equal to startup root | present in ATM-driven run | not observed | Bash hook starts at project root before directory drift |
| `PostToolUse(Bash)` after `cd` | `20260329T203144.881734Z-posttooluse-bash{,.env}.json` | `.../test-harness/hooks/claude` | present and still equal to startup root | present in ATM-driven run | not observed | later hook `cwd` can drift while `CLAUDE_PROJECT_DIR` stays pinned to startup root |
| `PreToolUse(Agent)` | `20260329T203357.612677Z-pretooluse-agent{,.env}.json` | project root | present and equal to startup root | present in ATM-driven run | not observed | Agent hook receives the same stable project-root env |
| `Stop` after drift | `20260329T203149.133073Z-stop{,.env}.json` | `.../test-harness/hooks/claude` | present and still equal to startup root | present in ATM-driven run | not observed | `Stop` may carry drifted `cwd`; root signal remains stable |
| `SessionEnd` after drift | `20260329T203149.289632Z-session-end{,.env}.json` | `.../test-harness/hooks/claude` | present and still equal to startup root | present in ATM-driven run | not observed | `SessionEnd` may also carry drifted `cwd`; root signal remains stable |
| `SessionStart(source="resume")` | `20260329T204833.204858Z-session-start{,.env}.json` | project root | present and equal to resumed runtime root | present in ATM-driven run | not observed | resumed runtime still receives `CLAUDE_PROJECT_DIR`; in this capture it matches the resumed root and preserves the prior `session_id` |
| `PreCompact` | `20260329T211418.974201Z-pre-compact{,.env}.json` | project root | present and equal to project root | absent in plain manual terminal run | not observed | `PreCompact` is env-backed now; root signal stayed stable without ATM routing vars |
| `SessionStart(source="compact")` | `20260329T211435.352660Z-session-start{,.env}.json` | project root | present and equal to project root | absent in plain manual terminal run | not observed | compact-return session still receives stable `CLAUDE_PROJECT_DIR` |
| `SessionEnd(reason="clear")` | `20260329T211532.933293Z-session-end{,.env}.json` | project root | present and equal to project root | absent in plain manual terminal run | not observed | `/clear` ends the prior session with reason `clear` and stable project-root env |
| `SessionStart(source="clear")` | `20260329T211532.964966Z-session-start{,.env}.json` | project root | present and equal to project root | absent in plain manual terminal run | not observed | the new cleared session receives stable `CLAUDE_PROJECT_DIR`; ATM routing vars are not implicit in a plain manual terminal |

## Immediate Design Implications

- `SessionStart(source="startup")` is the only current capture-backed place to
  establish immutable session root for a fresh runtime instance.
- Later hook `cwd` values are operational context only; they must not rewrite
  immutable session root.
- `CLAUDE_PROJECT_DIR` is present in the captured hooks above and remained
  stable when `cwd` drifted after `cd`.
- `ATM_IDENTITY` and `ATM_TEAM` are present in ATM-driven runs, but absent in a
  plain manual terminal run unless exported explicitly.
- `CLAUDE_PLUGIN_ROOT` was not observed in this env-backed pass and should
  remain unverified until a capture proves otherwise.

## Need-To-Address

- The current repo-level prompt guidance says:
  - `project_root_dir chains from CLAUDE_PROJECT_DIR; do not substitute cwd`
- That is **not** sufficient as the final design statement.
- The current discussion-established design is stricter and needs to replace
  the prompt wording in control documents:
  - immutable root is established at `SessionStart(source="startup")`
  - later `cwd` is current-directory context only
  - inbound `CLAUDE_PROJECT_DIR` is a required cross-check against immutable
    root, not a silent fallback source
  - `sc-hooks` should normalize project-root context for all downstream
    consumers, even if Claude drops the env var on some hooks later
  - any divergence between immutable root and inbound `CLAUDE_PROJECT_DIR`
    must log prominently for investigation
- The earlier doc-only correction pass happened before env capture existed and
  should be treated as intermediate, not final authority.
- `compact` and `clear` are now env-backed, but the automation path still needs
  work:
  - manual harness sessions produced the needed env snapshots
  - the current scripted `compact` / `clear` probes did not reproduce those
    artifacts automatically
  - if we want those surfaces in unattended regression capture, the helper
    needs another iteration
