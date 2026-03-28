# Claude Haiku Follow-Up Capture Notes

## Scope

This note records the targeted follow-up harness pass on
`feature/s9-harness-followup`.

Targets:

- confirm `PreToolUse(Agent)` remains the correct spawn surface name
- capture `SessionStart(source="resume")`
- probe `SessionStart(source="clear")`
- run one bounded `Notification(idle_prompt)` probe with the corrected matcher
  and idle timeout

## Results

### PreToolUse(Agent)

- status: confirmed
- evidence:
  - approved fixture `fixtures/approved/pretooluse-agent.json`
  - content-level validation in `claude/tests/test_fixture_validation.py`

### SessionStart(source="resume")

- status: captured
- method:
  - launched a prompt-driven Claude/Haiku session under harness-local settings
  - allowed the session to exit normally
  - resumed the returned `session_id` with `claude --resume <session_id> -p`
- raw evidence:
  - `captures/raw/20260328T041636.162713Z-session-start.json`
- associated end-of-turn evidence from the resumed session:
  - `captures/raw/20260328T041637.483194Z-stop.json`
  - `captures/raw/20260328T041637.711673Z-session-end.json`
- approved fixture promotion:
  - `fixtures/approved/session-start-resume.json`

### SessionStart(source="clear")

- status: not captured in this automated pass
- reason:
  - an automated PTY attempt sending `/clear` did not produce
    `SessionStart(source="clear")`; it only yielded a new `startup` session
    at `captures/raw/20260328T041916.304626Z-session-start.json`
  - reliable capture still appears to require a manual interactive `/clear`
    session under harness-local settings
- next step:
  - perform one manual harness-local `/clear` run and promote the resulting
    `SessionStart(source="clear")` payload if captured

### Notification(idle_prompt)

- status: bounded probe executed; no raw notification payload was produced
- method:
  - `run-interactive-capture.py notification`
  - `matcher = ""`
  - full idle timeout window
- interpretation:
  - keep the surface wired
  - do not promote it into implementation-required behavior without a live
    payload fixture
  - this bounded probe produced no new `*-notification.json` files in
    `captures/raw/`
