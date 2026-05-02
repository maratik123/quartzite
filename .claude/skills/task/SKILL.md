---
name: task
description: "Full task workflow from user description: interview → spec → design → design-review → impl → verify → self-review. Use when the user describes a task directly (no GitHub issue). Steps are strictly ordered and cannot be skipped."
disable-model-invocation: true
argument-hint: "[short task description]"
allowed-tools: Bash(cargo build) Bash(cargo test *) Bash(cargo clippy *) Bash(cargo fmt *) Bash(cargo doc *) Bash(git diff *) Bash(git rev-parse *) Bash(git checkout *) Bash(git branch *) Bash(git add *) Bash(git commit *) Bash(git push *) Bash(gh issue list *) Bash(gh issue view *) Bash(gh issue create *) Bash(gh pr create *) Bash(gh pr view *)
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
5. **Verify the spec carries `**Tracked in:**`** — if missing, run Step 5a (find or create tracking issue) and add the URL to the spec header before continuing.
6. If a `.progress.md` was moved: treat it as an active task — read it and resume from `## Next action` (same as the RESUME path above).
7. Otherwise (no progress file): skip Steps 1–7 and jump directly to Step 8 (spec + design already exist).

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

#### Step 5a: Find or create the tracking issue

Every spec **must** carry a `**Tracked in:**` link to an open GitHub issue.

1. Search existing open issues for matches:
   ```bash
   gh issue list --state open --search "<keyword>"
   ```
   Use 1–3 keywords from the task scope (crate names, feature names, key types).
2. **If a candidate exists** → present to the user: "I found #N: '<title>' — should this PR be tracked there?"
   - User confirms → use that issue.
   - User says no → continue to step 3.
3. **No suitable issue** → propose a new one to the user:
   - Title: matches the planned spec name (e.g. `feat(<crate>): <short description>`)
   - Body: brief background + scope items distilled from Step 2/3
   - Show the proposed title and body, ask user to confirm before running `gh issue create ...`
4. Capture the issue URL — it goes into the spec header in Step 5b.

> **Skip 5a only if the user explicitly states "no tracking issue".** Note the reason in the spec header (`**Tracked in:** none — <reason>`).

#### Step 5b: Write the spec

Create `ai-docs/plans/YYYY-MM-DD-name.spec.md` with `## Acceptance Criteria` (required).

Spec format:

```markdown
# [Task name]

**Source:** user description
**Date:** [YYYY-MM-DD]
**Tracked in:** https://github.com/<owner>/<repo>/issues/<N>

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
Agent(subagent_type="general-purpose", prompt="
  Read .claude/agents/design.md and follow it.
  Spec: ai-docs/plans/YYYY-MM-DD-name.spec.md
  Research codebase, produce Design Document.
")
```

Result: `ai-docs/plans/YYYY-MM-DD-name.design.md`

### Step 7: Design review

```
Agent(subagent_type="general-purpose", prompt="
  Read .claude/agents/design-review.md and follow it.
  Design: ai-docs/plans/YYYY-MM-DD-name.design.md
  Spec: ai-docs/plans/YYYY-MM-DD-name.spec.md
")
```

Verdict: GO / ITERATE / STOP.
- **GO** → proceed to Step 8.
- **ITERATE** → back to Step 6 (max 3 rounds total).
- **STOP** → fundamental flaw with the approach. Surface the verdict and `Issues` table to the user, do not start Step 8. Wait for direction (e.g., narrow scope, change approach, abandon).

### Design Amendment (re-entrant — triggered from Step 8 or Step 11)

If implementation (Step 8) reveals a necessary deviation from the design, **or** a self-review
finding (Step 11) requires a design change rather than a code fix:

1. **Stop** the current step immediately. Do not silently continue with the deviated approach.
2. **Surface to user:** describe what changed and why the design must be updated. Wait for approval.
3. Update `ai-docs/plans/YYYY-MM-DD-name.design.md` to reflect the new approach.
4. Re-run design review — same as Step 7 (max 3 rounds total across all design-review runs):
   ```
   Agent(subagent_type="general-purpose", prompt="
     Read .claude/agents/design-review.md and follow it.
     Design: ai-docs/plans/YYYY-MM-DD-name.design.md
     Spec: ai-docs/plans/YYYY-MM-DD-name.spec.md
     Context: design was amended during implementation / self-review — describe what changed.
   ")
   ```
5. **On GO** → resume from the step that triggered the amendment:
   - Triggered from Step 8 → resume Step 8 (continue the remaining subtasks)
   - Triggered from Step 11 → mark the finding `✅ Fixed (design amended)`, then return to Step 10
6. **On ITERATE** → fix the design and re-run design review (counts against the 3-round limit).
7. **On STOP** → surface to user; do not proceed until the design issue is resolved.

> Silently implementing a deviation without triggering Design Amendment — FORBIDDEN.

---

### Step 8: Implementation

> First action: verify both spec and design (with GO verdict) exist. Missing = previous steps incomplete.

- **Create a feature branch immediately** — before writing any code or making any commits:
  ```bash
  git checkout -b feat/YYYY-MM-DD-name
  ```
  Use the same date-name as the spec file. Record the branch name in the progress file.
- **Before every `git commit` in this step:** run `git branch --show-current` and confirm it is NOT `master`. If it is — stop immediately, do not commit, apply the recovery procedure in AGENTS.md.
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
  5. If N=3 of M≥5 → handoff via Agent (see `/context-reset`)
- Unknown API → read sources → grep codebase → ask user. Don't guess.
- Bug report during impl → activate `/bugfix`, then return here.
- Implementation reveals design must change → trigger **Design Amendment** above, then resume here.

### Step 9: Verify

1. `cargo build` — compiles clean
2. `cargo test` — all green
3. `cargo fmt -- --check` — no formatting drift
4. `cargo clippy -- -D warnings` — clean
5. `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` — no doc errors or warnings (matches CI)
6. For each AC — confirm covered by test or manual verification
7. Show summary table:

```
| # | Criterion | Test / Verification | Status |
|---|-----------|---------------------|--------|
| AC1 | ... | tests::name | ✅ PASS |
```

8. On ALL PASS → proceed to Step 9.5

### Step 9.5: Update documentation

Update content files only — **do not move spec/design to `done/` yet** (that happens at Step 12):

1. **`ai-docs/context.md`** — update any section touched by this task:
   - Resolve open questions that were answered during implementation
   - Update the Plans list (add ✅ to implemented crates)
   - Add new architectural decisions to the Key Decisions table

2. **`README.md`** — update the status table if a new crate was implemented

Then proceed to Step 10.

### Step 10: Self-review loop (max 3 rounds)

Spawn the self-review agent:

```
Agent(subagent_type="general-purpose", prompt="
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
- **Requires a design change** → trigger **Design Amendment** above (user approval required); on return mark `✅ Fixed (design amended)`.
- **Object to it** (finding is wrong or intentionally out of scope):
  - `nit` / `minor`: agent may object autonomously — write reason, mark `⚠️ Objected: <reason>`.
  - `major` / `blocker`: **surface to user first** before objecting. User must approve the objection.

After all findings are resolved (`✅ Fixed` or `⚠️ Objected`):
1. `cargo build` — must compile
2. `cargo test` — all green
3. `cargo clippy -- -D warnings` — clean
4. Update `.progress.md`
5. Return to Step 10.

### Step 12: Finalise docs, commit, and create PR

1. Confirm `git branch --show-current` is **not** `master`. If it is — stop, do not push, tell the user, apply the AGENTS.md recovery procedure.
2. **Finalise INDEX.md and move plan files:**
   - Change the plan row status to `✅ implemented (N tests)`
   - Move spec/design files to `ai-docs/plans/done/`
   - Update dependency tree and **Suggested next steps**
3. `cargo build` — ensures `Cargo.lock` is refreshed and included if changed.
4. Stage all changed files: implementation files from `## Files touched`, `context.md`, `README.md`, `ai-docs/learnings.md` (if modified), updated `INDEX.md`, and spec/design now in `done/`.
5. Commit:
   ```
   feat(<crate>): <short imperative description>

   <1-3 lines: what changed and why; key ACs covered>
   N new tests; all M tests green.
   ```
6. `git push -u origin <branch>`
7. `gh pr create --title "..." --body "$(cat <<'EOF' ... EOF)"` — body must include:
   - **Summary** (bullet list of what changed)
   - **Tracking** — reference the issue captured in the spec's `**Tracked in:**` field:
     - PR fully resolves the issue → `Closes #<N>`
     - PR partially addresses or is related (multi-PR effort, shared umbrella issue) → `Refs #<N>` (no `Closes`)
     - Spec was written with `Tracked in: none` → omit this section
   - **Test plan** (checklist: one line per AC, plus clippy/build)
8. Post the PR URL to the user.

**FORBIDDEN:** declaring done with uncovered ACs · skipping design review · writing code before confirmed spec · deleting `.progress.md` before self-review APPROVE · pushing from master branch · silently deviating from design without triggering Design Amendment

## Gate checklist

| Before | Check |
|---|---|
| Step 3 | All description items extracted? |
| Step 4 | Scope confirmed? |
| Step 5a | Tracking issue identified or created? URL captured? |
| Step 5b | All decisions confirmed? Every AC verifiable? Spec includes `**Tracked in:**`? |
| Step 6 | Spec saved? ACs confirmed? |
| Step 8 | Design doc with GO? Test Design section present? |
| Step 8 start | Feature branch created? Run `git branch --show-current` before every `git commit` — must not be `master`. `base_commit` + `branch` recorded in progress file? |
| Each subtask | `cargo build` ✅? Tests run? `.progress.md` updated? |
| Step 9 | `cargo build` ✅? `cargo test` green? `cargo fmt -- --check` clean? `cargo clippy -- -D warnings` clean? `cargo doc --no-deps --workspace` clean? All ACs covered? |
| Step 9.5 | context.md + README.md updated? (spec/design NOT moved yet — happens at Step 12) |
| Step 10 | Self-review APPROVE before deleting progress file? |
| Step 11 | `major`/`blocker` objections confirmed by user? Design change → Design Amendment triggered? |
| Design Amendment | User approved the amendment? Design review returned GO before resuming? |
| Step 12 | Branch ≠ master? INDEX.md ✅? spec/design moved to done/? `Cargo.lock` refreshed? PR body references the tracking issue (`Closes #N` or `Refs #N`)? PR created and URL posted? |
