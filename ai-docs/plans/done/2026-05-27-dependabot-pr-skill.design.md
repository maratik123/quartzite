# Design: `/dependabot-pr` skill

**Issue:** #577
**Date:** 2026-05-27
**Round:** 2 (round-1 ITERATE addressed)

> **Round-2 changes (one-line summary):** moved the `/pr-ci-failed` carve-out
> stop-point from "STOP after Step 4" to **"STOP between Step 3 and Step 4"**
> (Blocker 1), re-derived the verdict-translation table on `(class, reproducer
> outcome)` only, removed the inheritance claim and re-derived the parent's
> `allowed-tools` minus `git add` / `git commit` / `git push` (Blocker 2), and
> picked option (b) sentinel-comment for the propagation safety net (Major 3).

## Approach

Author a new project skill `.claude/skills/dependabot-pr/SKILL.md` as a **thin
orchestrator** modelled on the pair of `/pr-commented` + `/pr-ci-failed` (same
preamble shape, same allowed-tools spine, same Step 0 / Step 1 progress-file
opening, same AXIOM-2 PR-body read after side effects). The skill drives **one
round** per invocation; subsequent state changes (Dependabot rebases, CI re-runs)
require user re-invocation. No multi-round Claude-internal loops.

The skill resolves into a **single decision matrix** keyed by
`(diff-scope × CI-state)`. Diff-scope is `lockfile-only | scope-drift`;
CI-state is `all-green | red | pending`. Each cell maps to exactly one
terminal action. The matrix is the heart of the skill — every preceding step
(preconditions, snapshot, classification) feeds it, every succeeding step
(side-effecting action + AXIOM-2 re-read + progress close-out) consumes it.

### Key design-phase decisions

1. **Matrix presentation: compact table + per-cell bodies in `reference.md`.**
   The 2 × 3 table fits in SKILL.md (single screen). Each cell lists the
   terminal action verbatim. Per-cell *bodies* (the multi-step recipe for the
   `lockfile-only × red` delegation hand-off, the bail-with-issue template, the
   confirm-merge message template) extract to `reference.md` to keep SKILL.md
   under the 200-line soft target. Same pattern as `/pr-ci-failed`.

2. **Verdict-translation contract with `/pr-ci-failed`: STOP BETWEEN Step 3 and
   Step 4 + read child's progress-file Step-3 record + apply translation
   table.** This is the core of the round-2 redesign (Blocker 1 fix).

   **Why round-1's "STOP after Step 4" was incoherent.** Inspecting
   `.claude/skills/pr-ci-failed/SKILL.md` Step 4 (lines 195–203) shows two
   incompatible side-effects already executed by the time Step 4 finishes:
   - **Inline-fix classes** (`fmt`/`clippy`/`doc`/`actionlint`) — Step 4
     *applies the fix to the workspace* (line 199: "Edit the offending
     file(s); re-run the reproducer until green. Stage explicitly by name").
     Those edits would land on the Dependabot branch's checked-out tree with
     no cleanup path. KD-8 forbids any push to `dependabot/*`, so a workspace
     fix applied here strands edits in a tree we cannot push.
   - **Delegation classes** (`test`/`build`) — Step 4 *delegates to `/bugfix`*
     (line 200: "`/bugfix` owns the full Trace → Root cause → Fix →
     self-review → push loop and exits with the fix landed"). By the time
     control returns to the parent, the bot branch has already been pushed
     to. KD-8 violated.

   The only safe stop-point is **before Step 4** — i.e., after Step 3
   completes (reproducer ran or NO-REPRODUCE recorded), the child must EXIT
   without entering Step 4. No fix is applied, no delegation, no push.

   Three options were considered (round 1's option A and option B remain
   unchanged); option C is re-framed:

   - **(A)** Modify `/pr-ci-failed` to emit a structured verdict and skip its
     own Step 4 when invoked under a Dependabot parent. **Rejected** — KD-14
     forbids modifying `/pr-ci-failed` to know about Dependabot.
   - **(B)** Have `/dependabot-pr` re-implement CI classification inline (copy
     the eight-class table + reproducer logic). **Rejected** — duplicates
     ~150 lines of workflow-time content; future drift between the two
     surfaces is guaranteed.
   - **(C) [Chosen] Delegate to `/pr-ci-failed` with a prompt-level
     "exit-between-Step-3-and-Step-4" directive; parent reads the child's
     progress-file Step-3 record and infers the verdict.** The parent skill
     spawns `/pr-ci-failed` with an explicit instruction in the prompt:
     *"You are running under `/dependabot-pr` on a Dependabot-owned branch.
     EXIT between Step 3 and Step 4 — i.e., after recording your Step-3
     reproducer outcome (PASS or NO-REPRODUCE) into the progress file's
     decisions log. Do NOT enter Step 4. Do NOT apply a fix. Do NOT delegate
     to `/bugfix`. Do NOT commit or push. Your job ends with the Step-3
     decisions-log line; the parent skill consumes that line and translates
     it into the user-facing action."*

   **KD-14 honesty argument.** This IS a behaviour change for the child
   delivered via the spawn prompt, not via code. The argument it stays
   prompt-level (and therefore in-bounds of KD-14: *"`/pr-ci-failed` itself
   must NOT be modified to know about Dependabot"*) rests on three points:
   - The directive itself is GENERIC: "exit between Step 3 and Step 4". It
     does not reference Dependabot inside the child's executed instructions
     — the child only sees a stop-point cutoff. Any parent skill could issue
     the same directive for an unrelated reason (e.g. a hypothetical future
     `/pr-bisect` skill that needs Step-3 reproduction without Step-4 fix).
   - `/pr-ci-failed`'s own source files (`SKILL.md`, `reference.md`) are
     UNTOUCHED. No file in `.claude/skills/pr-ci-failed/` mentions
     Dependabot.
   - The child's existing control flow already supports exiting after Step 3
     in the NO-REPRODUCE branch (line 191: *"Mark the round's `**Self-review:**`
     as `n/a (no-reproduce path)` and exit. Do NOT push anything."*). The
     carve-out merely extends that exit-after-Step-3 path to cover the
     REPRODUCE branch as well, by prompt instruction rather than by code
     change.

   The same KD-14 honest disclaimer surfaces in the Risks section
   (`/pr-ci-failed` Step 3 shape changes) and in the propagation pointer
   (sentinel comment, Major-3 resolution).

   **Verdict-translation table — re-derived under STOP-between-Step-3-and-4.**
   The available inputs are now `(class, reproducer outcome)` only — no "fix
   locality" dimension, because no fix has been applied yet. Re-mapped
   per the reviewer's suggestion:

   | Class | Reproducer outcome | Inferred verdict | Parent's terminal action |
   |---|---|---|---|
   | `fmt` / `clippy` / `doc` / `actionlint` | REPRODUCES locally | Bumped crate exposed a new lint / removed an item — workspace fix needed but Dependabot can only regenerate the bump | **bail-with-issue** (user owns whether to reject the bump, pin, or fix the workspace separately). |
   | `fmt` / `clippy` / `doc` / `actionlint` | does NOT reproduce | Transient (CI environment difference) | **`@dependabot rebase`** comment; close round |
   | `build` | REPRODUCES locally | Dep API change broke our code; Dependabot can't help | **bail-with-issue** (user owns: pin / patch / drop the bump) |
   | `build` | does NOT reproduce | Transient | **`@dependabot rebase`** comment |
   | `test` | REPRODUCES locally | Real regression — semantic change in dep | **bail-with-issue** (KD-5: never silently fork the bump) |
   | `test` | does NOT reproduce | Transient or flaky test | **`@dependabot rebase`** comment |
   | `coverage` / `other` | (any) | Insufficient signal for automation | **pause-for-user**; print child's surfaced log + parent's diagnostic context |

   The user-facing behaviour set is unchanged from round 1: three terminal
   actions (`@dependabot rebase`, `@dependabot recreate`, bail-with-issue),
   plus the `coverage` / `other` pause-for-user. The cleaner input domain
   `(class, reproducer outcome)` removes the round-1 ambiguity about "fix
   locality" — there's nothing to inspect, because no fix was applied. This
   remapping is the reviewer's suggested re-derivation, applied verbatim
   (modulo a small change: round-1's "recreate by default" reading was too
   aggressive — Dependabot's `recreate` semantics regenerate the entire bump
   rather than apply a fix, so `fmt`/`clippy`/`doc`/`actionlint` × REPRODUCES
   defaults to bail-with-issue, not `@dependabot recreate`).

   **Child-exit-step caveat (per design-review round 2 recommendation).** The
   verdict-translation table reads the child's `**Class:**` field at Step 2
   AND the Step-3 decisions-log line. The child does NOT always reach Step 3
   — when `**Class:**` is `other` (and when `coverage` degrades because the
   local environment cannot run `cargo llvm-cov`, per `/pr-ci-failed`
   SKILL.md line 177), the child pauses at Step 2 and never enters Step 3.
   In both cases the parent's table maps `coverage` / `other` × (any
   reproducer outcome) → pause-for-user, so the routing is robust to "Step 3
   never ran". The `/pr-ci-failed` delegation-prompt template in subtask 2
   MUST document this exit-step variation as a precondition table at the
   top, so future readers can recognise that "Class field is sufficient input
   for the coverage/other row even when Step 3 didn't run".

   The child's verdict is **inferred** from `(class, reproducer outcome)`
   recorded in the child's progress file. Default path: the child uses its
   built-in fallback `ai-docs/ci-fixes/pr-<N>.progress.md` (see Q3 — KD-14
   forces this). The parent reads the child's Step-3 record after the child
   exits.

3. **SKILL.md size: extract to `reference.md`.** Estimated SKILL.md size with
   matrix + delegation block + bail templates inline is ~280 lines (above the
   200-line soft target). Extract per-cell bodies + delegation prompt template
   + bail-with-issue body template to `reference.md`. SKILL.md keeps the
   compaction-recovery callout, preconditions, the 2 × 3 matrix table, and the
   per-step narrative. This follows the `/pr-ci-failed` precedent and avoids
   triggering a new `ai-docs/skill-size-exemptions.md` entry.

4. **YAML frontmatter — re-derived from the parent's actual needs (Blocker 2
   fix).** Round 1 incorrectly claimed the child inherits the parent's
   `allowed-tools`. That is structurally wrong: each skill declares its own
   `allowed-tools` in its own frontmatter, and spawning a child via the Skill
   Tool does NOT extend the child's tool surface from the parent. The
   inheritance claim is **dropped**.

   The parent's `allowed-tools` is re-derived from what the parent itself
   invokes:

   - `name: dependabot-pr`
   - `disable-model-invocation: true` (mirrors `/pr-commented`'s shape; user
     must explicitly type the slash command).
   - **Removed from the `/pr-ci-failed` baseline:**
     - `Bash(git push *)` — the parent never pushes (KD-8: never push to
       `dependabot/*`).
     - `Bash(git commit *)` — the parent never commits (it only edits a
       gitignored progress file).
     - `Bash(git add *)` — corollary; with no commit, no staging.
     - `gh pr merge` — already excluded per KD-4 (user runs the merge
       manually).
   - **Added over the `/pr-ci-failed` baseline:**
     - `Bash(gh pr comment *)` — for `@dependabot rebase` / `@dependabot
       recreate` / bail-with-comment.
     - `Bash(gh issue create *)` — for bail-with-issue per KD-5.
   - **Kept** (read-side git + cargo gates the parent itself runs):
     `Bash(cargo build)`, `Bash(cargo test *)`,
     `Bash(cargo clippy *)`, `Bash(cargo fmt *)`, `Bash(cargo doc *)`,
     `Bash(actionlint *)` (kept defensively — workflow YAML inspection in
     scope-drift), `Bash(git status *)`, `Bash(git diff *)`,
     `Bash(git log *)`, `Bash(git rev-parse *)`, `Bash(git branch *)`,
     `Bash(git fetch *)`, `Bash(git merge-base *)`,
     `Bash(gh pr view *)`, `Bash(gh pr checks *)`, `Bash(gh pr edit *)`,
     `Bash(gh run view *)`, `Bash(gh run list *)`, `Bash(gh api *)`.

   The runtime enforcement against pushing to `dependabot/*` lives in the
   anti-patterns list per AC10 (force-push to `dependabot/*` is already
   listed; we add a parallel "never `git push` to `dependabot/*`" entry to
   make the absence of `Bash(git push *)` from `allowed-tools` defensible
   in code review). With `Bash(git push *)` out of `allowed-tools`, the
   harness would refuse the command anyway — the anti-pattern is the
   human-readable mirror.

5. **`actionlint` and workflow-gate concerns.** Not applicable to v1.
   `dependabot/cargo/*` branches never touch `.github/workflows/*.yml` by
   definition (KD-2: cargo only). If a `scope-drift` cell triggers and the
   drift somehow includes a workflow file, the bail-with-comment route
   catches it (KD-7: the skill never edits in scope-drift; it bails).
   `actionlint` in `allowed-tools` is kept defensively for any scope-drift
   inspection (read-only diagnosis); the parent itself never *edits* a
   workflow.

6. **Compaction-recovery preamble (Variant A).** Matches `/pr-commented` and
   `/pr-ci-failed` shape verbatim. Probe sequence: locate progress file via
   PR-linkage glob (`ai-docs/dependabot/pr-<N>.progress.md`), read top-to-bottom,
   re-enter from top of body. Cross-link to
   `.claude/skills/context-reset/SKILL.md § Compaction recovery (re-entry)`.

7. **Progress file: `ai-docs/dependabot/pr-<N>.progress.md` (gitignored, KD-9).**
   No `/task` progress file is ever co-located — Dependabot PRs are not
   `/task`-tracked, so the PR-linkage glob (`grep -l "Tracked in:.*#${PR_NUM}\b"
   ai-docs/plans/done/*.spec.md ai-docs/plans/*.spec.md`) always returns empty
   for Dependabot PRs. The skill therefore unconditionally creates / appends to
   `ai-docs/dependabot/pr-<N>.progress.md` — no fallback path complication.
   The skill creates `ai-docs/dependabot/` if missing. `.gitignore` adds the
   directory line in the same PR.

   The **child** `/pr-ci-failed` writes into its own fallback path
   `ai-docs/ci-fixes/pr-<N>.progress.md` (the child's Step-1 fallback fires
   because the PR isn't `/task`-tracked). The parent reads that file
   post-return to extract the child's Step-3 decisions-log line, then
   re-records the verdict into its own progress file. Two gitignored progress
   files per delegated round; both cleaned up by `/pr-merged` on merge.

8. **AXIOM-2 PR-body re-read.** Fires after any side-effecting action the
   parent skill takes: posting `@dependabot rebase` / `@dependabot recreate`,
   posting a bail-with-comment, creating a bail-with-issue. The confirm-merge
   route (`lockfile-only × all-green`) prints a command for the user and
   pauses — the user's eventual `gh pr merge` is a USER action, so AXIOM-2 in
   that round is the re-read after the parent's last side-effecting action
   (which is none if the round terminates at confirm-pause). On the next
   invocation (re-entry), the preamble + Step 0 snapshot covers the re-read
   naturally.

### Rejected alternative approaches (skill-level)

- **(A) Inline CI classification (no `/pr-ci-failed` delegation).** Rejected:
  duplicates ~150 lines of classification table + reproducer logic; future
  drift; KD-3 explicitly mandates delegation.
- **(B) Auto-merge on `× all-green` without confirm-pause.** Rejected: KD-4
  explicit. Confirm-first is the safety net against silent landings of bumps
  that haven't been eyeballed.
- **(C) Multi-PR batch mode (iterate over all open Dependabot PRs).** Rejected:
  KD scope, "one PR per invocation". Adds complexity; user can re-invoke per
  PR.
- **(D) Single SKILL.md with no `reference.md`.** Rejected on size grounds —
  inline per-cell bodies push past 200 lines.
- **(E) Round-1's "STOP after Step 4" carve-out.** Rejected: incoherent with
  `/pr-ci-failed`'s actual Step 4 control flow — inline-fix classes apply
  workspace edits before the parent can intercept, and delegation classes
  hand off to `/bugfix` which pushes end-to-end. KD-8 violation in both
  branches. Replaced by "STOP between Step 3 and Step 4" (chosen above).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Author `.claude/skills/dependabot-pr/SKILL.md` body — frontmatter (with `allowed-tools` re-derived per KD-4 in § Approach point 4), commit-authorisation callout, compaction-recovery callout, scope, preconditions, Workflow Steps 0–8 narrative, the `(diff-scope × CI-state)` 2 × 3 matrix table (with terminal actions only, no per-cell bodies), anti-patterns list (incl. "never `git push` to `dependabot/*`" mirror of the absent `Bash(git push *)` allowed-tool), gate checklist. Cross-link per-cell bodies to `reference.md`. Adds `.gitignore` entry for `ai-docs/dependabot/`. | `.claude/skills/dependabot-pr/SKILL.md`, `.gitignore` | — |
| 2 | Author `.claude/skills/dependabot-pr/reference.md` — per-cell bodies for each matrix cell (lockfile-only × all-green, lockfile-only × red, lockfile-only × pending, scope-drift × all-green, scope-drift × red, scope-drift × pending), the `/pr-ci-failed` delegation-prompt template (with the **"EXIT between Step 3 and Step 4"** carve-out instruction AND a precondition table at the top of the template noting that `class = other` and degraded `coverage` exit at Step 2 — see § Approach point 2 child-exit-step caveat), the verdict-translation table from § Approach point 2 (the `(class, reproducer outcome)` re-derived form), the bail-with-issue body template, the bail-with-comment body template, the confirm-merge message template. | `.claude/skills/dependabot-pr/reference.md` | 1 |
| 3 | Propagation updates: AGENTS.md `## Agent Docs` table row for `.claude/skills/dependabot-pr/SKILL.md`; `ai-docs/claude-tools-hierarchy.md` § 3c Project-defined Skills table row for `dependabot-pr` (with name-clash check against §§ 3a/3b embedded + marketplace inventories — `dependabot-pr` does not clash); inline sentinel comment in `.claude/skills/pr-ci-failed/SKILL.md` near Step 3/4 boundary cross-referencing the carve-out in `.claude/skills/dependabot-pr/reference.md` (per Major-3 option (b), see Risks). | `AGENTS.md`, `ai-docs/claude-tools-hierarchy.md`, `.claude/skills/pr-ci-failed/SKILL.md` | 1, 2 |

Three atomic tasks. Total scope well under the 7-task ceiling. No further
decomposition needed; SKILL.md authoring + `reference.md` authoring are
coupled-by-link but separable as discrete edits; propagation is the
mechanical final pass.

## Handoff plan

- **(a) When grouping is required** — every M ≥ 1. The `## Handoff plan`
  section is mandatory for every design, including single-group designs.
- **(b) Maximum group size** — 3 consecutive subtasks. Non-terminal groups
  MUST be exactly 3.
- **(c) Handoff destination** — `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
  Named in prose at every boundary, including the entry into the first group.
- **(d) Terminal-group sizing** — `1..=3`. The last group may be smaller than
  the cap; sizes outside `1..=3` are a design defect.

With **M = 3** the plan resolves to a single terminal group:

- **Group A:** subtasks 1–3 — terminal group (3 subtasks; equals the 3-subtask
  cap, within the `1..=3` range). No handoff between groups; the single group
  completes Step 8 in its own `/context-reset` subagent per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
  The parent `/task` Step 8 spawns one `Agent` Tool call for this group, and
  the spawned `/context-reset` subagent is the one entered at the start of
  Group A per the every-group handoff contract.

## Risks

- **Risk: `/pr-ci-failed`'s control flow changes such that the "EXIT between
  Step 3 and Step 4" carve-out instruction no longer cleanly pauses
  execution.** Round-1 mitigation overstated the safety net — the
  Spec-Amendment sync group in AGENTS.md is anchored at
  `.claude/skills/task/SKILL.md`, NOT at `/pr-ci-failed`, and there is no
  named sync-group row for `/pr-ci-failed` ↔ `/dependabot-pr`. The only
  Propagation-Rule mechanism that fires is the catch-all "Any other
  instruction file" grep row, which depends on the editor of `/pr-ci-failed`
  remembering to run `grep -rn "<changed-keyword>"
  .claude/agents/ .claude/skills/ .claude/rules/ AGENTS.md
  ai-docs/agent-writing-style.md` and recognising the resulting
  `/dependabot-pr` hits as a downstream consumer.

  **Mitigation (option (b) — chosen):** inline a **sentinel comment** in
  `.claude/skills/pr-ci-failed/SKILL.md` near the Step 3 / Step 4 boundary,
  cross-referencing the carve-out location in
  `.claude/skills/dependabot-pr/reference.md`. Shape (illustrative;
  exact wording lives in subtask 3):

  ```
  > **Downstream consumer note.** A separate skill (`/dependabot-pr`) spawns
  > this skill with a prompt instruction to EXIT between Step 3 and Step 4 —
  > see `.claude/skills/dependabot-pr/reference.md` § "/pr-ci-failed
  > delegation-prompt template" for the carve-out. If you restructure the
  > Step 3 / Step 4 boundary, update that template too. This comment is a
  > pointer only — `/pr-ci-failed` does NOT know about Dependabot.
  ```

  **Why (b) over (a) (new named sync-group row in AGENTS.md):** option (a)
  is a larger surface-area edit and adds maintenance burden to the
  Propagation Rule's already-long sync-group table. Option (b) is a small
  in-place comment that the next maintainer of `/pr-ci-failed` will see
  literally adjacent to the code they're editing. The actual precedent for
  cross-skill consumer pointers is the AGENTS.md Propagation-Rule sync-group
  rows themselves (e.g. the Triage / Review / Interview group rows pointing
  sibling instruction surfaces at each other). The sentinel comment is a
  smaller, inline form of the same idea: a pointer between sibling
  instruction surfaces, embedded next to the boundary it protects.

  **KD-14 honesty check:** the sentinel comment IS the kind of thing that
  could be read as "`/pr-ci-failed` knows about Dependabot". The honest
  argument it stays in bounds:
  - The comment is a **pointer**, not a behavioural branch. The child's
    executed instructions are unchanged.
  - The comment names Dependabot in prose ONLY as a label for the consumer
    skill; nothing in the comment changes the child's flow conditional on
    "is this Dependabot?".
  - The comment exists to protect the consumer from silent contract drift,
    not to add behaviour. This is the same model as
    `ai-docs/workflow.md`'s "PR body vs. tracking-issue body" cross-link
    notes — pointers between sibling instruction surfaces are normal.

  If a future maintainer disagrees with this reading, option (a) remains
  available as an upgrade path (add the named sync-group row in AGENTS.md
  and delete the sentinel comment).

- **Risk: verdict-translation table mis-classifies a `class = test` failure
  with NO-REPRODUCE as transient when it is a real regression that simply
  doesn't reproduce locally (e.g., requires a specific runner-OS or a clean
  cache).** Mitigation: KD-5 enforces "never silently fork the bump"; the
  worst-case outcome of a mis-classified NO-REPRODUCE → `@dependabot rebase`
  is one wasted Dependabot rebase cycle, NOT a silent regression landing in
  master (the user still confirms merge per KD-4, and on the next
  invocation the CI state will re-surface red if the issue is real). The
  table's REPRODUCE-locally row defaults to bail-with-issue for `test`, so
  the asymmetric risk falls on the side of false-positive transient
  classifications, which the next round catches.

- **Risk: the `lockfile-only × all-green` cell prints the wrong merge
  command (e.g., `--squash` or `--rebase`).** Mitigation: the confirm-merge
  template hard-codes `gh pr merge --merge <N>` (AGENTS.md `## Workflow`
  merge-strategy rule); anti-patterns list explicitly forbids `--squash` /
  `--rebase` in the printed command (AC10).

- **Risk: a future Dependabot PR with multiple commits (e.g., a rebase
  produces a second commit) makes the snapshot Step 0 list commits the
  parent skill cannot reason about.** Mitigation: Step 0 records the
  commit list verbatim into the progress file but does NOT make
  routing decisions based on commit count — routing is purely
  `(diff-scope × CI-state)`. Documented as a gate at the bottom of Step 1.

- **Risk: name clash with the embedded `Skills` inventory in
  `ai-docs/claude-tools-hierarchy.md` §§ 3a / 3b.** Mitigated by the
  subtask-3 name-clash check (per AGENTS.md AXIOM on project-defined name
  clashes). Verified pre-design: `dependabot-pr` does not appear in §§ 3a
  (embedded skills) or 3b (ast-index-marketplace plugin skills).

- **Risk: progress-file-format drift between `/pr-ci-failed`'s round section
  (which the child writes when it exits between Step 3 and Step 4) and the
  parent skill's progress-file schema.** Mitigation: the parent's progress
  file is a separate file (`ai-docs/dependabot/pr-<N>.progress.md`) keyed
  by the parent's round counter, NOT the child's round counter. The
  child's Step-1 fallback path (`ai-docs/ci-fixes/pr-<N>.progress.md`) is
  what the child writes into; the parent reads that file AFTER the child
  returns and re-records the verdict into its own progress file. Two
  progress files per delegated round — both gitignored, both deleted on PR
  merge by `/pr-merged`'s cleanup logic (no change required there).

  Specifically, the parent reads the child's `### Decisions log (round M)`
  Step-3 bullet (the line prefixed `Step 3:` per `/pr-ci-failed` SKILL.md
  line 193). That bullet records reproducer outcome ("reproduced" or "NO
  REPRODUCE, surfaced to user"). Combined with the `**Class:**` field set
  at Step 2, the parent has `(class, reproducer outcome)` — both inputs
  to the verdict-translation table.

- **Risk: parent's `allowed-tools` accidentally re-acquires `Bash(git push *)`
  via a future edit (e.g., a maintainer copies from `/pr-ci-failed`'s
  frontmatter without removing the line).** Mitigation: the anti-pattern
  list in SKILL.md explicitly forbids `git push` to `dependabot/*` (AC10);
  self-review on any future edit will catch the contradiction between
  anti-patterns and `allowed-tools`. The harness allowlist is the
  belt-and-suspenders.

## Test Design

This task authors a skill — no Rust unit / integration test surface. The
`/task` Steps 9–10 self-review + design-review chain is the gate; the
acceptance criteria in the spec are the contract. End-to-end validation is
explicit in AC14:

- **Location:** Manual dry-run, captured in the implementation PR's body.
- **Entry point:** The new `/dependabot-pr` skill, invoked against PR #574
  (the motivating real-world case).
- **Scenarios:**
  - Happy path: `lockfile-only × red` → delegate to `/pr-ci-failed` with the
    "EXIT between Step 3 and Step 4" carve-out prompt → child completes
    Steps 0–3 (snapshot, classify, reproduce) and exits → parent reads
    child's Step-3 record from `ai-docs/ci-fixes/pr-574.progress.md` →
    parent applies verdict-translation table on `(class, reproducer
    outcome)` → parent posts the correct `@dependabot` command OR opens a
    bail-with-issue. Verify NO commit / push happened on the bot branch.
  - Edge case 1: `lockfile-only × all-green` → local `cargo build` validates
    lockfile resolves → parent prints `gh pr merge --merge <N>` command and
    pauses.
  - Edge case 2: `scope-drift` (synthetic) → parent posts bail-with-comment;
    no merge command printed.
  - Edge case 3: `lockfile-only × pending` → parent prints pause-for-user
    message; no side effect.
  - Bail path 1: branch is `dependabot/github_actions/*` → preconditions bail
    with non-cargo-ecosystem message (KD-2).
  - Bail path 2: branch is `dependabot/cargo/*` but tree is dirty →
    preconditions bail.
- **Fixtures / helpers needed:** None for the dry-run (uses live PR #574).
  Future regression tests would require a recorded PR fixture
  (out of v1 scope).

The propagation-rule checks (subtask 3) are validated by `grep`:

- `grep -n "dependabot-pr" AGENTS.md` must return ≥ 1 hit (Agent Docs row).
- `grep -n "dependabot-pr" ai-docs/claude-tools-hierarchy.md` must return ≥ 1
  hit (§ 3c Project-defined Skills row).
- `grep -n "ai-docs/dependabot/" .gitignore` must return 1 hit (the new
  directory pattern, modelled on `/ai-docs/ci-fixes/`).
- `grep -n "dependabot-pr" .claude/skills/pr-ci-failed/SKILL.md` must return
  ≥ 1 hit (the sentinel comment from Major-3 option (b)).
- Name-clash grep: `grep -n "dependabot-pr" ai-docs/claude-tools-hierarchy.md`
  in §§ 3a + 3b must return 0 hits (no embedded or marketplace clash).
- `allowed-tools` audit: `grep -n "git push\|git commit\|git add\|gh pr merge"`
  inside the YAML frontmatter block of
  `.claude/skills/dependabot-pr/SKILL.md` must return 0 hits.

## Open questions

All round-1 open questions are now resolved in this round; no remaining
product-owner questions block implementation.

- **Q1 (KD-14 interpretation — is the prompt-level carve-out within bounds
  of "do NOT modify `/pr-ci-failed` to know about Dependabot"?)** —
  **RESOLVED via iteration.** Round-1's "STOP after Step 4" stop-point was
  rejected by design-review as incoherent with `/pr-ci-failed`'s actual
  Step-4 control flow (Blocker 1). The chosen replacement is
  "EXIT between Step 3 and Step 4" — the carve-out is delivered via the
  spawn prompt as a GENERIC stop-point directive (not Dependabot-specific
  baked into the child's source). KD-14 honesty argument detailed in §
  Approach point 2.
- **Q2 (`cargo build` vs `cargo test` for `lockfile-only × all-green`)** —
  **RESOLVED.** Design default (`cargo build` only) matches spec KD-4
  literal text: *"after a local `cargo build` validates the new lockfile
  resolves"*. CI green covers the test surface; running `cargo test`
  locally before every confirm-merge would be a multi-minute delay and
  duplicates CI work. No product-owner input required.
- **Q3 (child progress-file path — parent-controlled vs child's built-in
  fallback)** — **RESOLVED.** Design default (child's built-in path
  `ai-docs/ci-fixes/pr-<N>.progress.md`) is KD-14-forced: passing a
  parent-controlled path to the child would require modifying the child to
  accept a path override, which violates KD-14. Two gitignored progress
  files per delegated round is the accepted cost.
- **Q4 (spec-side open questions)** — **RESOLVED (spec-deferred).**
  Compatibility-score badge, per-dep auto-merge allow-list, and
  pre-design tracking-issue decision are v2-deferred per the spec body
  and require no design-time action.
