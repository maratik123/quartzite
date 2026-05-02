---
name: task
description: "Full task workflow from user description: interview → spec → design → design-review → impl → verify → self-review. Use when the user describes a task directly (no GitHub issue). Steps are strictly ordered and cannot be skipped."
disable-model-invocation: true
argument-hint: "[short task description]"
allowed-tools: Bash(cargo build) Bash(cargo test *) Bash(cargo clippy *) Bash(cargo fmt *) Bash(git diff *) Bash(git rev-parse *) Bash(git checkout *) Bash(git branch *) Bash(git add *) Bash(git commit *) Bash(git push *) Bash(gh pr create *) Bash(gh pr view *)
---

Full workflow for a user-described task. Steps execute **strictly in sequence** — proceeding to N+1 before N is complete is FORBIDDEN.

## ⚡ First: check for active task

```bash
ls ai-docs/plans/*.progress.md 2>/dev/null
```

**If found → RESUME (skip steps 1–7):**
1. Read the `.progress.md` file
2. Read spec — only `## Acceptance Criteria`
3. Read only files from `## Files touched`
4. Jump to `## Next action`
5. Tell user: "Found active task [X], resuming from [subtask Y]"

---

## ⚡ Second: check for deferred plan activation

If `$ARGUMENTS` contains words like "activate", "start", "proceed" **and** a matching plan exists in `ai-docs/plans/deferred/`:

1. Identify the matching `*.spec.md` (and `*.design.md` if present) in `ai-docs/plans/deferred/`.
2. Move them to `ai-docs/plans/`:
   ```bash
   mv ai-docs/plans/deferred/YYYY-MM-DD-name.spec.md ai-docs/plans/
   mv ai-docs/plans/deferred/YYYY-MM-DD-name.design.md ai-docs/plans/     # if exists
   mv ai-docs/plans/deferred/YYYY-MM-DD-name.progress.md ai-docs/plans/   # if exists
   ```
3. Update `ai-docs/plans/INDEX.md`: move the plan row from the **Deferred plans** table to the **Active plans** table and mark its status as `🟢 ready` (or `🟡 spec-only` if no design).
4. Tell the user: "Activated plan [name] — moved spec (and design) to `ai-docs/plans/`."
5. If a `.progress.md` was moved: treat it as an active task — read it and resume from `## Next action` (same as the RESUME path above).
6. Otherwise (no progress file): skip Steps 1–7 and jump directly to Step 8 (spec + design already exist).

---

### Step 1: Get the task description

If `$ARGUMENTS` is provided — use it as the initial task description.
Otherwise — ask the user: "What do you want to implement or change?"

Read the description carefully. If it references code, grep the codebase to understand context before proceeding.

### Step 2: Extract scope

Every requirement = a separate numbered item. **Dropping an item without confirmation — FORBIDDEN.**

### Step 3: Confirm scope

Show scope list. For each item: in scope / out of scope / deferred. Max 3 questions at once.

### Step 4: Confirm key decisions

Any ambiguity or approach choice → ask the user. **Silently substituting your own decision — FORBIDDEN.**

Red flags (STOP and ask):
- Interpreting ambiguity two different ways
- Choosing between technically equivalent approaches
- Adding something not in the description
- Treating an item as "unimportant"

### Step 5: Save spec + acceptance criteria

Create `ai-docs/plans/YYYY-MM-DD-name.spec.md` with `## Acceptance Criteria` (required).

Spec format:

```markdown
# [Task name]

**Source:** user description
**Date:** [YYYY-MM-DD]

## Scope
## Out of scope
## Deferred
- what | why

## Key decisions
| Question | Decision |
|---|---|

## Technical constraints

## Acceptance Criteria
| # | Criterion |
|---|-----------|
| AC1 | [specific, verifiable condition] |

## Open questions
```

AC rules:
- ✅ "Function returns `Err` if input is empty"
- ✅ "Event is emitted when state transitions to Ready"
- ❌ "Test `foo_test` exists"
- ❌ "`cargo test` passes green"

One business requirement = ONE AC. Show to user for confirmation.

### Step 6: Design agent

> First action: confirm `ai-docs/plans/YYYY-MM-DD-name.spec.md` exists.

```
Task(subagent_type="general-purpose", prompt="
  Read .claude/agents/design.md and follow it.
  Spec: ai-docs/plans/YYYY-MM-DD-name.spec.md
  Research codebase, produce Design Document.
")
```

Result: `ai-docs/plans/YYYY-MM-DD-name.design.md`

### Step 7: Design review

```
Task(subagent_type="general-purpose", prompt="
  Read .claude/agents/design-review.md and follow it.
  Design: ai-docs/plans/YYYY-MM-DD-name.design.md
  Spec: ai-docs/plans/YYYY-MM-DD-name.spec.md
")
```

Verdict: GO / ITERATE / STOP. On ITERATE → back to Step 6 (max 3 rounds).

### Step 8: Implementation

> First action: verify both spec and design (with GO verdict) exist. Missing = previous steps incomplete.

- **Create a feature branch immediately** — before writing any code or making any commits:
  ```bash
  git checkout -b feat/YYYY-MM-DD-name
  ```
  Use the same date-name as the spec file. Record the branch name in the progress file.
- Create `ai-docs/plans/YYYY-MM-DD-name.progress.md` at start (see `/context-reset` for format)
- **Record base commit** in the progress file immediately:
  ```
  base_commit: <output of `git rev-parse HEAD`>
  branch: feat/YYYY-MM-DD-name
  ```
- After each subtask:
  1. `cargo build` — must compile
  2. `cargo test test_name` — if subtask adds tests
  3. `cargo fmt`; `cargo clippy -- -D warnings`
  4. Update `.progress.md`
  5. If N=3 of M≥5 → handoff via Task (see `/context-reset`)
- Unknown API → read sources → grep codebase → ask user. Don't guess.
- Bug report during impl → activate `/bugfix`, then return here.

### Step 9: Verify

1. `cargo test` — all green
2. `cargo clippy -- -D warnings` — clean
3. For each AC — confirm covered by test or manual verification
4. Show summary table:

```
| # | Criterion | Test / Verification | Status |
|---|-----------|---------------------|--------|
| AC1 | ... | tests::name | ✅ PASS |
```

5. On ALL PASS → proceed to Step 9.5

### Step 9.5: Update documentation

Update the following files to reflect the completed implementation:

1. **`ai-docs/plans/INDEX.md`**
   - Change the plan row status to `✅ implemented (N tests)`
   - Move spec/design files to `ai-docs/plans/done/` (`git mv`)
   - Update dependency tree and **Suggested next steps**

2. **`ai-docs/context.md`** — update any section touched by this task:
   - Resolve open questions that were answered during implementation
   - Update the Plans list (add ✅ to implemented crates)
   - Add new architectural decisions to the Key Decisions table

3. **`README.md`** — update the status table if a new crate was implemented

Then proceed to Step 10.

### Step 10: Self-review loop (max 3 rounds)

Spawn the self-review agent:

```
Task(subagent_type="general-purpose", prompt="
  Read .claude/agents/self-review.md and follow it.
  Spec: ai-docs/plans/YYYY-MM-DD-name.spec.md
  Design: ai-docs/plans/YYYY-MM-DD-name.design.md
  Progress: ai-docs/plans/YYYY-MM-DD-name.progress.md
  base_commit is recorded in the progress file.
")
```

**On APPROVE:** delete `.progress.md` → proceed to Step 12.

**On REJECT:** proceed to Step 11. After Step 11, loop back here.

**After round 3 with REJECT:** surface all remaining `⬜ Open` findings to the user and ask how to proceed. Do not delete `.progress.md` until resolved.

### Step 11: Review fixes

For each `⬜ Open` finding in the latest `## Self-Review (Round N)` section of the progress file:

- **Fix it** → mark `✅ Fixed` in the progress file, implement the change.
- **Object to it** (finding is wrong or intentionally out of scope):
  - `nit` / `minor`: agent may object autonomously — write reason, mark `⚠️ Objected: <reason>`.
  - `major` / `blocker`: **surface to user first** before objecting. User must approve the objection.

After all findings are resolved (`✅ Fixed` or `⚠️ Objected`):
1. `cargo build` — must compile
2. `cargo test` — all green
3. `cargo clippy -- -D warnings` — clean
4. Update `.progress.md`
5. Return to Step 10.

### Step 12: Create PR

1. Confirm `git branch --show-current` is **not** `master`. If it is — stop, do not push, tell the user, apply the AGENTS.md recovery procedure.
2. `cargo build` — ensures `Cargo.lock` is refreshed and included if changed.
3. Stage all changed files: implementation files from `## Files touched` in the progress file, plus doc files updated in Step 9.5 (`INDEX.md`, `context.md`, `README.md`, spec/design in `done/`).
4. Commit:
   ```
   feat(<crate>): <short imperative description>

   <1-3 lines: what changed and why; key ACs covered>
   N new tests; all M tests green.
   ```
5. `git push -u origin <branch>`
6. `gh pr create --title "..." --body "$(cat <<'EOF' ... EOF)"` — body must include:
   - **Summary** (bullet list of what changed)
   - **Test plan** (checklist: one line per AC, plus clippy/build)
7. Post the PR URL to the user.

**FORBIDDEN:** declaring done with uncovered ACs · skipping design review · writing code before confirmed spec · deleting `.progress.md` before self-review APPROVE · pushing from master branch

## Gate checklist

| Before | Check |
|---|---|
| Step 3 | All description items extracted? |
| Step 4 | Scope confirmed? |
| Step 5 | All decisions confirmed? Every AC verifiable? |
| Step 6 | Spec saved? ACs confirmed? |
| Step 8 | Design doc with GO? Test Design section present? |
| Step 8 start | Feature branch created (`git branch --show-current` ≠ master)? `base_commit` + `branch` recorded in progress file? |
| Each subtask | `cargo build` ✅? Tests run? `.progress.md` updated? |
| Step 9 | `cargo test` green? clippy clean? All ACs covered? |
| Step 9.5 | INDEX.md updated? spec/design moved to done/? context.md + README.md current? |
| Step 10 | Self-review APPROVE before deleting progress file? |
| Step 11 | `major`/`blocker` objections confirmed by user? |
| Step 12 | Branch ≠ master? `Cargo.lock` refreshed? PR created and URL posted? |
