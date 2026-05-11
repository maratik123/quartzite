---
name: triage
description: "Batched promotion of untracked rows to gh issues; drains _inbox.md; reconciles md ↔ gh issue divergence (bridge ships in Issue C). Default threshold ≥ 3 unhandled rows."
argument-hint: "[N — override default threshold]"
disable-model-invocation: true
allowed-tools: Bash(gh issue create *) Bash(gh issue edit *) Bash(gh issue list *) Bash(gh issue view *) Bash(gh api *) Bash(grep *) Bash(rg *) Read Edit
---

Launch the `triage-runner` subagent. The subagent reads `.claude/agents/triage-runner.md` for full instructions.

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

<!-- Issue C (#205) fills in this section with the md ↔ gh issue
divergence-detection workflow. Until C lands, `/triage` ships the
promotion + drain flows above; no drift detection runs. -->

_Not yet implemented. See [Issue #205](https://github.com/maratik123/quartzite/issues/205)._

## Run-output summary

At the end of every `/triage` run the subagent emits:

- Status table covering all 10 row sources with candidate counts (before / after).
- List of issues created (`#N` + one-line title each).
- List of rows declined (file path + `Item` cell content).
- List of inbox actions taken (sort / promote / drop, with destination thematic file when applicable).
- Concurrent-edit aborts (if any), listing the affected files + diff snippet.
- `ai-docs/deferred-items.md` row-count diff.

Context from user (if any): $ARGUMENTS
