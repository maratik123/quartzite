---
name: pr-merged
description: "After a PR merge: switch to master, pull, and delete the local PR branch."
disable-model-invocation: true
allowed-tools: Bash(git checkout master) Bash(git pull) Bash(git pull *) Bash(git branch -d *) Bash(git branch --show-current) Bash(git status) Bash(git status --porcelain)
---

Current branch: !`git branch --show-current`

Working tree:
```!
git status --porcelain
```

If the current branch is `master`, stop and tell the user this skill must be run while standing on the merged PR branch.

If `git status --porcelain` above is non-empty (any modified, staged, or untracked entries), stop and ask the user how to proceed (commit, stash, discard, ignore). Do not run any further commands until the user answers.

Otherwise, run in order:

1. `git checkout master`
2. `git pull`
3. `git branch -d <previous-branch>` — always `-d`, never `-D`. If `-d` refuses because the branch is not fully merged, stop and report the message; do not force-delete.
