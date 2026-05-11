# `/triage` skill — batched promotion + `_inbox.md` drain + widget-backlog source

**Source:** issue #204
**Date:** 2026-05-11
**Tracked in:** #204

This is umbrella issue **B** of the four-issue process-improvements meta-plan
([`ai-docs/plans/2026-05-10-process-improvements.md`](2026-05-10-process-improvements.md)).
B ships third in the strict sequence **A1 (#202) → A2 (#203) → B (#204) → C (#205)**.
A1 and A2 have both merged to `master` at base commit `24849b4`; the new AGENTS.md
AXIOM (Workflow § anchored at `_inbox.md`) declares `/task` Step 12 and `/triage`
as the only writers of `_inbox.md`, and this issue creates the second writer.
`_inbox.md` already exists with 294 backfilled rows under the 4-column schema
`| Item | Source | Section | Tracked |`.

The meta-plan's *Locked-in decisions*, *Issue B*, and *Risks and mitigations*
sections are the source of truth for every design-affecting choice; this spec
lifts them so the design phase has a self-contained brief.

## Scope

1. **New skill file: `.claude/skills/triage/SKILL.md`** with the frontmatter
   already drafted in the meta-plan:

   ```
   ---
   name: triage
   description: "Batched promotion of untracked rows to gh issues; drains _inbox.md; reconciles md ↔ gh issue divergence (bridge ships in Issue C). Default threshold ≥ 3 unhandled rows."
   argument-hint: "[N — override default threshold]"
   disable-model-invocation: true
   allowed-tools: Bash(gh issue create *) Bash(gh issue edit *) Bash(gh issue list *) Bash(gh issue view *) Bash(gh api *) Bash(grep *) Bash(rg *) Read Edit
   ---
   ```

   The skill body launches the `triage-runner` subagent (per the precedent
   shape of `improve/SKILL.md` launching `self-improve`).

2. **New agent file: `.claude/agents/triage-runner.md`** with frontmatter
   `model: opus`. Mirrors `.claude/agents/self-improve.md`'s structure (deep
   analysis subagent, reads md, proposes diffs, applies after user
   confirmation). Mutation scope is **strictly limited to `ai-docs/deferred/**`**
   and `gh issue create / edit` calls — no other file writes, no code edits.

3. **Walk all 10 row sources** on a `/triage` run:
   `ai-docs/deferred/{signals-slots,properties,macros-codegen,object-tree,threading-runtime,future-crates,ci-docs-workflow,python}.md`
   (8 thematic) **+** `widget-backlog.md` **+** `_inbox.md`.

4. **Promote untracked rows.** Across the 8 thematic files and `_inbox.md`,
   rows whose `Tracked` cell holds `—` are batched-approval candidates. In
   `widget-backlog.md`, rows with status `🟡 v2` are batched-approval
   candidates (the `Tracked` ref, when written, lands in the existing `Notes`
   cell — no schema change to widget-backlog).

   Promotion flow (per-row user approval, batched `gh issue create`):
   - Agent drafts title + body per row from the `Item` cell text and the
     linked `Source` spec.
   - Agent runs **one** bulk `gh issue list --state all --json number,title --limit 500`
     query upfront and dedupes proposed titles against existing open+closed
     issues by exact title match.
   - Pagination watchdog: if the response length is ≥ 0.9× the limit (≥ 450),
     warn the user and recommend raising `--limit` or implementing pagination
     before proceeding.
   - Agent presents the full batch as a table to the user.
   - User approves a subset.
   - Batched `gh issue create` for the approved rows only.
   - Agent writes the new `#N` back into the appropriate cell:
     - 8 thematic files and `_inbox.md` → the `Tracked` column (cell 4 in both).
     - `widget-backlog.md` → `Notes` cell, prepending `tracked: #N — ` to the
       existing notes content.
   - Rows the user declines receive an implicit-by-decline rewrite (single
     user action per row, no separate write-confirmation):
     - 8 thematic files and `_inbox.md` → `Tracked` cell rewritten to literal
       `untracked`.
     - `widget-backlog.md` → `Notes` cell rewritten to
       `untracked (declined YYYY-MM-DD): <prev>` where `<prev>` is the
       existing notes content.

5. **`_inbox.md` drain (per-entry, NOT routed through the cell-iteration
   sweep above).** `_inbox.md` rows are carved out of the promotion sweep to
   avoid double-handling. The drain step is canonical:
   - Per-entry user prompt (one row at a time, since each decision is
     consequential).
   - Three actions per row: **sort into a thematic file**, **promote to a gh
     issue**, or **drop**.
   - **Sort** = remove the row from `_inbox.md` and append it to the chosen
     thematic file in the schema of that file (cell 4 stays `—` since the
     row remains untracked at the thematic-file level — the user can promote
     it on a future `/triage` run via the standard cell-iteration sweep).
   - **Promote** = batched into the same approval flow as cell 4 above (the
     batched `gh issue create` covers both the cell-iteration approvals and
     the drain-step promotions in one call). On approval, the row migrates
     to the most appropriate thematic file with the new `#N` written into
     cell 4; on decline, the row migrates to a thematic file with cell 4
     set to `untracked`. The user picks the destination thematic file as
     part of the drain prompt.
   - **Drop** = physically remove the row from `_inbox.md`. (Drop is reserved
     for rows the user judges should never have been propagated — wrong shape,
     duplicate of an already-thematic row that the dedupe missed, etc. The
     `untracked` decline marker is for legitimate rows the user reviewed and
     chose not to promote; those go through Sort.)

6. **Concurrent-edit guard.** `/triage` re-reads each md file's relevant row
   text immediately before its rewrite step. If the content snapshot no longer
   matches the read taken at the start of the session, abort the rewrite with
   a diff showing the divergence, and surface the affected file to the user.
   Mtime is **not** part of the check (filesystem and editor behaviour make
   it unreliable; content-snapshot is the stronger invariant).

7. **Update `ai-docs/deferred-items.md` row counts** at the end of each
   `/triage` run.

8. **Default threshold ≥ 3 unhandled rows**, tunable via skill arg
   (`/triage [N]`). Matches `/improve`'s default. The threshold gates whether
   the agent recommends running `/triage` at all on a given session — below
   the threshold, the agent exits with a brief status report (no batched
   approval prompt).

9. **Skill body shape leaves room for the future *Bridge* section.**
   Section structure of `triage/SKILL.md` must be organised so Issue C
   (#205) can add a *Bridge* section cleanly. Recommended layout: *Trigger
   and threshold* → *Cell-iteration sweep* → *`_inbox.md` drain* → *(Bridge:
   added in Issue C)* → *Run-output summary*. The Issue C placeholder is a
   one-line comment in the skill body, not an empty section.

10. **Sync-group propagation in the same PR:**
    - `AGENTS.md` *Agent Docs* table — add row for the new skill + agent.
    - `AGENTS.md` *Propagation Rule* — add the new sync-group entry:
      `triage/SKILL.md` ↔ `triage-runner.md` ↔ `next/SKILL.md`
      (`/next` already mentions `/triage` in its *Candidates needing
      `/triage`* section per Issue A1; mutual updates required going
      forward).
    - `.claude/skills/improve/SKILL.md` — one-line cross-reference linking
      to `/triage`, noting the shared batched-approval + threshold-trigger
      patterns and the divergence in mutation scope.

## Out of scope

- **md ↔ gh issues bridge / drift detection.** Ships in Issue C (#205). B
  introduces only: promotion + `_inbox.md` drain + `widget-backlog.md` rule.
  The *Bridge* section placeholder lives in `triage/SKILL.md` from B but
  contains no logic yet.
- **`/improve` patterns not borrowed.** The meta-plan documents three:
  (a) post-application eval gate — user-in-loop per row IS the eval;
  (b) hook escalation at ≥3 occurrences — `/triage` mutates data not rules,
  no rule to escalate; (c) `learnings.md`-style decision log — md file state
  (`#N` / `untracked` / `—` markers) IS the log. Do not relitigate these.
- **Folding `/improve` and `/triage` into a unified `/groom` skill.**
  Explicit non-goal per meta-plan.
- **Webhook-driven mirror, CI gate, Rust binary, or shell script.** `/triage`
  is pure skill-prompt logic invoked as an opus subagent — same shape as
  `/improve`.
- **Reshaping `widget-backlog.md` schema.** Tracked refs continue to live
  in the existing `Notes` cell. No 4th column added.
- **Renaming the `Tracked` column to `Issue` across the 9 deferred files.**
  Future cosmetic improvement noted in the meta-plan; out of scope here.
- **`/next` edits.** A1 already taught `/next` to surface untracked rows
  in the *Candidates needing `/triage`* section. B's PR may need a one-line
  cross-reference touch-up in `/next` per the new sync-group rule, but no
  behavioural change.
- **Backfill of `_inbox.md`.** Already done in A2 (294 rows present).
- **Date-stamping declined rows in thematic files / `_inbox.md`** (only
  `widget-backlog.md` carries the `YYYY-MM-DD` per the cell-format above).
  Adding date stamps to thematic-file `untracked` cells is a future
  enhancement noted in the meta-plan; out of scope here.

## Deferred

- Date-stamps on thematic-file / `_inbox.md` `untracked` markers for audit
  purposes | Out of scope for v1 of `/triage`; meta-plan *Open implementation
  knobs* notes this | no separate issue — fold into a future cosmetic PR
  when audit need surfaces.
- `Tracked` column rename to `Issue` across all 9 deferred files | The
  `untracked` decline-marker token clashes semantically with column name
  `Tracked` | yes — a separate cosmetic PR if the friction surfaces in
  `/triage` v1.
- Pagination beyond `--limit 500` | Current corpus is 64 live issues
  (verified at meta-plan rev 3); pagination watchdog warns at ≥ 0.9× | no
  separate issue — design phase decides whether to ship watchdog only or
  ship pagination from day one.

## Key decisions

| Question | Decision |
|---|---|
| Skill / agent shape? | Mirror `.claude/skills/improve/SKILL.md` + `.claude/agents/self-improve.md`. `disable-model-invocation: true` on the skill; `model: opus` on the agent. Skill body launches the agent (one-liner, no logic). |
| Mutation scope of `triage-runner`? | `ai-docs/deferred/**` writes + `gh issue create / edit` calls only. No code edits, no other instruction-file writes. |
| Row sources walked per run? | All 10: 8 thematic + `widget-backlog.md` + `_inbox.md`. |
| Promotion candidates in 8 thematic + `_inbox.md`? | Rows with `Tracked` cell = `—`. |
| Promotion candidates in `widget-backlog.md`? | Rows with status `🟡 v2`. Other statuses (`✅` first-pass, `🤔` undecided, `❌` dropped, `📭` future) are **not** triage candidates — design call needed first for `🤔` / `📭`; `✅` / `❌` are terminal. |
| Title / body drafting for `gh issue create`? | From row's `Item` cell text + linked `Source` spec. Exact prompt template is a design-phase concern. |
| Title-dedupe strategy? | Single bulk `gh issue list --state all --json number,title --limit 500` per run; exact title match against existing open+closed issues. Pagination watchdog at ≥ 0.9× limit. |
| `_inbox.md` drain vs cell-iteration sweep? | `_inbox.md` rows are carved out of the sweep (canonical handler = drain step). Drain is per-entry, three actions: *sort* / *promote* / *drop*. |
| Drain "sort" semantics? | Remove row from `_inbox.md`, append to user-chosen thematic file with cell 4 = `—`. The row can be promoted on a future `/triage` run via the standard sweep. |
| Drain "drop" semantics? | Physically remove the row from `_inbox.md`. Reserved for rows that should never have been propagated (wrong shape, duplicate, etc.). Distinct from the `untracked` decline marker, which records a legitimate review-and-decline decision. |
| Drain "promote" semantics? | Batched into the same `gh issue create` flow as cell-iteration approvals (one bulk call covers both). On approval/decline, the row migrates to a user-chosen thematic file with `#N` or `untracked` in cell 4. |
| Approval UX? | **Batched** for cell-iteration sweep (one table-of-decisions across all 8 thematic files + widget-backlog candidates). **Per-entry** for `_inbox.md` drain. The two flows merge at the `gh issue create` step (one bulk API call). |
| Decline-marker token in thematic files + `_inbox.md`? | Literal `untracked` written to the `Tracked` cell. |
| Decline-marker rewrite in `widget-backlog.md`? | Prepend `untracked (declined YYYY-MM-DD): ` to the existing `Notes` cell. Date is the day of the `/triage` run. |
| Concurrent-edit guard? | Content-snapshot re-read immediately before each rewrite. Abort with diff on mismatch. Mtime not part of the check. |
| Default threshold? | ≥ 3 unhandled rows. Tunable via `/triage [N]`. Below threshold, `/triage` exits with brief status, no approval prompt. |
| What counts as an "unhandled" row for threshold purposes? | Rows that would be candidates for the promotion sweep — i.e. `Tracked` = `—` across 8 thematic + `_inbox.md`, plus `🟡 v2` rows in `widget-backlog.md`. `_inbox.md` rows count individually toward the threshold. |
| Patterns NOT borrowed from `/improve`? | Eval gate (user-in-loop is the eval); hook escalation (no rules to escalate); `learnings.md` decision log (md state IS the log). |
| Bridge logic? | Out of scope here. Section placeholder + one-line forward-note in `triage/SKILL.md`. Issue C (#205) fills it in. |
| Sync-group additions in this PR? | `triage/SKILL.md` ↔ `triage-runner.md` ↔ `next/SKILL.md`. AGENTS.md *Agent Docs* row + *Propagation Rule* row. One-line cross-reference in `improve/SKILL.md`. |

## Technical constraints

- **A1 (#202) and A2 (#203) have both merged to `master`** (base commit
  `24849b4`). The AGENTS.md AXIOM declaring `/task` Step 12 and `/triage`
  as the only writers of `_inbox.md` is in place; B fulfils the AXIOM by
  introducing the second writer.
- **`_inbox.md` already exists** with 294 backfilled rows under the 4-column
  schema `| Item | Source | Section | Tracked |`. The drain step reads this
  file's existing rows.
- **`/triage` must read every row of every `ai-docs/deferred/*.md` file**
  including `_inbox.md` and `widget-backlog.md` — the parser must anchor on
  column-header context, not bare substrings, to avoid the `widget-backlog.md`
  prose-hit class of false positive (one prose hit exists at
  `widget-backlog.md:89` per meta-plan *Background — what was found*).
- **All `_inbox.md` writes during a `/triage` run go through the same
  prompt-driven write path that A2 wired into Step 12.** The AGENTS.md
  AXIOM forbids hand-edits; `/triage`'s writes are agent-driven and
  AXIOM-compliant by virtue of being inside the `triage-runner` flow.
- **Allow-list narrowing** in the skill frontmatter (`allowed-tools:`) is
  a per-skill deliberate narrowing for clarity. `.claude/settings.json`
  already permits `Bash(gh *)` globally — the skill-level narrowing does
  not introduce a new permission, only documents the intended subset.
- **Single `gh issue list` call per run.** This is both a performance
  guarantee (1 API call regardless of corpus size; GitHub's 5000/h rate
  limit is well within budget) and a correctness invariant — multiple calls
  could race against concurrent issue mutations. Pagination watchdog
  enforces visibility at ≥ 0.9× limit.
- **Content-snapshot guard, NOT mtime.** Editor save-without-modify and
  filesystem touch operations bump mtime without changing content; content
  hashing is the stronger invariant for concurrent-edit detection.
- **Mirror `self-improve.md` discipline literally.** Issue B's design phase
  must read `.claude/agents/self-improve.md` and enumerate similarities and
  differences in the design doc, with the goal of keeping the divergences
  minimal and intentional.
- **Working branch** `feat/2026-05-11-triage-skill` is already created (per
  AGENTS.md AXIOM 1 — feature branch before any file edit).
- **`widget-backlog.md` parser rules** must enumerate the 5-status taxonomy
  and choose `🟡 v2` as the only candidate state. Design phase locks the
  rules (e.g. how to extract `Item` from `| Widget | Status | Notes |`
  vs the `| Item | Source | … |` shape of the other files).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | **Skill + agent files exist with correct frontmatter.** `.claude/skills/triage/SKILL.md` exists with frontmatter exactly matching the meta-plan-specified block (`name: triage`, `description: …`, `argument-hint: "[N — override default threshold]"`, `disable-model-invocation: true`, `allowed-tools: …` listing exactly `gh issue create/edit/list/view`, `gh api`, `grep`, `rg`, `Read`, `Edit`). `.claude/agents/triage-runner.md` exists with `model: opus` in frontmatter. Verification: `ls`; `head -10` of each file; `grep '^model:' .claude/agents/triage-runner.md` returns `model: opus`. |
| AC2 | **`/triage` reports a status table covering all 10 row sources.** Running `/triage` against current data produces an opening status table whose row-source labels enumerate all 10 sources: the 8 thematic files, `widget-backlog.md`, and `_inbox.md`. Each row shows the file name and the count of triage-candidate rows in it. Verification: manual run on current data; output table contains 10 row-source labels. |
| AC3 | **Cell-iteration sweep proposes promotion for `—` rows with per-row approval and single bulk `gh issue list` dedupe.** Running `/triage` proposes `gh issue create` for at least one `—` row across the 8 thematic files; the run log shows exactly one `gh issue list --state all --json number,title --limit 500` invocation; each proposed row requires explicit user approval. Verification: manual run; inspect run log; verify exactly one `gh issue list` call and one approval prompt per proposed row. |
| AC4 | **Widget-backlog `🟡 v2` promotion writes `tracked: #N — <prev>` into `Notes`.** A `🟡 v2` row in `tests/fixtures/process-improvements/widget-backlog-promotion.md` (fixture committed in this PR; mirrors the A2 fixture-pattern at `tests/fixtures/process-improvements/`) approved for promotion ⇒ `Notes` cell rewritten with `tracked: #N — <previous notes>`. Row's `Status` column unchanged. Table format intact. Verification: dry-run against fixture; `git diff` shows expected cell mutation; `Status` cell byte-identical pre/post. |
| AC5 | **`_inbox.md` drain is per-entry, not routed through the sweep, and supports sort / promote / drop.** Manual dry-run against current `_inbox.md` (294 rows) shows each inbox row triggers exactly one drain-step prompt (not two — the cell-iteration sweep does NOT also propose the same row). Three drain actions are available per row: **sort** (row migrates to a chosen thematic file with cell 4 = `—`; row removed from `_inbox.md`), **promote** (row migrates to a chosen thematic file with cell 4 = `#N` or `untracked` after the batched `gh issue create`), **drop** (row physically removed from `_inbox.md`). Verification: dry-run on a 3-row subset of `_inbox.md`; verify each row prompts exactly once; verify the three actions produce the documented file mutations. |
| AC6 | **`ai-docs/deferred-items.md` row counts updated at end of run.** After a `/triage` run that mutates ≥ 1 row, `git diff ai-docs/deferred-items.md` shows the count columns updated to match the actual post-run row counts across all 9 deferred files (`_inbox.md` row count reflects drained-and-removed rows). Verification: manual run; compare `git diff` against `wc -l` per deferred file. |
| AC7 | **Declined rows receive `untracked` / `untracked (declined YYYY-MM-DD): <prev>` so subsequent runs skip them.** Running `/triage` against a fixture with one `—` row in a thematic file and one `🟡 v2` row in widget-backlog, both declined: the thematic-file row's cell 4 = literal `untracked`; the widget-backlog row's `Notes` cell prepends `untracked (declined 2026-05-11): ` (or whatever the run date is) to the existing notes. A subsequent dry-run of `/triage` on the mutated fixture does **not** re-propose either row. Verification: fixture at `tests/fixtures/process-improvements/triage-decline.md`; two-step dry-run; `git diff` of mid-state shows the expected tokens; second dry-run output omits the rewritten rows from the candidate set. |
| AC8 | **Concurrent-edit content-snapshot guard aborts with diff on mismatch.** `/triage` re-reads each md file's relevant row text immediately before its rewrite step. If the row text no longer matches the read snapshot, the rewrite is aborted, a diff is surfaced to the user, and the affected file is named. Verification: manual scenario — start `/triage`, modify a candidate row's `Item` text via direct editor between approval and rewrite, confirm `/triage` aborts that row's rewrite and prints a diff identifying the file. Mtime is not used; verify by also touching the file (no content change) and confirming `/triage` proceeds. |

## Open questions

- **None blocking design.** The meta-plan resolved every design-affecting
  question across three opus-subagent reviews. Single residual ambiguity
  was the drain-step "sort" / "drop" semantics, which the meta-plan
  describes only as "sort / promote / drop"; the defensible defaults
  (sort = move to thematic file with cell 4 = `—`; drop = physical row
  removal) are recorded as Key decisions above. If implementation reveals
  a counter-example, the design phase can revisit via Design Amendment.
- **Bridge section placeholder shape.** A one-line forward-note is the
  baseline; the design phase may choose to emit a more substantive
  comment block describing the section's intended contents. Either is
  acceptable; the constraint is that Issue C (#205)'s additions land
  cleanly without restructuring B's skill body.
