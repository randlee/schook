#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path


SECTIONS = (
    (
        "General",
        (
            ("help", "Show this help."),
            ("build", "Build the full workspace."),
            ("install", "Run the local provider cutover install surface."),
            ("install local-cutover", "Write local Claude/Codex/Gemini cutover configs."),
            ("test", "Run the full workspace test suite."),
            ("test hooks claude", "Run the Claude hook harness pytest suite."),
            ("test hooks codex", "Run the Codex hook harness pytest suite."),
            ("test hooks gemini", "Run the Gemini hook harness pytest suite."),
            ("clean", "Remove workspace build artifacts."),
            ("ci", "Run the local CI-equivalent command set."),
        ),
    ),
    (
        "Formatting",
        (
            ("fmt", "Check Rust formatting."),
            ("fmt check", "Check Rust formatting."),
            ("fmt write", "Format the Rust workspace in place."),
            ("fmt apply", "Format the Rust workspace in place."),
        ),
    ),
    (
        "Lint",
        (
            ("lint", "Run the repo lint surface."),
            ("lint fmt", "Run only the format gate."),
            ("lint clippy", "Run only Clippy with warnings denied."),
            ("lint sc-boundary", "Run the provider-normalization boundary lint wrapper."),
        ),
    ),
)


def render_help(repo_name: str) -> str:
    lines = [
        f"{repo_name} task runner",
        "",
        "Usage:",
        "  just <recipe>",
        "",
    ]
    width = max(len(name) for _, recipes in SECTIONS for name, _ in recipes)
    for section_name, recipes in SECTIONS:
        lines.append(f"{section_name}:")
        for name, description in recipes:
            lines.append(f"  {name.ljust(width)}  {description}")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def main() -> int:
    repo_name = Path(__file__).resolve().parent.parent.name
    print(render_help(repo_name), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
