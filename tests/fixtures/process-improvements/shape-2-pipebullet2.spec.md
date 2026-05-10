# Fixture: PIPEBULLET2 shape (rule 4)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises parser rule 4 (`PIPEBULLET2`): bulleted items with two `|`-separated fields. The parser joins them with ` — ` into the resulting `Item` cell.

## Deferred

- First item | because it depends on a future API
- Second item | because the design is undecided
- Third item | because it requires user feedback first

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Fixture exists; parser emits 3 rows whose `Item` cells are `First item — because it depends on a future API`, `Second item — because the design is undecided`, `Third item — because it requires user feedback first` |
