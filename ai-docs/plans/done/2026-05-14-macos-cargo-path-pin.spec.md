# Fix macOS `cargo` → `rustup-init` shadowing in CI

**Source:** issue #340
**Date:** 2026-05-14
**Tracked in:** #340

## Problem statement

CI jobs running on `macos-latest` (`Clippy`, `Test`, and — by extension — any `dtolnay/rust-toolchain`-using job that targets macOS) fail with:

```
error: unexpected argument 'clippy' found
Usage: rustup-init[EXE] [OPTIONS]
```

The macOS runner's `cargo` binary on `PATH` resolves to `rustup-init` instead of the real rustup-managed cargo shim. The failure was occasional on PR #339 earlier runs (job `76008621587`, run `25866130248`); as of commit `ba52781` it is **constant** and has spread from `Clippy (macos-latest)` to `Test (macos-latest)` (run `25869160291`, jobs `76019385408` and `76020156434`). Re-running no longer unblocks. Ubuntu and Windows pass on the same commit. PR #339 is blocked by the required-checks gate until this is fixed.

Hypothesised cause: an interaction between `Swatinem/rust-cache@v2` (which can restore `.cargo/bin`) and the `dtolnay/rust-toolchain` install step leaves a stale or wrong cargo shim earlier on `PATH` than the freshly-installed rustup-managed one.

Resolution (per interview): replace the toolchain-install action across **every** workflow that installs Rust, switching to `actions-rust-lang/setup-rust-toolchain@v1` — a maintained action whose install logic is known to set the correct shim ordering on macOS, and which bundles `Swatinem/rust-cache` so cache + toolchain are managed end-to-end by one action.

## Scope

1. Replace `dtolnay/rust-toolchain@stable` (and `@nightly` in `coverage.yml`) with `actions-rust-lang/setup-rust-toolchain@v1` in every workflow file that installs a Rust toolchain.
2. Affected files (every toolchain-install site — five workflow files, 11 total install steps; counts verified 2026-05-14 via `grep -n "dtolnay/rust-toolchain" .github/workflows/*.yml`):
   - `.github/workflows/ci.yml` — 7 sites: jobs `format`, `build`, `test`, `clippy`, `gpu-tests`, `docs`, `features`.
   - `.github/workflows/base_benchmarks.yml` — 1 site: job `benchmark_base_branch`.
   - `.github/workflows/coverage.yml` — 1 site: job `coverage` (nightly).
   - `.github/workflows/docs.yml` — 1 site: job `build` (separate from `ci.yml`'s docs job — this one publishes to GitHub Pages).
   - `.github/workflows/fork_pr_benchmarks_run.yml` — 1 site: job `benchmark_fork_pr_branch`.
3. Preserve the `components:` selections at each call site:
   - `format` job (ci.yml) → `components: rustfmt`.
   - `clippy` job (ci.yml) → `components: clippy`.
   - `coverage` job (coverage.yml) → preserve the nightly toolchain channel; add `components: llvm-tools-preview` if currently relied on implicitly by `cargo llvm-cov` (verify in design — see Open questions).
   - All other sites → no components.
4. Pin the new action to the major-version floating tag `@v1` (live registry confirms latest is `v1.16.1` released 2026-05-08; floats forward within major per AGENTS.md Dependency Versions rules for actions).
5. **Accept the action's default `RUSTFLAGS=-D warnings`** (Round 2 Q1). Do **not** override via `rustflags: ""`. This extends `-D warnings` to `cargo build`, `cargo test`, `cargo bench`, `cargo doc`, and `cargo llvm-cov` invocations within affected jobs. Any newly-surfaced compiler warnings on the feature-branch push that are small / mechanical (≤ ~20 LOC, no design choice) are fixed in this PR; substantive warnings (touching design decisions, public API, or non-trivial logic) become separate follow-up issues. The design agent records the small-fix budget as a concrete LOC ceiling.
6. **Disable the explicit `Swatinem/rust-cache@v2` steps in `ci.yml` and let the new action's bundled cache replace them** (Round 2 Q2). Configure the bundled cache via the action's `cache-shared-key:` / `cache-workspaces:` inputs to reproduce the current per-job key topology:
   - `build` / `test` / `clippy` / `docs` jobs → `cache-shared-key: ${{ runner.os }}-stable`.
   - `gpu-tests` job → `cache-shared-key: ${{ runner.os }}-stable-gpu`.
   - `features` job → `cache-shared-key: ${{ runner.os }}-stable-features-${{ matrix.features }}`.
   - `format` job → no current cache step; leave the bundled cache at its default key (or `cache: false` if the format job is fast enough not to benefit — design decides).
7. **`actions/cache@v5` entries in `coverage.yml` and `docs.yml` — replace with the bundled cache, configured via `cache-shared-key:` to carry the existing custom key strings** (Round 3 Q1: "Bundled + keys"). One cache layer; preserves today's per-job key topology end-to-end:
   - `coverage.yml` (`coverage` job, nightly) → `cache-shared-key: ${{ runner.os }}-cargo-coverage-${{ hashFiles('**/Cargo.lock') }}`. The current step's `restore-keys: ${{ runner.os }}-cargo-coverage-` partial-match behaviour is supplied by the bundled cache's own segment-based restore logic; if the design phase finds the bundled cache cannot reproduce the exact restore-prefix behaviour, fall back to `cache-key:` (additional segment on the auto-job key) plus an explicit comment in the design noting the divergence.
   - `docs.yml` (`build` job) → `cache-shared-key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`. Same fallback rule.
   - Remove the explicit `actions/cache@v5` step in both files after the swap.
8. Verify each modified workflow with `actionlint <file>` before commit (AGENTS.md AXIOM gate).
9. Confirm the macOS Clippy / Test failures are gone on the feature-branch push.

## Out of scope

- Switching the cache action (`Swatinem/rust-cache@v2`) to a different cache provider — orthogonal to the toolchain-action swap.
- Investigating the upstream Swatinem/rust-cache × dtolnay PATH interaction in depth — the swap sidesteps it.
- Adding macOS or Windows lanes to workflows that are currently Linux-only (`base_benchmarks.yml`, `coverage.yml`, `docs.yml`, `fork_pr_benchmarks_run.yml`).
- Changes to non-toolchain steps (sccache, DX12 stack install, xvfb, Bencher, codecov upload, Pages deploy).
- Substantive code changes triggered by latent warnings the new `RUSTFLAGS=-D warnings` default surfaces — those become separate follow-up issues per item 5 above.

## Deferred

- Pinning `actions-rust-lang/setup-rust-toolchain` to a specific minor (e.g. `@v1.16.1`) | currently using major-floating per project convention; revisit if a future minor release ships a regression | no separate issue.
- Auditing per-job `RUSTFLAGS` strictness uniformly across the workspace (today's behaviour was non-uniform — only the `clippy` lint flag enforced `-D warnings`; the swap unifies it) | post-merge follow-up if any latent-warning fallout is non-trivial | new issue per item 5 if pursued.

## Key decisions

| Question | Decision |
|---|---|
| Fix approach (Round 1 Q1) | **Action swap.** Replace `dtolnay/rust-toolchain` with `actions-rust-lang/setup-rust-toolchain@v1` across every workflow file. Rationale: the alternative surgical PATH-pin step addresses the symptom but leaves the upstream action interaction with `Swatinem/rust-cache` intact; the swap moves to a maintained action that integrates the cache + PATH handling end-to-end. |
| Scope of edits (Round 1 Q2) | **All workflows.** Every `dtolnay/rust-toolchain` invocation across the 5 workflow files (11 install steps total). Linux-only workflows are included for consistency and future-proofing against a macOS matrix lane being added later. |
| New action version pin | `@v1` major-floating. Live registry verified 2026-05-14: `v1.16.1` (2026-05-08). Per AGENTS.md Dependency Versions rules, GitHub Actions follow major-floating convention. |
| Toolchain channel selection | Preserve per-site channel: `@stable` everywhere except `coverage.yml` which keeps `@nightly` (taiki-e `cargo-llvm-cov` requires nightly for `--doctests`). Translate to `toolchain: stable` / `toolchain: nightly` input on the new action. |
| Components preservation | Preserve `components: rustfmt` on `format`, `components: clippy` on `clippy`. For `coverage`, design step verifies whether `llvm-tools-preview` needs to be added explicitly (taiki-e/install-action installs `cargo-llvm-cov` which may already pull what it needs; explicit selection is the safer choice). |
| `RUSTFLAGS=-D warnings` default (Round 2 Q1) | **Accept default.** The new action sets `RUSTFLAGS=-D warnings` on all `cargo` invocations within a job; do not override. Extends `-D warnings` to `cargo build`, `cargo test`, `cargo bench`, `cargo doc`, `cargo llvm-cov`. Latent-warning fallout handled per Scope item 5 (small fixes in this PR; substantive ones spun out). |
| Bundled cache vs. explicit `Swatinem/rust-cache@v2` in ci.yml (Round 2 Q2) | **Config bundled.** Drop the six explicit `Swatinem/rust-cache@v2` steps in `ci.yml`; configure the new action's bundled cache via `cache-shared-key:` to reproduce today's per-job key topology (`${{ runner.os }}-stable`, `${{ runner.os }}-stable-gpu`, `${{ runner.os }}-stable-features-${{ matrix.features }}`). |
| `actions/cache@v5` in coverage.yml + docs.yml (Round 3 Q1) | **Bundled + keys.** Replace both `actions/cache@v5` steps with the new action's bundled cache, configured via `cache-shared-key:` carrying the existing custom key strings (`${{ runner.os }}-cargo-coverage-${{ hashFiles('**/Cargo.lock') }}` for coverage; `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}` for docs). One cache layer; preserves today's per-job key topology. Action.yml input verified 2026-05-14: `cache-shared-key` overrides the auto-job key — exactly the semantics needed. Fallback rule (cache-key segment if restore-prefix behaviour can't be reproduced) recorded in Scope item 7. |
| Apply changes on a feature branch | Branch `feat/2026-05-14-macos-cargo-path-pin` per AGENTS.md Workflow AXIOM 1. |
| `actionlint` gate | Run `actionlint <file>` for **every** modified workflow file before commit (`actionlint .github/workflows/ci.yml .github/workflows/base_benchmarks.yml .github/workflows/coverage.yml .github/workflows/docs.yml .github/workflows/fork_pr_benchmarks_run.yml`). |

## Technical constraints

- `actionlint` MUST pass on every modified workflow file (AGENTS.md AXIOM).
- Migration MUST not regress passing jobs on Ubuntu and Windows.
- macOS Clippy / Test jobs MUST go green; aggregate `clippy-pass` / `test-pass` gates MUST go green on the same run.
- Coverage workflow MUST continue to upload to Codecov (`secrets.CODECOV_TOKEN` still consumed by `codecov/codecov-action@v6`).
- Bencher workflows MUST continue to call `cargo bench --workspace` after the swap (bencher's CLI lives outside the toolchain action and is installed via `bencherdev/bencher@main`).
- Docs deployment workflow MUST continue to publish `target/doc` via `actions/deploy-pages@v5` and retain `RUSTDOCFLAGS=-D warnings -D missing-docs` (separate env var; not affected by the action's `RUSTFLAGS` default).
- Live registry version verified 2026-05-14 for `actions-rust-lang/setup-rust-toolchain` — `v1.16.1` released 2026-05-08, `@v1` major tag is current.
- Live `action.yml` input names verified 2026-05-14: `cache: true|false`, `cache-shared-key:` (overrides auto-job key), `cache-key:` (additional segment on auto-job key), `cache-workspaces:`, `toolchain:`, `components:`, `rustflags:` (default `-D warnings`). Source-of-truth for spec wording.
- The `RUSTFLAGS=-D warnings` default applies per-job, set by the action's setup step into `$GITHUB_ENV`; jobs that do not install Rust via the new action are unaffected.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `actionlint` exits 0 against every modified workflow file (5 files total). |
| AC2 | `Clippy (macos-latest)` job passes on the feature-branch push (no `unexpected argument 'clippy' found` / `rustup-init` error). |
| AC3 | `Test (macos-latest)` job passes on the same push (no `unexpected argument 'test' found` / `rustup-init` error). |
| AC4 | Aggregate `Clippy`, `Test`, `Build`, `GPU tests`, `Docs`, `Feature matrix`, `Format` gate jobs all report success on the feature-branch push. |
| AC5 | Ubuntu and Windows variants of `Clippy`, `Test`, `Build`, and `GPU tests` continue to pass. |
| AC6 | `Coverage` workflow (if triggered on the push) completes successfully and uploads lcov to Codecov; nightly toolchain still installed; `cargo llvm-cov` still runs `--doctests`. |
| AC7 | `Base Benchmarks` and `Run Benchmarks` workflows continue to invoke `cargo bench --workspace` without error if triggered (bench invocation itself is unchanged). |
| AC8 | `Docs` (publish-to-Pages) workflow continues to build and publish target/doc with `RUSTDOCFLAGS=-D warnings -D missing-docs` retained. |
| AC9 | No `dtolnay/rust-toolchain` reference remains anywhere under `.github/workflows/` (`grep -r "dtolnay/rust-toolchain" .github/workflows/` returns empty). |
| AC10 | No explicit `Swatinem/rust-cache@v2` step remains in `ci.yml`, and no explicit `actions/cache@v5` step remains in `coverage.yml` or `docs.yml` (the new action's bundled cache replaces all three). `cache-shared-key:` values on the new action match the per-job key strings recorded in Scope items 6 and 7. |
| AC11 | Any newly-surfaced compiler warnings on the feature-branch push that exceed the design agent's small-fix LOC budget are filed as separate follow-up issues (linked from the PR description) rather than fixed in this PR. |
| AC12 | The PR description summarises the swap, calls out the resolution of all Round-1 / Round-2 / Round-3 decisions (action swap, RUSTFLAGS default, bundled-cache strategy, coverage/docs cache strategy), and links the green runs that demonstrate AC2–AC8. |

## Open questions

- Whether `coverage.yml` needs an explicit `components: llvm-tools-preview` after the swap can be settled by the design agent during implementation (mechanical verification: run the job, observe failure mode if any, then pin if needed). Defensible default: add it explicitly to remove uncertainty.
- Concrete small-fix LOC ceiling for latent-warning fallout (Scope item 5) is set by the design agent based on what surfaces on the first push — defensible default ~20 LOC, but the design phase may revise upward/downward once actual fallout is visible.
- Whether the bundled cache's segment-based restore behaviour fully reproduces the existing `restore-keys: ${{ runner.os }}-cargo-coverage-` partial-prefix semantics, or whether the fallback (cache-key segment on auto-job key) is needed for coverage/docs — design agent verifies during implementation (Scope item 7 fallback rule).
