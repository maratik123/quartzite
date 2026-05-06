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

### Output (both modes)

- **Recommendation:** title + link or file path + a 2–4 sentence rationale (scope, readiness, why now; in small mode, also why it counts as small and which larger work it sets up).
- **Runner-ups (2–3):** one line each, with the reason each ranked lower.
