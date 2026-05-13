# `.progress.md` format (canonical)

Single source of truth for the progress-file format. `/task`, `/code-review`, `/pr-commented`, and the `review-findings` / `self-review` agents all read and write it; the **required** fields below must be present in every progress file regardless of which workflow created it.

```markdown
# Progress: [task name] — ACTIVE
_Updated: YYYY-MM-DD HH:MM_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** [branch name]
**base_commit:** [git rev-parse HEAD output]
**Last build:** PASS / FAIL / not run

<!-- Optional, /task only — omit for /code-review: -->
**Issue:** [#number or URL]
**Spec:** ai-docs/plans/YYYY-MM-DD-name.spec.md

## Next action

**Do this immediately:** [one concrete sentence — file + what to do]

## Subtasks

- [x] 1. done thing
- [x] 2. done thing
- [ ] 3. current/next thing  ← CURRENT
- [ ] 4. pending

## Key discoveries (don't re-investigate)

- [finding]: [why it matters / what we decided]

## AC Status

| AC | Status |
|----|--------|
| AC1 | PASS / FAIL / NOT_TESTED |

## Files touched

- `src/foo.rs` — what changed
```

**Required fields** (read by `self-review` at handoff): `**Branch:**`, `**base_commit:**`, `**Last build:**`.
**Optional fields** (added by `/task` only): `**Issue:**`, `**Spec:**`.

## Lifecycle

- **Created by:** `/task` Step 8 (start of implementation) or `/code-review` Phase 1 (review-findings agent).
- **Extended by:** subtask updates (Step 8 per-subtask); `self-review` agent appends `## Self-Review (Round N)` sections at each round (Step 10); `/pr-commented` appends `## Comment cycle round M` sections after merge-base on reviewer comments.
- **Gitignored:** `/ai-docs/plans/**/*.progress.md` and `/ai-docs/pr-comments/` in `.gitignore`. Never committed.
- **Deleted by:** `/pr-merged` after the PR merges (uses PR-linkage to derive the spec name → progress-file path).
- **Exception:** `/code-review`'s own `ai-docs/plans/YYYY-MM-DD-code-review.progress.md` is deleted explicitly by `/code-review` SKILL on self-review APPROVE — a separate lifecycle from `/task`'s.
