# CI: fix macOS Clippy `rustup-init` cache pollution

**Source:** issue #340
**Date:** 2026-05-15
**Tracked in:** #340

## Scope

1. Eliminate the intermittent `Clippy (macos-latest)` failure where `cargo clippy` resolves to `rustup-init` and prints `error: unexpected argument 'clippy' found`.
2. Mitigation mechanism (per Q1): append the GitHub-hosted runner image version to every `cache-shared-key` that today reads `${{ runner.os }}-...-v2`, so that any runner-image bump (e.g. macOS 15.7.4 → 15.7.5, which the user identified as the trigger) invalidates the cache and forces a clean restore — preventing a polluted `~/.cargo/bin/cargo` shim built on the previous image from being restored onto a fresh image. Narrowest blast radius among the candidates considered.
3. Mitigation scope (per Q2): apply the same cache-key change to **every cargo job on every OS leg of every workflow** that uses `actions-rust-lang/setup-rust-toolchain@v1` with a `cache-shared-key`. This protects Ubuntu and Windows from an unknown future regression of the same shape. Concretely, the following call sites are in scope:
   - `.github/workflows/ci.yml` — `build`, `test`, `clippy`, `gpu-tests` (key `${{ runner.os }}-stable-gpu-v2`), `docs`, `features` (key embeds matrix features).
   - `.github/workflows/coverage.yml` — single Linux job, key `${{ runner.os }}-cargo-coverage-v2`.
   - `.github/workflows/docs.yml` — single Linux job, key `${{ runner.os }}-cargo-v2`.
4. Add a diagnostic positive-shape check step on every cargo-running job that prints `cargo --version` (and, where appropriate, `rustc --version`) **after** the toolchain-install step and **before** the first `cargo <subcommand>` invocation, so a future regression surfaces as a visible "this is rustup-init" line instead of a confusing argparse error.
5. Run `actionlint` on every modified workflow file before committing (AGENTS.md AXIOM).

## Out of scope

- Replacing `actions-rust-lang/setup-rust-toolchain@v1` with `dtolnay/rust-toolchain` (issue body's secondary hypothesis is moot — the current workflow already uses the more robust action). Re-evaluation of the action choice is a separate concern.
- Wholesale rewrite of the CI matrix.
- Tweaking `sccache` configuration (orthogonal cache layer).
- Bumping the `-v2` suffix to `-v3`. The image-version segment will already force a one-time miss on the first run after merge; a manual generation bump on top would be redundant.

## Deferred

(none — Q1, Q2, Q3 resolved in round 1; previously-deferred "broaden to all OS legs" item is now in Scope.)

## Key decisions

| Question | Decision |
|---|---|
| Which mitigation mechanism neutralises the polluted `~/.cargo/bin/cargo` shim on macOS? | **Cache key + image version.** Append the runner's image version to every `cache-shared-key` so a runner-image bump invalidates the cache. Matches the user's stated hypothesis ("Seems the problem with macos 15.7.5, in 15.7.4 it is good. May be need to add macos version to cache key"). Narrowest blast radius — no PATH hacks, no action swap, no per-step shell shimming. |
| Which jobs / workflow files receive the mitigation? | **All cargo jobs on all OS legs of every workflow.** Applied uniformly to `ci.yml` (`build`, `test`, `clippy`, `gpu-tests`, `docs`, `features`), `coverage.yml`, `docs.yml`. Defensive — protects against an unknown future Ubuntu/Windows regression of the same shape. |
| AC1 verification target — how many consecutive green macOS CI runs are needed to consider #340 closed? | **3 consecutive runs**, combined with the AC2 positive `cargo --version` diagnostic. Quick verdict; the diagnostic step turns a recurrence into a visible signal rather than a stochastic re-test. |
| Which CI workflow file(s) need editing? | `.github/workflows/ci.yml`, `.github/workflows/coverage.yml`, `.github/workflows/docs.yml`. All three currently call `actions-rust-lang/setup-rust-toolchain@v1` with a `cache-shared-key` and are exposed to the same vector. |
| Which toolchain-install action is in play? | `actions-rust-lang/setup-rust-toolchain@v1` on every job. The issue body's `dtolnay/rust-toolchain` reference is incorrect; design must reason about `actions-rust-lang` semantics. |
| Reproduction reliability | Failure is intermittent; cannot be reproduced on-demand. Acceptance is therefore defined as (a) no recurrence across the 3-run window (AC1) plus (b) a positive-shape `cargo --version` diagnostic (AC2). |
| Pre-publish posture (AGENTS.md § *API Stability*) | Not applicable — CI-only change, no public-API surface. |

## Technical constraints

- AGENTS.md AXIOM: `actionlint .github/workflows/<file>.yml` MUST pass before `git add` on every modified workflow file.
- AGENTS.md / `ai-docs/dependency-versions.md`: any new or bumped action version must be checked against the live registry **before** writing the version string; for any **load-bearing claim about an action's behaviour** (e.g. "the runner image version is exposed as env var `X`"), the design agent must verify the variable name and exposure mechanism by inspecting `actions/runner-images` documentation or a live workflow run, not by relying on remembered knowledge. README narrative is not evidence.
- The current macOS cache key `${{ runner.os }}-stable-v2` does not vary by macOS image minor version — a cache produced on `macos-latest` image 15.7.4 can be restored onto a runner now on image 15.7.5. The fix must make the key vary by the image-version identifier.
- `actions-rust-lang/setup-rust-toolchain@v1` runs `rustup toolchain install` / `rustup component add` **after** `Swatinem/rust-cache@v2` restores `~/.cargo/bin`. Whether it overwrites a polluted shim or silently inherits it is unspecified in the README; the design agent must verify by inspecting `action.yml` + `src/main.ts` of the action (per `ai-docs/dependency-versions.md` recipe) and document the verification. If the action does NOT overwrite the shim, the cache-key change alone is the entire mitigation; if it DOES, the cache-key change is belt-and-braces. Either way the cache-key change is correct.
- The image-version segment will force a one-time cache miss on the first run on each OS after merge. This is expected and acceptable (bounded one-time cost; cache repopulates on the master push).
- AC2's diagnostic step must run **after** the toolchain-install step and **before** the first `cargo <subcommand>` step on the job; on success it prints e.g. `cargo 1.xx.0 (...)`, on a regression it prints the `rustup-init` argparse error and fails the step early with a clear cause.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | After the PR merges to master, the `Clippy (macos-latest)` job does not fail with `error: unexpected argument 'clippy' found` on at least **3 consecutive** PR + master runs of `ci.yml`. (Combined with AC2's positive check.) |
| AC2 | A diagnostic step running `cargo --version` (and `rustc --version` where useful) is added to every cargo-running job on every OS leg, positioned after the toolchain-install step and before the first `cargo <subcommand>`. Output shows a real cargo version string (e.g. `cargo 1.xx.0 (...)`). The step exists on `build`, `test`, `clippy`, `gpu-tests`, `docs`, `features` in `ci.yml`, plus the corresponding jobs in `coverage.yml` and `docs.yml`. |
| AC3 | Every `cache-shared-key` value in `ci.yml`, `coverage.yml`, `docs.yml` that previously read `${{ runner.os }}-...-v2` now includes the runner image version segment (exact env var / expression to be picked by the design agent — likely `${{ env.ImageVersion }}` or the equivalent verified against `actions/runner-images`). The change is applied to every OS leg uniformly (Ubuntu, macOS, Windows). |
| AC4 | `actionlint` passes cleanly on every modified workflow file. |
| AC5 | Each modified workflow file carries a near-step comment citing issue #340 and the macOS-image-version hypothesis, so a future reader understands why the image-version segment is in the cache key and why the diagnostic step exists. |
| AC6 | No regression on `Ubuntu` or `Windows` legs of any modified job — they continue to run; their first run after merge is a one-time cache miss (expected), after which the new key warms. |
| AC7 | The `features` job's `cache-shared-key` (which already embeds `${{ matrix.features }}`) is updated with the image-version segment in a position that keeps the per-feature partitioning intact (i.e. image-version is added to the key without merging the per-feature buckets). |

## Open questions

(none — design-affecting ambiguities resolved across round 1.)
