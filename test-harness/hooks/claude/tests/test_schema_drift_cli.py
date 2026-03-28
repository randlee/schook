import json
import subprocess
import sys

import pytest


def _run_cli(*args: str, workdir) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "test-harness/hooks/run-schema-drift.py", *args],
        cwd=workdir,
        text=True,
        capture_output=True,
    )


@pytest.mark.provider_claude
def test_run_schema_drift_cli_passes_for_claude(tmp_path, claude_root) -> None:
    repo_root = claude_root.parents[2]
    output_dir = tmp_path / "claude"
    result = _run_cli("claude", "--output-dir", str(output_dir), workdir=repo_root)
    assert result.returncode == 0, result.stderr

    payload = json.loads(result.stdout)
    assert payload["status"] == "PASS"
    assert payload["provider"] == "claude"

    drift_files = sorted((output_dir / "drift-history").glob("*.json"))
    assert len(drift_files) == 1
    report_dir = claude_root / "reports" / payload["run_timestamp"]
    assert (report_dir / "schema-drift-report.html").exists()
    assert (report_dir / "schema-drift-report.json").exists()
    assert any(path.endswith(".xhtml") for path in payload["section_paths"])


@pytest.mark.provider_claude
def test_run_schema_drift_cli_errors_for_unknown_provider(tmp_path, claude_root) -> None:
    repo_root = claude_root.parents[2]
    output_dir = tmp_path / "unknown"
    result = _run_cli("unknown", "--output-dir", str(output_dir), workdir=repo_root)
    assert result.returncode == 2
