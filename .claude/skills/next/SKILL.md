---
name: next
description: "Recommend one task to work on next — an open GitHub issue or a ready plan from ai-docs/plans/INDEX.md — with rationale and 2–3 runner-ups. Pass `small` to limit to quick wins / groundwork that prepares the codebase for larger milestones."
argument-hint: "[small]"
disable-model-invocation: true
---

## Open GitHub issues

```!
gh issue list --limit 50 --state open --json number,title,labels,updatedAt
```

## Plan index

```!
cat ai-docs/plans/INDEX.md
```

## Deferred-file backlog (8 thematic files + widget-backlog)

The deferred store is canonical **JSONL** (one JSON object per line). Each block
below surfaces the *untracked candidates* for one file via a baked-in `jq`
one-liner — the candidate set is identical to the former markdown `cat` output
(see *Deferred-file rows* below for the field semantics).

Thematic files — untracked rows are `tracked=="—"`:

```!
jq -c 'select(.tracked=="—")' ai-docs/deferred/ci-docs-workflow.jsonl ai-docs/deferred/future-crates.jsonl ai-docs/deferred/macros-codegen.jsonl ai-docs/deferred/object-tree.jsonl ai-docs/deferred/properties.jsonl ai-docs/deferred/python.jsonl ai-docs/deferred/signals-slots.jsonl ai-docs/deferred/threading-runtime.jsonl
```

`widget-backlog.jsonl` carries two row kinds in one file — widget rows
(`kind=="widget"`, candidate when `emoji_status=="🟡 v2"`) and topic-area
thematic rows (no `kind`, candidate when `tracked=="—"`):

```!
jq -c 'select(.kind=="widget") | select(.emoji_status=="🟡 v2")' ai-docs/deferred/widget-backlog.jsonl
```

```!
jq -c 'select(.kind!="widget") | select(.tracked=="—")' ai-docs/deferred/widget-backlog.jsonl
```

## Task

Mode: `$ARGUMENTS` — if this is the literal string `small`, apply **small mode** below; otherwise apply **default mode**.

### Default mode (no argument)

Pick ONE item to recommend next from the issues and plans above.

Selection rules:
- Prefer plans marked 🟢 ready (no blockers in the "Blocked by" column).
- Prefer items that unblock the most other plans — consult the "Dependency order" section of `INDEX.md`.
- A time-sensitive GitHub issue (bug, regression, security) outranks a plan of comparable readiness.
- Skip items marked 🔴 blocked or 🟡 spec-only without a design.
- Skip GitHub issues carrying the `blocked` label (see *Blocked-issues label* below) — body text like "Blocked by: #N" is not visible here, so the label is the canonical signal.
- Skip GitHub issues carrying the `ui-design` label (see *UI-designer label* below) — work cannot proceed in this harness until the human designer hands back assets.

### Small mode (`/next small`)

Recommend ONE **small** item — the goal is to lay groundwork for upcoming larger milestones, not to start a milestone itself.

Selection rules:
- Prefer scope: bugfix, refactor, cleanup, docs polish, small dependency upgrade, or a single-crate change.
- Prefer items that unblock or de-risk a larger plan further down the dependency chain — consult the "Dependency order" section of `INDEX.md` and pick prerequisites of bigger blocked plans.
- Skip items marked 🔴 blocked or full-milestone plans (multi-crate, design-heavy).
- Skip GitHub issues carrying the `blocked` label (see *Blocked-issues label* below).
- Skip GitHub issues carrying the `ui-design` label (see *UI-designer label* below).
- 🟡 spec-only items qualify only if writing the design itself is the small task.
- If an issue bundles one small sub-item with larger ones, recommend it scope-narrowed to the small sub-item and call out that the issue should be split.

### Blocked-issues label

This skill fetches issues via `gh issue list --json number,title,labels,updatedAt` — labels are visible, **issue bodies are not.** A "Blocked by: #N" line in an issue body therefore has no effect on `/next`. The convention is:

- After opening or triaging a new issue that depends on another open issue, run `gh issue edit <N> --add-label blocked` (creating the label first via `gh label create blocked` if the repo doesn't have it yet).
- When the blocking dependency is resolved, run `gh issue edit <N> --remove-label blocked`.
- `/next` filters out any issue whose `labels` array contains `blocked` in both default and small modes.

### UI-designer label

Issues that need an out-of-harness designer pass (Figma asset, visual spec, `design-system/` work) carry the `ui-design` label (color `#E91E63`, description "Design-system designer pass / visual spec work required"). Like `blocked`, the label is the canonical signal because issue bodies are not visible to this skill. The convention is:

- When an issue is identified as needing a human designer pass, run `gh issue edit <N> --add-label ui-design` (the label already exists in this repo; created 2026-05-23).
- When the design-system assets land and the issue can proceed in-harness, run `gh issue edit <N> --remove-label ui-design`.
- `/next` filters out any issue whose `labels` array contains `ui-design` from Recommendation and Runner-ups in both default and small modes, and surfaces them in the *Candidates for UI-designer handoff (informational)* section instead.

### Deferred-file rows (8 thematic + widget-backlog)

The blocks above already filter to the candidate set via `jq`; this classification
documents the field semantics behind those filters (deferred store is JSONL —
`ci-docs-workflow.jsonl`, `future-crates.jsonl`, `macros-codegen.jsonl`,
`object-tree.jsonl`, `properties.jsonl`, `python.jsonl`, `signals-slots.jsonl`,
`threading-runtime.jsonl`, `widget-backlog.jsonl`):

1. **Tracked vs. untracked.** Two row kinds (`kind` key absent ⇒ thematic;
   `kind=="widget"` ⇒ widget):
   - **Thematic rows** (the 8 thematic files AND the no-`kind` topic-area rows in
     `widget-backlog.jsonl`) — field `tracked`: `#N` ⇒ tracked, `—` ⇒ untracked
     (the `jq 'select(.tracked=="—")'` filter above). Emoji-status legend (`🟡 v2`
     etc.): ✅ first pass / 🟡 v2 / 🤔 undecided / ❌ dropped / 📭 future.
   - **`widget-backlog.jsonl` widget rows** (`kind=="widget"`) — `emoji_status`
     `🟡 v2` ⇒ untracked candidate (the `jq 'select(.emoji_status=="🟡 v2")'`
     filter above); `notes` containing literal `tracked: #N` ⇒ tracked. Other
     `emoji_status` values (`✅` / `🤔` / `❌` / `📭`) ⇒ skip — not candidates.
2. **Double-recommendation guard.** If a tracked row's `#N` is already in the `gh issue list` candidate set, the deferred-file row is **not** re-listed as a separate item — at most one supplementary one-liner under that issue's recommendation cites the deferred row.
3. **`jq` filters the candidate set precisely.** The `jq` blocks above already
   exclude every non-candidate row (the former markdown prose hit `> spec.
   Tracked: TBD` is not a JSONL row at all, so it cannot leak in). Treat each
   emitted JSON object as one candidate; read `.item` (thematic) or `.widget`
   (widget) for the title and `.source_path` for the source citation.
4. **Output the *Candidates needing `/triage`* section.** Untracked rows surface in a new section titled **Candidates needing `/triage`** in the output (see *Output (both modes)* below). They are **never** the top-line recommendation or a runner-up — only listed for situational awareness, with a brief note that `/triage` ships in Issue B (#204); until then, the user can act on a candidate manually via `/interview`.

### Output (both modes)

- **Recommendation:** title + link or file path + a 2–4 sentence rationale (scope, readiness, why now; in small mode, also why it counts as small and which larger work it sets up).
- **Runner-ups (2–3):** one line each, with the reason each ranked lower.
- **Candidates needing `/triage` (informational):** any untracked rows from the deferred files. Title each row with the row's `.item` (thematic) or `.widget` (widget) text and cite the source via `.source_path` (thematic) or the originating file (`widget-backlog.jsonl`). **Items in this section are never the top-line recommendation or a runner-up** — they are listed for situational awareness only. End the section with a one-sentence reminder that `/triage` ships in Issue B (#204) and until then the user can act on a candidate manually via `/interview`.
- **Candidates for UI-designer handoff (informational):** any open GitHub issue whose `labels` array contains `ui-design`. Format each row as `#N — <title> (<link>): <one-line rationale>` — default rationale "needs design-system visual spec / designer pass" unless the issue body's first line gives a more specific cue. **Items in this section are never the top-line recommendation or a runner-up** — they are listed for situational awareness only. End the section with a one-sentence reminder that items here need an out-of-harness designer pass (Figma / `design-system/` folder) and unblock once the designer's PR lands and the `ui-design` label is removed. **Section always renders** — when zero issues carry the label, the body is the single line `No issues currently labelled \`ui-design\`.` (schema stability for `/next` consumers).
