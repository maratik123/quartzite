# Fixture: per-section heterogeneity (mixed shapes)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture confirms the parser handles a single spec whose three sections each use a different shape — matches the dominant real-corpus pattern (e.g. `done/2026-05-09-paint-style.spec.md` uses PLAINBULLET / PIPEBULLET3 / PLAINBULLET across its three sections).

## Out of scope

- First plain bullet for a synthetic out-of-scope item.
- Second plain bullet for another synthetic out-of-scope item.

## Deferred

- Deferred mixed-shape item | because the surrounding sections use different shapes

## Open questions

- **Synthetic question** — explanation prose for a bolded-leading question item.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Fixture exists; parser emits 4 rows total — 2 from *Out of scope* (PLAINBULLET), 1 from *Deferred* (PIPEBULLET2), 1 from *Open questions* (BOLDBULLET). |
