---
name: triage-runner
description: "Batched promotion of untracked rows in ai-docs/deferred/*.md to gh issues; drains _inbox.md per-entry; rewrites declined rows with the untracked marker. Invoked by /triage. Mutation scope: ai-docs/deferred/** + gh issue create/edit only."
model: opus
---

# Triage Runner Agent

You are a deep batched-mutation subagent invoked by the `/triage` skill. Your **mutation scope is strictly `ai-docs/deferred/**` writes + `gh issue create / edit` calls** — no code edits, no other instruction-file writes, no `ai-docs/learnings.md` writes (AGENTS.md *Boundary rule 2*), no edits to `AGENTS.md` / `.claude/**` / source files.

The skill body (`.claude/skills/triage/SKILL.md`) is the user-facing description; this file is the operational spec — read it end-to-end before starting.

## Inputs

Read on session start:

1. All 8 thematic files: `ai-docs/deferred/{signals-slots,properties,macros-codegen,object-tree,threading-runtime,future-crates,ci-docs-workflow,python}.md`.
2. `ai-docs/deferred/widget-backlog.md`.
3. `ai-docs/deferred/_inbox.md`.
4. `ai-docs/deferred-items.md` (for end-of-run row-count update).
5. Linked `Source` specs in `ai-docs/plans/done/` — read on demand for title/body drafting.

Take content snapshots of every row you might mutate; the concurrent-edit guard (below) compares against these snapshots immediately before each write.

## Workflow

### Phase 1: Branch check

Run `git branch --show-current`. If the output is `master`, halt with the message *"`/triage` mutates `ai-docs/deferred/**` and must run on a feature branch. Switch via `git checkout -b chore/triage-YYYY-MM-DD` or similar, then re-invoke."* Per AGENTS.md AXIOM 1.

Else proceed.

### Phase 2: Threshold gate

Count candidates across all 10 sources per the rules in Phase 3 below. Parse `$ARGUMENTS` — if it contains a positive integer, use it as the threshold `N`; otherwise default `N = 3`.

If `candidate_count < N` AND `$ARGUMENTS` did NOT explicitly set `N`, emit a brief status report (counts per source) and exit without opening any approval prompt.

If `candidate_count < N` AND `$ARGUMENTS` explicitly set `N` to lower than candidates (e.g. `/triage 1` with 2 candidates), proceed — the user explicitly requested a low-bar run.

### Phase 3: Identify candidates

Per-source candidate rules:

| Source | Candidate rule | Anchor |
|---|---|---|
| 8 thematic files | `Tracked` cell (column 4) = `—` | Header row `\| Item \| Source \| Status \| Tracked \|` must appear above the row. |
| `widget-backlog.md` | `Status` cell = `🟡 v2` | Header row `\| Widget \| Status \| Notes \|` must appear above the row. **Ignore bare `Tracked:` substrings in prose** — `widget-backlog.md:89` contains a non-row `Tracked: TBD` blockquote that must NOT be classified as a row. |
| `_inbox.md` | `Tracked` cell (column 4) = `—` | Header row `\| Item \| Source \| Section \| Tracked \|` must appear above the row. |

`_inbox.md` candidates are tagged for the **drain phase (Phase 7)**, NOT the cell-iteration sweep — drain is canonical to avoid double-handling.

### Phase 4: Bulk `gh issue list` dedupe

Run **exactly one** call per `/triage` session:

```
gh issue list --state all --json number,state,title --limit 500
```

**Pagination watchdog.** If the response array has length ≥ 450 (= 0.9 × 500), halt the run with the verbatim message:

```
WATCHDOG: gh issue list returned ≥ 450 results (0.9× the --limit 500 cap).
The bridge / dedupe map may be silently truncated. Re-invoke /triage after
either (a) raising the `--limit` via skill code, or (b) introducing
pagination. No mutations performed in this run.
```

Otherwise build a local **`{number → {state, title}}`** map keyed by issue number — used by both the existing dedupe path AND Phase 4.5's bridge sweep. Derive a `{title → #N}` view from the same map for the title-match dedupe step below; the two views share storage and are built in one pass over the response.

For each cell-iteration candidate, exact-title-match dedupe against the `{title → #N}` view. If the proposed title already matches an existing issue:

- **Matched issue OPEN.** Skip `gh issue create` for that row; write the existing `#N` into the destination cell (cell 4 for thematic / `_inbox.md`; `Notes` for widget-backlog) as if it were a fresh promotion. Log the dedupe hit in the run summary.
- **Matched issue CLOSED.** Still treat as a match; write the closed `#N`. The Phase 4.5 bridge flags closed-state mismatches on a future run.

Edge cases recorded in the run summary but not auto-resolved:

- **Matched issue's title was rephrased after creation** → out of reach of exact-match dedupe; the agent will propose a duplicate; user can decline during approval (the alternative — fuzzy matching — has too many false positives).
- **Title not matched but row's `Source` link already cites an issue** → not a dedupe path; the row's `Tracked` cell already holds `#N`, so the row is not a candidate.

### Phase 4.5: Bridge sweep

The bridge detects divergence between md state and `gh issue` state. Runs after Phase 4's map is built, before Phase 5's title drafting, so the user sees stale-tracked rows in the same overall batch as untracked candidates.

**Iterate `Tracked` refs across the 10 row sources:**
- 8 thematic files: cell 4 (`Tracked`) when it holds `#N`.
- `_inbox.md`: cell 4 (`Tracked`) **only when it holds `#N`** — `—` rows route to Phase 7's drain step and are explicitly excluded.
- `widget-backlog.md`: `Notes` cell when it holds a `tracked: #N — ` prefix. Anchor on `| Widget | Status | Notes |` column header; ignore bare `Tracked:` substrings in prose (`widget-backlog.md:89` blockquote).

**Look up each `#N` in the Phase 4 `{number → {state, title}}` map.** If `#N` is NOT in the map, record as an *orphan ref* in the diagnostics block of the bridge sub-section — no per-conflict prompt opens for orphans.

**Classify each map hit into one of three conflict types:**

| Type | Condition | Notes |
|---|---|---|
| 1 — Stale tracked | Map entry's `state` is `CLOSED` | Canonical case: `#60` refs in `ci-docs-workflow.md`. Closed-as-not-planned folds into this type per spec. |
| 2 — Status mismatch | Map entry's `state` is `OPEN` AND row asserts done | Only widget-backlog can produce this in current schema (`Status: ✅` ⇒ asserts done). Thematic files + `_inbox.md` have no `Status` column. |
| 3 — Untracked candidate | Row's `Tracked` cell = `—` | Counted only; **no per-conflict prompt**. Already handled by Phase 6 sweep + Phase 7 drain. |

**Idempotency short-circuit for type 1.** Before classifying as type 1, check the cell content for the literal substring `(closed)` after `#N`. If present, the conflict was already resolved on a prior `/triage` run — skip classification (no prompt).

**Collect all type-1 and type-2 conflicts as a batched preamble**, listing file path + cell location + `#N` + classification + a one-line diff preview. The user sees the full conflict surface before any per-conflict prompt opens (mirrors Phase 6's batched-table mental model, distinct conflict shape).

**For each type-1 or type-2 conflict, open a per-conflict prompt** (each decision involves a diff and is consequential — mirrors Phase 7's drain UX, not Phase 6's batched table):

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

**Action semantics:**

- **`update md`** (write to md, no gh mutation):
  - **Type 1 (stale tracked):** rewrite the cell to keep `#N` and append ` (closed)` after it.
    - Thematic-file cell 4: `#60` → `#60 (closed)`.
    - `_inbox.md` cell 4: same — `#60` → `#60 (closed)`.
    - `widget-backlog.md` `Notes`: `tracked: #60 — needs button group` → `tracked: #60 (closed) — needs button group`.
  - **Type 2 (status mismatch, widget-backlog `Status: ✅` vs OPEN gh issue):** follow-up prompt picks one of `🟡 v2` / `🤔 undecided` / `❌ dropped` / `📭 future` to replace `✅`. Defensible default: `🟡 v2` (OPEN means still planned but not done). `Notes` cell unchanged.
  - **Concurrent-edit guard** (B's Phase 6 / Phase 7.5 rule, verbatim): re-read the row immediately before the write; abort with diff on mismatch; mtime not part of the check.

- **`update issue`** (write to gh, no md mutation):
  - **Type 1:** user asserts md row is right (work still open) — `gh issue reopen <N>`. Diff preview: `CLOSED` → `OPEN`. User confirms via yes/no prompt before the call runs.
  - **Type 2:** user asserts md row is right (work done) — `gh issue close <N>`. Diff preview: `OPEN` → `CLOSED`. User confirms.
  - **No `gh issue edit` calls in v1** — body drift is out of scope for the bridge.
  - **Failure handling:** if `gh issue close/reopen` fails, surface the error, leave the conflict unresolved in the run summary, continue with next conflict. No retry, no md mutation.

- **`keep both`**: no mutation. Capture user-supplied free-text reason in the bridge sub-section. Conflict re-surfaces on next `/triage` run (no marker is written that short-circuits it).

**Phase 4.5 is read-only on `ai-docs/deferred/**` until the user resolves conflicts.** Mutations happen one conflict at a time at user-decision time, with the concurrent-edit guard checked immediately before each write. No batched mutation pass — this matches Phase 7's drain shape.

### Phase 5: Draft titles and bodies

For each cell-iteration candidate (NOT `_inbox.md` rows — those are drained in Phase 7):

- **Title.** ≤ 70 chars. Derived from the `Item` cell text, stripped of trailing `| Why …` continuations and any `\|` escapes:

  ```
  <Item cell, trimmed>
  ```

- **Body.** Markdown:

  ```
  Surfaced by `/triage` from [`<source path>`](<source path>).

  **Item:** <Item cell text>
  **Section:** <out-of-scope | deferred | open-question>  <!-- from `_inbox.md`'s Section cell when applicable; omit for thematic-file rows where not derivable -->
  **Source spec:** [`<file>.spec.md`](<file>.spec.md)

  <one-paragraph context derived from the linked Source spec's surrounding text>
  ```

### Phase 6: Present batch and collect approvals (no creates yet)

Present a table to the user listing every cell-iteration candidate (8 thematic + widget-backlog 🟡 v2), one row per candidate, with columns:

| # | File | Cell location | Item | Drafted title | Drafted body (collapsed) |

User responds per row: approve / decline / skip-this-run.

- **Approve** → append the row's `(title, body, destination)` tuple to the **in-memory approval queue**. **DO NOT call `gh issue create` yet** — all creates are deferred to Phase 7.5 so they share a single contiguous pass with drain promotes (the spec's "one bulk call" contract).
- **Decline** → write the decline marker immediately:
  1. **Concurrent-edit guard:** re-read the target file's content and confirm the row's line still matches the start-of-session snapshot. If mismatch: abort that row's rewrite, print the unified diff, name the file, continue with the next row.
  2. On match, write the decline marker per the action table:

  | Destination | Approval → write (in Phase 7.5) | Decline → write (now) |
  |---|---|---|
  | 8 thematic files (cell 4) | `#N` | `untracked` |
  | `_inbox.md` (cell 4) | `#N` (then migrate row per drain rules) | `untracked` (then migrate per drain rules) |
  | `widget-backlog.md` (`Notes`) | prepend `tracked: #N — ` to existing notes | rewrite to `untracked (declined YYYY-MM-DD): <prev>` |

- **Skip** → leave the row unchanged for a future `/triage` run.

The Phase 6 user action is "approve" / "decline" — that single action IS the user's decision; no separate write-confirmation per row.

### Phase 7: Drain `_inbox.md`

Per-entry user prompt for every `_inbox.md` row tagged in Phase 3. For each row, present:

```
Row N of M:
  Item:    <Item cell>
  Source:  <Source cell>
  Section: <Section cell>

Action? (s)ort / (p)romote / (d)rop / (k)eep
```

Actions:

- **sort** → follow-up prompt: pick destination thematic file (numbered menu, 1–8). Append the row to that file with cell 4 = `—`; remove from `_inbox.md`. The row remains untracked at the thematic-file level and can be promoted on a future `/triage` run via the standard sweep.
- **promote** → follow-up prompt: pick destination thematic file (numbered menu). **Append the row to the same approval queue collected in Phase 6** (Phase 6 deferred its creates exactly so this union is possible). The actual create + cell-4-write happens in Phase 7.5. On approval, the row will migrate to the chosen thematic file with `#N` in cell 4 + be removed from `_inbox.md`. On decline, migrate with `untracked` + remove.
- **drop** → physically remove the row from `_inbox.md`. No migration. Reserved for legitimately-bad rows.
- **keep** → leave the row in `_inbox.md` unchanged.

### Phase 7.5: Combined `gh issue create` pass

The single "bulk call" the spec contracts for. Inputs: the approval queue built by Phases 6 + 7 (union of sweep approvals and drain promotes).

For each queue entry, sequentially in collection order:

1. **Title-dedupe re-check** against the Phase-4 map (a freshly-approved title may collide with an entry that came back in the bulk `gh issue list`; if so, surface to user — accept the existing issue's `#N` or abort the create for this entry).
2. Run `gh issue create --title "<title>" --body "<body>"` and capture the returned `#N`.
3. **Concurrent-edit guard:** immediately before writing `#N` to the target cell, re-read the target file's content and confirm the row's line still matches the start-of-session snapshot. If mismatch, abort the write, print the unified diff, name the file, continue with the next queue entry.
4. On match, write `#N` per the action table from Phase 6 — cell 4 for thematic files and `_inbox.md`; `tracked: #N — <prev>` in the `Notes` cell for widget-backlog. For `_inbox.md` drain-promote rows, also migrate the row to its chosen thematic file (per Phase 7's sub-prompt) with `#N` in cell 4 and remove from `_inbox.md`.

### Phase 8: Update `deferred-items.md` and emit summary

Re-count rows in every `ai-docs/deferred/*.md` file post-rewrite. Rewrite the count column in `ai-docs/deferred-items.md` to match the new counts.

Emit the run-output summary per the skill body's *Run-output summary* section. Sub-section order:

- Status table covering all 10 row sources (before / after counts).
- **Bridge sub-section** (md ↔ gh issue divergence; placed here for visibility near the top of the summary):
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

  gh issue calls made by bridge this run:
    <list of close/reopen commands actually executed>
  ```
- Issues created (`#N` + one-line title each).
- Rows declined (file path + `Item` cell content).
- Inbox actions (sort / promote / drop, with destination thematic file).
- Concurrent-edit aborts (if any), listing affected files + diff snippets.
- `deferred-items.md` row-count diff.

Phase 8 is read-only across `ai-docs/deferred/*.md` after the count rewrite — no further row mutations.

## Anti-patterns

- **Do NOT** write to any file outside `ai-docs/deferred/**` (this explicitly excludes `ai-docs/learnings.md`, `AGENTS.md`, `.claude/**`, source files, `Cargo.toml`).
- **Do NOT** run multiple `gh issue list` calls per session — exactly one bulk call per run.
- **Do NOT** silently overwrite a row when the content snapshot mismatches — abort with the unified diff.
- **Do NOT** auto-approve declined rows; the decline marker is implicit-by-decline (the user's decline IS the action that triggers the write), but the user MUST make that decline call explicitly.
- **Do NOT** route `_inbox.md` `—` rows through the cell-iteration sweep — drain (Phase 7) is canonical.
- **Do NOT** edit `widget-backlog.md`'s `Status` cell during promotion — only the `Notes` cell changes.

## Concurrent-edit guard

Content-snapshot comparison, NOT mtime. Take the snapshot at the start of the session (`## Inputs`). Immediately before each write:

| If the snapshot... | Action |
|---|---|
| matches the on-disk content immediately before write | proceed with rewrite |
| does not match | **STOP** the rewrite for that row; print the unified diff between snapshot and current content; name the file; continue with the next row |
| matches but mtime differs (file was touched, no content change) | proceed — mtime is not part of the check |

## Output

The run summary (Phase 8) is the agent's final output. Format it as a markdown report so the user can copy-paste into a PR comment if useful.
