# CI sccache layer for compiler-artefact caching

**Source:** issue #178
**Date:** 2026-05-08
**Tracked in:** #178

## Scope

Add [`mozilla-actions/sccache-action`](https://github.com/Mozilla-Actions/sccache-action) to every merge-gate job in `.github/workflows/ci.yml` that compiles Rust. sccache caches compiler output (object files / `rlib`s) keyed on source-content hash — complements the existing `actions/cache@v5` cargo cache, which caches the registry and `target/` keyed on `Cargo.lock` hash. Both layers stay; sccache hits when cargo decides to recompile but the source bytes haven't changed.

In-scope jobs (transitively required for merge per `master` branch-protection list — Format, Build, Test, Clippy, Docs, Feature matrix):

| Job | Matrix? | Compiles Rust? | Add sccache? |
|---|---|---|---|
| `build` | ubuntu / macos / windows | ✅ | ✅ |
| `test` | ubuntu / macos / windows | ✅ | ✅ |
| `clippy` | ubuntu / macos / windows | ✅ | ✅ |
| `docs` | ubuntu only | ✅ (`cargo doc`) | ✅ |
| `features` | matrix | ✅ | ✅ |

## Out of scope

- `format` job in ci.yml — runs `cargo fmt --check` only; no compilation, sccache wouldn't help
- `*-pass` aggregator jobs (`build-pass`, `test-pass`, `clippy-pass`, `features-pass`, `roadmap-sync-pass`) — pure `needs:`-wait jobs, no compile
- `roadmap-sync` — runs the gen-roadmap script, no Rust compile
- `coverage.yml` — does not gate merge; coverage instrumentation may interact with sccache in ways that need separate evaluation
- `docs.yml` (Pages-deploy workflow) — does not gate merge; rebuilds docs only on master push
- `base_benchmarks.yml`, `fork_pr_benchmarks_run.yml`, `fork_pr_benchmarks_track.yml` — don't gate merge; benchmark wall-time should be measured on a stable build, not a sccache-affected one
- Self-hosted sccache backend (S3 / Azure) — overkill at this CI volume
- Replacing `actions/cache@v5` with `Swatinem/rust-cache@v2` — separate concern, file as follow-up if desired

## Deferred

- `Swatinem/rust-cache@v2` migration | smarter cargo-side caching with better eviction | follow-up issue if observed gain warrants it
- Coverage workflow sccache integration | needs separate validation that sccache doesn't perturb coverage instrumentation | follow-up
- Self-hosted sccache backend | only worthwhile at much higher CI volume than this project has | re-evaluate if GHA cache exhaustion becomes an issue

## Key decisions

| Question | Decision |
|---|---|
| Which CI workflows get sccache? | Only `ci.yml` merge-gate compile jobs. Other workflows out of scope this PR. |
| Which sccache backend? | GHA-backed (default for `mozilla-actions/sccache-action`) |
| Failure mode if sccache breaks at runtime | Fail loud (default sccache-action behaviour) — do NOT add `continue-on-error`. Surfaces regressions immediately. |
| Formal Windows-runtime measurement (≥30% reduction)? | Dropped — ship and observe in GitHub Actions UI |
| Cache size tuning (`SCCACHE_CACHE_SIZE`)? | Leave default. Tune later if GHA repo cache (10 GB total) shows pressure. |
| Action version pinning | Per AGENTS.md § Dependency Versions: registry-query before pinning (`gh api /repos/Mozilla-Actions/sccache-action/releases --jq '.[0].tag_name'`); pin to live-current major. Verify Node-runtime currency on the action's `action.yml`. |

## Technical constraints

- **`actionlint` gate** — per AGENTS.md, every modified `.github/workflows/*.yml` must pass `actionlint` before staging. Required for this PR.
- **Cargo cache stays** — keep the existing `actions/cache@v5` block in each job. Don't delete or alter it. sccache layers under it: cargo cache covers `target/` and registry; sccache covers compiler output. Complementary.
- **Per-job placement** — sccache-action's `Run sccache-cache` step must run AFTER `dtolnay/rust-toolchain` and BEFORE the `actions/cache@v5` restore (so cargo cache restore sees `RUSTC_WRAPPER` already set if needed). The action sets `RUSTC_WRAPPER=sccache` automatically.
- **Matrix consistency** — same sccache version across all matrix entries (no per-OS overrides at this stage). Per-OS tuning is a follow-up if the data warrants it.
- **No CI volume increase** — adding sccache must not require a new workflow file or new job. Strict in-place modification of `ci.yml`.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | sccache action added to all merge-gate compile jobs in `ci.yml`: `build` (matrix), `test` (matrix), `clippy` (matrix), `docs`, `features` (matrix). Verified by reading the diff: every such job has a `mozilla-actions/sccache-action@<pinned>` step before its compile step. |
| AC2 | `actionlint .github/workflows/ci.yml` passes clean on the modified file (per AGENTS.md required-gate) |
| AC3 | sccache-action version pinned to the live-current major: tag pulled via `gh api /repos/Mozilla-Actions/sccache-action/releases --jq '.[0].tag_name'` at task time, not from training memory. Action version explicitly cited in the spec / PR body alongside the date verified. |
| AC4 | sccache stats visible in job logs on at least one PR-CI run — confirms the cache is wired up and the wrapper is active. (sccache-action emits stats automatically at end-of-job by default.) |
| AC5 | All required CI checks pass on the PR (Format, Build, Test, Clippy, Docs, Feature matrix). Fail-loud failure mode means a sccache misconfiguration would block merge — by design. |

## Open questions

- **Cache contention with existing 10 GB GHA repo cache** — current cargo cache + sccache cache + benchmarks cache + coverage cache may collectively exceed the GHA repo limit, causing eviction churn that erodes the win. Worth observing post-merge: if cache hit rates show high churn, tune `SCCACHE_CACHE_SIZE` per OS.
- **Does sccache benefit `cargo clippy`?** — clippy runs the full type-check / borrow-check pipeline; sccache caches up to and including codegen. Clippy may not actually drive codegen, in which case sccache hit rate on the clippy job will be lower than on build/test. Worth observing in stats.
- **`cargo doc` interaction** — `cargo doc` invokes `rustdoc`, not `rustc` directly. sccache's wrapping is on `rustc` via `RUSTC_WRAPPER`. doc may bypass sccache entirely. Acceptable if so — `docs` job is shorter than build/test anyway.
