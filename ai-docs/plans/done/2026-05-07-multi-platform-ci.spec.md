# Multi-platform CI runners (Windows + macOS)

**Source:** issue #133
**Date:** 2026-05-07
**Tracked in:** #133

## Scope

- Add `strategy.matrix.os: [ubuntu-latest, macos-latest, windows-latest]` to `build`, `test`, `clippy` jobs
- Set `runs-on: ${{ matrix.os }}` on those three jobs
- Set `strategy.fail-fast: false` on those three jobs
- Verify cache keys include `${{ runner.os }}` (already present — confirm no change needed)
- `format` job stays on `ubuntu-latest` only (rustfmt output is platform-independent; uses `cargo fmt`)

## Out of scope

- `docs` and `features` jobs — stay on `ubuntu-latest` only (not in issue scope)
- Rust toolchain matrix (stable only; toolchain matrix is a separate concern)
- Windows path-length / line-ending issues (file follow-up if they surface at runtime)

## Deferred

- `--no-default-features` cross-OS coverage | not in this issue | track separately if OS coverage needed

## Key decisions

| Question | Decision |
|---|---|
| Which jobs get the OS matrix? | `build`, `test`, `clippy` only — `format`, `docs`, `features` stay Linux |
| `fail-fast` behaviour | `false` — so a Windows-only failure doesn't cancel the macOS run |
| Cache partitioning | Per-OS via `${{ runner.os }}` in key — already present, no change |
| Toolchain matrix? | No — stable only, matching current shape |
| Formatter tool | `cargo fmt` (not `rustfmt` directly) |

## Technical constraints

- Single-file edit: `.github/workflows/ci.yml`
- YAML must pass `actionlint` (run locally if available; skip gracefully if not installed)
- After merge, CI must show 9 passing runs (3 jobs × 3 OSes)

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `build`, `test`, `clippy` jobs use `strategy.matrix.os: [ubuntu-latest, macos-latest, windows-latest]` and `runs-on: ${{ matrix.os }}` |
| AC2 | `format` job remains on `ubuntu-latest` only, unchanged |
| AC3 | `strategy.fail-fast: false` is set on each of the three matrix jobs |
| AC4 | Cache keys include `${{ runner.os }}` so per-OS caches don't cross-contaminate |
| AC5 | YAML is lint-clean (`actionlint` if available locally) |
| AC6 | `docs` and `features` jobs are not modified |

## Open questions

_None._
