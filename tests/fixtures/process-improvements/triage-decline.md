# Fixture: triage decline markers (AC7)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises AC7: declining a thematic-shape row writes the literal token `untracked` into the `Tracked` cell; declining a widget-backlog-shape `🟡 v2` row prepends `untracked (declined YYYY-MM-DD): ` to the existing `Notes` cell. A second `/triage` dry-run on the mutated fixture must NOT re-propose either row (the marker tokens short-circuit candidate classification on subsequent runs).

## Section A — thematic shape

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Synthetic deferred item awaiting triage decision | `tests/fixtures/process-improvements/triage-decline.md` | deferred | — |

## Section B — widget-backlog shape

| Widget | Status | Notes |
|---|---|---|
| `SyntheticWidget` | 🟡 v2 | needs synthetic backing model |

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC7 | Two-step dry-run of `/triage` against this fixture: (1) decline both rows. (2) Re-run `/triage` on the mutated fixture. After step 1: Section A's `Tracked` cell = literal `untracked`; Section B's `Notes` cell = `untracked (declined 2026-05-11): needs synthetic backing model` (date is the day of the `/triage` run). After step 2: candidate list excludes both rows. Table format intact. |
