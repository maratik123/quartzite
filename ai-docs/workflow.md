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

## Staging learnings.md during PR commits

**Before every `git commit` during a PR task**, check `git status` for `ai-docs/learnings.md`. If it appears modified or untracked, stage it together with the related code changes — learnings are part of the task deliverable and must be visible in the PR diff.

**After every push** (CI fix, reviewer-comment fix, self-review fix): if a learning entry was written *after* the last code commit landed, give it its own commit on the feature branch in the same turn — do not leave `learnings.md` as an unstaged working-tree change waiting to be bundled with the next code change.

Order: write learning → `git add ai-docs/learnings.md` → commit → push.

## No --no-verify

**Never** use `git commit --no-verify` (or any other hook-skipping flag). If a hook fails, fix the underlying issue.

## TDD + lint-changed-files

Plan first. Tests before prod code (TDD). Lint changed files.

## #[cfg(test)] requirement for substantial logic

Any file with substantial logic (~50+ lines of non-trivial code) must have a `#[cfg(test)] mod tests` block. No exceptions for generator, codegen, or utility files. **Exceptions:**

- Files under `examples/` are runnable demos, not library code — no `#[cfg(test)]` block required.
- Files under `benches/` declared with `[[bench]] harness = false` (criterion bench binaries) — `criterion_main!` replaces the test runner, so `#[cfg(test)]` items would never be invoked. No `#[cfg(test)]` block required.

## Markdown link tracing after generate/move

After generating or moving any markdown file with relative links to siblings (`../`, `../../`), trace at least one link by hand or with `realpath` before committing. From `ai-docs/deferred/file.md`: `..` reaches `ai-docs/`, `../..` reaches the repo root.

## Merge strategy

Merge PRs with a merge commit (`gh pr merge --merge`). Never squash or rebase-merge.

## Cargo.lock refresh before commit

Run `cargo build` before committing so `Cargo.lock` is refreshed and included in the commit when it changes.

## Explicit-file staging

Stage files explicitly by name. **Never** use `git add -A` or `git add .` — they pull in unintended files (IDE state, accidental scratch files).

## CI-fix commit self-review (parent rule)

**CI-fix commits get self-review too.** Any code change made in response to a CI failure — even a one-liner in test code — is subject to the same self-review requirement as the original implementation commits. Before pushing a CI-fix commit, spawn the `self-review` agent. The `/task` Step 10 self-review loop applies to every code-producing commit on the branch, not just the initial implementation batch. The inline-review checklist lives in [§ Self-review checklist for CI-fix commits](#self-review-checklist-for-ci-fix-commits).

## "Too simple" step-skip rule (parent rule)

**No "too simple" step-skip in `/task`.** Steps 6 (design), 7 (design-review), 10 (self-review) are mandatory regardless of diff size. `/task` Step 12 sub-step 1 enforces mechanically via `**current_step:**` in the progress file; explicit user authorisation is the only bypass. Recurrence-date log lives in [§ "Too simple" step-skip recurrences](#too-simple-step-skip-recurrences).

## Spec-Amendment group

Extracted from the AGENTS.md § Propagation Rule *Spec-Amendment group* row so the parent table cell can stay short. The Spec-Amendment / Design-Amendment recipe lives in `.claude/skills/task/SKILL.md` Steps 7 / 11; every downstream "fix" skill (`/pr-commented`, `/pr-ci-failed`, `/master-ci-failed`) and the `self-review` subagent carry the same recipe.

**Mechanical detection trigger:** *"the fix commit's diff includes a `.spec.md` OR `.design.md` file under `ai-docs/plans/` (active or `done/`)"*. When that trigger fires, the change is an Amendment, not an ordinary finding — the spec/design must round-trip back through `design` + `design-review` before commit.

**Recurrence record:** the trigger recurred in 2026-05-21 inside the `/task` Step 11 self-review fix flow — a fix commit touched `ai-docs/plans/*.spec.md` while in a non-Amendment branch of the workflow. The 2026-05-21 recurrence is the canonical reference for why each downstream skill's fix step explicitly re-checks the trigger before commit.

**Sync-group sister files** (must co-evolve when the recipe changes): `.claude/skills/task/SKILL.md`, `.claude/skills/pr-commented/SKILL.md`, `.claude/skills/pr-ci-failed/SKILL.md`, `.claude/skills/master-ci-failed/SKILL.md`, `.claude/agents/self-review.md`.
