# Fixture: PIPEBULLET3 shape (rule 3)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises parser rule 3 (`PIPEBULLET3`): bulleted items with three `|`-separated fields. The parser keeps fields 1 + 2 (joined with ` — `) and drops field 3 (typically "Separate issue?" metadata).

## Deferred

- Item alpha | reason A | yes
- Item beta | reason B | no
- Item gamma | reason C | folded into another issue

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Fixture exists; parser emits 3 rows whose `Item` cells are `Item alpha — reason A`, `Item beta — reason B`, `Item gamma — reason C`. The third field is dropped. |
