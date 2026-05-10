# Fixture: all three sections, single item each (AC2)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This is the **AC2 fixture**: one *Out of scope* item, one *Deferred* item, one *Open questions* item, each in a different shape. A single Step-12 run on this fixture must append exactly 3 rows to `_inbox.md` with `Section` cells `out-of-scope`, `deferred`, `open-question` in that order.

## Out of scope

- Plain bullet item describing something out of scope for the synthetic task.

## Deferred

- Deferred item example | because the design phase punted it

## Open questions

- Plain bullet question about a synthetic edge case.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Fixture exists; Step-12 dry-run appends exactly 3 rows with `Section` cells `out-of-scope`, `deferred`, `open-question` in that order. |
