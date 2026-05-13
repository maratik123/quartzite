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
