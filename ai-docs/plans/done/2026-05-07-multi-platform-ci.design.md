# Design: Multi-platform CI runners (Windows + macOS)

**Issue:** #133
**Date:** 2026-05-07

## Approach

Add an `os` matrix dimension to the `build`, `test`, and `clippy` jobs in
`.github/workflows/ci.yml`. Each of those three jobs gains:

```yaml
strategy:
  fail-fast: false
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
runs-on: ${{ matrix.os }}
```

The `format`, `docs`, and `features` jobs remain on `ubuntu-latest` unchanged.

Cache keys already contain `${{ runner.os }}` (lines 39, 58, 79, 98, 129–130
of the current file), so per-OS cache partitioning is already correct — no
cache-key edits required.

### Alternatives considered

**Multi-file split** — extracting reusable job templates into a `workflow_call`
composite. Rejected as over-engineering for a three-job, single-axis matrix
with no shared steps beyond what already exists.

**`include`/`exclude` matrix entries** — using a more complex matrix to run
`clippy` only on Linux. Rejected — the spec explicitly puts `clippy` in the
matrix set, and running it on all three OSes catches platform-specific
compiler diagnostics.

**`fail-fast: true` (default)** — rejected per spec; a Windows-only failure
must not cancel the macOS run.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `strategy` block and change `runs-on` on the `build` job | `.github/workflows/ci.yml` | — |
| 2 | Add `strategy` block and change `runs-on` on the `test` job | `.github/workflows/ci.yml` | — |
| 3 | Add `strategy` block and change `runs-on` on the `clippy` job | `.github/workflows/ci.yml` | — |
| 4 | Verify `format`, `docs`, `features` jobs are untouched; run `actionlint` if available | `.github/workflows/ci.yml` | 1, 2, 3 |

All three edits (tasks 1–3) touch the same file but are independent regions
and can be applied in one commit. Task 4 is a local validation step, not a
separate commit.

## Risks

- **Windows `~/.cargo` path**: the cache `path` block uses `~/.cargo/registry`
  and `~/.cargo/git`. On Windows runners, `~` resolves correctly for
  `actions/cache@v4` because GitHub expands it to `%USERPROFILE%`. No change
  needed, but if caching misses appear post-merge, switching to
  `${{ env.CARGO_HOME }}` is the mitigation path.
- **Windows line-ending / path-separator issues in tests**: out of scope per
  spec; tracked as a follow-up if failures surface.
- **YAML validity**: a misplaced `strategy` or wrong indentation level would
  silently break the workflow. Mitigation: run `actionlint` (task 4) before
  pushing.
- **Job-name collision**: GitHub Actions appends `(ubuntu-latest)` etc. to the
  job name automatically when a matrix is present. No branch-protection rules
  reference the bare job names (`build`, `test`, `clippy`) in a way that would
  break — but if any required-status-check rules exist they must be updated to
  the matrix-suffixed names. Verify in the repository's branch-protection
  settings after merging.

## Test Design

This task has no Rust source changes — there is no `#[cfg(test)]` module to
write. Validation is CI-level:

- **Local lint:** `actionlint .github/workflows/ci.yml` (if installed) must
  exit 0.
- **Post-merge CI check:** GitHub Actions run must show 9 successful jobs:
  `Build (ubuntu-latest)`, `Build (macos-latest)`, `Build (windows-latest)`,
  and the analogous triples for `Test` and `Clippy`; plus the unmodified
  `Format`, `Docs`, and `Feature matrix (*)` jobs.
- **Unchanged-job smoke-check:** confirm `format`, `docs`, and `features` job
  definitions are byte-identical to their pre-change state (diff review in PR).

## Open questions

- Are there any branch-protection required-status-check rules referencing the
  bare job names `build`, `test`, or `clippy`? If so, they must be updated to
  the matrix-suffixed names (e.g. `build (ubuntu-latest)`) after this PR
  merges. Check the repository settings before merging.
