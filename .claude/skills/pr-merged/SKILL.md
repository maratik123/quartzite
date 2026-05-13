---
name: pr-merged
description: "After a PR merge: switch to master, pull, delete the merged branch's local progress files, and delete the local PR branch."
disable-model-invocation: true
allowed-tools: Bash(git checkout master) Bash(git pull) Bash(git pull *) Bash(git branch -d *) Bash(git branch --show-current) Bash(git status) Bash(git status --porcelain) Bash(gh pr list *) Bash(grep -l *) Bash(basename *) Bash(rm -f *) Bash(rmdir *)
---

Current branch: !`git branch --show-current`

Working tree:
```!
git status --porcelain
```

If the current branch is `master`, stop and tell the user this skill must be run while standing on the merged PR branch.

If `git status --porcelain` above is non-empty (any modified, staged, or untracked entries), stop and ask the user how to proceed (commit, stash, discard, ignore). Do not run any further commands until the user answers.

Otherwise, run in order. **Capture `<previous-branch>` from the `Current branch:` value above before step 1** — once `git checkout master` runs, `git branch --show-current` no longer returns it.

1. `git checkout master`
2. `git pull`
3. **Delete the merged branch's local progress files** (gitignored agent artefacts; no longer needed). Use the PR-linkage path: find the merged PR for `<previous-branch>`, locate the spec by its `Tracked in: #<N>` marker, derive the progress-file base name from the spec filename.

   ```bash
   PREV_BRANCH=<previous-branch>
   PR_NUM=$(gh pr list --state merged --head "$PREV_BRANCH" --json number --jq '.[0].number // empty')
   if [ -n "$PR_NUM" ]; then
     SPEC_PATH=$(grep -l "Tracked in:.*#${PR_NUM}\b" ai-docs/plans/done/*.spec.md ai-docs/plans/*.spec.md 2>/dev/null | head -n1)
     if [ -n "$SPEC_PATH" ]; then
       SPEC_BASE=$(basename "$SPEC_PATH" .spec.md)
       rm -f "ai-docs/plans/${SPEC_BASE}.progress.md"
       rm -f "ai-docs/plans/deferred/${SPEC_BASE}.progress.md"
     fi
     rm -f "ai-docs/pr-comments/pr-${PR_NUM}.progress.md"
   fi
   rmdir ai-docs/pr-comments 2>/dev/null || true
   ```

   If `PR_NUM` is empty (branch was merged outside `gh`, or PR is closed-not-merged): skip this step silently — there's nothing reliable to derive paths from. Surface a one-line note: `pr-merged: no merged PR found for <previous-branch>; skipping progress-file cleanup.`

4. `git branch -d <previous-branch>` — always `-d`, never `-D`. If `-d` refuses because the branch is not fully merged, stop and report the message; do not force-delete.
