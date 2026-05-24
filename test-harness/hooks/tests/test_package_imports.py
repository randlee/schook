from importlib import import_module

import pytest


@pytest.mark.provider_claude
@pytest.mark.provider_codex
@pytest.mark.provider_gemini
def test_cross_provider_harness_packages_import_cleanly() -> None:
    import_module("test_harness.hooks.claude.tests.test_harness_imports")
    import_module("test_harness.hooks.codex.models.payloads")
    import_module("test_harness.hooks.gemini.capture")
