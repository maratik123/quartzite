# Fixture: TABLE shape (rule 2)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises parser rule 2 (`TABLE`): a markdown table whose first non-blank line is a `|`-prefixed header. The parser skips the header + separator rows and extracts each body row's column 1 (with column 2 appended via ` — `; columns 3+ ignored).

## Deferred

| What | Why | Separate issue needed? |
|------|-----|------------------------|
| Table item one | first reason | yes |
| Table item two | second reason | no |
| Table item three | third reason | folded into another issue |

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Fixture exists; parser emits 3 rows whose `Item` cells are `Table item one — first reason`, `Table item two — second reason`, `Table item three — third reason`. The third column is dropped. |
