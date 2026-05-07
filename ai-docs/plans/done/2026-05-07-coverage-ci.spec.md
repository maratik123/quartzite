# CI: Code coverage reporting via cargo-llvm-cov + Codecov

**Source:** issue #134
**Date:** 2026-05-07
**Tracked in:** #134

## Scope

- New `.github/workflows/coverage.yml` with a single `coverage` job
- Runs on push to `master` and on PRs targeting `master`
- Platform: `ubuntu-latest` only (coverage instrumentation is platform-independent)
- Install `cargo-llvm-cov` via `taiki-e/install-action`
- Run `cargo llvm-cov --workspace --lcov --output-path lcov.info --doctests`
- Exclude `examples/` from coverage (no test logic there)
- Upload via `codecov/codecov-action@v5` with `token: ${{ secrets.CODECOV_TOKEN }}`
- New `codecov.yml`: `coverage.status.project.target: auto`, `threshold: 1%`, `informational: true` (no auto-fail), comment-only mode
- Codecov badge added to `README.md` under existing badges

## Out of scope

- Multi-OS coverage (Windows/macOS) — duplicate work, ubuntu-only is sufficient
- Auto-fail on coverage drop — comment-only, informational only
- Special proc-macro crate handling — `cargo-llvm-cov` v0.5+ handles proc-macros automatically
- Benchmarks, release workflow (tracked separately in #135, #136)

## Deferred

- Threshold tightening (below 1%) | after project matures | no new issue needed
- Per-crate coverage targets | not needed until more crates land | no new issue needed

## Key decisions

| Question | Decision |
|----------|----------|
| Separate file vs. add to ci.yml? | Separate `coverage.yml` — coverage is a distinct concern |
| Codecov or Coveralls? | Codecov — better Rust support, GitHub-native PR comments, free OSS tier |
| Run on push + PRs? | Both — push-to-master provides baseline; PRs need baseline for delta |
| Include doctests? | Yes (`--doctests`) — skipping under-counts public-API coverage |
| Include examples? | No (`--exclude quartzite-examples`) — runnable demos, no test logic |

## Technical constraints

- Requires `CODECOV_TOKEN` secret to be added to the repo settings before the job can upload
- `codecov/codecov-action@v5` is the current major version; pin to `v5` (floating tag, not a SHA)
- `cargo-llvm-cov` needs a nightly or stable compiler with llvm-tools; `taiki-e/install-action` handles this
- `coverage.yml` must not block merges (no branch protection rule depending on it); it is informational

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `coverage.yml` exists and defines a `coverage` job triggered on push to `master` and on `pull_request` targeting `master` |
| AC2 | Job installs `cargo-llvm-cov`, runs `cargo llvm-cov --workspace --lcov --output-path lcov.info --doctests`, and uploads `lcov.info` via `codecov/codecov-action@v5` |
| AC3 | `codecov.yml` exists with `coverage.status.project.target: auto`, `threshold: 1%`, and `informational: true` (no auto-fail on drop) |
| AC4 | `README.md` contains a Codecov badge linking to the project's Codecov page |
| AC5 | `ci.yml` is unchanged (coverage logic lives entirely in `coverage.yml`) |

## Open questions

_(none)_
