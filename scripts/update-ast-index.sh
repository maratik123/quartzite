#!/usr/bin/env bash
# update-ast-index.sh — refresh the local ast-index DB.
#
# Invoked by the PreToolUse Bash hook in .claude/settings.json on `git commit`.
# Always exits 0 so a failure here never blocks a commit (AC6, AC7).
#
# - No-op when `ast-index` is not installed on the host.
# - Runs `ast-index update` (incremental); on the missing-DB sentinel
#   `Index not found` (ast-index v3.42.0 writes it to stdout with exit 0)
#   falls back to a one-shot `ast-index rebuild`.

set -eu

command -v ast-index >/dev/null 2>&1 || exit 0
out=$(ast-index update 2>&1) || true
case "$out" in *"Index not found"*) ast-index rebuild >/dev/null 2>&1 || true ;; esac
exit 0
