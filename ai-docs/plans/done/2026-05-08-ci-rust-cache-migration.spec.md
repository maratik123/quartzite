# CI: migrate to Swatinem/rust-cache@v2; tune sccache size and cache-key strategy

**Source:** issue #183
**Date:** 2026-05-08
**Tracked in:** #183

## Scope

1. Replace `actions/cache@v5` with `Swatinem/rust-cache@v2` in 5 compile jobs:
   `build`, `test`, `clippy`, `docs`, `features`.
2. Configure rust-cache inputs:
   - `shared-key`:
     - `build`, `test`, `clippy`, `docs`: `${{ runner.os }}-stable`
     - `features`: `${{ runner.os }}-stable-features-${{ matrix.features }}`
   - `save-if: ${{ github.ref == 'refs/heads/master' }}`
   - All other inputs left at upstream defaults
     ([action.yml](https://github.com/Swatinem/rust-cache/blob/v2.9.1/action.yml)):
     `cache-targets: true`, `cache-bin: true`, `cache-all-crates: false`,
     `cache-workspace-crates: false`, `add-rust-environment-hash-key: true`,
     `cache-on-failure: false`, `prefix-key: v0-rust`,
     `env-vars: CARGO CC CFLAGS CXX CMAKE RUST` (default).
3. Preserve step ordering: `checkout` → toolchain install → sccache install
   (`mozilla-actions/sccache-action@v0.0.10`) → rust-cache → build/test/clippy/etc.
4. Add `SCCACHE_CACHE_SIZE: "2G"` to each compile job's per-job `env:` block
   (alongside the existing `SCCACHE_GHA_ENABLED` / `RUSTC_WRAPPER` from PR #182).
5. Sccache stays as-is — keep `SCCACHE_GHA_ENABLED: "true"` and
   `RUSTC_WRAPPER: "sccache"` from PR #182.

## Out of scope

- Changing PR #182's `RUSTC_WRAPPER` / `SCCACHE_GHA_ENABLED` env vars.
- Workflow-level `env:` block migration (samply variant).
- Authoring an internal composite action (Mozilla pattern).
- Removing sccache (Mozilla "rust-cache only" pattern).
- Touching non-compile jobs: `format`, `roadmap-sync`, and the `*-pass`
  aggregator jobs (`build-pass`, `test-pass`, `clippy-pass`, `features-pass`,
  `roadmap-sync-pass`).

## Deferred

(none — issue did not call out separate-issue candidates)

## Key decisions

| Question | Decision |
|---|---|
| shared-key strategy | Mozilla per-OS shared. One cache entry per OS+toolchain for `build`/`test`/`clippy`/`docs`; one per matrix entry for `features`. ~7 entries total. Matches `mozilla/actions/rust` composite pattern. Trade-off: jobs share `target/` on master save (last-writer-wins), accepted for low cache pressure. |
| AC4 verification depth | Structural only — `cache restored` / `cache miss` log line + sccache stats output observable in CI logs. Wall-clock comparison is informational, not pass/fail (CI timing too noisy for hard assertion). |
| Keep sccache | Yes — samply pattern, not Mozilla "either-or". Quartzite is active-dev with frequent Cargo.lock churn; rust-cache invalidates wholesale on lockfile change, sccache catches "lock changed but source bytes identical" compilation. |
| sccache cache size | `"2G"` per job (matches samply). Caps sccache so rust-cache has ~8G headroom within GHA 10G per-repo limit. |
| Step order around sccache install | `sccache-action` install BEFORE `rust-cache`. Functionally equivalent either way (sccache install is binary-only; doesn't read/write cargo state). Choosing "after sccache install" mirrors PR #182's existing pattern and Mozilla's composite action ordering. |
| `CARGO_INCREMENTAL=0` (set by rust-cache) | Accept. Documented behaviour; appropriate for CI; reduces cache size. No explicit `env:` override needed. |

## Technical constraints

- GHA per-repo cache limit is 10 GB total. Budget: ~2 GB sccache + ~8 GB
  rust-cache (3 OS × ~2 GB target/ + matrix entries). Eviction-churn risk
  exists but is bounded by `save-if: master only` (feature-branch runs read
  only; never write).
- rust-cache must run AFTER toolchain install — it reads the active rustc
  version into the cache key. Order is enforced by the step sequence in each
  job; no per-step gating needed.
- rust-cache automatically sets `CARGO_INCREMENTAL=0` for the build step.
  Workflow code does not need to set or unset this.
- Workspace crates themselves are NOT cached (only their dependencies).
  Workspace rebuilds every run. Acceptable for our small workspace.
- Cache key incorporates env-var values via the default `env-vars` input
  (`CARGO CC CFLAGS CXX CMAKE RUST` prefixes). `RUSTC_WRAPPER=sccache` value
  participates in the key — toggling sccache produces a separate cache entry,
  preventing cross-contamination.
- `actionlint` must pass on the modified workflow file (AGENTS.md gate).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `actions/cache@v5` is not referenced anywhere in `.github/workflows/ci.yml`. |
| AC2 | `Swatinem/rust-cache@v2` is used in jobs `build`, `test`, `clippy`, `docs`, and `features`, with `shared-key` and `save-if` configured per the Key Decisions table. |
| AC3 | `SCCACHE_CACHE_SIZE: "2G"` is set in the per-job `env:` block of each job that declares `RUSTC_WRAPPER` and `SCCACHE_GHA_ENABLED` (i.e. the same 5 compile jobs). |
| AC4 | On a CI run after the cache is populated (post-merge master push, or any subsequent PR/master run), each compile job's logs show: (a) a rust-cache `Cache restored …` or `Cache miss …` line, (b) sccache `Compile requests …` / `Cache hits …` stats output at the end of the build step (when at least one rustc invocation occurred). Wall-clock improvement is not a pass/fail condition. |
| AC5 | `actionlint .github/workflows/ci.yml` passes cleanly. |

## Verification protocol

The PR itself will see a cold cache (no master save yet). AC4 is verified by:

1. Inspecting the PR's CI logs for the structural markers (rust-cache "Cache
   miss" line; sccache stats present). Cold-run timing is not the test.
2. After merge, inspecting the master push's CI logs for the same markers
   (still cache miss; saves to cache on success).
3. On the subsequent CI run (next PR opened, or next master push), inspecting
   the logs for `Cache restored` (rust-cache) and elevated `Cache hits` count
   (sccache).

Step 3 is observed manually post-merge; it does not block this PR.

## Open questions

(none)
