---
name: task
description: "Full task workflow from a user description OR a GitHub issue number: interview → spec → design → design-review → impl → verify → self-review. Steps are strictly ordered and cannot be skipped."
disable-model-invocation: true
argument-hint: "[issue-number | task description]"
allowed-tools: Bash(cargo build) Bash(cargo test *) Bash(cargo clippy *) Bash(cargo fmt *) Bash(cargo doc *) Bash(git diff *) Bash(git rev-parse *) Bash(git checkout *) Bash(git branch *) Bash(git add *) Bash(git commit *) Bash(git push *) Bash(gh issue list *) Bash(gh issue view *) Bash(gh issue create *) Bash(gh issue comment *) Bash(gh pr create *) Bash(gh pr view *)
---

Full workflow for a task. Steps execute **strictly in sequence** — proceeding to N+1 before N is complete is FORBIDDEN.

> **Commit authorization.** The default rule "only commit when the user explicitly asks" does **not** apply inside this workflow. Commits at Step 8 (per subtask) and the commit + `git push` + `gh pr create` at Step 12 are pre-authorized by `/task` itself — perform them without an extra prompt. Pause to confirm only if the situation is ambiguous beyond the prescribed step (e.g., commits would touch master, files outside the task scope, or sensitive paths).

The task may originate from either:
- a **GitHub issue number** (e.g. `/task 42` or `/task #42`) — `/interview` reads the issue body during Steps 1–5
- a **user description** (e.g. `/task add foo to bar`) or empty (`/interview` interviews the user)

> **⚡ Compaction recovery check — read FIRST on every invocation.**
> If you are re-entering this skill after auto-compaction (a
> summary/compaction block appears at the top of context, or workflow
> context feels thin), STOP before any tool call and:
>
> 1. **Locate the durable-state file via this skill's active-state probe**
>    — run the preamble glob (`ls ai-docs/plans/*.progress.md 2>/dev/null`) and apply the validation it
>    documents (stale-merge, branch-match, or PR-linkage as the preamble
>    prescribes). The probe both finds the path AND decides whether to
>    RESUME, delete, park, or treat the situation as fresh.
> 2. Once the probe identifies the correct durable-state file
>    (the matched `ai-docs/plans/<spec-base>.progress.md`), read it **top-to-bottom in one
>    pass** — every line, including older sections and the `## Decisions
>    log` section. Do not skim. The recorded `current_step` is a
>    cross-check, never an instruction to skip the read.
> 3. **Then re-enter this skill from the top of its body** — let the
>    preamble's probe / validation / RESUME sequence route control. Do
>    NOT jump to a numbered Step directly; the preamble owns the routing.
>    The probe will land you at the right next action without re-doing
>    completed work.
>
> If the probe finds no matching durable-state file (or returns a
> validated "no active task" result), this is a fresh invocation —
> proceed normally.
>
> See `.claude/skills/context-reset/SKILL.md` § **Compaction recovery
> (re-entry)** for the canonical handoff rationale.

## ⚡ First: check for active task

```bash
ls ai-docs/plans/*.progress.md 2>/dev/null
```

> **Re-entry-after-compaction case (lost-arguments path).** If `$ARGUMENTS` is empty (lost to
> compaction) AND the glob above finds a matching file with `**entry_args:**` recorded, treat the recorded
> `entry_args` as the canonical entry reference. The `⚡ Second` and `⚡ Third` preambles must NOT fire on the **lost-arguments path** — they require a positive match against the live `$ARGUMENTS`, which by definition is unavailable on re-entry. Only `⚡ First` (active-task probe) is allowed to route a lost-arguments re-entry. If the probe finds no matching progress file, the re-entry is "fresh"; surface this to the user (do NOT proceed to Steps 1–5 silently, because the user's original task is unknown).

**If found → validate the match BEFORE jumping to RESUME.** The probe is a flat glob — it matches any `.progress.md` regardless of git branch or merge state. Two failure modes have already burned cycles in this repo:

1. **Stale-merge.** The matched progress file's task already merged via a GitHub-UI merge that bypassed `/pr-merged` (gitignored `.progress.md` survived). RESUME-ing into this points at a completed task instead of starting the new one. _See `ai-docs/learnings.md` 2026-05-13 stale-`.progress.md` entry._
2. **Wrong-branch parallel PR.** The matched progress file belongs to an unrelated in-flight PR on a different feature branch. RESUME-ing here cross-contaminates the two flows. _See `ai-docs/learnings.md` 2026-05-14 branch-aware-probe entry._

**Validation sequence (run before the RESUME jump):**

1. Read `**Branch:**` and `**base_commit:**` from the matched `.progress.md`.
2. **Stale-merge check.** `git merge-base --is-ancestor <base_commit> origin/master` — if exit code is `0`, the task's base commit is now an ancestor of `origin/master`, meaning the work merged. Stale candidate.
3. **Branch-match check.** Compare the progress file's `**Branch:**` against `git branch --show-current`. If they differ, the user is on a different branch from the progress file's owner. Wrong-branch candidate.
4. If **either** check signals a mismatch, surface the situation to the user with three options and wait for direction — do NOT jump to RESUME:
   - **delete** — `rm ai-docs/plans/<base>.progress.md`, then proceed with the new task (Steps 1–7 or whichever applies).
   - **park** — `mv ai-docs/plans/<base>.progress.md ai-docs/plans/<base>.progress.md.parked`; the `.parked` suffix takes it out of the glob, allowing the new `/task` to start cleanly. Restore via the reverse `mv` later.
   - **RESUME anyway** — user explicitly chooses to ignore the mismatch (rare; typically only when reviving an interrupted task whose branch happens to be re-checked-out).
5. If both checks pass (base_commit NOT in origin/master AND branch matches), proceed to the RESUME flow below.

**RESUME flow (skip Steps 1–7) — only after validation passes:**
1. Read the `.progress.md` file
2. Read spec — only `## Acceptance Criteria`
3. Read only files from `## Files touched`
4. Jump to `## Next action`
5. Tell user: "Found active task [X], resuming from [subtask Y]"

---

## ⚡ Second: check for deferred plan activation

> **Guard sentence.** This preamble fires only when `$ARGUMENTS` is **non-empty**. On a lost-arguments re-entry (empty `$ARGUMENTS` after compaction), do NOT enter this preamble — fall through to `⚡ First`'s active-task probe instead. See `⚡ First`'s lost-arguments clause.

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
5. **Verify the spec carries `**Tracked in:**`** — if missing, run `/interview`'s tracking-issue resolution to find or create one and add it to the spec header before continuing.
6. If a `.progress.md` was moved: treat it as an active task — read it and resume from `## Next action` (same as the RESUME path above).
7. Otherwise (no progress file): skip Steps 1–7 and jump directly to Step 8 (spec + design already exist).

---

## ⚡ Third: bare-issue activation of a matching deferred spec

> **Guard sentence.** This preamble fires only when `$ARGUMENTS` is **non-empty** (a bare integer). On a lost-arguments re-entry (empty `$ARGUMENTS` after compaction), do NOT enter this preamble — fall through to `⚡ First`'s active-task probe instead. See `⚡ First`'s lost-arguments clause.

> **AXIOM — When `$ARGUMENTS` is a bare gh issue number, search deferred specs for `**Tracked in:** #N` BEFORE launching the interview.**
> The keyword trigger above ("activate", "start", "proceed") does NOT fire on a bare integer, so `/task 47` would otherwise enter the interview machinery and create a spurious `*.state.md` file even when `ai-docs/plans/deferred/2026-05-01-paint-style.spec.md` already carries `**Tracked in:** #47`. Catch this case here.
>
> | If `$ARGUMENTS` resolves to... | Action |
> |---|---|
> | A bare issue number (`/task 47` or `/task #47`) AND a deferred spec exists with `**Tracked in:** #N` matching that number | Run this phase's activation sequence below — do NOT launch the interview, do NOT create a state file. |
> | A bare issue number AND no matching deferred spec | Fall through to the Steps 1–5 interview phase (the issue's body becomes the interview seed). |
> | Free text / keyword-triggered activation / empty args | Skip this phase; the active-task probe above (if applicable) or Steps 1–5 cover those entry modes. |

Activation sequence (bare-issue → matching deferred spec):

1. Parse `$ARGUMENTS` — strip leading `#`, confirm it's a positive integer `N`.
2. Load issue body: `gh issue view <N> --json title,body,state,labels` (also used to surface the issue's `blocked` label per the separate AGENTS.md learning, when that rule fires).
3. Grep deferred specs for the tracking reference:
   ```bash
   grep -l "^\*\*Tracked in:\*\* #<N>\b" ai-docs/plans/deferred/*.spec.md
   ```
   If grep returns **zero matches**: fall through to Steps 1–5 (interview-driven flow). The issue body loaded in step 2 is available as context for the interview.
   If grep returns **one match**: continue with step 4.
   If grep returns **multiple matches** (unexpected — `**Tracked in:**` should be 1:1 with an issue): surface the list to the user and ask which spec to activate before proceeding.
4. Move the matched spec (and its `*.design.md` / `*.progress.md` siblings if present) from `ai-docs/plans/deferred/` to `ai-docs/plans/`:
   ```bash
   mv ai-docs/plans/deferred/YYYY-MM-DD-name.spec.md ai-docs/plans/
   mv ai-docs/plans/deferred/YYYY-MM-DD-name.design.md ai-docs/plans/     # if exists
   mv ai-docs/plans/deferred/YYYY-MM-DD-name.progress.md ai-docs/plans/   # if exists
   ```
5. Update `ai-docs/plans/INDEX.md`: move the plan row from **Deferred plans** to **Active plans**; status `🟢 ready` (or `🟡 spec-only` if no design exists yet).
6. Surface the spec's existing `## Acceptance Criteria` table to the user verbatim and ask: *"Confirm these ACs, or revise before continuing?"* — wait for the user's response before proceeding. Any revisions must be applied to the spec file before Step 6 launches.
7. **Do NOT run the interview.** **Do NOT create an `*.state.md` interview state file.** **Do NOT re-resolve the tracking issue** — the spec's existing `**Tracked in:** #<N>` is authoritative.
8. If a `.progress.md` was moved (rare — a prior `/task` run on this spec was interrupted): treat the activated task as a resume and jump to the RESUME path's `## Next action`.
9. Otherwise jump directly to **Step 6** (design phase) — the spec exists, ACs are confirmed, the interview phase is satisfied.

---

### Steps 1–5: Spec creation (delegated to `/interview`)

`/task` does not duplicate the interview workflow. Scope extraction, key-decision confirmation, tracking-issue resolution, spec writing, and the cross-link comment are owned by `/interview` (`.claude/skills/interview/SKILL.md`). Treat these five steps as a single delegated phase.

**Already have a spec?** If a saved spec for this task already exists under `ai-docs/plans/` (e.g. the user previously ran `/interview` to draft the spec without implementing), confirm with the user that this is the spec to implement, then **skip to Step 6** — do not re-run the interview.

**Otherwise, run the interview** by invoking the `interview` skill via the Skill tool, passing the original `$ARGUMENTS` through:

```
Skill(skill="interview", args="$ARGUMENTS")
```

The interview will:

- detect entry mode (issue ref / free text / empty) and load the issue body if applicable
- extract and confirm scope (in / out / deferred)
- ask any clarifying questions (max 4 rounds, max 3 questions per round)
- resolve or create the tracking GitHub issue
- save the spec at `ai-docs/plans/YYYY-MM-DD-name.spec.md` with `**Tracked in:** #N` and an `## Acceptance Criteria` table
- post a cross-link comment on the tracking issue (unless `Tracked in: none`)

When the Skill call returns, `/interview`'s instructions and the saved spec are both in conversation context. Resume with the next paragraph of `/task` (the spec-only check, then Step 6).

**Spec-only run.** If the user wants to stop after the interview ("just draft the spec, defer the implementation"), move the spec to `ai-docs/plans/deferred/`, update `INDEX.md` (move the row to **Deferred plans**, status `🟡 spec-only`), and **do not proceed to Step 6**. The spec can be picked up later via the deferred-plan-activation preamble above.

**Before Step 6:** confirm `ai-docs/plans/YYYY-MM-DD-name.spec.md` exists and the user has approved its `## Acceptance Criteria`.

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

> First action: verify both spec and design (with GO verdict) exist AND that **every note / minor / recommendation from the latest design-review GO verdict has been written back into the design document**. "Applied in code later" is NOT the same as "resolved in the design"; the design doc is the implementation contract. Scan the most recent `## Self-Review (Round N)` / `## Verdict: GO` block emitted by `.claude/agents/design-review.md` — for each `## Issues` row of `Severity: note` / `minor` and each `## Recommendations` bullet, confirm the corresponding API table / helper list / risk table / decomposition section of `ai-docs/plans/YYYY-MM-DD-name.design.md` was updated to match. If any note is unresolved → stop, edit the design doc to incorporate it (and re-run design-review if the change is non-trivial per the Design Amendment rule), and only then begin coding. Missing spec, missing design, missing GO verdict, OR unresolved GO-notes = previous steps incomplete.

- **Create a feature branch immediately** — before writing any code or making any commits:
  ```bash
  git checkout -b feat/YYYY-MM-DD-name
  ```
  Use the same date-name as the spec file. Record the branch name in the progress file.
- **Before every `git commit` in this step:** run `git branch --show-current` and confirm it is NOT `master`. If it is — stop immediately, do not commit, apply the recovery procedure in AGENTS.md.
- **Before every `git commit` in this step:** check `git status` for `ai-docs/learnings.md`. If modified or untracked, stage it together with the related code changes — learnings are part of the task deliverable and must be visible in the PR diff.
- Create `ai-docs/plans/YYYY-MM-DD-name.progress.md` at start using the canonical format spec at [`ai-docs/templates/progress-format.md`](../../../ai-docs/templates/progress-format.md) — required fields: `**Branch:**`, `**base_commit:**`, `**Last build:**`, `**current_step:**`, `**last_passed_gate:**`, `**entry_args:**`, plus a `## Decisions log` h2 section. Optional: `**parent_skill:**` (when `/task` itself was invoked from another skill — rare). For `/task` flows also include `**Issue:**` and `**Spec:**`.
- **Record base commit, branch, and `entry_args`** in the progress file header immediately:
  ```
  **Branch:** feat/YYYY-MM-DD-name
  **base_commit:** <output of `git rev-parse HEAD`>
  **current_step:** Step 8 — Implementation start
  **last_passed_gate:** cargo build | <ISO-8601 UTC timestamp> | <commit SHA from git rev-parse HEAD>
  **entry_args:** <original $ARGUMENTS at /task entry — bare issue ref (`#348`/`348`), `activate paint-style`, free text (`add foo to bar`), or `(none)` for empty entry>
  ```
  The `**entry_args:**` field is recorded ONCE at Step 8 creation and **read-only thereafter** — Steps 9–12 do NOT touch it. On a lost-arguments re-entry (empty `$ARGUMENTS` after compaction), this recorded value is the canonical entry reference per `⚡ First`'s lost-arguments clause.
- After each subtask:
  1. `cargo build` — must compile
  2. `cargo test test_name` — if subtask adds tests
  3. `cargo fmt`; `cargo clippy --workspace -- -D warnings`
  4. Update `.progress.md` — **at this subtask boundary, rewrite `**current_step:**` to `Step 8 — subtask N of M complete`; rewrite `**last_passed_gate:**` to `cargo clippy --workspace -- -D warnings | <ISO-8601 UTC timestamp> | <commit SHA from git rev-parse HEAD>`; append a `## Decisions log` bullet for any non-trivial choice made during this subtask (one line, prefixed `Step 8 subtask N:`; omit if none).**
  5. **N=3 of M≥5 handoff gate (binding, not optional).** If this just-completed subtask is the **3rd** of a planned total **M≥5**, STOP further coding in this conversation turn. The next action MUST be a `/context-reset` handoff spawned via `Agent`. Skipping this gate has caused compaction mid-task to silently drop the `/task` step contract — Step 10 (self-review) was omitted on PR #339 because the compaction summary did not reproduce the strict sequence faithfully. Re-state the gate to yourself before deciding: *"I completed subtask N of M. N == 3? M >= 5? If both yes → handoff. No exceptions for 'almost done' or 'one more quick subtask'."* See `.claude/skills/context-reset/SKILL.md` for the handoff protocol.
- Unknown API → read sources → grep codebase → ask user. Don't guess.
- Bug report during impl → activate `/bugfix`, then return here.
- Implementation reveals design must change → trigger **Design Amendment** above, then resume here.
- **Local FAIL investigation before push (AGENTS.md workflow corollary).** When `cargo test` returns `FAILED`, identify the specific failing test (`grep "FAILED"` on the output) and reproduce it in isolation (`cargo test test_name_substring -- --nocapture`) before deciding the failure was transient. A subsequent green run is NOT proof of transience — different test-thread assignments or environment vars (DISPLAY, WAYLAND_DISPLAY) can flip the result. Only accept "transient" when the test is known flaky AND multiple reruns are consistently green. _See `ai-docs/learnings.md` 2026-05-11 entry for the winit-`EventLoop::new()`-on-worker-thread case._

### Step 9: Verify

1. `cargo build` — compiles clean
2. `cargo test` — all green
3. `cargo fmt -- --check` — no formatting drift
4. `cargo clippy --workspace -- -D warnings` — clean (`--workspace` is required so leaf crates outside the default dep tree, e.g. `quartzite-renderer`, are linted; the bare form misses them)
5. `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` — no doc errors or warnings (matches CI; `--all-features` so intra-doc links into every feature-gated module — `serde`-gated `snapshot`, `style`, `widgets`, … — resolve regardless of which feature gates them)
6. **actionlint gate** — if this task created or modified any `.github/workflows/*.yml` file, run `actionlint <file>` (or pass every changed workflow file in one invocation) and require a clean pass. Skip the gate only when no workflow file was touched. See AGENTS.md *Build & Test → Workflow files*.
7. **Panic-index sync.** Scan new/modified production sources for documented or direct panic sites and update `ai-docs/panic-index.md` if any are introduced:
   - `rg '^\s*///\s*#\s*Panics' <changed-files>` — documented panic behaviour (primary signal; always present when a panic exists)
   - `rg '\.expect\(|\.unwrap\(|panic!' <changed-files>` filtered to lines outside `#[cfg(test)]` modules — direct panic call sites
   For each new hit, add an entry to `ai-docs/panic-index.md` (location, trigger, invariant, why not `Result`, preferred fix). Stage `panic-index.md` with the implementation commit. Skip when this task added no new production panics.
8. For each AC — confirm covered by test or manual verification
9. Show summary table:

```
| # | Criterion | Test / Verification | Status |
|---|-----------|---------------------|--------|
| AC1 | ... | tests::name | ✅ PASS |
```

10. On ALL PASS → proceed to Step 9.5

**Write progress at this step boundary** before further tool calls: rewrite `**current_step:**` to `Step 9 — Verify (ALL PASS)`; rewrite `**last_passed_gate:**` to `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features | <ISO-8601 UTC timestamp> | <commit SHA from git rev-parse HEAD>`; append a `## Decisions log` bullet recording panic-index additions, if any (one line, prefixed `Step 9:`; omit if none).

### Step 9.5: Update documentation

Update content files only — **do not move spec/design to `done/` yet** (that happens at Step 12):

1. **`ai-docs/context.md`** — update any section touched by this task:
   - Resolve open questions that were answered during implementation
   - Update the Plans list (add ✅ to implemented crates)
   - Add new architectural decisions to the Key Decisions table

2. **`README.md`** — update the status table if a new crate was implemented

**Write progress at this step boundary** before further tool calls: rewrite `**current_step:**` to `Step 9.5 — docs updated`; append a `## Decisions log` bullet recording any open questions resolved in `context.md` (one line, prefixed `Step 9.5:`; omit if none).

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

**On APPROVE:** proceed to Step 12. The progress file is gitignored and **stays in the working tree** — it persists for `/pr-commented` to extend across any subsequent reviewer-comment rounds, and is deleted by `/pr-merged` after the PR merges. Do NOT `rm` it here. **Write progress at this step boundary** before further tool calls: rewrite `**current_step:**` to `Step 10 — self-review APPROVE (Round N)`; append a `## Decisions log` bullet recording the round count (one line, prefixed `Step 10:`).

**On REJECT:** proceed to Step 11. After Step 11, loop back here. **Write progress at this step boundary** before further tool calls: rewrite `**current_step:**` to `Step 10 — self-review REJECT (Round N), addressing findings`; append a `## Decisions log` bullet recording the finding count + severity breakdown (one line, prefixed `Step 10:`).

**After round 3 with REJECT:** surface all remaining `⬜ Open` findings to the user and ask how to proceed.

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
3. `cargo clippy --workspace -- -D warnings` — clean
4. Update `.progress.md`
5. **PR body sync (unconditional).** Run `gh pr view <N> --json title,body` and re-read the body. Decide *after* reading whether `gh pr edit` is needed: edit if the body contradicts the new commits (renames, scope drift, AC status flips, cited counts that drifted), skip if it's still accurate. Never skip the read. Applies to every push during this step — not just review-fix commits that change public API. See AGENTS.md *Workflow*.
6. **Resolve fixed review threads (unconditional).** For every PR review comment addressed by a `✅ Fixed` finding in this round, follow the GraphQL recipe in [`ai-docs/workflow.md` → PR review comment resolution](../../../ai-docs/workflow.md#pr-review-comment-resolution) verbatim — reply via REST, query unresolved thread IDs via `reviewThreads`, then `resolveReviewThread` each fixed thread and verify `isResolved: true`. Threads behind `⚠️ Objected` findings stay open. Skipping this sub-step has caused the same correction twice (`ai-docs/learnings.md` entries #33 and #44) — the recipe is already documented; this sub-step exists so it is actually consulted.
7. **Write progress at this step boundary** before further tool calls: rewrite `**current_step:**` to `Step 11 — review fixes complete (Round N)`; rewrite `**last_passed_gate:**` to `cargo clippy --workspace -- -D warnings | <ISO-8601 UTC timestamp> | <commit SHA from git rev-parse HEAD>`; append a `## Decisions log` bullet recording any `⚠️ Objected` rationale or Design-Amendment trigger (one line, prefixed `Step 11:`; omit if none).
8. Return to Step 10.

### Step 12: Finalise docs, commit, and create PR

1. **Confirm `.progress.md` is NOT staged.** It is gitignored by `.gitignore` (`/ai-docs/plans/**/*.progress.md`), so `git status` should not list it. If it appears as a tracked-modified entry, `git rm --cached` it once and re-stage; if it appears as staged-untracked, unstage. The file MUST remain in the working tree (it persists for `/pr-commented`; `/pr-merged` deletes it post-merge) but MUST NOT enter the commit.
2. Confirm `git branch --show-current` is **not** `master`. If it is — stop, do not push, tell the user, apply the AGENTS.md recovery procedure.
3. **Finalise INDEX.md and move plan files:**
   - Change the plan row status to `✅ implemented (N tests)`
   - Move spec/design files to `ai-docs/plans/done/`
   - Update dependency tree and **Suggested next steps**
4. **Inbox propagation — parse the just-finalised spec (and its design if present) and append rows to `ai-docs/deferred/_inbox.md`.**
   - Run the parser (per *Inbox propagation* rules below) against `ai-docs/plans/done/YYYY-MM-DD-name.spec.md`.
   - Run the parser against `ai-docs/plans/done/YYYY-MM-DD-name.design.md` if that file exists.
   - Build the live dedupe set `H`: every `Source`-cell path appearing in the 8 thematic files (`ai-docs/deferred/{signals-slots,properties,macros-codegen,object-tree,threading-runtime,future-crates,ci-docs-workflow,python}.md`). `widget-backlog.md` is NOT in `H` — its rows are tracked via the `Notes` cell, not via thematic-file membership.
   - For each candidate row, dedupe at *file* granularity: if the candidate's `Source` path is in `H`, skip the entire file (all of its sections); otherwise append the row to `ai-docs/deferred/_inbox.md` below the existing body.
   - Emit one `WARN: <spec-path> :: <section heading> — unrecognised body shape, no rows emitted` line to stdout for any section whose body matches none of the six shape rules; the row count for that section is zero and Step 12 continues normally.
   - The Step 12 commit (sub-step 7 below) stages `_inbox.md` alongside the existing artefacts.
5. **Regenerate dependent artefacts.** If any source for an auto-generated file
   was touched in this commit, regenerate now and stage the artefact in the same
   commit so the CI sync-gate runs green on first push:
   - `ai-docs/plans/INDEX.md` or `ai-docs/plans/done/**` changed → run
     `bash scripts/gen-roadmap.sh` and stage `ROADMAP.md`.
   - When new generators land, append the source→artefact relationship to
     `ai-docs/context.md` (the auto-derived-artefact registry) and add a bullet
     here.
6. `cargo build` — ensures `Cargo.lock` is refreshed and included if changed.
7. Stage all changed files: implementation files from `## Files touched`, `context.md`, `README.md`, `ai-docs/learnings.md` (if modified), updated `INDEX.md`, regenerated artefacts (e.g. `ROADMAP.md`), `ai-docs/deferred/_inbox.md` (rows appended in sub-step 4), and spec/design now in `done/`.
8. Commit:
   ```
   feat(<crate>): <short imperative description>

   <1-3 lines: what changed and why; key ACs covered>
   N new tests; all M tests green.
   ```
9. `git push -u origin <branch>`
10. `gh pr create --title "..." --body "$(cat <<'EOF' ... EOF)"` — body must include:
    - **Summary** (bullet list of what changed)
    - **Tracking** — reference the issue captured in the spec's `**Tracked in:**` field:
      - PR fully resolves the issue → `Closes #<N>` (auto-closes on merge)
      - PR partially addresses or is related (multi-PR effort, shared umbrella issue) → `Refs #<N>` (no `Closes`)
      - Spec was written with `Tracked in: none` → omit this section
    - **Test plan** (checklist: one line per AC, plus clippy/build)
11. Post the PR URL to the user.
12. **Write progress at this step boundary** before further tool calls: rewrite `**current_step:**` to `Step 12 — PR opened (PR #<N>)`; append a `## Decisions log` bullet recording the PR number and the spec/design `done/` move (one line, prefixed `Step 12:`).

After the PR is created, the unconditional PR-body re-read rule (AGENTS.md *Workflow*) applies to any subsequent push on this branch: `gh pr view <N>` first, then `gh pr edit` only if the body now contradicts the diff.

**Reviewer comments arrive after Step 12** — run `/pr-commented` to address them. Do **not** re-enter `/task` for routine reviewer feedback; `/pr-commented` handles one comment round per invocation (read threads → classify → fix in one commit → self-review → push → reply/resolve) and is re-invocable for each subsequent round. Architectural-rework requests in comments are the one exception: `/pr-commented` bails on them, at which point a fresh `/task` design-review cycle is the correct response.

#### Inbox propagation — parser rules and per-row mapping

The Step 12 sub-step 4 parser specification lives in a dedicated reference file: **[inbox-propagation.md](inbox-propagation.md)**. It covers the six shape rules (NONE / TABLE / PIPEBULLET3 / PIPEBULLET2 / BOLDBULLET / PLAINBULLET), the unrecognised-shape warning behaviour, the per-row mapping format (4-cell `_inbox.md` row), and the file-level dedupe rule against the 8 thematic files. Load it on demand when implementing or modifying Step 12's propagation logic.

**FORBIDDEN:** declaring done with uncovered ACs · skipping design review · writing code before confirmed spec · `rm`ing `.progress.md` from within `/task` (it's gitignored and lives until `/pr-merged`) · staging `.progress.md` into a commit · pushing from master branch · silently deviating from design without triggering Design Amendment

## Gate checklist

| Before | Check |
|---|---|
| Steps 1–5 | Spec saved at `ai-docs/plans/YYYY-MM-DD-name.spec.md`? `**Tracked in:** #N` present (or `none` with reason)? Cross-link comment posted on the tracking issue (unless tracking skipped)? ACs confirmed by user and verifiable? See `/interview` gate checklist for the full per-step list. |
| Step 6 | Spec exists? ACs confirmed? Not a "spec-only / defer" run? |
| Step 8 | Design doc with GO? Test Design section present? **Every note / minor / recommendation from the GO verdict written back into the design doc?** |
| Step 8 start | Feature branch created? Run `git branch --show-current` before every `git commit` — must not be `master`. `base_commit` + `branch` recorded in progress file? |
| Each subtask | `cargo build` ✅? Tests run? `.progress.md` updated? |
| Step 9 | `cargo build` ✅? `cargo test` green? `cargo fmt -- --check` clean? `cargo clippy --workspace -- -D warnings` clean (note: `--workspace`, not bare)? `cargo doc --no-deps --workspace --all-features` clean (note: `--all-features`, not bare and not a hand-picked subset — every feature-gated public module/re-export must be enabled so intra-doc links resolve)? `actionlint` clean on every changed `.github/workflows/*.yml` (skip if none changed)? Any new `# Panics` doc section / `.unwrap()` / `.expect()` / `panic!` outside `#[cfg(test)]` → `ai-docs/panic-index.md` updated and staged (skip when no new production panics)? All ACs covered? |
| Step 9.5 | context.md + README.md updated? (spec/design NOT moved yet — happens at Step 12) |
| Step 10 | Self-review APPROVE? (Progress file persists in working tree — gitignored — until `/pr-merged`. Do NOT `rm` it here.) |
| Step 11 | `major`/`blocker` objections confirmed by user? Design change → Design Amendment triggered? `gh pr view <N>` re-read after every push (unconditional) — `gh pr edit` only if body contradicts new commits? |
| Design Amendment | User approved the amendment? Design review returned GO before resuming? |
| Step 12 | Branch ≠ master? INDEX.md ✅? spec/design moved to done/? `_inbox.md` parsed and appended (or warning logged for unrecognised shape) and staged? Auto-derived artefacts regenerated and staged (e.g. `ROADMAP.md` from `INDEX.md`)? `Cargo.lock` refreshed? PR body references the tracking issue (`Closes #N` or `Refs #N`)? PR created and URL posted? |
