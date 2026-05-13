#!/usr/bin/env bash
#
# Delete the local progress files belonging to the merged branch passed as $1.
#
# Run by `/pr-merged` skill (.claude/skills/pr-merged/SKILL.md, step 3) after
# `git checkout master && git pull`, before `git branch -d <previous-branch>`.
#
# Derivation (PR linkage per design Q2 of PR #300):
#   1. `gh pr list --state merged --head <branch>` -> merged PR number
#   2. `grep -l "Tracked in: #N"` in ai-docs/plans/done/ + ai-docs/plans/
#      -> matching /task spec file
#   3. `basename <spec>.spec.md` -> spec-base name
#   4. Delete ai-docs/plans/<spec-base>.progress.md (if it exists)
#   5. Delete ai-docs/pr-comments/pr-<N>.progress.md (the fallback path used
#      by /pr-commented for PRs not produced by /task)
#   6. `rmdir ai-docs/pr-comments` -- opportunistic cleanup; non-fatal if the
#      directory still has unrelated files
#
# Failure modes:
# - `PR_NUM` empty (branch merged outside `gh`, or PR is closed-not-merged):
#   prints a one-line note and exits 0 (nothing reliable to derive paths from).
# - `SPEC_PATH` empty (no matching /task spec; e.g. manual PR without /task):
#   skips the /task-progress-file deletion silently and proceeds to the
#   /pr-comments fallback path.
# - rm -f is silent on missing files (intentional — files may not exist).
# - rmdir failing on non-empty directory is expected and ignored.
#
# Deferred-task progress files (ai-docs/plans/deferred/) are intentionally
# NOT touched -- deferral is its own workflow and a deferred task has no
# merged PR to drive cleanup.

set -uo pipefail

PREV_BRANCH="${1:-}"
if [ -z "${PREV_BRANCH}" ]; then
  printf 'cleanup-progress.sh: usage: cleanup-progress.sh <previous-branch>\n' >&2
  exit 2
fi

PR_NUM=$(gh pr list --state merged --head "${PREV_BRANCH}" --json number --jq '.[0].number // empty')

if [ -z "${PR_NUM}" ]; then
  printf 'pr-merged: no merged PR found for %s; skipping progress-file cleanup.\n' "${PREV_BRANCH}"
  exit 0
fi

SPEC_PATH=$(grep -l "Tracked in:.*#${PR_NUM}\b" ai-docs/plans/done/*.spec.md ai-docs/plans/*.spec.md 2>/dev/null | head -n1)

if [ -n "${SPEC_PATH}" ]; then
  SPEC_BASE=$(basename "${SPEC_PATH}" .spec.md)
  rm -f "ai-docs/plans/${SPEC_BASE}.progress.md"
fi

rm -f "ai-docs/pr-comments/pr-${PR_NUM}.progress.md"

# Opportunistic cleanup -- fails non-fatally if directory has other files
# or does not exist. Exit code intentionally ignored.
rmdir ai-docs/pr-comments 2>/dev/null || true

exit 0
