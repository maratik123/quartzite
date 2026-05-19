# Workflow

This page extracts long narrative passages from [`AGENTS.md` § Workflow](../AGENTS.md#workflow). The AXIOMs and short bullet rules stay in AGENTS.md.

## PR review comment resolution

After pushing fixes, resolve only the comments that were addressed by a code change. Comments where you posted an objection (explaining why no change was made) must **not** be resolved — leave them for the reviewer to accept or push back on.

**Mechanics (GitHub stores review threads, not just comments — REST `/pulls/{N}/comments` does not expose `isResolved`; use GraphQL):**

1. Reply to each comment via `gh api repos/<OWNER>/<REPO>/pulls/<N>/comments/<comment-id>/replies -f body='...'`.
2. Query unresolved thread IDs:
   ```bash
   gh api graphql -f query='{ repository(owner:"<OWNER>", name:"<REPO>") { pullRequest(number:<N>) { reviewThreads(first:50) { nodes { id isResolved path comments(first:1) { nodes { databaseId body } } } } } } }'
   ```
   Filter to `isResolved == false` and match each thread to the comment it was opened on (via `comments.nodes[0].databaseId` or `path`). Never guess thread IDs — `NOT_FOUND` means the ID is wrong, not that resolution is unavailable.
3. Resolve each fixed thread:
   ```bash
   gh api graphql -f query='mutation { resolveReviewThread(input:{threadId:"<id>"}) { thread { isResolved } } }'
   ```
   Verify `isResolved: true` in the response.
4. Skip threads where you posted an objection — those stay open for the reviewer.

## PR body vs. tracking-issue body

The upstream tracking **issue**'s title and body are the user's original problem statement — do not rewrite them. Communicate scope changes via `gh issue comment` instead.

## Recovery from destructive-git-commands

**Never** use `git reset --hard` — it silently discards uncommitted work (working-tree changes, untracked files). Use one of these instead:

- `git reset --soft HEAD~N` — preserves working tree; commits become staged
- `git stash` — saves uncommitted changes before switching branches
- `git cherry-pick` — moves specific commits to another branch
- Backup branch — `git checkout -b backup/...` before any destructive operation

## Self-review checklist for CI-fix commits

Any code change made in response to a CI failure — even a one-liner in test code — is subject to the same self-review requirement as the original implementation commits (parent rule lives in [`AGENTS.md` § Workflow](../AGENTS.md#workflow) under the `**CI-fix commits get self-review too.**` bullet). Before pushing a CI-fix commit, spawn the `self-review` agent; at minimum, run an inline review covering:

- correct idiom used
- no semantics broken
- no adjacent lint issues missed
- commit message accurate

The `/task` Step 10 self-review loop applies to every code-producing commit on the branch, not just the initial implementation batch.

## "Too simple" step-skip recurrences

The `**No "too simple" step-skip in /task.**` rule (Steps 6 / 7 / 10 are mandatory regardless of diff size) has recurred 2026-05-07 / 2026-05-14 / 2026-05-16. The recurrence record is kept here so the parent rule in AGENTS.md stays a one-line directive.
