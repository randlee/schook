#!/usr/bin/env bash
# check-req-ids.sh — compare requirement IDs between two git refs or directories
#
# Usage:
#   ./scripts/check-req-ids.sh <ref-a> <ref-b> [docs-path]
#
# Examples:
#   ./scripts/check-req-ids.sh develop integrate/s9-hook-harness
#   ./scripts/check-req-ids.sh develop integrate/s9-hook-harness docs/
#
# Prints IDs present in ref-a but missing from ref-b (dropped in B).
# Prints IDs present in ref-b but missing from ref-a (new in B).
# Exit 0 if no drops; exit 1 if any ID from A is missing in B.

set -euo pipefail

REF_A="${1:-}"
REF_B="${2:-}"
DOCS_PATH="${3:-docs}"

if [[ -z "$REF_A" || -z "$REF_B" ]]; then
  echo "Usage: $0 <ref-a> <ref-b> [docs-path]" >&2
  exit 2
fi

# Regex pattern covering all requirement/ADR/policy ID namespaces in use
PATTERN='HKR-[0-9]+|REQ-SHK-[A-Z]+-[0-9]+|ADR-SHK-[A-Z]+-[0-9]+|ADR-SHK-[0-9]+|PLC-[0-9]+|DEF-[0-9]+|GAP-[0-9]+|BND-[0-9]+[a-z]?|OBS-[0-9]+'

extract_ids() {
  local ref="$1"
  # List all .md files under docs-path at that ref, extract IDs, sort unique
  git ls-tree -r --name-only "$ref" -- "$DOCS_PATH" \
    | grep '\.md$' \
    | xargs -I{} git show "$ref:{}" 2>/dev/null \
    | grep -oE "$PATTERN" \
    | sort -u
}

echo "Extracting IDs from $REF_A ..."
IDS_A=$(extract_ids "$REF_A")

echo "Extracting IDs from $REF_B ..."
IDS_B=$(extract_ids "$REF_B")

DROPPED=$(comm -23 <(echo "$IDS_A") <(echo "$IDS_B"))
ADDED=$(comm -13 <(echo "$IDS_A") <(echo "$IDS_B"))

echo ""
echo "=== IDs in $REF_A missing from $REF_B (DROPPED) ==="
if [[ -z "$DROPPED" ]]; then
  echo "  (none)"
else
  echo "$DROPPED" | sed 's/^/  /'
fi

echo ""
echo "=== IDs in $REF_B not in $REF_A (NEW) ==="
if [[ -z "$ADDED" ]]; then
  echo "  (none)"
else
  echo "$ADDED" | sed 's/^/  /'
fi

echo ""
if [[ -n "$DROPPED" ]]; then
  COUNT=$(echo "$DROPPED" | wc -l | tr -d ' ')
  echo "RESULT: FAIL — $COUNT ID(s) dropped from $REF_A not found in $REF_B"
  exit 1
else
  echo "RESULT: PASS — all IDs from $REF_A are present in $REF_B"
  exit 0
fi
