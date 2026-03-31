import json
import os
import subprocess
import sys
from pathlib import Path

import pytest


def _run_cli(*args: str, workdir) -> subprocess.CompletedProcess[str]:
    pythonpath = str(workdir)
    if existing := os.environ.get("PYTHONPATH"):
        pythonpath = f"{pythonpath}{os.pathsep}{existing}"
    return subprocess.run(
        [sys.executable, "test-harness/hooks/run-schema-drift.py", *args],
        cwd=workdir,
        env={**os.environ, "PYTHONPATH": pythonpath},
        text=True,
        capture_output=True,
    )


def _claude_version(workdir) -> str:
    result = subprocess.run(
        ["claude", "--version"],
        cwd=workdir,
        text=True,
        capture_output=True,
        check=True,
    )
    return result.stdout.strip()


@pytest.mark.provider_claude
def test_run_schema_drift_cli_passes_for_claude(tmp_path, claude_root) -> None:
    repo_root = claude_root.parents[2]
    output_dir = tmp_path / "claude"
    result = _run_cli("claude", "--output-dir", str(output_dir), workdir=repo_root)
    assert result.returncode == 0, result.stderr

    payload = json.loads(result.stdout)
    assert payload["status"] == "PASS"
    assert payload["provider"] == "claude"
    assert payload["claude_version"] == _claude_version(repo_root)

    drift_files = sorted((output_dir / "drift-history").glob("*.json"))
    assert len(drift_files) == 1
    report_dir = claude_root / "reports" / payload["run_timestamp"]
    assert (report_dir / "schema-drift-report.html").exists()
    assert (report_dir / "schema-drift-report.json").exists()
    report_payload = json.loads((report_dir / "schema-drift-report.json").read_text(encoding="utf-8"))
    assert report_payload["claude_version"] == payload["claude_version"]
    assert any(path.endswith(".xhtml") for path in payload["section_paths"])


@pytest.mark.provider_claude
def test_run_schema_drift_cli_errors_for_unknown_provider(tmp_path, claude_root) -> None:
    repo_root = claude_root.parents[2]
    output_dir = tmp_path / "unknown"
    result = _run_cli("unknown", "--output-dir", str(output_dir), workdir=repo_root)
    assert result.returncode == 2
