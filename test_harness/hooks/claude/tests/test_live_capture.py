import os

import pytest


@pytest.mark.live_capture
@pytest.mark.provider_claude
def test_live_capture_marker_requires_opt_in() -> None:
    if os.environ.get("SCHOOK_ENABLE_LIVE_CAPTURE") != "1":
        pytest.skip("live provider capture is opt-in; rerun with -m live_capture and SCHOOK_ENABLE_LIVE_CAPTURE=1")

    assert os.environ["SCHOOK_ENABLE_LIVE_CAPTURE"] == "1"
