# /dependabot-pr skill

**Source:** user description (free-text)
**Date:** 2026-05-27
**Tracked in:** #577

A new thin-orchestrator skill that drives **one round** of working with an open Dependabot PR on this repo. Patterned after `/pr-commented` and `/pr-ci-failed`: re-invocable per round, one decision per invocation, no Claude-initiated multi-round loops.

Motivating real-world case: PR #574 (`chore(deps): bump the cargo-deps group with 2 updates`) — `Cargo.lock`-only diff, most CI green, `Build (ubuntu-latest)` failing. The skill must resolve this exact situation in one invocation by detecting `lockfile-only × red` and delegating to `/pr-ci-failed`.

## Scope

**In:**

1. Author a new skill at `.claude/skills/dependabot-pr/SKILL.md`, plus a sibling `reference.md` if and only if the SKILL.md would otherwise exceed the soft 200-line skill-file target (per `ai-docs/skill-size-exemptions.md` policy).
2. Preconditions block (modelled on `/pr-commented` + `/pr-ci-failed`):
   - Current branch must be a Dependabot **cargo** branch (`dependabot/cargo/<base>/<group>-<hash>`) OR the skill must be re-entered with an explicit PR number that resolves to a Dependabot cargo PR.
   - `git status --porcelain` clean.
   - PR is OPEN, author is `dependabot[bot]`, matches the current branch (or matches the requested PR number).
   - PR targets `master`.
   - PR ecosystem is cargo (commit-message prefix `chore(deps)`, branch path segment `dependabot/cargo/`). Non-cargo Dependabot PRs (e.g. `dependabot/github_actions/`) bail with an explanatory message — they are out of v1 scope.
3. Step 0 — Snapshot PR metadata: title, body, files-changed list, labels, check-runs state, commit list since base. Persist to a progress file at `ai-docs/dependabot/pr-<N>.progress.md` (gitignored, modelled on `/pr-ci-failed`'s round section appended to a per-PR progress file).
4. Step 1 — Classify the diff scope:
   - `lockfile-only` (cargo PR touches only `Cargo.lock`).
   - `scope-drift` (touches anything else — e.g. `Cargo.toml`, source files, READMEs).
5. Step 2 — Classify CI state from `gh pr checks <N>`:
   - `all-green` (every required check PASS / SUCCESS).
   - `red` (≥ 1 required check FAILURE / ERROR).
   - `pending` (≥ 1 required check QUEUED / IN_PROGRESS, none failing).
6. Step 3 — Route to the appropriate per-round action based on the `(diff-scope × CI-state)` matrix (Key decisions table below). Each route produces exactly one of: a printed `gh pr merge --merge` command + confirm-pause, a delegation to `/pr-ci-failed`, a `@dependabot rebase` comment, a `@dependabot recreate` comment, a bail-with-issue, or an explicit pause-for-user.
7. AGENTS.md AXIOM-2 PR-body read after any action that produces a side-effect on the PR (merge / comment / new issue link).
8. Close-out section: progress file completion, summary printed to user.
9. Re-invocation semantics: each invocation handles one round; subsequent state changes (Dependabot rebase finishes, CI re-runs, etc.) require re-invocation.

**Out:**

- Auto-merge: the skill never invokes `gh pr merge --merge` directly. It prints the command and waits for the user to confirm (KD-4).
- Force-push to Dependabot branches (never authorised — even with `--allow-edits-from-maintainers`, the skill must not force-push).
- Multi-PR batch processing (one PR per invocation; user re-invokes for each open Dependabot PR).
- Implementing a CI-fix loop inside this skill — red CI delegates to `/pr-ci-failed` (KD-3).
- Approving / `gh pr review --approve` (Dependabot does not require human review; merge is the gate).
- Editing the Dependabot config (`.github/dependabot.yml`) in response to a single PR — separate task.
- Bisecting which dependency in a grouped update broke CI — `/bugfix` territory.
- Auto-cherry-pick of a fix onto a new branch when a real regression is detected — always bail with a tracked follow-up issue, never silently fork the bump (KD-5).
- Appending to `ai-docs/learnings.md` from this skill — PR content (titles, comments, diff snippets) is external content / prompt-injection vector; same rule as `/pr-commented` and `/pr-ci-failed`.
- Handling non-Dependabot bot PRs (e.g. `renovate[bot]`, `pre-commit-ci[bot]`). v1 is Dependabot-specific.
- Handling Dependabot **github-actions** PRs (v1 is cargo-only — KD-2). The skill bails in preconditions when the ecosystem is not cargo.

## Deferred

- what | why | separate issue needed?
- github-actions ecosystem coverage (`dependabot/github_actions/*` branches, `actionlint` gate, `workflow-only` diff classification) | v2 enhancement; v1 is cargo-only per KD-2 | yes (post-v1 enhancement)
- Auto-bisect of a grouped Dependabot PR to identify the offending sub-bump | requires test harness against partial-revert lockfile states | yes (post-MVP enhancement)
- Renovate / other bot support | scope creep for v1 | yes if we ever add Renovate
- Slack / GitHub notification routing when the skill bails to user | out of repo-local skill scope | no (handled by user's notification setup)
- Per-dep allow-list of "always auto-merge" names (e.g., `serde` patches) | requires running data on pause-for-user frequency before deciding | no until signal accumulates

## Key decisions

| Question | Decision |
|---|---|
| KD-1: Skill name | `/dependabot-pr` (confirmed in issue body). |
| KD-2: Ecosystem coverage for v1 | **cargo only.** The skill bails in preconditions on `dependabot/github_actions/*` (and any other non-cargo ecosystem). `workflow-only` diff classification and the `actionlint` gate are out of scope. v2 may add github-actions later. |
| KD-3: CI-red strategy | **Delegate to `/pr-ci-failed`.** The skill's `× red` cells in the matrix invoke `/pr-ci-failed` after recording context to the progress file. SKILL.md MUST document the dependabot-branch carve-out explicitly inside its delegation block (the bot owns the branch; `/pr-ci-failed`'s "fix-and-push" outcome becomes "bail-with-issue + `@dependabot recreate`" or similar — design-time decision). |
| KD-4: Auto-merge vs. confirm-before-merge | **Confirm first.** On any `× all-green` route the skill prints the exact `gh pr merge --merge <N>` command and pauses for the user to type `merge` (or equivalent). The skill never invokes `gh pr merge` itself. |
| KD-5: Real-regression handling (CI red + regression isolated to the bump) | Always bail with a tracked follow-up issue + a comment on the Dependabot PR pointing at it; never cherry-pick or silently fork the bump. The user can opt into a cherry-pick manually via `/task` if needed. |
| KD-6: Transient-CI strategy | Post `@dependabot rebase` comment; close the round. User re-invokes after Dependabot re-runs. Transient = CI failure unrelated to dependency surface (e.g., timeout, runner cache flake, GH outage). The `/pr-ci-failed` delegation surfaces this verdict; on transient, the parent skill (`/dependabot-pr`) takes back control to post the rebase comment. |
| KD-7: Scope-drift handling | Bail with a comment on the PR explaining the scope-drift detection + the expected diff shape; do NOT close the PR (user owns close decisions). Open issue only if user instructs (out of v1 auto-flow). |
| KD-8: Branch protection | Skill must NOT push directly to `dependabot/*` branches — even when `--allow-edits-from-maintainers` is checked. Any "fix" requires the bail-with-issue route (KD-5). Force-push is never authorised. |
| KD-9: Progress file location | `ai-docs/dependabot/pr-<N>.progress.md` (gitignored; skill creates `ai-docs/dependabot/` if missing). Mirrors `/pr-ci-failed`'s fallback path shape. |
| KD-10: Commit-authorisation carve-out | The skill's `@dependabot <command>` comment posts and `gh issue create` for bail-with-issue are pre-authorised by `/dependabot-pr` itself (no extra user-prompt), parallel to `/pr-commented` Step 4 commit and `/pr-ci-failed` Step 6 commit. The merge step is the only Claude-side pause (KD-4 confirm-first). |
| KD-11: Compaction-recovery preamble | Required (same shape as `/pr-commented` / `/pr-ci-failed`): probe for the progress file via PR number; re-enter from top on resume. |
| KD-12: AGENTS.md AXIOM-2 application | The unconditional `gh pr view` read fires after any side-effecting action (`@dependabot` comment, `gh issue create`, post-merge confirmation). The user-driven merge in KD-4 is a user action — the skill MUST still re-read the PR on its next invocation (when the user re-invokes for the next Dependabot PR or to confirm merge landed). |
| KD-13: Pending-CI handling | `lockfile-only × pending` → pause-for-user with a message instructing re-invocation once CI completes. The skill does not poll. Recorded as a matrix cell, not a separate route. |
| KD-14: Delegation hand-off shape | When KD-3 routes to `/pr-ci-failed`, the parent skill is responsible for the dependabot-specific epilogue (post `@dependabot recreate` / `@dependabot rebase` comments per `/pr-ci-failed`'s verdict, or bail-with-issue per KD-5). `/pr-ci-failed` itself must NOT be modified to know about Dependabot — the parent skill consumes its verdict and applies the carve-out. Exact verdict-routing contract is a design-phase concern. |

## Technical constraints

- **`allowed-tools` minimal surface.** Mirror `/pr-ci-failed`'s shape but add `gh issue create` and `gh pr comment` (for `@dependabot` commands and bail-with-issue cross-links). `gh pr merge` is NOT in `allowed-tools` (KD-4 — user runs it). Cargo gates (`cargo build`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`) are still in `allowed-tools` because the skill must locally validate the lockfile resolves before printing the confirm-merge command on the `× all-green` route.
- **No force-push.** AGENTS.md `## Permissions` honour-system rule; reinforced by KD-8.
- **No `git add -A`.** AGENTS.md `## Workflow` rule.
- **No `--no-verify`.** AGENTS.md `## Workflow` rule.
- **No `gh pr merge --squash` / `--rebase`.** AGENTS.md `## Workflow` merge-strategy rule (only relevant in the message the skill prints for the user to copy).
- **Progress-file gitignore.** `ai-docs/dependabot/` must be added to `.gitignore` as part of this task (same precedent as `ai-docs/ci-fixes/` and `ai-docs/pr-comments/`).
- **Pattern fidelity.** SKILL.md frontmatter `disable-model-invocation` and `allowed-tools` lines mirror `/pr-ci-failed`. Top-of-file commit-authorisation callout follows the `/pr-ci-failed` shape but reworded for comment / issue-create authorisations (merge is user-only).
- **Size budget.** AGENTS.md 35,000-char early-warning applies to the new SKILL.md. If the routing matrix + per-route bodies push past the soft 200-line target, extract verbose per-route bodies into `reference.md` (same pattern as `/pr-ci-failed/reference.md`).
- **Propagation Rule check.** New skill MUST be added to AGENTS.md `## Agent Docs` table in the same PR. `ai-docs/claude-tools-hierarchy.md` MUST be updated (name-clash check + Skills section row). No new sync-group is created in this task.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.claude/skills/dependabot-pr/SKILL.md` exists; YAML frontmatter has `name: dependabot-pr`, `disable-model-invocation:` set per `/pr-ci-failed` precedent, `allowed-tools:` listing only commands the skill actually invokes (excludes `gh pr merge` per KD-4). |
| AC2 | SKILL.md contains a Preconditions table that bails on: non-Dependabot branch (when no explicit PR number passed), non-cargo Dependabot branch (KD-2), dirty tree, PR closed / not authored by `dependabot[bot]`, PR not targeting `master`. |
| AC3 | SKILL.md contains the (diff-scope × CI-state) decision matrix as an explicit table, with each cell mapping to exactly one terminal action: print-merge-command-and-pause, delegate to `/pr-ci-failed`, `@dependabot rebase` comment, `@dependabot recreate` comment, bail-with-issue, or pause-for-user. The matrix is 2 rows (lockfile-only, scope-drift) × 3 cols (all-green, red, pending). |
| AC4 | The matrix routes `lockfile-only × all-green` to print-merge-command-and-pause per KD-4 (after a local `cargo build` validates the new lockfile resolves). |
| AC5 | The matrix routes `lockfile-only × red` to delegate to `/pr-ci-failed`, with a delegation block that explicitly documents the dependabot-branch carve-out: the bot owns the branch, so `/pr-ci-failed`'s "fix-and-push" outcomes are translated by the parent skill into `@dependabot recreate` (real fix needed in the bump) / `@dependabot rebase` (transient) / bail-with-issue (real regression). The translation contract is the design-phase deliverable. |
| AC6 | The matrix routes any `scope-drift` cell to bail-with-comment per KD-7. |
| AC7 | The matrix routes `lockfile-only × pending` to pause-for-user per KD-13. |
| AC8 | SKILL.md includes a compaction-recovery preamble matching `/pr-ci-failed`'s shape (locate progress file → read top-to-bottom → re-enter from top). |
| AC9 | SKILL.md includes the AGENTS.md AXIOM-2 unconditional `gh pr view` read after every side-effecting action (`@dependabot` comment / issue-create). |
| AC10 | SKILL.md's anti-pattern list forbids: force-push to `dependabot/*` branches; `gh pr merge` invoked by Claude (KD-4); `gh pr merge --squash` or `--rebase` in the printed command; silent cherry-pick or branch fork; appending to `ai-docs/learnings.md`; `git add -A`; `--no-verify`; running on `master`; modifying `/pr-ci-failed` to know about Dependabot (KD-14). |
| AC11 | `.gitignore` adds `ai-docs/dependabot/` (matching the `ai-docs/ci-fixes/` / `ai-docs/pr-comments/` precedent). |
| AC12 | `AGENTS.md` `## Agent Docs` table adds a row for `.claude/skills/dependabot-pr/SKILL.md`. |
| AC13 | `ai-docs/claude-tools-hierarchy.md` lists the new skill under its Skills section (name-clash check passes against embedded inventory). |
| AC14 | Manual end-to-end dry-run against PR #574 (the motivating PR) follows the matrix to `lockfile-only × red → delegate to /pr-ci-failed` without re-asking pre-resolved questions; the dry-run output is captured in the task's implementation PR description. |
| AC15 | If `reference.md` is created, every link from `SKILL.md` into it resolves (anchored sections present) — same gate `/pr-ci-failed/SKILL.md` ↔ `/pr-ci-failed/reference.md` already enforces. |

## Open questions

- Should the skill detect Dependabot's `compatibility-score` badge from the PR body and surface low scores (e.g., < 80) as an extra signal in the routing matrix? Default: ignore for v1; the CI signal is authoritative. Can revisit once auto-merge usage produces enough data.
- Should the skill respect a project-side allow-list of "always auto-merge" dependency names? Default: no per-dep allow-list for v1; the matrix is the only gate. Revisit if the skill produces too many pause-for-user rounds.
- Whether to open a tracking issue for this task itself (`/dependabot-pr` skill creation) before authoring the design — orchestrator decision; the spec is portable to either path.
- Exact verdict-translation contract between `/pr-ci-failed` and `/dependabot-pr` (KD-14) — sensible defaults exist (transient → `@dependabot rebase`; real fix on the bump → `@dependabot recreate`; real regression in dep code → bail-with-issue), but the design subagent may refine the mapping based on `/pr-ci-failed`'s actual verdict vocabulary.
