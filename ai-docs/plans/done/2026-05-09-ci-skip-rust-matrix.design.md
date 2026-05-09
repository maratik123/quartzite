# Design: CI — skip Rust matrix on instruction-only PRs

**Issue:** #190
**Spec:** [`2026-05-09-ci-skip-rust-matrix.spec.md`](2026-05-09-ci-skip-rust-matrix.spec.md)
**Date:** 2026-05-09

## Approach

The workflow `.github/workflows/ci.yml` is restructured so that the heavy
Rust matrix jobs (`build`, `test`, `clippy`, `docs`, `features`) only run
when a path-filter has determined the PR / push touched Rust-relevant
files. Branch-protection check names are preserved by aggregator jobs
(`build-pass`, `test-pass`, `clippy-pass`, `docs-pass`, `features-pass`)
which now treat `result == 'skipped'` as success, so instruction-only PRs
finish quickly while still satisfying the six required contexts
(`Format`, `Build`, `Test`, `Clippy`, `Docs`, `Feature matrix`).

**Mechanism**: a new job `changes` runs `dorny/paths-filter@v4` on
`ubuntu-latest`, declares one boolean output `rust`, and is added to
`needs:` of every gated job. Each gated job carries
`if: needs.changes.outputs.rust == 'true'`. Aggregator scripts are
rewritten to fail only on `failure` / `cancelled` and pass on
`success` / `skipped`.

**Why a `changes` job + job-level `if:` (not workflow-level
`paths-ignore`)**: workflow-level `paths-ignore` would prevent the
workflow from triggering at all on instruction-only PRs, which leaves
required status checks in `expected` state forever and blocks merge.
The `changes`-driven approach lets the workflow run, mark matrix jobs
as `skipped`, and have aggregators report `success` so branch protection
is satisfied. This is the well-known GitHub idiom for the "skipped vs.
success on required checks" trap.

**Live-version verification (per AGENTS.md § Dependency Versions)**:
- `dorny/paths-filter` latest release: `v4.0.1` (queried 2026-05-09);
  `action.yml` declares `using: 'node24'` — modern runtime, current major.
- All other actions remain at their currently-pinned major (no change).

**Rejected alternatives**:

1. **Workflow-level `paths-ignore:`** — rejected because GitHub treats
   "workflow did not run" differently from "workflow ran and skipped";
   required checks remain pending and PR cannot merge.
2. **Filter at `on: pull_request:` + duplicate `on: push:`** — same
   skipped-vs-pending failure mode, plus duplicates trigger config.
3. **`tj-actions/changed-files`** — supersedes `paths-filter` for some
   use cases but pulls in heavier git history scanning and has had
   security incidents (2024 supply-chain). `dorny/paths-filter` is
   simpler, smaller-surface, and matches the spec's named pin.
4. **Move the matrix into a callable workflow gated by an outer
   `workflow_call`** — overkill for this scope; reshapes the public
   check graph more than necessary and complicates branch-protection
   maintenance.
5. **Reuse a single generic `pass` aggregator parametrised by job
   name** — GitHub Actions has no good way to template `needs:` on a
   matrix-of-aggregators; explicit one-aggregator-per-matrix-job is
   the maintainable shape and matches what's already in the file.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add the `changes` job (top of `jobs:`, before `format`) using `dorny/paths-filter@v4` with one output `rust` driven by the path patterns from spec § Scope item 2. The step gets `id: filter` so outputs can be referenced. The job declares `outputs.rust: ${{ steps.filter.outputs.rust }}`. Runs on `ubuntu-latest`; uses `actions/checkout@v6` only when needed (paths-filter v4 supports `pull_request` events without a checkout, but on `push` events a checkout is required — include checkout unconditionally for simplicity, as per the action's README guidance). | `.github/workflows/ci.yml` | — |
| 2 | Add `needs: changes` and `if: needs.changes.outputs.rust == 'true'` to the `build` job. No other changes to the job body. | `.github/workflows/ci.yml` | 1 |
| 3 | Add `needs: changes` and `if: needs.changes.outputs.rust == 'true'` to the `test` job. | `.github/workflows/ci.yml` | 1 |
| 4 | Add `needs: changes` and `if: needs.changes.outputs.rust == 'true'` to the `clippy` job. | `.github/workflows/ci.yml` | 1 |
| 5 | Rename the existing `docs` job's `name:` from `Docs` to `Docs (build)` so the new `docs-pass` aggregator can own the public `Docs` name without check-name collision. Add `needs: changes` and `if: needs.changes.outputs.rust == 'true'` to the same job. | `.github/workflows/ci.yml` | 1 |
| 6 | Add `needs: changes` and `if: needs.changes.outputs.rust == 'true'` to the `features` job. | `.github/workflows/ci.yml` | 1 |
| 7 | Reshape the four existing aggregators (`build-pass`, `test-pass`, `clippy-pass`, `features-pass`) so the bash check passes only when the underlying job is `success` OR (the underlying job is `skipped` AND the `changes` job itself succeeded). Add `changes` to each aggregator's `needs:` list so its result is observable. Pattern (single line, exits 1 on failure):<br>`c='${{ needs.changes.result }}'; r='${{ needs.<job>.result }}'; if [[ "$c" != "success" ]] || [[ "$r" != "success" && "$r" != "skipped" ]]; then echo "changes=$c <job>=$r"; exit 1; fi`<br>This guards AC4 against the `changes`-failed cascade (every gated job auto-skips, and a naive `success || skipped` check would falsely report aggregator success). | `.github/workflows/ci.yml` | 1, 2, 3, 4, 6 |
| 8 | Add a new `docs-pass` aggregator with `name: Docs`, `needs: [changes, docs]`, `if: always()`, and the same `changes`-aware bash logic from Task 7 (require `changes == success` AND `docs ∈ {success, skipped}`). This job is what branch protection will see for the `Docs` required context. | `.github/workflows/ci.yml` | 1, 5 |
| 9 | Run `actionlint .github/workflows/ci.yml` and resolve any reported issues; the workflow is the only file changed in this PR. | `.github/workflows/ci.yml` | 7, 8 |

Total: 9 atomic tasks, all in one file. Under the 7-task threshold for splitting (the threshold is for *complex* multi-file decomposition; here all changes are localised co-edits to one workflow file, so they're presented separately for reviewability but are intended to be made in a single commit/PR).

## Risks

- **Risk: required-context name collision during transition.** The
  existing `docs` job is named `Docs`; the new `docs-pass` aggregator
  is also named `Docs`. If both exist with the same `name:` at any
  point, GitHub will report two checks with the same name, and branch
  protection's match becomes ambiguous.
  **Mitigation:** Task 5 renames the underlying `docs` job's `name:` to
  `Docs (build)` *before* (or in the same diff as) Task 8 introduces
  the `docs-pass` aggregator with `name: Docs`. The implementer must
  apply both edits in the same commit; they cannot be split across
  pushes. Verification: `actionlint` and a live PR check showing only
  one check with name `Docs` (the aggregator).
- **Risk: `dorny/paths-filter@v4` behaviour on `push` to master.** On
  `push` events the action compares against the push's `before` commit;
  when a PR is merged with a merge commit, the `before` is master's tip
  before the merge, so changes are detected correctly. On a force-push
  scenario `before` may point to a non-existent commit, causing the
  action to fall back to comparing against `HEAD~1`.
  **Mitigation:** documented behaviour, no special config needed; if a
  force-push to master ever occurs (server-blocked today), the worst
  case is a full matrix run — safe failure mode.
- **Risk: skipping the matrix on master push misses cache regeneration.**
  The spec acknowledges this and accepts it: an instruction-only push
  to master changes nothing the cache covers, so no cache work is
  meaningfully missed. The next code-touching push will warm the cache
  as before.
- **Risk: `actionlint` rejects the new `if:` expression syntax.**
  `if: needs.changes.outputs.rust == 'true'` is the canonical pattern
  documented in the GitHub Actions context expressions reference and
  is well-supported by `actionlint`. **Mitigation:** Task 9 runs
  `actionlint` as the gate.
- **Risk: an undocumented Rust-relevant path is missed by the filter,
  causing a code-affecting change to skip the matrix.** The spec's
  filter set (`**/*.rs`, `**/Cargo.toml`, `Cargo.lock`,
  `.github/workflows/**`, `rust-toolchain*`) covers everything that
  could affect a Rust build today. **Mitigation:** any future Rust
  artefact added to the repo (e.g. `.cargo/config.toml`, `clippy.toml`)
  must be added to the filter in the same PR that introduces it. Add a
  brief comment above the filter listing this contract.
- **Risk: `roadmap-sync` and `roadmap-sync-pass` left untouched, but
  their aggregator does not need skipped-as-success because the
  underlying job runs unconditionally.** No real risk — by spec they
  remain unchanged. Mention in the design so the implementer doesn't
  attempt symmetry edits that would be no-ops.
- **Risk: `format` job left untouched, ditto.** Same — runs
  unconditionally per spec, no changes needed. Format job has no
  aggregator today (the job's `name: Format` is itself the required
  context); preserve that.
- **Risk: `dorny/paths-filter@v4` token permissions on `pull_request`
  events.** On `pull_request` events the action calls the GitHub REST
  API to compute the changed-file list; this requires the
  `pull-requests: read` scope on `GITHUB_TOKEN`. The repo's `ci.yml`
  has no top-level `permissions:` block today, so it relies on the
  default `GITHUB_TOKEN` scopes, which include `pull-requests: read`
  for non-fork PRs in this repo's settings. **Mitigation:** acceptable
  to rely on defaults (matches the pattern used elsewhere in this
  workflow). For robustness against a future tightening of default
  scopes, the implementer MAY add an explicit
  `permissions: { pull-requests: read }` block on the `changes` job —
  cheap, narrow, and self-documenting. Not required for AC.

## Test Design

This is a workflow-only change with no Rust source touched, so there are
no `cargo test` cases to add. Validation is split across static checks
and live CI behaviour.

### Static gate (mandatory, blocks merge)

- **Tool:** `actionlint .github/workflows/ci.yml`
- **Entry point:** the modified workflow file
- **Scenarios:**
  - All YAML, expression, and shell syntax valid.
  - No deprecated action versions or expression-syntax errors.
  - Bash heredoc-style inline scripts pass the embedded shellcheck rules.
- **Fixtures:** none — single-file static analysis.
- **Failure mode:** any `error` or `warning` from `actionlint` is a
  blocker per AGENTS.md § Build & Test "AXIOM — `actionlint` MUST pass
  before `git add`".

### Live CI verification (per spec § Verification protocol)

These are out-of-tree validations performed on the PR before merge.
They are part of the acceptance evidence, not unit tests.

- **Scenario 1 — instruction-only PR (AC1, AC3, AC6):**
  - Touch one file under `ai-docs/` only.
  - Push, open PR.
  - Expect: `changes` job logs `rust=false`; `build`/`test`/`clippy`/
    `docs`/`features` all `Skipped`; all five `*-pass` aggregators
    report `success`; the six required contexts (`Format`, `Build`,
    `Test`, `Clippy`, `Docs`, `Feature matrix`) all `success`; total
    runner wall-clock < 60 s.
- **Scenario 2 — code-affecting PR (AC2, AC4):**
  - Touch any `.rs` file (e.g. add a comment).
  - Push, open PR.
  - Expect: full matrix runs as today; all aggregators only succeed
    when underlying jobs do.
- **Scenario 3 — `Cargo.toml`-only PR (negative test, AC2):**
  - Bump a comment or whitespace in `Cargo.toml` (no `.rs` change).
  - Push, open PR.
  - Expect: `changes` job logs `rust=true`; full matrix runs.
- **Scenario 4 — workflow-only PR (recursive case):**
  - The PR that lands this change itself touches
    `.github/workflows/ci.yml`, which matches `.github/workflows/**`
    in the filter — so the matrix runs on this PR. This is the
    correct behaviour and is also the live AC2/AC4 evidence for the
    PR introducing the rule.

### Aggregator failure-propagation matrix (AC4)

The reshape rewrites the bash check from
`if [[ "$r" != "success" ]]; then exit 1; fi`
to
`c='${{ needs.changes.result }}'; r='${{ needs.<job>.result }}'; if [[ "$c" != "success" ]] || [[ "$r" != "success" && "$r" != "skipped" ]]; then exit 1; fi`.
Treating `skipped` as success is conditional on `changes` itself having
succeeded — otherwise a `changes` failure would silently cascade to
`skipped` on every gated job and falsely report aggregator success.
The transition table the implementer should keep in mind:

| `changes` result | Underlying job result | Old aggregator | New aggregator |
|---|---|---|---|
| `success` | `success` | `success` | `success` |
| `success` | `skipped` (rust filter false — instruction-only PR) | `failure` (regression today) | `success` (intended) |
| `success` | `failure` | `failure` | `failure` |
| `success` | `cancelled` | `failure` | `failure` |
| `failure` | `skipped` (auto, because `needs: changes` failed) | n/a | `failure` (intended — guards AC4 against `changes`-failure cascade) |
| `cancelled` | `skipped` (auto) | n/a | `failure` |
| any | `null` / unevaluated | n/a | `failure` (defensive — empty `$r` fails the `!=` check) |

No new tests are added to the Rust workspace.

## Open questions

- **Force-rerun escape hatch** (carried from spec § Open questions).
  Default position is "no" — contributors who want to validate the
  matrix against an instruction change can include a trivial whitespace
  edit to a `.rs` file. If this proves friction-creating in practice,
  open a follow-up issue to add a `[force-ci]` PR-title token or a
  `workflow_dispatch` trigger; not part of this design.
- **Filter contract documentation.** Should the path-filter list carry
  an inline YAML comment naming the contract ("any future Rust artefact
  must be added here in the same PR that introduces it")? Recommendation:
  yes — one-line comment above the filter is cheap insurance against
  silent drift. Defer to implementer / reviewer if they prefer to keep
  the YAML uncommented.
