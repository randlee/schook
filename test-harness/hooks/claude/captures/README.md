# Claude Captures

Raw and normalized run artifacts are written here by the harness.

Phase 1 creates the directory structure and hook writers. Later phases add
approved capture runs.

Capture artifacts in this tree may include verbatim machine-local paths such as
home directories, temp locations, and worktree roots from the session where
the capture was recorded. Those path literals are expected evidence artifacts,
not portable contract requirements. In particular, `raw/` files are
intentionally unredacted and are not portable across environments.
