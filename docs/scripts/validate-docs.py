#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


REQUIRED_FILES = (
    "codex-capture-checklist.md",
    "codex-findings-ledger.md",
    "gemini-capture-checklist.md",
    "gemini-findings-ledger.md",
    "normalization-checklist.md",
    "normalization-findings-ledger.md",
    "readiness.md",
    "release-checklist.md",
    "sprint-N1.md",
    "sprint-N2.md",
    "sprint-N3.md",
    "sprint-N4.md",
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def read(path: Path) -> str:
    require(path.is_file(), f"missing required file: {path}")
    return path.read_text(encoding="utf-8")


def require_status_not_pending(name: str, content: str) -> None:
    match = re.search(r"Status:\n- `([^`]+)`", content)
    require(match is not None, f"{name}: missing Status block")
    require(match.group(1) != "PENDING", f"{name}: status must not remain PENDING")


def readiness_allows_pending_verdict(content: str) -> bool:
    return (
        "`release_verdict`: `PENDING`" in content
        and "`authorized_by`: `TBD — integration author updates at merge time per ADR-SHK-007`" in content
    )


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: python3 docs/scripts/validate-docs.py docs/phase-N/")

    root = Path(sys.argv[1]).resolve()
    require(root.is_dir(), f"phase directory not found: {root}")

    contents: dict[str, str] = {}
    for filename in REQUIRED_FILES:
        contents[filename] = read(root / filename)

    for filename in (
        "codex-capture-checklist.md",
        "codex-findings-ledger.md",
        "gemini-capture-checklist.md",
        "gemini-findings-ledger.md",
        "normalization-checklist.md",
        "normalization-findings-ledger.md",
        "release-checklist.md",
    ):
        require_status_not_pending(filename, contents[filename])

    readiness = contents["readiness.md"]
    require(
        "release_verdict" in readiness or "release verdict:" in readiness,
        "readiness.md: missing final verdict record",
    )
    require(
        "`NO_GO`" in readiness
        or "`GO`" in readiness
        or "`PARTIAL_GO`" in readiness
        or readiness_allows_pending_verdict(readiness),
        "readiness.md: final verdict not recorded",
    )

    sprint_n4 = contents["sprint-N4.md"]
    require("status: complete" in sprint_n4, "sprint-N4.md: frontmatter status must be complete")
    require("status: complete" in sprint_n4.split("```yaml", 1)[1], "sprint-N4.md: yaml status must be complete")

    print("docs validation: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
