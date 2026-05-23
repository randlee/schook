import os

import pytest


@pytest.mark.live_capture
@pytest.mark.provider_codex
def test_live_capture_marker_requires_opt_in(request: pytest.FixtureRequest) -> None:
    # Intentional smoke test: this file proves the explicit opt-in gate and
    # marker wiring for live provider capture without invoking Codex.
    if os.environ.get("SCHOOK_ENABLE_LIVE_CAPTURE") != "1":
        pytest.skip("live provider capture is opt-in; rerun with -m live_capture and SCHOOK_ENABLE_LIVE_CAPTURE=1")

    assert os.environ["SCHOOK_ENABLE_LIVE_CAPTURE"] == "1"
    assert any(mark.name == "live_capture" for mark in request.node.iter_markers())
