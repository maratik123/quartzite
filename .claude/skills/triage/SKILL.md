---
name: triage
description: "Batched promotion of untracked rows to gh issues; drains _inbox.md; reconciles md ↔ gh issue divergence via the bridge sweep. Default threshold ≥ 3 unhandled rows."
argument-hint: "[N — override default threshold]"
disable-model-invocation: true
allowed-tools: Bash(gh issue create *) Bash(gh issue edit *) Bash(gh issue close *) Bash(gh issue reopen *) Bash(gh issue list *) Bash(gh issue view *) Bash(gh api *) Bash(grep *) Bash(rg *) Read Edit
---

Launch the `triage-runner` subagent. The subagent reads `.claude/agents/triage-runner.md` for full instructions.

## Progress file

A multi-turn `/triage` run persists state to `ai-docs/triage/triage-YYYY-MM-DD.progress.md` (local-only / gitignored under `/ai-docs/triage/**/*.progress.md`). The file mirrors the canonical schema at `ai-docs/templates/progress-format.md` and stores:

- Phase 4 dedupe map summary (`{number → {state, title}}` counts).
- Phase 4.5 bridge classifications (type-1 / type-2 / type-3 lists + per-conflict user resolutions as they land).
- Phase 6 / Phase 7 candidate partitions (approve / decline / sort / promote / drop / keep — including any user-edited tweaks to the proposed split).
- `## Next action` — the phase the next subagent invocation should resume from.

Lifecycle (mirrors `/task` and `/pr-commented` progress files):

- **Created** by `triage-runner` at Phase 1.5 (after the branch check, before threshold gate). If the file already exists on the current branch when the subagent starts, it is read at Phase 1 and the run resumes from `## Next action` instead of restarting from scratch.
- **Extended** by the subagent as each phase produces durable state (dedupe map, classifications, partitions, per-conflict resolutions).
- **Deleted** by `triage-runner` after Phase 8's run summary emits successfully — same shape as `/pr-merged`'s `scripts/cleanup-progress.sh` mechanic for `/task` / `/pr-commented` files.

Subagent context isolation makes classification state unrecoverable across invocations unless persisted; the progress file is what makes a `/triage` run resumable across compaction or fresh-subagent spawn.

## Trigger and threshold

Default threshold is **≥ 3 unhandled rows** across the 10 row sources. Tunable via `/triage [N]` — passing `N` overrides the default. Below the threshold the subagent exits with a brief status report; no approval prompt opens.

"Unhandled" counts rows with `Tracked` = `—` across the 8 thematic files + `_inbox.md`, plus `🟡 v2` rows in `widget-backlog.md`. `_inbox.md` rows count individually toward the threshold.

Manual invocation always proceeds regardless of threshold — the `[N]` argument can explicitly raise *or* lower the gate (e.g. `/triage 1` drains anything; `/triage 100` forces the threshold to skip a small batch).

## Cell-iteration sweep

The subagent walks the 8 thematic files (`signals-slots.md`, `properties.md`, `macros-codegen.md`, `object-tree.md`, `threading-runtime.md`, `future-crates.md`, `ci-docs-workflow.md`, `python.md`) + `widget-backlog.md`. `_inbox.md` is **NOT** in this sweep — its rows are handled per-entry in the drain step below.

- Candidates: `Tracked` cell = `—` (thematic files) or `Status` = `🟡 v2` (widget-backlog). The `Tracked` column header anchors classification — bare `Tracked:` substrings in prose are ignored (`widget-backlog.md:89` is the canonical example).
- For each candidate, the subagent drafts a title + body from the row's `Item` cell text and the linked `Source` spec.
- **Single** bulk `gh issue list --state all --json number,title --limit 500` query upfront; proposed titles are deduped against existing open + closed issues by exact title match. Pagination watchdog at ≥ 0.9 × the limit warns the user.
- The subagent presents the full batch as a table; the user approves a subset (per-row decisions, but in one table).
- All approved creates from the sweep AND from the drain step's *promote* action are collected and run together in a single contiguous `gh issue create` pass at the end of the run (the spec's "one bulk call" contract).
- On approval: `#N` is written into the appropriate cell — cell 4 (`Tracked`) for thematic files, or `Notes` cell prepended with `tracked: #N — ` for widget-backlog.
- On decline: an implicit-by-decline write — `untracked` into cell 4 (thematic) or `untracked (declined YYYY-MM-DD): <prev>` into `Notes` (widget-backlog). Single user action per row; no separate write confirmation.

## `_inbox.md` drain

`_inbox.md` rows are handled per-entry — **not** routed through the cell-iteration sweep above (drain is canonical to avoid double-handling).

One prompt per row, four actions:

- **sort** — remove row from `_inbox.md`; append to a user-chosen thematic file (numbered menu) with cell 4 = `—`. The row remains untracked at the thematic-file level and can be promoted on a future `/triage` run via the standard sweep.
- **promote** — queue the row into the same combined `gh issue create` pass as the sweep; on approval the row migrates to a user-chosen thematic file with `#N` in cell 4; on decline migrates with `untracked` in cell 4. Either way the row leaves `_inbox.md`.
- **drop** — physically remove the row from `_inbox.md`. No migration. Reserved for legitimately-bad rows (wrong shape, duplicate that dedupe missed, etc.). Distinct from `untracked`, which records legitimate review-and-decline.
- **keep** — leave the row in `_inbox.md` unchanged for a later `/triage` session.

## Bridge

After the bulk `gh issue list` call and before the cell-iteration sweep, the subagent walks every `Tracked`-column ref across the 10 row sources (cell 4 in the 8 thematic files + `_inbox.md` **only** when the cell holds `#N`; the `Notes` cell in `widget-backlog.md` when the cell holds a `tracked: #N —` prefix) and looks up each `#N` in the local `{number → {state, title}}` map built in Phase 4. `_inbox.md` rows whose `Tracked` cell = `—` are explicitly excluded — those route to the per-entry drain step.

Three conflict types reported (no silent overwrite — every type-1 and type-2 conflict surfaces a diff and asks the user):

- **Stale tracked.** Row's `Tracked` cell holds `#N` and the map reports that issue is CLOSED. Canonical example: the `#60` references in `ci-docs-workflow.md`.
- **Status mismatch.** `widget-backlog.md` row's `Status` cell = `✅` but the linked `#N` issue is OPEN. (Thematic files have no `Status` column; this direction does not fire for them — the reverse case, thematic-file row with `#N` that closed-as-not-planned, folds into stale-tracked.)
- **Untracked candidate count.** Row's `Tracked` cell = `—`. Reported as a count for situational awareness only — these rows are already handled by the cell-iteration sweep (thematic + widget-backlog) and the `_inbox.md` drain step.

For each detected type-1 or type-2 conflict, the user picks one of three actions (per-conflict prompt, mirroring the drain step's per-entry shape):

- **`update md`** — rewrite the md cell to reflect gh state. Type-1 rewrites leave `#N` in place and append ` (closed)` inline; type-2 rewrites the widget-backlog `Status` cell to a non-done status the user picks from a follow-up prompt (default `🟡 v2`). Concurrent-edit guard (content-snapshot, not mtime) inherited from the cell-iteration sweep.
- **`update issue`** — close or reopen the gh issue to match the md row. Before any `gh issue close` / `gh issue reopen` call, the bridge surfaces a diff preview (current state → proposed state) and requires explicit user confirmation. The bridge **never** silently rewrites issue state or body.
- **`keep both`** — record the divergence in the run output with a user-supplied reason; make no mutation. The conflict re-surfaces on the next `/triage` run.

Issues that exist in `gh` but have no md row anywhere are explicitly **not** flagged — asymmetric drift is by design.

The bridge appends a sub-section to the run-output summary listing every conflict, its type, the user's resolution, and any `gh issue close` / `gh issue reopen` calls made. See `.claude/agents/triage-runner.md` Phase 4.5 for the operational specification.

## Run-output summary

At the end of every `/triage` run the subagent emits:

- Status table covering all 10 row sources with candidate counts (before / after).
- List of issues created (`#N` + one-line title each).
- List of rows declined (file path + `Item` cell content).
- List of inbox actions taken (sort / promote / drop, with destination thematic file when applicable).
- Concurrent-edit aborts (if any), listing the affected files + diff snippet.
- `ai-docs/deferred-items.md` row-count diff.

Context from user (if any): $ARGUMENTS
