# Fixture: bridge divergence cases (AC3)

**Source:** fixture
**Date:** 2026-05-11
**Tracked in:** none — synthetic test fixture for `tests/fixtures/process-improvements/`

This fixture exercises AC3: the `/triage` bridge (Phase 4.5 in
`.claude/agents/triage-runner.md`) reports md ↔ gh divergence in both
directions. The synthetic issue numbers `#9001` and `#9002` below are
**not** real gh issues — they are interpreted via the mock map at the
bottom of this file. Manual scenario: run `/triage` with the bridge
instructed to treat the mock map as authoritative for this fixture;
verify the bridge output cites both rows as conflicts.

## Section A — widget-backlog shape (Status: ✅ vs OPEN gh issue)

| Widget | Status | Notes |
|---|---|---|
| `SyntheticDoneWidget` | ✅ first pass | tracked: #9001 — synthetic note carrying tracked-ref |

## Section B — thematic-file shape (Tracked: #N → CLOSED gh issue)

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Synthetic deferred item linked to stale issue | `tests/fixtures/process-improvements/divergence-cases.md` | deferred | #9002 |

## Mock map (manual scenario input)

For this fixture, the `/triage` bridge treats the following
`{number → {state, title}}` map as authoritative (no live `gh` call):

| Issue | State  | Title |
|-------|--------|-------|
| #9001 | OPEN   | Synthetic open issue cited by widget-backlog row |
| #9002 | CLOSED | Synthetic closed-as-not-planned issue cited by thematic row |

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC3 | Dry-run `/triage` against this fixture (with the mock map injected): the bridge sub-section reports both rows as conflicts. Section A's `SyntheticDoneWidget` row surfaces as **type 2 (status mismatch)** because `Status: ✅` and `#9001` is OPEN. Section B's row surfaces as **type 1 (stale tracked)** because `Tracked: #9002` and `#9002` is CLOSED. The widget-backlog `tracked: #9001 —` prefix is correctly anchored on the `Notes` cell (column-header rule); no other row classification fires. |
