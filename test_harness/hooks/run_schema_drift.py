from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from test_harness.hooks.claude.models.payloads import DriftErrorCode, ProviderStatus
from test_harness.hooks.claude.schema_drift import build_failure_report, run_drift as run_claude_drift
from test_harness.hooks.paths import HOOKS_ROOT


def _atomic_write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_name(f"{path.name}.tmp")
    try:
        tmp_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        tmp_path.replace(path)
    except Exception:
        if tmp_path.exists():
            tmp_path.unlink()
        raise


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run schema drift validation for a provider")
    parser.add_argument("provider", help="Provider to validate")
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Provider output directory root (defaults to test-harness/hooks/<provider>)",
    )
    args = parser.parse_args(argv)

    output_dir = args.output_dir or (HOOKS_ROOT / args.provider)
    if args.provider != "claude":
        report = build_failure_report(
            DriftErrorCode.PROVIDER_NOT_SUPPORTED,
            f"Provider {args.provider!r} is not supported by this Phase 3 implementation",
            source=args.provider,
        )
        _atomic_write_json(output_dir / "drift-history" / f"{report.run_timestamp}-drift.json", report.model_dump(mode="json"))
        print(json.dumps(report.model_dump(mode="json"), indent=2))
        return 2

    try:
        report = run_claude_drift(output_dir)
    except Exception as exc:
        report = build_failure_report(
            DriftErrorCode.CAPTURE_FAILED,
            str(exc),
            source="claude",
        )
        _atomic_write_json(output_dir / "drift-history" / f"{report.run_timestamp}-drift.json", report.model_dump(mode="json"))
        print(json.dumps(report.model_dump(mode="json"), indent=2))
        return 2

    print(json.dumps(report.model_dump(mode="json"), indent=2))
    if report.status == ProviderStatus.PASS:
        return 0
    if report.status == ProviderStatus.DRIFT:
        return 1
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
