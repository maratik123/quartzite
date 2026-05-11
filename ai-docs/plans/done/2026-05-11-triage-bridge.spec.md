# `/triage` md ↔ gh issues bridge — drift detection (closed refs, status mismatch)

**Source:** issue #205
**Date:** 2026-05-11
**Tracked in:** #205

This is umbrella issue **C** of the four-issue process-improvements meta-plan
([`ai-docs/plans/2026-05-10-process-improvements.md`](2026-05-10-process-improvements.md)).
C ships fourth (and last) in the strict sequence **A1 (#202) → A2 (#203) → B (#204) → C (#205)**.
A1, A2 and B have all merged to `master` at base commit `8456f34`; B introduced
`.claude/skills/triage/SKILL.md` (with a placeholder *Bridge* section pointing at
this issue) and `.claude/agents/triage-runner.md` (a 9-phase opus-subagent
runner with phases 1–8 + 7.5). After this issue lands the meta-plan is done.

The meta-plan's *Locked-in decisions*, *Issue C*, and *Risks and mitigations*
sections are the source of truth for every design-affecting choice; this spec
lifts them so the design phase has a self-contained brief.

## Scope

1. **Extend `.claude/skills/triage/SKILL.md`'s placeholder *Bridge* section**
   with the divergence-detection workflow. The section's parent file already
   exists (shipped in B); the placeholder is an HTML comment + italic
   forward-note pointing at #205, both of which this PR replaces with real
   content. Section ordering established in B is preserved:
   *Trigger and threshold* → *Cell-iteration sweep* → *`_inbox.md` drain* →
   ***Bridge*** → *Run-output summary*.

2. **Extend `.claude/agents/triage-runner.md`** with the bridge phase. B's
   runner has 9 phases (1–8 + 7.5); C inserts the bridge phase as a new
   numbered phase in the same workflow. The defensible default location
   (recorded in B's design *Open questions*) is **before the cell-iteration
   sweep** so the user sees stale-tracked rows in the same overall batch as
   untracked candidates. Design phase confirms or moves it.

3. **Reuse B's existing single bulk `gh issue list` call.** B's Phase 4
   already runs **exactly one** `gh issue list --state all --json number,title --limit 500`
   call per `/triage` session, with a pagination watchdog at ≥ 0.9× the limit
   (response length ≥ 450). C **extends the `--json` projection to include
   `state`** (i.e. `--json number,state,title`) so the bridge can detect
   closed-issue refs from the same map. The "exactly one bulk call per
   session" invariant remains — C does **not** add a second call.

4. **Build a local `{number → {state, title}}` map** from the same bulk-call
   response. The bridge iterates every `Tracked`-column ref (cell 4 in the
   8 thematic files; cell 4 in `_inbox.md`; the `Notes` cell in `widget-backlog.md`)
   and looks up each `#N` ref in the map.

5. **`_inbox.md` `Tracked` = `—` rows are explicitly excluded from the
   bridge sweep** (carved out per the meta-plan *Exception* — those rows go
   through B's Phase 7 drain step). However, the bridge **does still inspect
   `_inbox.md` rows whose `Tracked` cell holds `#N`** (a previous `/triage`
   may have promoted an inbox row; its drift still needs detection).

6. **Three conflict types reported (no silent overwrite — every conflict
   surfaces a diff and asks the user):**
   1. **Stale tracked.** Row's `Tracked` cell holds `#N`; the local map
      reports that issue is CLOSED. Current-data canonical example: the
      `#60` refs in `ci-docs-workflow.md` (verified CLOSED). For thematic
      files, "row implies open" is the default (thematic files have no
      `Status` column — `Tracked: #N` means "tracked-and-open" by
      construction). For `widget-backlog.md`, "row implies open" means the
      row's `Status` cell is **not** `✅` (`✅` semantically asserts done).
   2. **Status mismatch.** Row's prose / status cell asserts `✅ done`
      (widget-backlog only, since thematic-file rows have no Status
      column), but the linked `#N` issue is OPEN. Reverse case also
      flagged: thematic-file row whose `Tracked` cell holds `#N` pointing
      at an issue that closed-as-not-planned (vs. closed-as-completed) —
      design phase distinguishes via `gh issue view <N> --json stateReason`
      if needed, OR simplifies to "closed = stale tracked" and folds
      not-planned into type 1.
   3. **Untracked candidate.** Row's `Tracked` cell = `—`. This is **not a
      true bridge conflict** — it is the same set of rows the cell-iteration
      sweep (Phase 6) and the `_inbox.md` drain (Phase 7) already handle.
      Reported here as a **count** in the bridge summary for situational
      awareness only; no separate per-row prompt opens.

7. **Issues that exist in `gh` but have no md row anywhere are explicitly
   allowed** — not flagged. Asymmetric drift is by design (issues filed via
   `/interview`, `/bugfix`, or the web UI don't owe an md row).

8. **Per-conflict resolution UX.** For each detected conflict of type 1 or 2,
   the user picks one of three actions:
   - **`update md`** — rewrite the md cell to reflect the gh state. For
     stale-tracked (type 1), the design phase locks the exact rewrite
     (defensible default: leave `#N` in place, append `(closed)` marker
     inline, OR rewrite cell to `untracked` with a `# closed: was #N` log
     line in the run summary; design phase picks one and applies
     uniformly). For status-mismatch (type 2), update the row's status
     cell / prose to match the gh state.
   - **`update issue`** — rewrite the gh issue's state (close it if md
     says done; reopen if md says open). Before any `gh issue edit` /
     `gh issue close` / `gh issue reopen` call, the bridge **surfaces a
     diff preview** showing the current issue body / state and the
     proposed change. User must confirm explicitly. The bridge **never**
     silently rewrites issue body or state.
   - **`keep both`** — record the divergence in the run output with a
     user-supplied reason; make no mutation. The conflict will re-surface
     on the next `/triage` run unless one of the other two actions is
     chosen later.

9. **Pagination watchdog already exists in B's Phase 4** (halts the run at
   ≥ 450 results with the verbatim message). C inherits it unchanged — the
   bridge runs after Phase 4's watchdog check, so the bridge never sees a
   truncated map.

10. **Bridge runs as part of every `/triage` invocation.** It is **not** its
    own command. Same batched-approval shape as B's promotion flow (the
    bridge collects all detected conflicts up-front, presents them as a
    batch, and the user resolves them in one pass).

11. **Sync-group propagation in the same PR.** The Triage sync-group
    (`triage/SKILL.md` ↔ `triage-runner.md` ↔ `next/SKILL.md`) was
    established in B's AGENTS.md *Propagation Rule* update. C touches
    `triage/SKILL.md` and `triage-runner.md` — both members must move
    together. `next/SKILL.md` requires no behavioural change (the bridge
    has no `/next`-facing surface) but the Propagation Rule still applies:
    grep `/next`'s SKILL.md for any reference that needs syncing.

## Out of scope

- **Promotion / drain workflows.** B's Phases 5–7.5 (promotion sweep,
  `_inbox.md` drain, batched `gh issue create`) are untouched by C.
- **CI gate, webhook mirror, Rust binary, shell script.** Bridge stays
  pure skill-prompt logic invoked as an opus subagent — same shape as the
  rest of `/triage`.
- **`/next` behavioural changes.** A1 already taught `/next` to surface
  untracked rows in the *Candidates needing `/triage`* section; the bridge
  has no `/next`-facing surface. (The Propagation Rule still requires a
  grep of `next/SKILL.md` for textual references that may need syncing.)
- **Renaming the `Tracked` column to `Issue`.** Future cosmetic
  improvement noted in the meta-plan; out of scope here.
- **Reshaping `widget-backlog.md` schema.** Tracked refs continue to live
  in the `Notes` cell; no 4th column added.
- **Fuzzy matching of `Tracked` cell values.** Bridge only resolves exact
  `#N` refs found in the cell. Free-text prose mentioning issue numbers
  in passing is not a tracked-ref (anchored by column-header context,
  same rule as B).
- **Auto-resolution of conflicts.** Every conflict requires a per-conflict
  user decision (`update md` / `update issue` / `keep both`); the bridge
  never auto-resolves, never silently overwrites either side. This is the
  meta-plan's hardest constraint.
- **Closing the meta-plan.** After C lands, the meta-plan is done and the
  plan file moves to `ai-docs/plans/done/` — a follow-up housekeeping step
  outside this issue's scope.
- **Multi-call `gh issue list` patterns.** The single-bulk-call invariant
  established in B is preserved; C reuses the same response and does not
  introduce a second call.

## Deferred

- Granular distinction between *closed-as-completed* and
  *closed-as-not-planned* via `gh issue view <N> --json stateReason` |
  Defensible default is to collapse both into "stale tracked" (type 1).
  Refine if `/triage` runs surface ambiguity. | no separate issue — fold
  into a future `/triage` v2 enhancement.
- Auto-detection of *body drift* (md row text vs. gh issue body text out
  of sync) | Scope creep beyond the meta-plan's three locked conflict
  types. State-only drift is enough for v1 of the bridge. | no separate
  issue — future cosmetic enhancement if body drift becomes a real
  problem.
- Closing the meta-plan and moving the plan file to `done/` after C
  lands | Housekeeping step, separate from this implementation issue. |
  no separate issue — folded into the C PR's final commit per the
  meta-plan's *Approval gate*.

## Key decisions

| Question | Decision |
|---|---|
| Where does the bridge logic live? | Inside `/triage` only. **Not** a separate command. Bridge runs as part of every `/triage` invocation, before the cell-iteration sweep (defensible default per B's design *Open questions*; design phase confirms placement). |
| Bridge phase placement in `triage-runner.md`'s 9-phase workflow? | Defensible default: insert as a new phase **before Phase 5** (between Phase 4's bulk-`gh issue list` and Phase 6's batch presentation), so the user sees stale-tracked rows in the same overall batch as untracked candidates. Design phase locks the exact phase number. |
| `gh issue list` call strategy? | Reuse B's single bulk call. C **extends the `--json` projection from `number,title` to `number,state,title`** so the bridge can read state from the same map. **No second call.** Pagination watchdog inherited from B unchanged. |
| Local map shape? | `{number → {state, title}}`. Built once from the bulk-call response in Phase 4. Consumed by both the dedupe path (Phase 4 existing) and the bridge (new). |
| Tracked-ref iteration source? | `Tracked` cell (cell 4) in the 8 thematic files; cell 4 in `_inbox.md` **only when it holds `#N`** (the `—` rows route to drain); `Notes` cell in `widget-backlog.md`. Same column-header anchoring rule as B (no false-positive on the `widget-backlog.md:89` prose hit). |
| Conflict types in v1? | Three: **stale tracked** (md `#N` → CLOSED issue), **status mismatch** (md `✅ done` vs OPEN issue and reverse), **untracked-candidate count** (reported but not a per-row prompt — handled by the existing sweep + drain). |
| Status-mismatch surface? | Only widget-backlog rows carry an explicit done marker (`✅`); thematic files have no Status column. Status-mismatch type 2 is therefore primarily widget-backlog-flavoured; reverse-direction (thematic-file row's `Tracked` `#N` is CLOSED but row implies open) folds into type 1 (stale tracked). |
| Asymmetric drift (issue exists, no md row)? | **Silently accepted.** Not flagged. |
| Per-conflict resolution? | User picks one of three actions: `update md`, `update issue`, `keep both`. Each conflict shows a diff before any mutation. |
| `update md` mutation semantics? | For type 1 (stale tracked): defensible default — leave `#N` in place but append a `(closed)` marker inline, OR rewrite to `untracked` with a `closed: was #N` line in the run summary. Design phase picks one and applies uniformly. For type 2 (status mismatch): rewrite the row's status cell to match gh state. |
| `update issue` mutation semantics? | Run `gh issue close <N>` / `gh issue reopen <N>` (and/or `gh issue edit --body <…>` if body drift is in scope per Out of scope above — it is **not** for v1). User confirms a diff preview before the call runs. |
| `keep both` semantics? | No mutation. Reason captured in run output. Conflict re-surfaces on the next `/triage` run. |
| Resolution UX shape? | Per-conflict prompt (each decision is consequential and involves a diff preview). Same per-entry shape as B's `_inbox.md` drain step, distinct from B's batched cell-iteration sweep. (Meta-plan *Open implementation knobs* deferred this; the per-conflict default falls out of the diff-preview requirement — design phase can revisit via Design Amendment.) |
| Concurrent-edit guard for `update md` writes? | Inherits B's Phase-6-and-7.5 content-snapshot guard verbatim. Bridge `update md` writes go through the same re-read-and-compare path; mismatch ⇒ abort with diff. Mtime is not part of the check. |
| Pagination watchdog? | Inherited from B's Phase 4 unchanged (warns at ≥ 0.9 × `--limit`). |
| Bridge in run-output summary? | Adds a new sub-section to the existing summary listing every conflict, its type, the user's resolution, and any `update issue` calls made. |
| Sync-group footprint? | `triage/SKILL.md` ↔ `triage-runner.md` (both fattened in this PR). `next/SKILL.md` checked via grep for textual references that may need syncing; no behavioural change expected. |
| Single `gh issue list` invariant? | Preserved. C does **not** introduce a second call. The invariant is both a performance guarantee and a correctness invariant (multiple calls could race against concurrent mutations). |

## Technical constraints

- **A1 (#202), A2 (#203), and B (#204) have all merged to `master`**
  (base commit `8456f34`). `.claude/skills/triage/SKILL.md` exists with a
  placeholder *Bridge* section (HTML comment + italic forward-note
  pointing at #205). `.claude/agents/triage-runner.md` exists with 9
  phases (1–8 + 7.5). C extends both files in place.
- **`_inbox.md` already exists** with 311 body rows (294 backfilled by A2
  + 17 from B's own dogfooded Step 12 propagation). C's bridge respects
  B's carve-out: `_inbox.md` `Tracked` = `—` rows are **not** in the
  bridge sweep.
- **Single `gh issue list` call per session** — extending the `--json`
  projection to include `state` is a pure additive change to B's call.
  The pagination watchdog at ≥ 0.9 × `--limit` (≥ 450 results) is
  inherited from B's Phase 4 unchanged.
- **Map shape `{number → {state, title}}`** must be built once in
  Phase 4 and consumed by both the existing dedupe path and the new
  bridge phase — no duplicate map-builds.
- **Bridge `update md` writes** use B's content-snapshot concurrent-edit
  guard verbatim (re-read each md file's relevant row text immediately
  before its rewrite step; abort with diff on mismatch; mtime not part of
  the check).
- **`update issue` writes** must surface a diff preview before any
  `gh issue close` / `gh issue reopen` / `gh issue edit` call. The bridge
  never silently rewrites issue state — this is the meta-plan's hardest
  constraint, equal in weight to B's "never silently overwrite md"
  invariant.
- **Mutation scope of `triage-runner` is unchanged from B:**
  `ai-docs/deferred/**` writes + `gh issue create / edit / close / reopen`
  calls only. No code edits, no other instruction-file writes, no
  `ai-docs/learnings.md` writes (AGENTS.md *Boundary rule 2*).
- **Working branch** `feat/2026-05-11-triage-bridge` is already created
  (per AGENTS.md AXIOM 1).
- **Per-conflict prompt UX** mirrors B's `_inbox.md` drain (per-entry)
  shape, not B's cell-iteration sweep (batched table) shape — each bridge
  decision involves a diff preview and is consequential enough to warrant
  per-conflict attention.
- **Run-output summary** gains a new bridge sub-section (conflict list,
  resolutions, `update issue` calls made). Existing summary sections
  (status table, issues created, declined rows, inbox actions, edit
  aborts, `deferred-items.md` diff) are unchanged.
- **`gh issue` mutation tools** required: `gh issue close`,
  `gh issue reopen`, `gh issue edit`. The skill frontmatter's
  `allowed-tools` already covers `Bash(gh issue edit *)`. Design phase
  confirms whether `Bash(gh issue close *)` and `Bash(gh issue reopen *)`
  need explicit additions or are subsumed by `Bash(gh issue edit *)` /
  the global `Bash(gh *)` allow in `.claude/settings.json`.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | **Single bulk `gh issue list` call extended with `state` projection; bridge consumes the same map.** Running `/triage` against current data shows exactly **one** `gh issue list --state all --json number,state,title --limit 500` invocation per session (no second call). The bridge phase reads its `{number → {state, title}}` map from the same response. Verification: manual run on current data; inspect run log; verify call count = 1 and projection includes `state`. |
| AC2 | **Bridge flags the canonical stale-tracked case (`#60` in `ci-docs-workflow.md`).** Running `/triage` against current data reports the `#60` references in `ci-docs-workflow.md` as type-1 conflicts (stale tracked: `#60` is CLOSED). The conflict surfaces in the bridge sub-section of the run-output summary with file path, row item text, and the resolution prompt (`update md` / `update issue` / `keep both`). Verification: manual run on current data; output contains a conflict-line citing `ci-docs-workflow.md` and `#60`. |
| AC3 | **Status-mismatch (type 2) reported in both directions.** Running `/triage` against `tests/fixtures/process-improvements/divergence-cases.md` (synthetic widget-backlog fixture: one row with `Status: ✅` and a `Notes: tracked: #X` where `#X` is OPEN, plus one thematic-file row with `Tracked: #Y` where `#Y` is CLOSED-as-not-planned) reports both directions as conflicts. Verification: fixture committed in this PR (or extended from B's fixture if compatible); bridge output cites both rows. |
| AC4 | **Asymmetric drift accepted silently (issues without md rows are NOT flagged).** Running `/triage` against current data does NOT flag any open issue that has no corresponding row in any `ai-docs/deferred/*.md` file. Verification: pick at least one OPEN issue with no md ref (verify via `rg '#<N>\b' ai-docs/deferred/`); confirm absent from conflict report. |
| AC5 | **`_inbox.md` `Tracked` = `—` rows are NOT in the bridge sweep; `_inbox.md` rows with `Tracked: #N` ARE in the sweep.** Manual dry-run shows the bridge sub-section's conflict list contains zero entries from `_inbox.md` `—` rows (those route to drain). If `_inbox.md` contains a row with `Tracked: #N` pointing at a CLOSED issue, the bridge DOES flag it as type 1. Verification: fixture or current-data scenario; inspect bridge output's `_inbox.md` row count. |
| AC6 | **Per-conflict resolution prompts: `update md` / `update issue` / `keep both`; `update issue` surfaces a diff preview before the `gh issue` call.** For each conflict, the bridge surfaces a per-conflict prompt with the three actions. Choosing `update issue` triggers a diff preview (current issue state + body vs. proposed change) and requires explicit user confirmation before any `gh issue close` / `gh issue reopen` / `gh issue edit` call runs. Verification: manual scenario test — accept an `update issue` resolution; verify diff preview rendered; verify confirmation prompt; verify post-confirm the `gh issue` call executed. |
| AC7 | **Pagination watchdog (inherited from B) still active; bridge never sees a truncated map.** Running `/triage` with `--limit` set such that response length ≥ 0.9 × limit (e.g. `--limit 60` against the current 64-issue corpus, via skill code edit or a fixture) halts the run with B's verbatim WATCHDOG message; no bridge mutations occur. Verification: manual scenario test by lowering the limit; verify halt message; verify no `gh issue close` / `reopen` / `edit` and no md writes. |

## Open questions

- **Exact `update md` rewrite semantics for type-1 (stale tracked).**
  Two defensible defaults: (a) leave `#N` in place, append `(closed)`
  inline to the cell; (b) rewrite cell to `untracked` and log
  `closed: was #N` in the run summary. Design phase picks one and
  applies uniformly across thematic files and `_inbox.md`. Either
  choice satisfies AC2; revisit via Design Amendment if the choice
  produces friction in practice.
- **Exact phase number for the bridge in `triage-runner.md`.** B's
  design *Open questions* recorded the defensive answer: drift detection
  runs **before** the cell-iteration sweep so the user sees stale-tracked
  rows in the same batch as untracked candidates. Concrete phase number
  (e.g. Phase 4.5 vs. renumbering everything after Phase 4) is a design-
  phase placement detail; either is acceptable.
- **`gh issue close` / `gh issue reopen` allow-list status.** Skill
  frontmatter currently lists `Bash(gh issue create *)`,
  `Bash(gh issue edit *)`, etc. Design phase confirms whether the
  bridge needs explicit additions for `Bash(gh issue close *)` and
  `Bash(gh issue reopen *)` or whether `Bash(gh issue edit *)` /
  the global `Bash(gh *)` allow in `.claude/settings.json` is enough.
  No spec-affecting impact; design phase resolves.
- **Closed-as-not-planned vs closed-as-completed distinction.** Out of
  scope for v1 per the *Deferred* section; revisit if `/triage` runs
  surface ambiguity in the conflict reports.
- **Run-output summary placement for the bridge sub-section.** B's
  Phase 8 emits a single summary; the bridge sub-section's order within
  it (before / after the existing sub-sections) is a design-phase
  cosmetic choice. Defensible default: place after the status table and
  before the issues-created list, so conflicts surface near the top of
  the summary.
