# Fixture: BOLDBULLET shape (rule 5)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises parser rule 5 (`BOLDBULLET`): bulleted items beginning with a bolded leading term followed by an em-dash and prose. The parser keeps the bold formatting in the resulting `Item` cell.

## Out of scope

- **Term one** — explanation prose for the first item.
- **Term two** — explanation prose for the second item.
- **Term three** — explanation prose for the third item.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Fixture exists; parser emits 3 rows whose `Item` cells preserve the bolded leading term plus em-dash plus prose. |
