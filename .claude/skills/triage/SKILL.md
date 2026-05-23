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

- Phase 4 dedupe map summary (`{number → {state, title, labels, body}}` counts; the `labels` + `body` fields support the Phase 6.5 / Phase 7 UI-design gate's umbrella discovery + ranking — see `## Design-work classification gate` below).
- Phase 4.5 bridge classifications (type-1 / type-2 / type-3 lists + per-conflict user resolutions as they land).
- Phase 6 / Phase 7 candidate partitions (approve / decline / sort / promote / drop / keep — including any user-edited tweaks to the proposed split). Each per-row record carries a `design_link:` sub-field — `none` / `umbrella=#N` / `umbrella=#N (new)` / `skip-link` / `defer` — written immediately after the gate decision; on resume, rows already carrying the field are NOT re-prompted.
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
- **Single** bulk `gh issue list --state all --json number,state,title,labels,body --limit 500` query upfront; proposed titles are deduped against existing open + closed issues by exact title match. The map shape is `{number → {state, title, labels, body}}`. The `labels` + `body` fields are added so the Phase 6.5 / Phase 7 UI-design gate (see `## Design-work classification gate` below) can filter umbrellas by `state == "OPEN" ∧ "ui-design" ∈ labels` and rank them by keyword overlap against the umbrella `title + body` — all without a second `gh issue list` round-trip. The pagination watchdog at ≥ 0.9 × the limit and the "one bulk call per run" contract are preserved unchanged.
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

After the bulk `gh issue list` call and before the cell-iteration sweep, the subagent walks every `Tracked`-column ref across the 10 row sources (cell 4 in the 8 thematic files + `_inbox.md` **only** when the cell holds `#N`; the `Notes` cell in `widget-backlog.md` when the cell holds a `tracked: #N —` prefix) and looks up each `#N` in the local `{number → {state, title, labels, body}}` map built in Phase 4. The bridge consults only `state` + `title`; `labels` + `body` are inert here and serve the UI-design gate. `_inbox.md` rows whose `Tracked` cell = `—` are explicitly excluded — those route to the per-entry drain step.

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

## Design-work classification gate

Every row that reaches the Phase 7.5 `gh issue create` queue is first classified as **design-work** or **plain**, and design-work rows are linked to a `ui-design` umbrella before the create call runs. The classification fires per-row at the approval moment — **Phase 6** for sweep approvals and **Phase 7** for `_inbox.md` drain promotes — and is fully resolved BEFORE the row enters Phase 7.5. This ordering ensures a newly-created umbrella's `#N` is available to be cited in the row issue's `**Blocked by:** #N` body line within the same Phase 7.5 bulk pass.

**Classification trigger (Hybrid keyword + LLM auto-detect fallback).** Two branches:

- **Hit branch — keyword match.** A case-insensitive substring scan of the row's `Item` cell text concatenated with the row's `Source spec` filename against a hard-coded keyword list (the verbatim 23-entry list lives in `.claude/agents/triage-runner.md` adjacent to the gate code — see *Design-work classification keyword list* in that file). On a hit, the gate emits exactly one y/n confirm prompt of shape:

  ```
  Row matches design-work keyword `<hit>` — classify as design-work and require ui-design umbrella link? (y/n)
  ```

  `y` continues into the umbrella-selection prompt below; `n` marks the row plain and skips the gate.

- **No-hit branch — LLM auto-detect.** When the keyword scan returns no hits, the `/triage` orchestrator (Claude Code itself; `/triage` already runs inside Claude Code) reads (i) the row's `Item` cell text + (ii) the row's `Source spec` file (bounded to that ONE spec file, not the whole repo, not every spec in `ai-docs/plans/done/`, not any glob — see read-scope LIMIT below), infers a `DESIGN-WORK | PLAIN` classification with a one-line reason, and emits exactly one y/n confirm prompt with the inference as the default:

  ```
  Auto-classification: <DESIGN-WORK | PLAIN> (reason: <one-line summary>). Accept? (y/n)
  ```

  `y` honours the inference (continues to umbrella-selection if `DESIGN-WORK`; skips the gate if `PLAIN`); `n` flips to the other classification. No external API plumbing — uses the orchestrator's own LLM context.

  The `Source spec` link in the deferred row may appear in either of two forms — bare-path (`ai-docs/plans/done/<name>.spec.md`) or markdown-link (`[<text>](../plans/done/<name>.spec.md)`); both resolve to the same single file. The orchestrator reads exactly that one file (read-scope LIMIT: one row → one file). The inference output schema is exactly one token (`DESIGN-WORK` or `PLAIN`) plus a one-line reason (≤ 100 chars). See `.claude/agents/triage-runner.md` Phase 6.5 / Phase 7 gate section for the operational specification.

**Keyword list — dual role.** The same hard-coded 23-entry list in `.claude/agents/triage-runner.md` serves two roles: (i) **classification trigger** for the substring scan above, and (ii) **ranking signal** for the umbrella numbered menu — a shared keyword (case-insensitive substring hit on both the row's `Item` text AND a candidate umbrella's `title + body`) counts +1 toward that umbrella's overlap score, used to order the menu by descending score (ties broken by `#N` ascending). The full list lives only in `triage-runner.md` to keep the gate's classification logic and its menu-ranking logic in lock-step under a single source-of-truth.

**Umbrella discovery + ranking + numbered menu (design-work rows only).** Discovery uses the **Phase 4 dedupe map filtered to `state == "OPEN" ∧ "ui-design" ∈ labels`** — no new `gh issue list` round-trip. The menu shows ALL open `ui-design` umbrellas; ranking only affects ORDER, never inclusion (per AC11).

**Ranking computation.** For each candidate umbrella, the keyword-overlap score is `|{ kw ∈ KEYWORD_LIST : kw is substring of lowercase(row.Item_cell_text) ∧ kw is substring of lowercase(umbrella.title + " " + umbrella.body) }|` — a shared keyword (case-insensitive substring hit on BOTH the row's `Item` text AND the umbrella's `title + body`) counts +1; a shared NON-keyword token does not count.

**Ordering rule.** Sort by `score` **descending** (highest overlap first); tie-break by `#N` **ascending** (oldest umbrella first when scores tie). Deterministic across reruns.

**Menu render.** Numbered lines `1..N` in ranked order, each formatted `#<num> <title> — <truncated body summary>` (first 80 chars of the umbrella body's first non-empty paragraph; `…` if truncated; `<no body>` if empty), followed by three text options:

- **`new`** — create a new `ui-design` umbrella inline in the same Phase 7.5 pass.
- **`none`** — create the row's issue without any umbrella link.
- **`defer`** — skip creating this row's issue; return it to `_inbox.md` for a future `/triage` run.

**Numbered-pick branch — what happens.** When the user picks a number `i` (resolving to umbrella `#N`):

1. The chosen umbrella `#N` is captured as `link_to_umbrella` on the row's Phase 7.5 create-queue entry (persisted into the progress file's `design_link: umbrella=#N` field).
2. The row's drafted body has `**Blocked by:** #N\n\n` prepended at draft-construction time — so the child issue's body carries the back-reference from the moment `gh issue create` runs (the prefix is in the body string passed to `gh issue create`, NOT applied via a post-create edit).
3. After `gh issue create` returns the child `#C`, the labels `blocked` + `ui-design` are applied via `gh issue edit #C --add-label blocked --add-label ui-design`.
4. The umbrella `#N`'s body is fetched via `gh issue view #N --json body --jq .body`; under the `## Child issues (blocked on this epic)` anchor, a new bullet `- #<C> — <child-title>` is appended at the END of that section's bullet list (immediately before the next `## ` heading OR end-of-body if the section is final); the edited body is pushed back via `gh issue edit #N --body-file <tmpfile>`.

**Idempotency.** The umbrella body is scanned for the substring `#<C> ` (with a trailing-space sentinel to prevent `#54` matching `#549`) before the write — if `#<C> ` already appears under the anchor's section, the edit is a no-op (defensive against resume / re-promotion).

**Two distinct fallback sub-lists** (recorded separately in the Phase 8 run summary):

- **`Body-edit skipped — anchor absent`** — fires when the umbrella body has no `## Child issues (blocked on this epic)` substring (user-created umbrellas that diverge from the #539–#542 convention). Per-umbrella **structural state**: same on every `/triage` run until the umbrella body is hand-edited. A warning is emitted inline; the body edit is skipped; the umbrella is listed in Phase 8 under this sub-list with a one-line reminder.
- **`Body-edit failed — gh API error`** — fires when `gh issue edit #N --body-file <tmpfile>` returns non-zero (network error, rate limit, auth expiry, etc.). Per-run **transient state**: a re-run may succeed because the idempotency check (the `#<C> ` sentinel) is not yet satisfied. The child issue itself is already created with the back-reference; the umbrella is listed in Phase 8 under this sub-list with the captured `gh` stderr line so the maintainer can heal manually or via a re-run.

**Text-option branches.**

- **`new`** — two follow-ups (`Umbrella title:` / `Umbrella body:`) collect the new umbrella; an `umbrella`-kind entry enters the same Phase 7.5 queue with the `ui-design` label. The queue partitions `[umbrellas..., children...]` so each new umbrella's `#N` is known before its dependent child is created; the child then runs the numbered-pick branch's body-prefix + labels + umbrella body edit against that `#N`.
- **`none`** — the row's issue is created normally (no labels, no `**Blocked by:**`, no umbrella body edit). Recorded in Phase 8 *Design-link outcomes* as "design-work issue without umbrella link".
- **`defer`** — no `gh issue create` runs this run; the row is returned to `_inbox.md` (or left there). Recorded in Phase 8 as a deferred row.

See `.claude/agents/triage-runner.md` Phase 6.5 / Phase 7 gate section for the operational specification of the umbrella prompt, the body-edit machinery, and the progress-file `design_link:` audit trail.

## Run-output summary

At the end of every `/triage` run the subagent emits:

- Status table covering all 10 row sources with candidate counts (before / after).
- List of issues created (`#N` + one-line title each).
- List of rows declined (file path + `Item` cell content).
- List of inbox actions taken (sort / promote / drop, with destination thematic file when applicable).
- Concurrent-edit aborts (if any), listing the affected files + diff snippet.
- `ai-docs/deferred-items.md` row-count diff.

Context from user (if any): $ARGUMENTS
