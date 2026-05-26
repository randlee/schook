# Gemini Raw Capture Notes

Raw Gemini hook captures in this directory preserve the machine-local payloads
and environment snapshots from the capture session that produced them.

Implications:

- absolute paths, home-directory paths, and repo-checkout paths may be
  machine-local
- the files are retained as provider evidence, not as portable fixture
  contracts
- tests and docs should treat these captures as raw inputs that may contain
  capture-session-specific paths
