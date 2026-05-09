# CI: skip Rust matrix on instruction-only PRs

**Source:** issue #190
**Date:** 2026-05-09
**Tracked in:** #190

## Scope

1. Add a `changes` job to `.github/workflows/ci.yml` using
   `dorny/paths-filter@v4` (live current major as of 2026-05-09; node24
   runtime). The job runs on `ubuntu-latest`, checks out the repo, and exposes
   one boolean output `rust` driven by the path filter below.
2. Path-filter triggers (any match → `rust == true` → full Rust matrix runs):
   - `**/*.rs`
   - `**/Cargo.toml`
   - `Cargo.lock`
   - `.github/workflows/**`
   - `rust-toolchain*` (future-proof; not present today)
3. Anything else (notably `*.md`, `ai-docs/**`, `.claude/**`, `LICENSE*`,
   `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, top-level non-Rust scripts) does
   NOT match → `rust == false` → matrix jobs skipped.
4. Gate the following matrix / Rust jobs on `needs.changes.outputs.rust == 'true'`:
   - `build`
   - `test`
   - `clippy`
   - `docs`
   - `features`
5. Reshape the existing aggregator jobs (`build-pass`, `test-pass`,
   `clippy-pass`, `features-pass`) so that `result == 'success' || result == 'skipped'`
   passes; only `failure` / `cancelled` fails. Keep the public check name on the
   aggregator (the names that branch protection consumes — see § Technical
   constraints).
6. Add a `docs-pass` aggregator named `Docs` (matching the existing
   required-check naming pattern), and rename the underlying single-job `docs`
   to a non-conflicting `name:` (e.g. `Docs (build)`) so the aggregator owns
   the public `Docs` check name. Apply the same skipped-as-success logic.
7. `format` and `roadmap-sync` remain unconditional (cheap; ~15 s combined;
   `roadmap-sync` actively wants to validate `INDEX.md`-driven `ROADMAP.md`
   consistency).
8. Path filter applies on both `push` (to `master`) and `pull_request`
   triggers. Skipping the matrix on instruction-only master pushes is
   intentional: no source changed, so no cache regeneration is needed.

## Out of scope

- Path filtering for the separate workflows: `coverage.yml`, `docs.yml`,
  `base_benchmarks.yml`, `fork_pr_benchmarks_run.yml`,
  `fork_pr_benchmarks_track.yml`. Each is its own file with its own trigger;
  address separately if/when they become noisy.
- Removing `format` or `roadmap-sync` from instruction-only PRs.
- Adding a `Cargo.toml`-only fast-path. `Cargo.toml` changes can affect
  feature flags, dependency resolution, and workspace topology — safer to run
  the full matrix.
- Changing branch-protection settings on `origin`. The existing required
  contexts (`Format`, `Build`, `Test`, `Clippy`, `Docs`, `Feature matrix`)
  are preserved by aggregator naming.
- A `[force-ci]` / `workflow_dispatch` escape hatch for forcing the full
  matrix on a doc-only PR (see § Open questions).

## Deferred

- Force-rerun escape hatch — deferred to Open questions; separate issue
  needed only if the team wants explicit override semantics.

## Key decisions

| Question | Decision |
|---|---|
| Detection mechanism | `dorny/paths-filter@v4` — community-standard GHA path filter. Live current major as of 2026-05-09 (node24 runtime). Issue body cited `v3`; live registry query supersedes per AGENTS.md § Dependency Versions. |
| `paths-ignore:` at workflow level vs. job-level `if:` | Job-level `if:` driven by a `changes` job. Workflow-level `paths-ignore` would cause GHA to not run the workflow at all, leaving required `*-pass` aggregators in `expected` state forever — PR cannot merge. This is the well-known "skipped vs success on required checks" issue. |
| Aggregator behavior when matrix skipped | `success` if dependent job's result is `success` OR `skipped`; fail on `failure` / `cancelled`. Branch protection then sees `success` and allows merge. |
| `docs` job aggregator | Add `docs-pass` named `Docs` with skipped-as-success logic; rename underlying `docs` job's `name:` to avoid the GitHub check-name collision (current job is `Docs`, branch protection requires `Docs`, and the aggregator must own that name). |
| Filter scope: PR + push to master | Filter applies on both `push` and `pull_request`. Skipping the matrix on master push is fine: no source changed, so no cache save is missed in any meaningful sense. |
| `Cargo.toml` triggers full matrix | Always. Manifest changes can shift feature flags, dependency resolution, or workspace topology — full matrix is safer. (Listed explicitly in the path filter MUST-trigger set.) |
| `.github/workflows/**` triggers full matrix | Always. Workflow edits need real validation; skipping them would defeat the point of CI. |
| `format` / `roadmap-sync` stay unconditional | Cheap (~15 s combined). `roadmap-sync` validates `INDEX.md`/`ROADMAP.md` integrity which is touched by instruction-only PRs. Filtering would create false negatives. |

## Technical constraints

- Branch protection on `origin/master` requires these named contexts (verified
  via `gh api /repos/.../branches/master/protection`):
  `Format`, `Build`, `Test`, `Clippy`, `Docs`, `Feature matrix`.
  The redesigned workflow must produce a `success` result for each of these
  on every PR — including instruction-only PRs. Achieved by:
  - `Format` → unchanged; `format` job runs unconditionally.
  - `Build` → owned by `build-pass` aggregator (matches today's pattern).
  - `Test` → owned by `test-pass` aggregator.
  - `Clippy` → owned by `clippy-pass` aggregator.
  - `Docs` → owned by NEW `docs-pass` aggregator (rename underlying `docs`
    job's `name:` to avoid collision).
  - `Feature matrix` → owned by `features-pass` aggregator.
- `actionlint` must pass on the modified workflow file (AGENTS.md gate).
- The `changes` job adds a one-time fixed cost (`actions/checkout@v6` +
  paths-filter run) to every CI run. Empirically this is a few seconds; well
  under the AC1 budget.
- `dorny/paths-filter@v4` on `pull_request` events compares against the PR
  base branch; on `push` events it compares against the push's `before`
  commit. Default behavior is correct for both triggers; no special
  configuration needed.
- Workspace crates are NOT cached by `Swatinem/rust-cache@v2` — but on
  instruction-only PRs the matrix is skipped entirely, so cache state is not
  a factor.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | An instruction-only PR (touches only `.claude/**`, `ai-docs/**`, `*.md`, no `.rs` / `Cargo.toml` / `Cargo.lock` / `.github/workflows/**` / `rust-toolchain*` changes) completes CI in under 60 seconds total runner wall-clock — measured as the sum of all jobs that actually ran, not the matrix-skipped jobs' notional cost. |
| AC2 | A code-affecting PR (any `*.rs`, `Cargo.toml`, `Cargo.lock`, workflow, or `rust-toolchain*` change) still runs the full Rust matrix as today: `build`/`test`/`clippy` × 3 OS, `docs`, `features` × 4. |
| AC3 | All `*-pass` aggregator jobs (`build-pass`, `test-pass`, `clippy-pass`, `docs-pass`, `features-pass`) report `success` on instruction-only PRs (matrix skipped → aggregator success → branch protection allows merge). |
| AC4 | All `*-pass` aggregator jobs report `success` on code PRs ONLY when their underlying matrix jobs all succeed. A `failure` or `cancelled` in the underlying job MUST propagate to a `failure` on the aggregator. (No false-success regressions.) |
| AC5 | `actionlint .github/workflows/ci.yml` passes cleanly with no errors or warnings. |
| AC6 | The required branch-protection contexts (`Format`, `Build`, `Test`, `Clippy`, `Docs`, `Feature matrix`) all appear and report `success` on instruction-only PRs after this change. Verified by inspecting the live PR's check list. |

## Verification protocol

1. Live test on a docs-only branch: edit one file under `ai-docs/` (e.g.
   touch a comment), push, open PR. Verify:
   - `changes` job logs `rust == false`.
   - Matrix jobs (`build`/`test`/`clippy`/`docs`/`features`) all report
     `Skipped`.
   - Each `*-pass` aggregator reports `success`.
   - All six required-context checks report `success`.
   - Total runner wall-clock < 60 s.
2. Live test on a code-affecting branch: touch any `.rs` file, push, open PR.
   Verify the full matrix runs as today and all aggregators only succeed when
   underlying jobs succeed.
3. Negative test: a PR with a `Cargo.toml` change but no `.rs` — must still
   trigger the full matrix.
4. `actionlint .github/workflows/ci.yml` clean before merge.

## Open questions

- **Force-rerun escape hatch**: should there be an explicit way to force the
  full matrix on a doc-only PR (e.g. `[force-ci]` PR title token, or
  `workflow_dispatch`)? Default position: no — if a contributor wants to
  validate the matrix against an instruction change, they can include a
  trivial whitespace edit to a `.rs` file (which would also serve as a clear
  signal in the diff). Revisit if this proves friction-creating in practice.
- **`actionlint` validation of the `if:` expression syntax**: confirmed
  expressible as `if: needs.changes.outputs.rust == 'true'` per GHA docs;
  design phase will verify against the live `actionlint` rule set.
