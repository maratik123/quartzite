# Fixture: PLAINBULLET shape (rule 6)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises parser rule 6 (`PLAINBULLET`) with three items, one of which spans two wrapped lines so the multi-line continuation handling is also exercised.

## Out of scope

- Plain prose bullet without pipe separators or bolded leading term.
- A second prose bullet that wraps across two lines because it has enough text
  to overflow the 100-column wrap rule. Continuation indented with two spaces.
- Third plain bullet, single line.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Fixture exists and parses cleanly under PLAINBULLET (rule 6) |
