#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def sc_lint_repo_candidates(root: Path) -> list[Path]:
    return [
        root.parent / "sc-lint",
        root.parent.parent / "sc-lint",
        root.parent.parent.parent / "sc-lint",
    ]


def resolve_backend(root: Path) -> tuple[list[str], str]:
    installed = shutil.which("sc-lint-boundary")
    if installed:
        return [installed], f"homebrew:{installed}"

    for candidate in sc_lint_repo_candidates(root):
        cargo_toml = candidate / "Cargo.toml"
        if cargo_toml.exists():
            return (
                [
                    "cargo",
                    "run",
                    "-q",
                    "--manifest-path",
                    str(cargo_toml),
                    "-p",
                    "sc-lint-boundary",
                    "--",
                ],
                f"repo-local:{candidate}",
            )

    raise SystemExit(
        "sc-lint-boundary unavailable: install the Homebrew 0.1.0 binary or provide the ../sc-lint fallback"
    )


def analyze(command_prefix: list[str], target_root: Path, rule: str | None = None) -> dict:
    command = [
        *command_prefix,
        "analyze",
        "--root",
        str(target_root),
        "--format",
        "json",
    ]
    if rule is not None:
        command.extend(["--rule", rule])
    completed = subprocess.run(
        command,
        cwd=repo_root(),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip() or "unknown failure"
        raise SystemExit(f"sc-lint-boundary execution failed: {detail}")
    return json.loads(completed.stdout)


def expect_rule(command_prefix: list[str], target_root: Path, rule: str, expected_rule_id: str) -> None:
    payload = analyze(command_prefix, target_root, rule=rule)
    findings = payload.get("findings", [])
    if payload.get("status") != "fail":
        raise SystemExit(f"self-check expected fail status for {rule}, got {payload.get('status')}")
    if not any(finding.get("rule_id") == expected_rule_id for finding in findings):
        raise SystemExit(f"self-check for {rule} did not produce {expected_rule_id}")


def write_fixture_sources(root: Path) -> None:
    (root / "Cargo.toml").write_text(
        '[workspace]\nmembers = ["crates/example"]\nresolver = "2"\n',
        encoding="utf-8",
    )
    crate_dir = root / "crates" / "example"
    src_dir = crate_dir / "src"
    src_dir.mkdir(parents=True)
    (crate_dir / "Cargo.toml").write_text(
        '[package]\nname = "example"\nversion = "0.1.0"\nedition = "2024"\n',
        encoding="utf-8",
    )
    (src_dir / "lib.rs").write_text("mod owner; mod user; mod api; mod impls;\n", encoding="utf-8")
    (src_dir / "owner.rs").write_text(
        "#[sc_lint(boundary.internal_only)]\nstruct Secret;\n",
        encoding="utf-8",
    )
    (src_dir / "user.rs").write_text(
        "pub struct Uses(crate::owner::Secret);\n",
        encoding="utf-8",
    )
    (src_dir / "api.rs").write_text(
        "#[sc_lint(boundary.forbid_external_impls)]\n"
        "pub trait Tokenize {\n"
        "    fn tokenize(&self) -> usize;\n"
        "}\n\n"
        "pub struct Thing;\n",
        encoding="utf-8",
    )
    (src_dir / "impls.rs").write_text(
        "impl crate::api::Tokenize for crate::api::Thing {\n"
        "    fn tokenize(&self) -> usize {\n"
        "        1\n"
        "    }\n"
        "}\n",
        encoding="utf-8",
    )


def run_self_check(command_prefix: list[str]) -> None:
    with tempfile.TemporaryDirectory(prefix="schook-sc-boundary-") as tempdir:
        fixture_root = Path(tempdir)
        write_fixture_sources(fixture_root)
        expect_rule(command_prefix, fixture_root, "internal-only", "SCB-BOUNDARY-002")
        expect_rule(command_prefix, fixture_root, "forbid-external-impls", "SCB-BOUNDARY-003")


def main() -> int:
    root = repo_root()
    command_prefix, backend_label = resolve_backend(root)
    run_self_check(command_prefix)
    payloads = [
        analyze(command_prefix, root, rule="internal-only"),
        analyze(command_prefix, root, rule="forbid-external-impls"),
    ]
    findings: list[dict] = []
    statuses: list[str] = []
    for payload in payloads:
        statuses.append(payload.get("status", "unknown"))
        findings.extend(payload.get("findings", []))
    status = "pass" if all(value == "pass" for value in statuses) else "fail"
    print(f"sc-lint-boundary backend={backend_label} status={status} findings={len(findings)}")
    for finding in findings[:5]:
        rule_id = finding.get("rule_id", "UNKNOWN")
        message = finding.get("message", "").strip()
        print(f"- {rule_id}: {message}")
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
