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

```!
cat ai-docs/deferred/ci-docs-workflow.md
```

```!
cat ai-docs/deferred/future-crates.md
```

```!
cat ai-docs/deferred/macros-codegen.md
```

```!
cat ai-docs/deferred/object-tree.md
```

```!
cat ai-docs/deferred/properties.md
```

```!
cat ai-docs/deferred/python.md
```

```!
cat ai-docs/deferred/signals-slots.md
```

```!
cat ai-docs/deferred/threading-runtime.md
```

```!
cat ai-docs/deferred/widget-backlog.md
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

### Small mode (`/next small`)

Recommend ONE **small** item — the goal is to lay groundwork for upcoming larger milestones, not to start a milestone itself.

Selection rules:
- Prefer scope: bugfix, refactor, cleanup, docs polish, small dependency upgrade, or a single-crate change.
- Prefer items that unblock or de-risk a larger plan further down the dependency chain — consult the "Dependency order" section of `INDEX.md` and pick prerequisites of bigger blocked plans.
- Skip items marked 🔴 blocked or full-milestone plans (multi-crate, design-heavy).
- Skip GitHub issues carrying the `blocked` label (see *Blocked-issues label* below).
- 🟡 spec-only items qualify only if writing the design itself is the small task.
- If an issue bundles one small sub-item with larger ones, recommend it scope-narrowed to the small sub-item and call out that the issue should be split.

### Blocked-issues label

This skill fetches issues via `gh issue list --json number,title,labels,updatedAt` — labels are visible, **issue bodies are not.** A "Blocked by: #N" line in an issue body therefore has no effect on `/next`. The convention is:

- After opening or triaging a new issue that depends on another open issue, run `gh issue edit <N> --add-label blocked` (creating the label first via `gh label create blocked` if the repo doesn't have it yet).
- When the blocking dependency is resolved, run `gh issue edit <N> --remove-label blocked`.
- `/next` filters out any issue whose `labels` array contains `blocked` in both default and small modes.

### Deferred-file rows (8 thematic + widget-backlog)

Apply this classification to every row in the deferred files surfaced above (`ci-docs-workflow.md`, `future-crates.md`, `macros-codegen.md`, `object-tree.md`, `properties.md`, `python.md`, `signals-slots.md`, `threading-runtime.md`, `widget-backlog.md`):

1. **Tracked vs. untracked.** Schemas differ between the two file kinds:
   - **8 thematic files** (`signals-slots.md`, `properties.md`, `macros-codegen.md`, `object-tree.md`, `threading-runtime.md`, `future-crates.md`, `ci-docs-workflow.md`, `python.md`) — column 4 (`Tracked`): `#N` ⇒ tracked, `—` ⇒ untracked.
   - **`widget-backlog.md`** — `Status` column emoji `🟡 v2` ⇒ untracked candidate; `Notes` cell containing literal `tracked: #N` ⇒ tracked. Other emojis (`✅` / `🤔` / `❌` / `📭`) ⇒ skip — they are not in the candidate set at all.
2. **Double-recommendation guard.** If a tracked row's `#N` is already in the `gh issue list` candidate set, the deferred-file row is **not** re-listed as a separate item — at most one supplementary one-liner under that issue's recommendation cites the deferred row.
3. **Anchor on column-header context, not bare substrings.** The string `Tracked` appears once as prose in `widget-backlog.md` (`> spec. Tracked: TBD (file an issue when first item-view need surfaces).`). **Do not** treat this as a row — only rows inside an actual table count. Apply the same anchor for any future prose hits in other deferred files.
4. **Output the *Candidates needing `/triage`* section.** Untracked rows surface in a new section titled **Candidates needing `/triage`** in the output (see *Output (both modes)* below). They are **never** the top-line recommendation or a runner-up — only listed for situational awareness, with a brief note that `/triage` ships in Issue B (#204); until then, the user can act on a candidate manually via `/interview`.

### Output (both modes)

- **Recommendation:** title + link or file path + a 2–4 sentence rationale (scope, readiness, why now; in small mode, also why it counts as small and which larger work it sets up).
- **Runner-ups (2–3):** one line each, with the reason each ranked lower.
- **Candidates needing `/triage` (informational):** any untracked rows from the deferred files. Title each row with the row's `Item`-cell text and cite the source file. **Items in this section are never the top-line recommendation or a runner-up** — they are listed for situational awareness only. End the section with a one-sentence reminder that `/triage` ships in Issue B (#204) and until then the user can act on a candidate manually via `/interview`.
