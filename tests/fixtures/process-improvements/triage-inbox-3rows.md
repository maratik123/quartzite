# Fixture: _inbox.md drain (AC5)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises AC5: the `_inbox.md` drain step prompts the user **per-entry** (one prompt per row, not via the cell-iteration sweep), offering four actions (sort / promote / drop / keep). Three rows in distinct `Section` cells let a single dry-run exercise the three primary actions (sort row 1, promote row 2, drop row 3).

**Schema.** 4-column markdown table mirroring `ai-docs/deferred/_inbox.md` verbatim.

| Item | Source | Section | Tracked |
|------|--------|---------|---------|
| Synthetic out-of-scope inbox item awaiting drain | `tests/fixtures/process-improvements/triage-inbox-3rows.md` | out-of-scope | — |
| Synthetic deferred inbox item awaiting drain | `tests/fixtures/process-improvements/triage-inbox-3rows.md` | deferred | — |
| Synthetic open-question inbox item awaiting drain | `tests/fixtures/process-improvements/triage-inbox-3rows.md` | open-question | — |

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC5 | Dry-run `/triage` against this fixture: exactly **3** drain prompts (one per inbox row), **zero** cell-iteration-sweep prompts for these rows. Sort row 1 → row migrates to a chosen thematic file with cell 4 = `—`; removed from this fixture. Promote row 2 (approve) → row migrates to a chosen thematic file with cell 4 = `#N`; removed from this fixture. Drop row 3 → physically removed from this fixture. Final inbox-table row count = 0. |
