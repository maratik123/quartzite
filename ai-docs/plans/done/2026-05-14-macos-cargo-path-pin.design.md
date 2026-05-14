# Design: Fix macOS `cargo` → `rustup-init` shadowing in CI

**Issue:** #340
**Date:** 2026-05-14
**Spec:** [`ai-docs/plans/2026-05-14-macos-cargo-path-pin.spec.md`](2026-05-14-macos-cargo-path-pin.spec.md)
**Branch:** `feat/2026-05-14-macos-cargo-path-pin`

## Approach

**YAML-only PR. No Rust source changes expected.** Swap `dtolnay/rust-toolchain@{stable,nightly}` for `actions-rust-lang/setup-rust-toolchain@v1` at every install site across the five workflow files (11 sites total), and let the new action's bundled `Swatinem/rust-cache` replace the explicit cache layers in `ci.yml`, `coverage.yml`, and `docs.yml`. All Round-1 / Round-2 / Round-3 decisions are pre-settled in the spec; this design pins the **mechanical** open questions raised in the spec's *Open questions* section.

### Why this approach

- `actions-rust-lang/setup-rust-toolchain@v1.16.1` (released 2026-05-08, registry-verified 2026-05-14) is maintained, sets `$CARGO_HOME/bin` on `PATH` correctly on macOS, and bundles `Swatinem/rust-cache@v2.9.1` so the toolchain-install + cache-restore order is owned end-to-end by one action. This eliminates the upstream `dtolnay/rust-toolchain` × `Swatinem/rust-cache` PATH-ordering interaction that surfaces as `error: unexpected argument 'clippy' found` on `macos-latest`.
- Configuring the bundled cache with `cache-shared-key:` reproduces today's per-job key topology with a single cache layer per job — no parallel `actions/cache@v5` + `Swatinem/rust-cache@v2` stack to keep in sync.
- The action's default `RUSTFLAGS=-D warnings` (spec Round-2 Q1: accept) unifies the `-D warnings` posture across `cargo build` / `test` / `bench` / `doc` / `llvm-cov`, matching the project's clippy-strict stance.

### Rejected alternatives

| Alternative | Why rejected |
|---|---|
| Surgical `PATH` pin via `echo "$HOME/.cargo/bin" >> $GITHUB_PATH` after the toolchain step | Treats the symptom, not the upstream-action interaction. Future macOS runner-image updates could break it again. Spec Round-1 Q1 settles this. |
| Swap only on macOS-bearing workflows (Linux-only files left alone) | Inconsistent toolchain-install posture across workflows; future macOS-lane additions to today's Linux-only jobs would re-introduce the bug. Spec Round-1 Q2 settles this. |
| Override `rustflags: ""` on the new action | Keeps non-uniform strictness (only clippy enforces `-D warnings` today). Spec Round-2 Q1 settles this — accept the default. |
| Pin `@v1.16.1` exact tag instead of `@v1` major-floating | Diverges from AGENTS.md Dependency Versions rules for GitHub Actions. Spec Key-decisions row settles this. Deferred (see spec). |

## Mechanical-question resolutions

### Q1: Does `coverage.yml` need explicit `components: llvm-tools-preview`?

**Answer:** **Add it explicitly.** Defensible default; removes implicit `rustup component add` round-trip during the job.

**Evidence:** Read `taiki-e/cargo-llvm-cov/src/context.rs` (lines 120–186 for the detection / self-install branch; lines 557–579 for `ask_to_run`). On CI, if `llvm-cov` / `llvm-profdata` are absent from `$sysroot/lib/rustlib/$host_triple/bin`, the binary auto-runs `rustup component add llvm-tools-preview --toolchain <toolchain>`. The CI-vs-interactive split is performed **inside** `ask_to_run` itself (`context.rs:557-579`): the function checks `env::var_os("CI").is_some() || env::var_os("TF_BUILD").is_some()` and, when set, skips the interactive prompt branch (running the command non-interactively and emitting an `info!()` line instead). The outer `ask` argument is computed earlier from `CARGO_LLVM_COV_SETUP` — there is no `ask=false_in_ci` parameter. Net effect on GitHub Actions: the self-install (`rustup component add llvm-tools-preview --toolchain <toolchain>`) runs non-interactively. This is *why* the current `dtolnay/rust-toolchain@nightly` workflow works without the component being declared: cargo-llvm-cov self-installs it.

The swap does not change this fallback (cargo-llvm-cov still has it), but declaring `components: llvm-tools-preview` on the new action moves the install to the toolchain step where it is faster (bundled in `rustup toolchain install --profile minimal`) and cached alongside the toolchain — instead of a separate `rustup component add` call after `taiki-e/install-action` puts the binary on PATH. **Net win: faster, deterministic, no behaviour change if it fails to install (the cargo-llvm-cov self-install path is the fallback safety net).**

### Q2: Concrete LOC ceiling for the latent-warning fix budget

**Ceiling: 30 LOC across the whole PR.** Default in the spec is `~20 LOC`; revised modestly upward because the `-D warnings` default now applies to `cargo build` / `test` / `bench` / `doc` / `llvm-cov` simultaneously — five attack surfaces vs. clippy alone today. Anything above 30 LOC, or anything that touches a public API, a non-trivial control-flow path, or a documented design decision, becomes a separate follow-up issue per AC11.

If the first feature-branch push surfaces zero warnings (best case — the project already passes `cargo clippy --workspace -- -D warnings` and `cargo doc -D warnings -D missing-docs`), no follow-up issues are needed.

### Q3: Does the bundled cache's `shared-key` reproduce the current `restore-keys:` partial-prefix behaviour?

**Answer: Yes, natively — but only if the user-provided `cache-shared-key:` does not duplicate the lockfile hash. Let the bundled cache append the lockfile hash to `cacheKey` itself; the shared-key stays bare and lockfile-agnostic, exactly reproducing today's `restore-keys:` semantics.**

**Evidence:** Read `Swatinem/rust-cache/src/config.ts` (~lines 71–140 for key construction; ~lines 227–245 for the Cargo.lock `[[package]]` parsing branch that appends the lockfile hash into `cacheKey`) and `src/restore.ts` (lines 1–60; the `restoreCache(paths, cacheKey, [restoreKey])` call is at `src/restore.ts:42-48`).

- The bundled cache constructs two keys:
  - **`cacheKey`** (full key) = `prefix-key + shared-key + OS-arch + rust-env-hash + workspace-files-hash + lockfile-package-hash`. The bundled action **already** parses `**/Cargo.lock`'s `[[package]]` entries and appends that hash into `cacheKey` internally.
  - **`restoreKey`** (partial-match prefix) = `prefix-key + shared-key + OS-arch + rust-env-hash` (no workspace / lockfile component).
- `restore.ts` calls `cacheProvider.cache.restoreCache(paths, cacheKey, [restoreKey])` — the third argument is an array of partial-match prefixes. This is exactly the semantics of `actions/cache@v5`'s `restore-keys:` field.
- **Therefore:** putting `hashFiles('**/Cargo.lock')` inside `cache-shared-key:` would make the `restoreKey` lockfile-hash-specific. Any `Cargo.lock` change → new `restoreKey` prefix → no partial-restore fallback (cache miss on every lockfile bump). Today's `actions/cache@v5` step in `coverage.yml` already separates `key: ${{ runner.os }}-cargo-coverage-${{ hashFiles('**/Cargo.lock') }}` from `restore-keys: ${{ runner.os }}-cargo-coverage-` precisely to allow lockfile-agnostic partial restore. The bundled cache reproduces this for free when the shared-key omits the hash — the lockfile hash flows only into `cacheKey`, not `restoreKey`.

**Recommended `cache-shared-key:` values per job** (carry the existing custom string as the shared-key payload, bare and lockfile-agnostic; the action's `v0-rust-` prefix + OS/arch suffix + rust-env hash are appended automatically, and the lockfile hash flows into `cacheKey` automatically without appearing in the shared-key):

| File | Job | `cache-shared-key:` value |
|---|---|---|
| `ci.yml` | `format` | omit — leave bundled cache at default key (small job, no preserved key topology to match) |
| `ci.yml` | `build` / `test` / `clippy` / `docs` | `${{ runner.os }}-stable` |
| `ci.yml` | `gpu-tests` | `${{ runner.os }}-stable-gpu` |
| `ci.yml` | `features` | `${{ runner.os }}-stable-features-${{ matrix.features }}` |
| `coverage.yml` | `coverage` | `${{ runner.os }}-cargo-coverage` (lockfile hash omitted — see footnote) |
| `docs.yml` | `build` | `${{ runner.os }}-cargo` (lockfile hash omitted — see footnote) |
| `base_benchmarks.yml` | `benchmark_base_branch` | omit — no current cache step; default key is fine |
| `fork_pr_benchmarks_run.yml` | `benchmark_fork_pr_branch` | omit — no current cache step; default key is fine |

**Footnote on the coverage / docs lockfile-hash split:** Today's `actions/cache@v5` configuration uses `key: ${{ runner.os }}-cargo-coverage-${{ hashFiles('**/Cargo.lock') }}` paired with `restore-keys: ${{ runner.os }}-cargo-coverage-` — i.e., the lockfile hash appears in the **save-key** but NOT in the **restore-prefix**, so `Cargo.lock` bumps still fall back to a partial restore. The bundled `Swatinem/rust-cache` mirrors this split internally: the parsed-lockfile hash is appended to `cacheKey` (save) but not to `restoreKey` (restore prefix). Putting `hashFiles('**/Cargo.lock')` inside `cache-shared-key:` would *additionally* bake the hash into `restoreKey`, regressing today's partial-restore behaviour — so the shared-key stays bare.

**`cache-save-if:` (today's `save-if: ${{ github.ref == 'refs/heads/master' }}`):** preserve for the six `ci.yml` jobs that currently set it (`build` / `test` / `clippy` / `gpu-tests` / `docs` / `features`). The new action's input is `cache-save-if:` (passed through to `Swatinem/rust-cache`'s `save-if:`). For `coverage.yml` and `docs.yml`, today's `actions/cache@v5` always saves on PR — preserve that posture by not setting `cache-save-if:` (default `true`).

## Decomposition

The PR is decomposed into 4 atomic subtasks. Each modifies a disjoint set of workflow files; tasks 1–3 are independent in principle but task 1 (`ci.yml`) is by far the largest and is sequenced first so reviewer can validate the pattern before it propagates.

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Swap `ci.yml`: replace 7 `dtolnay/rust-toolchain` install sites + drop 6 explicit `Swatinem/rust-cache@v2` steps; route `cache-shared-key:` + `cache-save-if:` through the new action per the table above. | `.github/workflows/ci.yml` | — |
| 2 | Swap `coverage.yml`: replace 1 `dtolnay/rust-toolchain@nightly` site (add `components: llvm-tools-preview`); drop the `actions/cache@v5` step; route the existing key string through `cache-shared-key:`. | `.github/workflows/coverage.yml` | — |
| 3 | Swap `docs.yml`: replace 1 `dtolnay/rust-toolchain@stable` site; drop the `actions/cache@v5` step; route the existing key string through `cache-shared-key:`. | `.github/workflows/docs.yml` | — |
| 4 | Swap `base_benchmarks.yml` + `fork_pr_benchmarks_run.yml`: replace 1 `dtolnay/rust-toolchain` install site each (no cache steps to drop; let bundled cache use default keys). | `.github/workflows/base_benchmarks.yml`, `.github/workflows/fork_pr_benchmarks_run.yml` | — |

**Gate after each task:** run `actionlint <file>` on the touched workflow(s) **before** `git add` (AGENTS.md AXIOM). Commits land one-per-task to keep the diff bisectable if the macOS lane regresses.

**Cross-cutting gate after all four tasks (before opening the PR):**

```bash
grep -rn "dtolnay/rust-toolchain" .github/workflows/   # must return empty (AC9)
grep -rn "Swatinem/rust-cache" .github/workflows/      # must return empty (AC10)
grep -rn "actions/cache@" .github/workflows/           # must return empty (AC10) — actions/cache is also gone
actionlint .github/workflows/ci.yml .github/workflows/base_benchmarks.yml \
           .github/workflows/coverage.yml .github/workflows/docs.yml \
           .github/workflows/fork_pr_benchmarks_run.yml   # exits 0 (AC1)
```

Decomposition is 4 tasks — well within the 7-task guideline; no split into multiple issues.

## Risks

- **Latent compiler warnings surface on first push.** `RUSTFLAGS=-D warnings` now also applies to `cargo build` / `test` / `bench` / `doc` / `llvm-cov`. Mitigation: 30-LOC fix budget for trivial / mechanical fixes in this PR; anything above becomes a follow-up issue per AC11. Worst case: re-run after a fix push.
- **Cache key topology changes silently.** The new action prepends `v0-rust-` and appends OS/arch + rust-env hash to whatever `cache-shared-key:` we set. **This means the first-run caches will all miss** (the cache keys are not byte-identical to today's `actions/cache@v5` / `Swatinem/rust-cache@v2` keys). Mitigation: expected, one-time cost — first feature-branch push rebuilds the cache from scratch; subsequent pushes hit. Document in the PR description.
- **`base_benchmarks.yml` is a benchmarking workflow; first run after the swap may be slower (cache cold) and trip Bencher's `--threshold-upper-boundary 0.99` t-test alarm.** Mitigation: the workflow runs on `push: branches: master` only — it does NOT trigger on the feature branch. Cache becomes warm on the first post-merge run. PR description notes the expected one-off slowdown.
- **`coverage.yml` is `pull_request` + `push` triggered.** The first feature-branch push *will* trigger it. If the bundled cache + `llvm-tools-preview` swap regresses coverage, AC6 fails. Mitigation: `actionlint` catches syntax errors; `cargo-llvm-cov` has its own `llvm-tools-preview` self-install fallback (verified above) — even if the explicit `components:` line is wrong, the job still runs.
- **`docs.yml` runs on `push: branches: master` only** — feature-branch push will NOT validate it. Mitigation: AC8 is best-effort verified by reading the post-swap YAML and confirming `RUSTDOCFLAGS=-D warnings -D missing-docs` is retained on the `cargo doc` step; final validation is the first post-merge master push.
- **`fork_pr_benchmarks_run.yml` triggers on PR open/synchronise.** Will run on the feature branch's PR open. Same risk profile as `coverage.yml`: bench cold-start is slower but unlikely to fail.
- **`actions-rust-lang/setup-rust-toolchain@v1` major-tag drift.** A future v1.x release could ship a regression. Mitigation: project convention (AGENTS.md Dependency Versions) favours major-floating for actions; spec records the pin choice + deferred fallback ("pin v1.16.1 if a future minor ships a regression").
- **`Unbork mac` step runs `brew install bash` unconditionally on macOS.** This adds ~10–20s to every macOS job invocation. Mitigation: accepted cost; the action requires bash 4+ for its later steps and macOS's bundled bash is 3.x. No project-side mitigation needed.

## Test Design

This is a CI/YAML PR. All ACs are validated by CI job outcomes on the feature-branch push and (for AC8) post-merge. No `#[cfg(test)]` modules apply. The validation plan maps AC → evidence:

| AC | Evidence source | Pass criterion |
|---|---|---|
| AC1 | `actionlint .github/workflows/<each-file>.yml` run locally **and** as the workflow-parse step on push | Exit code 0 for all 5 files |
| AC2 | GitHub Actions run summary for `Clippy (macos-latest)` on the feature-branch push | Green check; logs show `rustc <ver>` and `cargo clippy --workspace -- -D warnings` succeeded (no `rustup-init` error) |
| AC3 | Same run, `Test (macos-latest)` job | Green check; logs show `cargo test --workspace` succeeded |
| AC4 | Same run, aggregate `Clippy` / `Test` / `Build` / `GPU tests` / `Docs (build)` / `Feature matrix` / `Format` gate jobs | All green |
| AC5 | Ubuntu and Windows lanes of `Clippy` / `Test` / `Build` / `GPU tests` in the same run | All green; logs show no regression vs. master |
| AC6 | `Coverage` workflow run on the feature-branch PR | Green; Codecov upload step succeeds; logs show `cargo llvm-cov --workspace --lcov --output-path lcov.info --doctests` runs against nightly toolchain; `llvm-tools-preview` is installed as part of the `Install Rust toolchain` step (visible in `rustup toolchain install` output) |
| AC7 | `Run Benchmarks` workflow run on the feature-branch PR | Green; `cargo bench --workspace` runs; results uploaded as artifacts. `Base Benchmarks` evidence deferred to first post-merge master push. |
| AC8 | `Docs` workflow's `build` job — **NOT** triggered on PR. Validate by inspecting the post-swap `docs.yml` content (the `Build docs` step keeps `env: RUSTDOCFLAGS: "-D warnings -D missing-docs"` and the `Build docs` step runs `cargo doc --no-deps --workspace --all-features`). Final empirical validation: first post-merge master push. |
| AC9 | `grep -rn "dtolnay/rust-toolchain" .github/workflows/` locally and as part of the PR description checklist | Empty output |
| AC10 | `grep -rn "Swatinem/rust-cache\|actions/cache@" .github/workflows/` locally | Empty output |
| AC11 | If any latent-warning fix exceeds 30 LOC or touches a public API / non-trivial control-flow path, open a follow-up issue and link it in the PR description; do NOT include the fix in this PR | Issue created and linked, or no warnings to address |
| AC12 | Final PR description revision before merge | Description summarises action swap, RUSTFLAGS-default acceptance, bundled-cache strategy for ci/coverage/docs, links the green-run URLs that demonstrate AC2–AC8 (AC8 link added post-merge) |

### Local pre-push checklist

Before `git push` on the feature branch:

```bash
# 1. actionlint gate — AGENTS.md AXIOM
actionlint .github/workflows/ci.yml \
           .github/workflows/base_benchmarks.yml \
           .github/workflows/coverage.yml \
           .github/workflows/docs.yml \
           .github/workflows/fork_pr_benchmarks_run.yml

# 2. Absence-of-old-references gate
grep -rn "dtolnay/rust-toolchain" .github/workflows/ && echo FAIL || echo OK
grep -rn "Swatinem/rust-cache"    .github/workflows/ && echo FAIL || echo OK
grep -rn "actions/cache@"          .github/workflows/ && echo FAIL || echo OK

# 3. Spot-check: action version pin
grep -n "actions-rust-lang/setup-rust-toolchain" .github/workflows/*.yml
# Every match must be `@v1` — no `@v1.16.1`, no `@main`, no `@<sha>`
```

## Open questions

None remaining. The three spec-level open questions are settled in *Mechanical-question resolutions* above:

1. `coverage.yml` `components: llvm-tools-preview` — **add explicitly** (defensible default; faster than the cargo-llvm-cov self-install fallback).
2. Latent-warning LOC ceiling — **30 LOC**.
3. Bundled-cache partial-prefix restore — **native, no fallback needed** (verified against `Swatinem/rust-cache/src/{config,restore}.ts`).
