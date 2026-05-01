---
name: task-issue
description: "Full ticket workflow: ticket → interview → spec → design → design-review → impl → verify → self-review. Steps are strictly ordered and cannot be skipped."
disable-model-invocation: true
argument-hint: "[issue-number]"
allowed-tools: Bash(gh issue view *) Bash(cargo build) Bash(cargo test *) Bash(cargo clippy *) Bash(cargo fmt *) Bash(git diff *) Bash(git rev-parse *)
---

Full workflow for working on a task (issue). Steps execute **strictly in sequence** — proceeding to N+1 before N is complete is FORBIDDEN.

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

### Step 1: Get the issue

`gh issue view $ARGUMENTS` — read description, comments, linked issues. Read EVERYTHING.

### Step 2: Extract scope

Every requirement = a separate numbered item. **Dropping an item without confirmation — FORBIDDEN.**

### Step 3: Confirm scope

Show scope list. For each: in scope / out of scope / deferred. Max 3 questions at once.

### Step 4: Confirm key decisions

Any ambiguity or approach choice → ask the product owner. **Silently substituting your own decision — FORBIDDEN.**

Red flags (STOP and ask):
- Interpreting ambiguity two different ways
- Choosing between technically equivalent approaches
- Adding something not in the issue
- Treating an item as "unimportant"

### Step 5: Save spec + acceptance criteria

Create `ai-docs/plans/YYYY-MM-DD-name.spec.md` with `## Acceptance Criteria` (required).

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

- Create `ai-docs/plans/YYYY-MM-DD-name.progress.md` at start (see `/context-reset` for format)
- **Record base commit** in the progress file immediately:
  ```
  base_commit: <output of `git rev-parse HEAD`>
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

5. On ALL PASS → proceed to Step 10

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

**On APPROVE:** delete `.progress.md` → done.

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

**FORBIDDEN:** declaring done with uncovered ACs · skipping design review · writing code before confirmed spec · deleting `.progress.md` before self-review APPROVE

## Gate checklist

| Before | Check |
|---|---|
| Step 3 | All issue items extracted? |
| Step 4 | Scope confirmed? |
| Step 5 | All decisions confirmed? Every AC verifiable? |
| Step 6 | Spec saved? ACs confirmed? |
| Step 8 | Design doc with GO? Test Design section present? |
| Step 8 start | `base_commit` recorded in progress file? |
| Each subtask | `cargo build` ✅? Tests run? `.progress.md` updated? |
| Step 9 | `cargo test` green? clippy clean? All ACs covered? |
| Step 10 | Self-review APPROVE before deleting progress file? |
| Step 11 | `major`/`blocker` objections confirmed by user? |
