# Step 3 — Sprint Scope Hardening (`chook`)

## Execute

**1. Render the message**

```bash
sc-compose render \
  --root .claude/skills/plan-hardening \
  --file 02-sprint-scope-hardening.xml.j2 \
  --var-file /tmp/plan-hardening-vars.json \
  --output /tmp/step-3-message.xml
```

The vars file or rendered task must include `step-2` fenced JSON as the
required input payload.
It must also carry current round metadata:
- `round_id`
- `round_index`
- `replay_nonce`
- `reviewed_commit`
- `previous_reviewed_commit`
- `findings_hash`

**2. Send to `chook`**

```bash
atm send chook --team schook --stdin < /tmp/step-3-message.xml
```

**3. Check the response**

Read the `chook` response and confirm it contains fenced JSON.
The expected output shape is specified inside
`02-sprint-scope-hardening.xml.j2`.
Do not proceed to Step 4 until that fenced JSON is present and well formed.
If the response is incomplete or malformed, send a correction request to
`chook` immediately.
Save the extracted fenced JSON to `/tmp/step-3.json`.

When you forward the result to `team-lead`, send a standalone ATM message that
contains only the fenced JSON block and no surrounding prose. Example:

```bash
cat <<'EOF' >/tmp/step-3-handoff.jsonmsg
```json
{
  "status": "PASS",
  "mode": "plan-hardening-sprint-scope",
  "round_id": "STEP3-R1",
  "round_index": 1,
  "reviewed_commit": "abc1234",
  "previous_reviewed_commit": "",
  "iterations": 0,
  "findings_resolved": 0,
  "final_finding_count": 0,
  "sprint_splits_added": 0,
  "docs_modified": [],
  "docs_created": [],
  "ready_for_next_step": true,
  "errors": []
}
```
EOF

atm send team-lead --team schook --from chook --stdin < /tmp/step-3-handoff.jsonmsg
```

**4. Route by status**

- `PASS` -> proceed to Step 4
- `FAIL` -> re-render and re-send Step 3 to `chook`
- if `chook` ACKs but responds as though the same already-fixed round is
  being replayed, increment `round_index`, update `round_id`, refresh
  `replay_nonce` with the current UTC timestamp, and re-render before
  re-sending

Maintain the round table after every Step 3 / Step 4 loop:

| Round | Step | Reviewer | reviewed_commit | status | blocking | important | minor | findings_hash | supersedes | Note |
|-------|------|----------|-----------------|--------|----------|-----------|-------|---------------|------------|------|

## Hard stops

- `step-2` fenced JSON from the Step 2 response is missing or malformed: do
  not advance; send a correction request immediately and identify the missing
  or malformed fields explicitly
- fenced JSON is missing or malformed: do not advance; send a correction
  request immediately and identify the missing or malformed fields explicitly
