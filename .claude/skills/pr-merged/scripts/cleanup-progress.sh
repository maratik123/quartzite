#!/usr/bin/env bash
#
# Delete the local progress files belonging to the merged branch passed as $1.
#
# Run by `/pr-merged` skill (.claude/skills/pr-merged/SKILL.md, step 3) after
# `git checkout master && git pull`, before `git branch -d <previous-branch>`.
#
# Derivation (PR linkage):
#   1. `gh pr list --state merged --head <branch>` -> merged PR number
#   2. `gh pr view <PR_NUM> --json body` -> PR body
#   3. `grep -oiE '(Close[sd]?|Fix(es|ed)?|Resolve[sd]?) #[0-9]+'` on the body
#      -> the first Closes/Fixes/Resolves #N reference. ISSUE_NUM is the
#      trailing digits. (Per /task + /interview convention every spec carries
#      `**Tracked in:** #<ISSUE_NUM>`, not #<PR_NUM>.)
#   4. `grep -lE "^\*\*Tracked in:\*\* #<ISSUE_NUM>"` in ai-docs/plans/done/
#      + ai-docs/plans/ -> matching /task spec file. The anchor on `^**Tracked
#      in:** ` prevents prose lines that mention "Tracked in:" alongside a
#      different #N (e.g. spec narrative discussing another spec's tracking
#      issue) from false-matching.
#   5. `basename <spec>.spec.md` -> spec-base name
#   6. Delete ai-docs/plans/<spec-base>.progress.md (if it exists)
#   7. Delete ai-docs/pr-comments/pr-<PR_NUM>.progress.md (the fallback path
#      used by /pr-commented for PRs not produced by /task)
#   8. `rmdir ai-docs/pr-comments` -- opportunistic cleanup; non-fatal if the
#      directory still has unrelated files
#
# Failure modes (all exit 0 -- workflow step 4 proceeds regardless):
# - `PR_NUM` empty (branch merged outside `gh`, or PR is closed-not-merged):
#   prints a one-line note to stdout and exits 0 (nothing reliable to derive
#   paths from).
# - `ISSUE_NUM` empty (PR body has no Closes/Fixes/Resolves #N line -- e.g.
#   manual non-/task PR, or `/task` PR whose body convention drifted):
#   prints a one-line warning to STDERR naming the PR and branch, skips the
#   /task-progress-file deletion, proceeds to the /pr-comments fallback path.
# - `SPEC_PATH` empty (issue number derived but no spec carries
#   `Tracked in: #<ISSUE_NUM>` -- e.g. PR closes a satellite issue without
#   its own /task spec): prints a one-line warning to STDERR naming the
#   issue number, skips the /task-progress-file deletion, proceeds to the
#   /pr-comments fallback path.
# - rm -f is silent on missing files (intentional -- files may not exist;
#   idempotent re-runs are safe).
# - rmdir failing on non-empty directory is expected and ignored.
#
# Multi-close PRs (rare; e.g. `Closes #289` + `Closes #277` on PR #295):
# the FIRST closing-keyword match wins. By convention the first-listed
# issue is the primary /task tracking issue; satellite closures are
# bundled cleanup items without their own spec.
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

ISSUE_NUM=$(gh pr view "${PR_NUM}" --json body --jq '.body' 2>/dev/null \
  | grep -oiE '(Close[sd]?|Fix(es|ed)?|Resolve[sd]?) #[0-9]+' \
  | head -n1 \
  | grep -oE '[0-9]+$')

if [ -z "${ISSUE_NUM}" ]; then
  printf 'pr-merged: PR #%s body has no Closes/Fixes/Resolves #N line; skipping spec-driven progress-file cleanup for %s.\n' "${PR_NUM}" "${PREV_BRANCH}" >&2
else
  SPEC_PATH=$(grep -lE "^\*\*Tracked in:\*\* #${ISSUE_NUM}\b" ai-docs/plans/done/*.spec.md ai-docs/plans/*.spec.md 2>/dev/null | head -n1)
  if [ -n "${SPEC_PATH}" ]; then
    SPEC_BASE=$(basename "${SPEC_PATH}" .spec.md)
    rm -f "ai-docs/plans/${SPEC_BASE}.progress.md"
  else
    printf 'pr-merged: no /task spec matches Tracked in: #%s (derived from PR #%s, branch %s); skipping spec-driven progress-file cleanup.\n' "${ISSUE_NUM}" "${PR_NUM}" "${PREV_BRANCH}" >&2
  fi
fi

rm -f "ai-docs/pr-comments/pr-${PR_NUM}.progress.md"

# Opportunistic cleanup -- fails non-fatally if directory has other files
# or does not exist. Exit code intentionally ignored.
rmdir ai-docs/pr-comments 2>/dev/null || true

exit 0
