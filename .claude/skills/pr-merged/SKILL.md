---
name: pr-merged
description: "After a PR merge: switch to master, pull, delete the merged branch's local progress files, and delete the local PR branch."
disable-model-invocation: true
allowed-tools: Bash(git checkout master) Bash(git pull) Bash(git pull *) Bash(git branch -d *) Bash(git branch --show-current) Bash(git status) Bash(git status --porcelain) Bash(.claude/skills/pr-merged/scripts/cleanup-progress.sh *)
---

> Near-stateless: no `.progress.md` discipline applies; re-entry consists of re-invoking the skill.

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
3. **Delete the merged branch's local progress files** (gitignored agent artefacts; no longer needed). Run the cleanup script:

   ```bash
   ${CLAUDE_SKILL_DIR}/scripts/cleanup-progress.sh <previous-branch>
   ```

   The script encapsulates the PR-linkage derivation (PR number → spec lookup → progress-file paths) and handles the failure modes:
   - **`PR_NUM` empty** (branch merged outside `gh`, or PR is closed-not-merged): prints `pr-merged: no merged PR found for <previous-branch>; skipping progress-file cleanup.` and exits 0.
   - **No matching `/task` spec** (manual PR without `/task`): skips the `/task`-progress-file deletion silently; the `/pr-comments` fallback path is still attempted.
   - **`rmdir` on `ai-docs/pr-comments`** is opportunistic — non-fatal if the directory has unrelated files or doesn't exist; exit code ignored.

   Deferred-task progress files (`ai-docs/plans/deferred/`) are intentionally NOT touched — deferral is its own workflow and a deferred task has no merged PR to drive cleanup. Source: [`scripts/cleanup-progress.sh`](scripts/cleanup-progress.sh).

4. `git branch -d <previous-branch>` — always `-d`, never `-D`. If `-d` refuses because the branch is not fully merged, stop and report the message; do not force-delete.
