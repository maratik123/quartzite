# Design: `/triage` md ↔ gh issues bridge — drift detection

**Issue:** [#205](https://github.com/maratik123/quartzite/issues/205)
**Spec:** [`2026-05-11-triage-bridge.spec.md`](2026-05-11-triage-bridge.spec.md)
**Date:** 2026-05-11

## Approach

This PR extends the two existing files shipped by Issue B in place:

- `.claude/skills/triage/SKILL.md` — replace the placeholder *Bridge* section
  (HTML comment + italic forward-note) with the divergence-detection prose at
  skill-prompt level.
- `.claude/agents/triage-runner.md` — extend Phase 4's `--json` projection
  (`number,title` → `number,state,title`) and its local-map shape
  (`{title → #N}` → `{number → {state, title}}` plus a derived
  `{title → #N}` view for the existing dedupe path); insert a new
  **Phase 4.5: Bridge sweep** between the bulk-list call (Phase 4) and the
  draft-titles step (Phase 5).

Every behavioural choice is already locked in the spec's *Key decisions*
table; this design phase only resolves the cosmetic / placement details the
spec explicitly defers ("design phase picks one and justifies"):

| Decision | Choice | Why |
|---|---|---|
| Bridge phase number | **Phase 4.5** (between Phase 4 and Phase 5; no renumber) | Minimal churn; signals "extends Phase 4's map" semantically; the `.5` precedent already exists in this file (Phase 7.5 = combined-create pass). Renumber-all (4 → 4, 5 → 6, 6 → 7, …) would touch every cross-reference in the agent file and the skill body for no readability gain. |
| Type-1 `update md` rewrite | **(a) leave `#N` in place, append `(closed)` inline marker** | Preserves the cite-able `#N` in the row (a future `gh issue view <N>` still resolves the trail); idempotent on a second `/triage` run (the bridge sees `#N` AND inline `(closed)` and short-circuits); preserves the cell-4-`Tracked` invariant byte-for-byte. Option (b) would lose the `#N` reference from the row (rewrite to `untracked`) and require a separate summary log line to recover provenance — extra mutation surface for no win. |
| `gh issue close *` / `gh issue reopen *` allow-list | **Add both explicitly to skill frontmatter** | The per-skill `allowed-tools` line is a deliberate narrowing of the globally-allowed `Bash(gh *)` in `.claude/settings.json` (per B's deliverables comment). Adding both verbs to the frontmatter keeps that narrowing intact and makes the bridge's mutation surface visible by reading the frontmatter alone — no developer has to cross-check `.claude/settings.json` to find out which `gh issue` verbs the skill calls. |
| Fixture interpretation | **Synthetic `#X` / `#Y` numbers + mock-map scenario** | Matches existing `tests/fixtures/process-improvements/*.md` convention (synthetic numbers, no live-gh dependency at fixture-author time). AC3 is a manual dry-run scenario in which the runner is told to treat the fixture map as authoritative for that run — the fixture documents the expected output, not a programmatic test harness. |
| Run-summary placement | **After status table, before issues-created list** | Defensible default from spec; conflicts surface near the top of the summary where the user looks first. |

### Rejected alternatives

- **Bridge as a second `gh issue list` call** (separate `--state closed`
  fetch). Rejected by spec (single-bulk-call invariant). Already covered by
  the `state` projection extension — closed issues come back in the same
  response.
- **Bridge as its own slash command (`/bridge`).** Rejected by spec ("Bridge
  runs as part of every `/triage` invocation. It is not its own command.")
- **Auto-resolution of conflicts (heuristic: if md is older than issue,
  trust issue).** Rejected by spec ("never silently overwrites either
  side" — the meta-plan's hardest constraint).
- **Body-drift detection (md row text vs. gh issue body).** Out of scope
  per spec *Deferred* — v1 is state-only.
- **Closed-as-not-planned vs. closed-as-completed distinction (via
  `stateReason`).** Out of scope per spec *Deferred* — both collapse into
  type 1 (stale tracked).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Extend Phase 4's `--json` projection from `number,title` to `number,state,title`; rebuild the local map as `{number → {state, title}}` with a derived `{title → #N}` view for the existing dedupe path. Preserve the pagination watchdog verbatim. | `.claude/agents/triage-runner.md` | — |
| 2 | Insert new **Phase 4.5: Bridge sweep** between Phase 4 and Phase 5. Body per *Phase 4.5 specification* below. Re-uses the map built in task 1; collects conflicts; presents per-conflict prompt; applies user resolutions; appends bridge sub-section to the run-output summary collected in Phase 8. | `.claude/agents/triage-runner.md` | 1 |
| 3 | Replace skill body *Bridge* section's HTML-comment + italic-forward-note placeholder with the real Bridge prose (high-level description; defers to the agent file for operational detail). Section ordering established in B is preserved. | `.claude/skills/triage/SKILL.md` | 2 |
| 4 | Extend skill frontmatter `allowed-tools` with `Bash(gh issue close *)` and `Bash(gh issue reopen *)`. | `.claude/skills/triage/SKILL.md` | — |
| 5 | Add synthetic divergence-cases fixture at `tests/fixtures/process-improvements/divergence-cases.md` with two sections (widget-backlog-shape + thematic-file-shape) and a mock-map block defining `#X`/`#Y` states for the manual scenario. | `tests/fixtures/process-improvements/divergence-cases.md` | — |
| 6 | Sync-group propagation grep + (likely no-op) confirmation. Grep `.claude/skills/next/SKILL.md` for textual references to the bridge / drift detection; if any are stale, sync. (No behavioural change expected — `/next`'s *Candidates needing `/triage`* surface is unchanged by the bridge.) | `.claude/skills/next/SKILL.md` (read; edit only if grep hits stale prose) | 3 |
| 7 | Close the meta-plan: move `ai-docs/plans/2026-05-10-process-improvements.md` to `ai-docs/plans/done/` and update its *Approval gate* section to mark the meta-plan done. House-keeping step in the Step 12 commit. | `ai-docs/plans/2026-05-10-process-improvements.md` → `ai-docs/plans/done/2026-05-10-process-improvements.md` | 3 |

7 atomic tasks. Within the "scope > 7 tasks ⇒ split issue" guidance from the
design agent template — this is one issue, intentionally so.

## Phase 4.5 specification (operational detail for task 2)

The agent file gets a new phase header `### Phase 4.5: Bridge sweep`,
inserted between Phase 4 and Phase 5. Body:

1. **Iterate `Tracked`-column refs across the 10 row sources.** Sources and
   anchors (re-using Phase 3's per-source rules):
   - 8 thematic files: cell 4 (`Tracked`) when it holds `#N`.
   - `_inbox.md`: cell 4 (`Tracked`) **only when it holds `#N`** — `—` rows
     route to Phase 7's drain step and are explicitly excluded from this
     iteration. (Spec *Exception* + AC5.)
   - `widget-backlog.md`: `Notes` cell when the cell holds a `tracked: #N — `
     prefix. The column-header anchor (`| Widget | Status | Notes |`) gates
     classification; bare `Tracked:` substrings in prose (the
     `widget-backlog.md:89` blockquote) are ignored, same rule as Phase 3.

2. **Look up each `#N` ref in the Phase 4 map.** If `#N` not present in the
   map, record as an *orphan ref* in the bridge sub-section's
   diagnostics block (not a conflict — could indicate an issue deleted
   from the project, which is rare but possible). No prompt opens for
   orphan refs.

3. **Classify each map hit into one of three conflict types:**

   | Type | Condition | Notes |
   |---|---|---|
   | 1 — Stale tracked | Map entry's `state` is `CLOSED`, regardless of row's status cell content (thematic files have no status column; widget-backlog rows may have any status). Spec folds closed-as-not-planned into this type. | Canonical case: `#60` refs in `ci-docs-workflow.md`. |
   | 2 — Status mismatch | Map entry's `state` is `OPEN` AND the row asserts done. Detection rule by source: <ul><li>**`widget-backlog.md`**: row's `Status` cell = `✅` (the only `Status` value that asserts done).</li><li>**Thematic files**: no `Status` column ⇒ this direction never fires; the file's rows can only fall into type 1 or type 3.</li><li>**`_inbox.md`**: no `Status` column ⇒ same as thematic.</li></ul> | Only widget-backlog can produce type 2 in current schema. |
   | 3 — Untracked candidate | Row's `Tracked` cell = `—`. **Not a per-conflict prompt** — counted only. Already handled by the Phase 6 sweep + Phase 7 drain. | Spec: "reported here as a count in the bridge summary for situational awareness only; no separate per-row prompt opens." |

4. **Collect all type-1 and type-2 conflicts in batch up-front; present as a
   batched preamble** (file path + cell location + `#N` + classification +
   one-line diff preview) so the user sees the full conflict surface before
   any per-conflict prompt opens. The preamble mirrors Phase 6's batched
   table for the cell-iteration sweep — same mental model, different
   conflict shape.

5. **For each type-1 or type-2 conflict, open a per-conflict prompt** (the
   spec mandates per-conflict because each decision involves a diff and is
   consequential; this mirrors Phase 7's drain UX, not Phase 6's batched
   table). The prompt shape:

   ```
   Conflict N of M — <type 1: stale tracked | type 2: status mismatch>
     File:     <path>
     Cell:     <line N: column 4 / Notes>
     Tracked:  #N — <issue title from map>
     Issue state: <CLOSED | OPEN>
     Row state:   <implied open | ✅ done>

     Diff preview:
       md:   <current row text>
       gh:   #N <title> [<state>]

   Action? (m)update md / (i)update issue / (k)keep both
   ```

6. **Action semantics:**

   - **`update md`** (write to md, no gh mutation):
     - **Type 1 (stale tracked):** rewrite the cell to leave `#N` in place
       and append ` (closed)` after it. Examples per source:
       - Thematic-file cell 4: `#60` → `#60 (closed)`.
       - `_inbox.md` cell 4: same — `#60` → `#60 (closed)`.
       - `widget-backlog.md` `Notes` cell: `tracked: #60 — needs button group`
         → `tracked: #60 (closed) — needs button group`.
       - **Idempotency:** on a future `/triage`, the bridge re-iterates
         these cells; if the cell content already contains `(closed)` after
         `#N`, the bridge short-circuits (already-resolved type-1). The
         agent's classifier checks for the literal substring `(closed)` in
         the cell as the "already-resolved" guard.
     - **Type 2 (status mismatch, widget-backlog `✅` vs. OPEN gh issue):**
       rewrite the row's `Status` cell from `✅` to one of the four
       non-done statuses (`🟡` / `🤔` / `❌` / `📭`) based on what the user
       picks in a follow-up prompt. Defensible default: `🟡 v2` (the issue
       being OPEN means it's still planned but not done — `v2` matches
       "deferred to a follow-up issue, definitely planned"). `Notes` cell
       unchanged.
     - **Concurrent-edit guard:** B's Phase-6-and-7.5 content-snapshot
       guard applies verbatim. Take snapshot of the file's relevant row
       at start of session (already done in B's `## Inputs`); re-read
       immediately before the write; abort with diff on mismatch; mtime
       not part of the check.

   - **`update issue`** (write to gh, no md mutation):
     - **Type 1 (stale tracked):** the user is asserting the md row is
       right (work still open) — call `gh issue reopen <N>`. Surface diff
       preview first: current issue state (`CLOSED`) → proposed state
       (`OPEN`). User confirms via a yes/no prompt before the call runs.
     - **Type 2 (status mismatch, widget-backlog `✅` vs. OPEN gh issue):**
       the user is asserting the md row is right (work done) — call
       `gh issue close <N>`. Diff preview: `OPEN` → `CLOSED`. User
       confirms.
     - **No `gh issue edit` calls in v1** — body drift is out of scope
       (spec *Out of scope*). The skill frontmatter retains `Bash(gh issue
       edit *)` from B for future use (Issue B already declared it; the
       bridge does not yet exercise it).
     - **Failure handling:** if the `gh issue close/reopen` call fails
       (network, permission), surface the error to the user, leave the
       conflict in the run summary as unresolved, continue with the next
       conflict. No retry, no md mutation.

   - **`keep both`**:
     - No mutation to md or gh.
     - Capture the user-supplied reason in the bridge sub-section of the
       run output (free-text prompt: "Reason for keeping both? ").
     - Conflict re-surfaces on the next `/triage` run (no marker is
       written that would short-circuit it). This is intentional —
       `keep both` is a "decide later" action, not a permanent classification.

7. **Append a bridge sub-section to Phase 8's run-output summary.** Placement
   within the existing summary: after the status table, before the issues-
   created list. Sub-section body:

   ```
   ## Bridge sub-section (md ↔ gh issue divergence)

   Conflicts detected: <total>
     Type 1 (stale tracked):   <count>
     Type 2 (status mismatch): <count>
     Type 3 (untracked count): <count>   # reported only, no per-row prompt

   Orphan #N refs (issue not in bulk-list map): <count>
     <list, one per line>

   Resolutions:
     update md:    <count>   <list: file + cell + #N + before/after>
     update issue: <count>   <list: #N + before-state → after-state>
     keep both:    <count>   <list: file + cell + #N + user reason>

   `gh issue` calls made by bridge this run:
     <list of close/reopen commands actually executed>
   ```

   The existing Phase 8 sub-sections (status table, issues created, declined
   rows, inbox actions, edit aborts, `deferred-items.md` diff) are
   unchanged.

8. **Phase 4.5 is read-only on `ai-docs/deferred/**` until the user resolves
   conflicts.** Mutations happen at user-decision time, one conflict at a
   time, with the concurrent-edit guard checked immediately before each
   write. No batched mutation pass — this matches Phase 7's drain shape,
   not Phase 6's deferred-create pass.

## Skill body *Bridge* section — exact replacement prose (task 3)

Replace lines 42–48 of `.claude/skills/triage/SKILL.md` (the HTML comment +
italic-forward-note placeholder) with:

```markdown
## Bridge

After the bulk `gh issue list` call and before the cell-iteration sweep, the
subagent walks every `Tracked`-column ref across the 10 row sources (cell 4
in the 8 thematic files + `_inbox.md` **only** when the cell holds `#N`; the
`Notes` cell in `widget-backlog.md` when the cell holds a `tracked: #N —`
prefix) and looks up each `#N` in the local `{number → {state, title}}` map
built in Phase 4. `_inbox.md` rows whose `Tracked` cell = `—` are explicitly
excluded — those route to the per-entry drain step.

Three conflict types reported (no silent overwrite — every type-1 and type-2
conflict surfaces a diff and asks the user):

- **Stale tracked.** Row's `Tracked` cell holds `#N` and the map reports
  that issue is CLOSED. Canonical example: the `#60` references in
  `ci-docs-workflow.md`.
- **Status mismatch.** `widget-backlog.md` row's `Status` cell = `✅` but
  the linked `#N` issue is OPEN. (Thematic files have no `Status` column;
  this direction does not fire for them — the reverse case, thematic-file
  row with `#N` that closed-as-not-planned, folds into stale-tracked.)
- **Untracked candidate count.** Row's `Tracked` cell = `—`. Reported as a
  count for situational awareness only — these rows are already handled by
  the cell-iteration sweep (thematic + widget-backlog) and the `_inbox.md`
  drain step.

For each detected type-1 or type-2 conflict, the user picks one of three
actions (per-conflict prompt, mirroring the drain step's per-entry shape):

- **`update md`** — rewrite the md cell to reflect gh state. Type-1
  rewrites leave `#N` in place and append ` (closed)` inline; type-2
  rewrites the widget-backlog `Status` cell to a non-done status the user
  picks from a follow-up prompt (default `🟡 v2`). Concurrent-edit guard
  (content-snapshot, not mtime) inherited from the cell-iteration sweep.
- **`update issue`** — close or reopen the gh issue to match the md row.
  Before any `gh issue close` / `gh issue reopen` call, the bridge
  surfaces a diff preview (current state → proposed state) and requires
  explicit user confirmation. The bridge **never** silently rewrites
  issue state or body.
- **`keep both`** — record the divergence in the run output with a
  user-supplied reason; make no mutation. The conflict re-surfaces on the
  next `/triage` run.

Issues that exist in `gh` but have no md row anywhere are explicitly
**not** flagged — asymmetric drift is by design.

The bridge appends a sub-section to the run-output summary listing every
conflict, its type, the user's resolution, and any `gh issue close` /
`gh issue reopen` calls made. See `.claude/agents/triage-runner.md` Phase
4.5 for the operational specification.
```

The section ordering established in Issue B is preserved:
*Trigger and threshold* → *Cell-iteration sweep* → *`_inbox.md` drain* →
**Bridge** → *Run-output summary*.

## Agent file Phase 4 extension — exact diff (task 1)

The Phase 4 body in `.claude/agents/triage-runner.md` currently reads (key
lines):

```
gh issue list --state all --json number,title --limit 500
```

and:

```
Otherwise build a local `{title → #N}` map.
```

Edit to:

```
gh issue list --state all --json number,state,title --limit 500
```

and replace the map-construction sentence with:

```
Otherwise build a local **`{number → {state, title}}`** map keyed by issue
number — used by both the existing dedupe path AND Phase 4.5's bridge sweep
(extended in Issue C). Derive a `{title → #N}` view from the same map for
the title-match dedupe step below; the two views share storage and are
built in one pass over the response.
```

The pagination watchdog block, the dedupe edge-cases block, and Phase 4's
overall structure are unchanged. The "exactly one bulk call per session"
invariant is preserved — the change is a pure additive `--json` projection
widening + a single map shape upgrade.

## Agent file Phase 4.5 — exact insertion point (task 2)

Insert the new phase header `### Phase 4.5: Bridge sweep` and the body
specified in the *Phase 4.5 specification* section above, immediately
between the existing Phase 4 block (ends with the dedupe edge-cases block)
and Phase 5's `### Phase 5: Draft titles and bodies` header.

## Skill frontmatter — exact change (task 4)

Current:

```yaml
allowed-tools: Bash(gh issue create *) Bash(gh issue edit *) Bash(gh issue list *) Bash(gh issue view *) Bash(gh api *) Bash(grep *) Bash(rg *) Read Edit
```

Change to:

```yaml
allowed-tools: Bash(gh issue create *) Bash(gh issue edit *) Bash(gh issue close *) Bash(gh issue reopen *) Bash(gh issue list *) Bash(gh issue view *) Bash(gh api *) Bash(grep *) Bash(rg *) Read Edit
```

The two new entries (`Bash(gh issue close *)`, `Bash(gh issue reopen *)`)
preserve the deliberate-narrowing pattern that B established — the skill's
mutation surface is fully described by its frontmatter without requiring a
cross-reference to `.claude/settings.json`.

## Fixture specification (task 5)

New file `tests/fixtures/process-improvements/divergence-cases.md`. Schema
follows existing fixture conventions (synthetic numbers, mock-map block,
explicit AC reference). Content:

```markdown
# Fixture: bridge divergence cases (AC3)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises AC3: the bridge reports type-2 (status mismatch) in
both directions. The synthetic issue numbers `#X` and `#Y` below are NOT
real gh issues — they are interpreted via the mock map at the bottom of
this file. Manual scenario: run `/triage` with the bridge instructed to
treat the mock map as authoritative for this fixture; verify the bridge
output cites both rows as conflicts.

## Section A — widget-backlog shape (Status: ✅ vs OPEN gh issue)

| Widget | Status | Notes |
|---|---|---|
| `SyntheticDoneWidget` | ✅ first pass | tracked: #X — synthetic note carrying tracked-ref |

## Section B — thematic-file shape (Tracked: #N → CLOSED gh issue)

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Synthetic deferred item linked to stale issue | `tests/fixtures/process-improvements/divergence-cases.md` | deferred | #Y |

## Mock map (manual scenario input)

For this fixture, the `/triage` bridge treats the following map as
authoritative:

| Issue | State | Title |
|---|---|---|
| `#X` | OPEN | Synthetic open issue cited by widget-backlog row |
| `#Y` | CLOSED | Synthetic closed-as-not-planned issue cited by thematic row |

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC3 | Dry-run `/triage` against this fixture (with the mock map injected): the bridge sub-section reports both rows as conflicts. Section A's `SyntheticDoneWidget` row surfaces as **type 2 (status mismatch)** because `Status: ✅` and `#X` is OPEN. Section B's row surfaces as **type 1 (stale tracked)** because `Tracked: #Y` and `#Y` is CLOSED. The widget-backlog `tracked: #X —` prefix is correctly anchored on the `Notes` cell (column-header rule). |
```

The mock-map block makes the fixture interpretable without depending on
live gh state — the manual scenario tells the runner to use the embedded
map. This matches the synthetic-fixture convention established by Issue B's
`triage-decline.md`, `widget-backlog-promotion.md`, and `triage-inbox-3rows.md`
fixtures (all of which use synthetic data with no live-gh dependency at
fixture-author time).

## Sync-group propagation (task 6)

The AGENTS.md *Propagation Rule* Triage sync-group (established by Issue B):
`.claude/skills/triage/SKILL.md` ↔ `.claude/agents/triage-runner.md` ↔
`.claude/skills/next/SKILL.md`. Tasks 1–4 already cover the first two
members. For `next/SKILL.md`:

```bash
grep -n 'bridge\|drift\|divergence' .claude/skills/next/SKILL.md
```

Expected hits: zero. The `/next` skill mentions `/triage` for the
*Candidates needing `/triage`* surface only — the bridge has no
`/next`-facing surface (no untracked-row creation, no row classification
change visible to `/next`). If the grep returns any hit on stale prose,
sync — otherwise the propagation requirement is satisfied by the grep
itself (the Propagation Rule's procedure step 1 is the grep; step 2 only
fires on matches).

## Meta-plan closure (task 7)

After the bridge ships, all four umbrella issues (A1, A2, B, C) are merged
and the meta-plan is done. House-keeping step (single commit, alongside the
Step 12 propagation commit):

1. **Move** `ai-docs/plans/2026-05-10-process-improvements.md` to
   `ai-docs/plans/done/2026-05-10-process-improvements.md` via `git mv`.
   (The user's task notes refer to the file as already-in-done; that is
   not the current state — the file is at the top level. The move is
   part of this task.)
2. **Update the moved file's *Approval gate* section** to add a fourth
   checkbox:

   ```markdown
   4. ✅ Meta-plan complete: all four umbrella issues merged (A1 #202, A2 #203, B #204, C #205). File moved to `done/` in the C PR's Step 12 commit.
   ```

3. **Update `ai-docs/plans/INDEX.md`** to reflect the move (the meta-plan's
   row migrates from the active section to the done section). This is the
   standard Step 12 behaviour for any plan landing in `done/`; no special
   handling for the meta-plan.

## Risks

- **Risk: Phase 4.5 number clashes with future renumber.** The `.5`
  numbering convention is sustainable for one or two insertions per file;
  a future C+1 issue would have to choose between Phase 4.25 (ugly) and a
  full renumber. *Mitigation:* the spec marks this as a v1 bridge — future
  expansions (body drift, stateReason distinction) are out of scope and
  unlikely to insert another phase between 4 and 5. If renumber is
  eventually needed, it lands in a focused refactor PR.

- **Risk: `(closed)` inline marker collides with prose that already
  contains the substring `(closed)`.** Vanishingly low — the bridge's
  short-circuit check requires `#N` followed by `(closed)` in the same
  cell; freeform prose rarely matches that adjacency. *Mitigation:* the
  classifier anchors on the regex `#\d+\s*\(closed\)` (issue-number-
  adjacent), not bare `(closed)`. If false positives surface in practice,
  promote to a more specific marker (e.g. `#N (closed YYYY-MM-DD)`) via
  Design Amendment.

- **Risk: `update issue` call fails mid-conflict-list (network /
  permission), leaving partial state.** *Mitigation:* per-conflict failure
  is logged in the bridge sub-section of the run summary and the loop
  continues; no retry; no md mutation occurs for a failed `gh issue`
  call. Re-running `/triage` re-surfaces the same conflict.

- **Risk: User picks `update md` on a type-1 widget-backlog row but the
  `Notes` cell snapshot drifted between session-start and write-time
  (concurrent web-UI edit).** *Mitigation:* the content-snapshot guard
  inherited from B (Phase 6 / 7.5) fires verbatim — abort the write with
  unified diff, leave the conflict in the run summary, continue.

- **Risk: Asymmetric drift case ("OPEN gh issue, no md row") accidentally
  flagged.** *Mitigation:* the iteration loop in step 1 of Phase 4.5 walks
  md cells, NOT gh-list entries. The map is queried only by md-cell ref,
  so issues with no md ref are never visited. This is structural — the
  loop body never sees an unreferenced gh issue.

- **Risk: `_inbox.md` `—` row accidentally enters the bridge sweep.**
  *Mitigation:* step 1 of Phase 4.5 explicitly filters `_inbox.md` cell-4
  values: a `—` short-circuits BEFORE the map lookup. AC5 verifies this.

- **Risk: Bridge phase runs even when the run halts via the pagination
  watchdog.** *Mitigation:* the watchdog (Phase 4) halts the run BEFORE
  Phase 4.5 starts. The phase order guarantees the bridge never sees a
  truncated map. AC7 verifies this.

- **Risk: The skill frontmatter additions (`Bash(gh issue close *)`,
  `Bash(gh issue reopen *)`) collide with the global `Bash(gh *)` rule in
  `.claude/settings.json` and confuse the permission grant.** *Mitigation:*
  Claude Code's allow-list is additive — a narrower per-skill `allowed-tools`
  entry simply restates a subset of the global permission and grants no
  new authority. Verified by reading existing per-skill `allowed-tools`
  entries (the `Bash(gh issue create *)` line in B's frontmatter already
  follows this pattern).

## Test Design

The bridge is prompt-only Rust-free code; there is no `#[cfg(test)]` module
to write. Verification is per-AC manual / fixture-driven, matching B's
precedent.

### AC verification recipes

| AC | Strategy | Mechanically-verifiable commands / manual scenario |
|---|---|---|
| AC1 (single bulk call extended with `state`) | Inspection of run log | Manual: run `/triage` on current data; in the run log, count occurrences of `gh issue list` invocations — MUST equal 1. Inspect the call's `--json` argument — MUST contain `state` (full projection `number,state,title`). Mechanically: `rg "gh issue list" <run-log>` returns exactly 1 line; that line contains `--json number,state,title`. |
| AC2 (canonical `#60` flag) | Manual run on current data | Manual: run `/triage` on current data. Inspect the bridge sub-section of the run-output summary; MUST contain **three** type-1 conflict lines citing `ci-docs-workflow.md` and `#60` (verified via `grep -c '#60' ai-docs/deferred/ci-docs-workflow.md` returning 3). Each line includes file path, row item text, and the per-conflict resolution prompt. Mechanical post-condition (if user picks `update md` on all three): `grep '#60 (closed)' ai-docs/deferred/ci-docs-workflow.md` returns 3 lines; `git diff ai-docs/deferred/ci-docs-workflow.md` shows only the inline marker appended after `#60` in each of the three rows; no other byte changes. |
| AC3 (status-mismatch both directions via fixture) | Dry-run against `tests/fixtures/process-improvements/divergence-cases.md` | Manual: run `/triage` with the fixture's mock map injected. Bridge sub-section MUST cite Section A's row as type-2 and Section B's row as type-1. Mechanically: `grep -E 'type.?1.+#Y' <run-output>` returns ≥ 1 line; `grep -E 'type.?2.+#X' <run-output>` returns ≥ 1 line. |
| AC4 (asymmetric drift silently accepted) | Manual scenario on current data | Pick at least one OPEN issue with no md ref. Find via: `gh issue list --state open --json number,title --limit 500 \| jq -r '.[].number' \| while read N; do if ! rg -q "#$N\b" ai-docs/deferred/ then echo "#$N"; fi; done` — pick any returned number. Run `/triage`; verify the bridge sub-section does NOT cite that number anywhere. |
| AC5 (`_inbox.md` `—` carve-out + `_inbox.md` `#N` inclusion) | Manual scenario + structural inspection | Manual scenario A (`—` carve-out): `_inbox.md` currently has 311 `—` rows (per spec). Run `/triage`; verify the bridge sub-section's conflict list contains **zero** entries from `_inbox.md` `—` rows (those route to drain — Phase 7). Mechanically: `grep -c "_inbox.md" <bridge-sub-section>` for `—` rows = 0. Manual scenario B (`_inbox.md` `#N` inclusion): construct a one-row inbox scenario (or extend `triage-inbox-3rows.md`) where one inbox row has `Tracked: #N` pointing at a CLOSED issue; verify the bridge DOES flag it as type 1. |
| AC6 (per-conflict prompts + diff preview before `gh issue` call) | Manual scenario | Trigger any type-1 conflict (the `#60` cases on current data work). At the per-conflict prompt, pick `update issue`. Verify: (a) diff preview rendered (current `CLOSED` → proposed `OPEN`); (b) explicit yes/no confirmation prompt; (c) only after user confirms does `gh issue reopen 60` execute. Mechanically: `rg "gh issue reopen\|gh issue close" <run-log>` shows the call AFTER the confirmation acknowledgment in the log timeline. |
| AC7 (pagination watchdog still active; bridge never sees truncated map) | Manual scenario via skill-code edit | Lower `--limit 500` to `--limit 60` in `.claude/agents/triage-runner.md` Phase 4 (or via a fixture-injected override); current gh corpus is 64 issues (verified via `gh issue list --state all --json number --limit 500 \| jq length`); 64 ≥ 0.9 × 60 = 54 ⇒ watchdog fires. Verify: B's verbatim WATCHDOG message rendered; `/triage` halts; no `gh issue close` / `reopen` / `edit` calls in the run log; no md writes (`git diff ai-docs/deferred/ -- .` returns empty). Restore the `--limit` after verification. |

### Manual-scenario fixtures already in repo (re-used, no new fixture for AC5–AC7)

- `tests/fixtures/process-improvements/triage-decline.md` — schema reference for thematic-shape row (no bridge-specific scenario, but documents the shape).
- `tests/fixtures/process-improvements/widget-backlog-promotion.md` — widget-backlog shape + prose-hit anchor reference.
- `tests/fixtures/process-improvements/triage-inbox-3rows.md` — `_inbox.md` shape; can be extended in-place for AC5 scenario B (one row gets `Tracked: #N` pointing at a synthetic CLOSED issue) without breaking the existing AC5 scenario in B.

## Open questions

- **Should the bridge surface orphan `#N` refs (cell cites an issue not in
  the bulk-list map — e.g. issue was hard-deleted from the project) as a
  fourth conflict type, or keep them in a diagnostics-only block?** The
  spec does not mention orphan refs. Defensible default: diagnostics-only
  block in the bridge sub-section (as specified above); promote to a
  fourth conflict type via Design Amendment if orphan refs surface in
  practice. (Today's data: zero orphans verified — every `#N` in
  `ai-docs/deferred/*.md` resolves via `gh issue view <N>`.)

- **Confirm widget-backlog `Notes`-cell `tracked: #N —` anchor regex.** The
  Phase 4.5 iteration step 1 needs an exact regex to extract `#N` from
  widget-backlog `Notes` cells. Defensible default: `\btracked:\s*#(\d+)\b`
  anchored at start-of-cell (so `... | tracked: #42 — foo` matches but
  free-text prose containing `tracked: #42` mid-cell does not — same rule
  as B's column-header anchor). Resolve at implementation time; design
  phase does not block on regex syntax.

- **Bridge sub-section placement in Phase 8's summary** (defensible default
  chosen above: after status table, before issues-created list) — confirm
  with reviewer if a different placement reads better. Cosmetic only;
  resolve via Design Amendment if needed.
