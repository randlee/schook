from test_harness.hooks.gemini.capture import capture_from_stdin


# Gemini may replay hook delivery; this capture path must stay safe on repeated stdin.
raise SystemExit(capture_from_stdin("before-tool"))
