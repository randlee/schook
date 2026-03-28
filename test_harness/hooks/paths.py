from __future__ import annotations

import os
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
HOOKS_ROOT = REPO_ROOT / "test-harness" / "hooks"
CLAUDE_ROOT = HOOKS_ROOT / "claude"
CLAUDE_FIXTURES_ROOT = CLAUDE_ROOT / "fixtures" / "approved"
CLAUDE_REPORTS_ROOT = CLAUDE_ROOT / "reports"
CLAUDE_DRIFT_HISTORY_ROOT = CLAUDE_ROOT / "drift-history"
CLAUDE_SCHEMA_ROOT = CLAUDE_ROOT / "schema"
ATM_HOME_ROOT = Path(os.environ.get("ATM_HOME") or Path.home())
GLOBAL_HTML_SKILL_ROOT = ATM_HOME_ROOT / ".claude" / "skills" / "html-report"
GLOBAL_HTML_AGENT_PATH = ATM_HOME_ROOT / ".claude" / "agents" / "html-report-generator.md"
