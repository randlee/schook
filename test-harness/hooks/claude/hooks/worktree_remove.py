#!/usr/bin/env python3
"""Capture Claude WorktreeRemove payloads."""

from _capture_common import run_capture_hook


if __name__ == "__main__":
    raise SystemExit(run_capture_hook("worktree-remove"))
