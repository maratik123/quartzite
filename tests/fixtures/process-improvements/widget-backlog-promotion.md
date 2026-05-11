# Fixture: widget-backlog promotion (AC4)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises the AC4 behaviour: a `🟡 v2` row in a `widget-backlog.md`-shape table, when approved for promotion via `/triage`, has its `Notes` cell rewritten with `tracked: #N — <previous notes>` while the `Status` cell stays byte-identical. The prose-hit line below verifies the column-header anchor (it must NOT be classified as a candidate row).

## Status legend

- ✅ **first pass** — in scope of the umbrella widget issue
- 🟡 **v2** — deferred to a follow-up issue, definitely planned
- 🤔 **undecided** — design call needed before scoping (paradigm question)
- ❌ **dropped** — explicitly will not implement
- 📭 **future** — interesting but no decision; revisit when need surfaces

## Widgets

| Widget | Status | Notes |
|---|---|---|
| `RadioButton` | 🟡 v2 | needs button group abstraction |
| `Label` | ✅ first pass | text + alignment |
| `KeySequenceEdit` | 📭 future | shortcut capture widget; low priority |

> **Paradigm question — Model/View vs alternative.** Prose hit follows; the
> column-header anchor in `/triage`'s Phase 3 must reject this line as a
> candidate row.
>
> Tracked: TBD (file an issue when first item-view need surfaces).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC4 | Approve the `RadioButton` (🟡 v2) row for promotion ⇒ `Notes` cell rewritten to `tracked: #N — needs button group abstraction` (where `#N` is the issue number returned by `gh issue create`). `Status` cell `🟡 v2` byte-identical pre/post. The `Label` (✅ first pass) and `KeySequenceEdit` (📭 future) rows are NOT classified as candidates. The prose-hit `Tracked: TBD …` line is NOT classified as a candidate row. Table format intact. |
