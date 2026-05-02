# Learnings

### 2026-05-02 — code-style — let chains are allowed and formattable in edition 2024

**What happened:** During the macros task, rustfmt errored on a let chain with "let chains are only allowed in Rust 2024 or later". The workspace uses `edition = "2024"`, Rust 1.95, and rustfmt 1.9.0 — all of which support let chains. The error was caused by running rustfmt without `--edition 2024` explicitly, or against a stale binary. The response was to replace let chains wholesale with match expressions as a blanket rule.

**Rule:** Let chains (`if let A = x && let B = y { ... }`) are valid in this codebase. Do not avoid them. Run `rustfmt` via `cargo fmt` (which picks up the workspace edition automatically) rather than invoking `rustfmt <file>` directly.

**Escalated?** AGENTS.md

### 2026-05-02 — process — do not touch IDE files unless explicitly asked

**What happened:** `.idea/quartzite.iml` had an uncommitted modification. Without being asked, it was added to `.gitignore` and removed from tracking.

**Rule:** Never add, remove, modify, or `.gitignore` IDE files (`.idea/`, `*.iml`, `.vscode/`, etc.) unless the user explicitly asks. Treat them as the user's domain.

**Escalated?** AGENTS.md, memory

### 2026-05-02 — process — "submit to PR" means push to remote, not merge

**What happened:** User said "submit to pr". Interpreted as merging the PR via `gh pr merge`. User meant pushing the local commits to the remote branch so they appear in the open PR.

**Rule:** "Submit to PR" (and similar: "push to PR", "add to PR") means `git push` the branch to remote. It does not mean merging. Only merge when the user explicitly says "merge" or "merge the PR".

**Escalated?** AGENTS.md, memory

### 2026-05-02 — process — "wtf" signals that the previous action was wrong

**What happened:** User said "add ide files". Interpreted as adding IDE files to `.gitignore`. User meant commit and track them. User responded "wtf?" to signal the action was wrong.

**Rule:** "wtf" (and similar expressions of surprise/frustration) means the last action was the opposite of what the user wanted. Stop immediately, ask what went wrong, and do not proceed until the intent is clarified.

**Escalated?** AGENTS.md, memory

### 2026-05-02 — process — never use git reset --hard; use soft reset, stash, cherry-pick, or backup branch

**What happened:** `git reset --hard origin/master` was used to move commits off local master to a feature branch. This discarded uncommitted changes to `ai-docs/learnings.md` that had not been staged.

**Rule:** Never use `git reset --hard`. Use one of these instead:
- `git reset --soft HEAD~N` — moves commits back to staged, preserves working tree
- `git stash` — saves uncommitted changes before switching branches
- `git cherry-pick` — moves specific commits to another branch
- Backup branch — `git checkout -b backup/...` before any destructive operation

**Escalated?** memory

### 2026-05-02 — process — always create a feature branch before committing; never commit directly to master

**What happened:** When the user said "submit PR", commits were already on local master. Instead of creating a feature branch first, `git push` was run directly against master — pushing the commits to origin/master. `master` is branch-protected (no force push), so the commits could not be removed after the fact and a proper PR became impossible.

**Rule:** When work is intended for a PR, create a feature branch (`git checkout -b feat/...`) *before* making any commits. Never commit to local master with the intention of later turning it into a PR.

Recovery — if commits were accidentally made on local master and not yet pushed (full procedure in `AGENTS.md`):
1. `git stash` — save any uncommitted changes
2. `git checkout -b feat/...` — branch off from current HEAD (carries the commits)
3. `git checkout master && git reset --soft origin/master && git restore --staged .` — soft-rewind local master to remote state without discarding work
4. Push the feature branch and open the PR from it; pop the stash on the feature branch if needed
5. **Never push master** — not even as an intermediate step. **Never use `git reset --hard`** — see the dedicated rule above.

Before any `git push`: run `git branch --show-current` and confirm it is **not** `master`. If it is master — stop, do not push, apply the recovery procedure above.

If "submit PR" is requested and commits are already pushed to origin/master: stop and tell the user — there is no recovery without a force push, which branch protection may block.

**Escalated?** AGENTS.md

### 2026-05-02 — process — create feature branch before committing at the start of Step 8

**What happened:** The auto-connection task completed all implementation steps on the working tree without ever committing. Only after the user asked "why didn't you create a PR?" was the branch created. The changes had to be recovered by checking out a feature branch from the unstaged state.

**Rule:** At the start of Step 8 (Implementation), immediately create a feature branch (`git checkout -b feat/...`) before writing any code. Record the branch name in the progress file. Do not wait until after self-review to create the branch.

**Escalated?** skill:task, skill:task-issue, hook (PreToolUse on `git commit` blocks master)

### 2026-05-02 — testing — any sufficiently large file requires unit tests

**What happened:** Three codegen files (`object/codegen.rs`, `object_impl/codegen.rs`, `meta_enum/codegen.rs`) were written without `#[cfg(test)]` modules. Gaps were caught in review and by the user. The original rule was codegen-specific, but the user generalised it: any file with substantial logic needs tests.

**Rule:** Any file with ~50+ lines of non-trivial code must have a `#[cfg(test)] mod tests` block. This applies equally to codegen, parse, util, and any other module — not just files named `codegen.rs`.

**Escalated?** AGENTS.md

### 2026-05-02 — process — propagate skill/agent fixes to all related files in the same operation

**What happened:** A fix to `self-review.md` was applied in isolation; `codebase-review.md` was only updated after the user pointed it out. Similarly, `/task` and `/task-issue` fixes were done together, but the code-review family was handled piecemeal.

**Rule:** When fixing a skill or agent, immediately propagate the change to all files in the same sync group before reporting done. Two sync groups:
- **Task group:** `skills/task/SKILL.md` ↔ `skills/task-issue/SKILL.md`
- **Review group:** `skills/code-review/SKILL.md` (workflow orchestrator) ↔ `agents/review-findings.md` (findings producer) ↔ `agents/self-review.md` (fix validator)

Note: `code-review` is a **skill** (user-facing workflow); `review-findings` and `self-review` are **agents** spawned by it. Do not refer to any of these as "code-review agent" — that conflates the skill with an agent. (A `diff-review` agent existed historically but was removed as orphan; do not reintroduce it without wiring it into a skill.)

**Escalated?** memory

### 2026-05-02 — process — self-review must not re-run cargo fmt or cargo clippy

**What happened:** The self-review agent checked `cargo fmt -- --check` and raised REJECT findings for formatting drift, even though both `cargo fmt` and `cargo clippy -- -D warnings` are already mandated after every subtask during Implementation (Step 8 of /task and /task-issue). This caused a spurious round-trip.

**Rule:** Self-review must not run or re-check `cargo fmt`, `cargo clippy`, `cargo build`/`check`, or `cargo test`. These are all guaranteed by the Implementation and Verify steps before self-review runs. Self-review scope: spec conformance, design conformance, test coverage, safety/correctness, style (Rust file conventions, allow attributes) — not build tooling.

**Escalated?** agent:self-review, agent:review-findings, skill:task, skill:task-issue, skill:code-review

### 2026-05-02 — process — propagate rule exemptions to agent/skill files in same task

**What happened:** When adding `quartzite-examples` exemptions to `AGENTS.md` (no `#![deny(missing_docs)]`, no `#[cfg(test)]`), the corresponding checks in `.claude/agents/self-review.md` and `.claude/agents/review-findings.md` were not updated. Future reviews would have incorrectly flagged the examples crate.

**Rule:** When a rule exemption is added to `AGENTS.md`, immediately propagate it to every agent/skill/settings file that enforces that rule. Check with `grep` across `.claude/agents/` and `.claude/skills/` before closing the task.

**Escalated?** no

### 2026-05-02 — process — check current branch before committing, not only before pushing

**What happened:** For the `docs/learnings-and-skill-fix` branch, commits were made directly to local master without checking `git branch --show-current` first. The error was caught at push time (branch protection rejected the push), not at commit time. The rule in AGENTS.md mentions checking before `git push`, but the correct mental model is: verify branch before `git commit`.

**Rule:** Run `git branch --show-current` and confirm it is **not** `master` before any `git commit` that is intended for a PR. A pre-push check is a last resort, not the primary safeguard. The commit should never happen on master — the push check only exists as a final gate.

**Escalated?** hook, skill:task, skill:task-issue

### 2026-05-02 — process — run cargo fmt --all after every code change, including post-self-review fixes

**What happened:** `cargo fmt --all -- --check` was run once during Step 9 (Verify). A self-review finding then triggered a code fix (Step 11). The fix was committed and pushed without re-running `cargo fmt --all`. CI failed on the formatting drift introduced by that fix.

**Rule:** Run `cargo fmt --all` (and re-check with `cargo fmt --all -- --check`) after *every* code change — including fixes made after self-review. The verify step (Step 9) is not a one-time gate; it must be re-run after any subsequent edit before committing. Never commit without a clean `cargo fmt --all -- --check` immediately before the commit.

**Escalated?** hook
