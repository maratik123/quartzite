# Design: CI — Code Coverage via cargo-llvm-cov + Codecov

**Issue:** #134
**Date:** 2026-05-07

## Approach

Add a separate `.github/workflows/coverage.yml` that runs `cargo-llvm-cov` on
`ubuntu-latest`, produces an LCOV report, and uploads it to Codecov via
`codecov/codecov-action@v5`. A companion `codecov.yml` at the repository root
configures informational-only status checks (no auto-fail). A Codecov badge is
appended to `README.md` alongside the implicit CI badge block.

### Why a separate file

`ci.yml` runs a multi-OS matrix and is gated by branch protection. Coverage is a
single-platform concern (LLVM instrumentation is platform-independent; measuring
on Ubuntu is sufficient) and must not block merges. Keeping it in a separate file
avoids polluting the CI matrix and makes the informational boundary obvious.

### `--exclude` flag — examples are not a workspace member

The spec says `--exclude quartzite-examples`, but that package does not exist as a
workspace member. The workspace has exactly these members: `quartzite` (root),
`quartzite-core`, `quartzite-macros`, `quartzite-runtime`, `quartzite-geometry`,
`quartzite-events`, `quartzite-event-types`. Examples are `[[example]]` targets
inside the root `quartzite` package. Passing `--exclude quartzite-examples` to
`cargo llvm-cov` would produce an error ("package ID specification ... did not
match any packages"). Omit the flag; example binaries have no `#[test]` items and
produce no coverage data in the LCOV report unless explicitly run with
`--examples`. Since `--examples` is not passed, examples are naturally excluded.

### Tool installation

`taiki-e/install-action@v2` with `tool: cargo-llvm-cov` installs the pre-built
binary in seconds. This action also installs `llvm-tools-preview` for the active
Rust toolchain automatically, so no separate `rustup component add` step is needed.

### Doctests

`--doctests` is included per the spec. `cargo-llvm-cov` v0.5+ supports doctest
coverage out of the box (it spawns a separate instrumented doctest binary).

### Codecov configuration

`codecov.yml` at the repo root controls Codecov behaviour independently of the
workflow file. Setting `informational: true` on the project status check prevents
Codecov from posting a failing GitHub check when coverage drops, while still
posting a PR comment with the delta.

### Rejected alternatives

- **Add to ci.yml**: rejected — mixes concerns and could create confusion if
  coverage is slow or flaky relative to the primary gate jobs.
- **Coveralls**: rejected — spec mandates Codecov; Codecov has better Rust/LCOV
  integration and free OSS tier.
- **Multi-OS coverage**: rejected per spec; out of scope.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `coverage.yml` workflow | `.github/workflows/coverage.yml` | — |
| 2 | Create `codecov.yml` configuration | `codecov.yml` | — |
| 3 | Add Codecov badge to README | `README.md` | 1 |

## Risks

- **`cargo-llvm-cov` not available on stable**: mitigation — `taiki-e/install-action` downloads
  a pre-built binary from GitHub Releases; it does not require nightly. No additional
  toolchain configuration is needed beyond stable.
- **Doctest coverage requires `llvm-tools-preview` component**: mitigation — `taiki-e/install-action`
  installs it automatically as a side-effect of installing `cargo-llvm-cov`.
- **`CODECOV_TOKEN` not set**: the upload step will fail with a 401 until the
  secret is added in repo Settings → Secrets. The job itself is informational and
  not branch-protected, so this only affects the coverage upload, not merges.
  Mitigation: document the prerequisite in the PR description.
- **Badge URL wrong before first upload**: the Codecov badge URL embeds the repo
  slug (`maratik123/quartzite`) and branch (`master`). It will show "unknown"
  until the first successful upload. This is cosmetic and self-resolves.
- **`--exclude quartzite-examples` flag**: as analysed above, passing this flag
  would cause `cargo llvm-cov` to error. The flag must not be used.

## Test Design

This task produces only CI workflow YAML files and a README edit — no Rust source
changes. There is no `#[cfg(test)]` module to write.

**Manual verification steps (post-merge):**
- Push to `master` triggers the `coverage` job; GitHub Actions log shows
  `cargo llvm-cov ... --doctests` completing and `lcov.info` being uploaded.
- A PR against `master` triggers the same job; Codecov posts a PR comment with
  coverage delta.
- Codecov project page at `https://codecov.io/gh/maratik123/quartzite` shows
  a coverage percentage.
- The badge in `README.md` resolves and displays a percentage after the first upload.

## Open questions

_(none)_
