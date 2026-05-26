# Claude Captures

Raw and normalized run artifacts are written here by the harness.

Phase 1 creates the directory structure and hook writers. Later phases add
approved capture runs.

Raw capture artifacts in raw/ may include machine-local paths such as home
directories, temp locations, and worktree roots from the capture session.
Those path literals are expected evidence artifacts, not portable contract
requirements.
