# Fixture: mangled section (AC4)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This is the **AC4 fixture**: the *Deferred* section contains content that fails every parser rule (no bullets, no table, no NONE sentinel). The parser must emit one warning, zero rows, and Step 12 must complete normally.

## Deferred

> A blockquote-style narrative that the parser cannot classify.
> Multiple paragraphs of free prose without any structural markers.
> No leading hyphen, no pipes, no bold-leading term, not a NONE sentinel.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Fixture exists; parser emits exactly one `WARN:` line citing this fixture's path and the `Deferred` heading; no row is appended to `_inbox.md` for this section; Step 12 completes successfully. |
