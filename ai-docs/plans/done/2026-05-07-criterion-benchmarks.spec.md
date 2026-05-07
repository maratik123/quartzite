# ci: benchmark workflow (criterion) with PR regression comments

**Source:** issue #135
**Date:** 2026-05-07
**Tracked in:** #135

## Scope

1. `quartzite-core/benches/` — criterion benchmarks for:
   - Signal emit: typed wrapper + dynamic via `emit!`
   - Property R/W via `read_property` / `write_property`
   - Object-tree lookup via `find_by_path` / `find_by_name`
2. `quartzite-core/Cargo.toml`: `criterion = "0.8"` dev-dep; one `[[bench]]` entry per file with `harness = false`
3. `.github/workflows/base_benchmarks.yml` — push-to-master baseline; `bencher run` with Student's t-test threshold
4. `.github/workflows/fork_pr_benchmarks_run.yml` — `pull_request` trigger; runs benches, uploads `benchmark_results.txt` + `event.json` artifacts (no secrets read)
5. `.github/workflows/fork_pr_benchmarks_track.yml` — `workflow_run` trigger; downloads artifacts, calls `bencher run --file`, posts PR comment

## Out of scope

- `quartzite-runtime/benches/` — event-loop timing dominated by OS scheduler noise on shared CI runners
- Multi-platform benchmarks — statistically incoherent across OS/allocator/scheduler differences
- Self-hosted runners

## Deferred

- `.github/workflows/fork_pr_benchmarks_closed.yml` (PR branch archival in Bencher) | not needed at first land | no separate issue needed

## Key decisions

| Question | Decision |
|---|---|
| Comment tooling | Bencher — secret already configured, official 3-workflow fork-PR pattern, statistical t-test model |
| Bencher action pin | `bencherdev/bencher@main` (official guide recommendation, user confirmed) |
| Criterion adapter | `rust_criterion` — parses criterion stdout; no custom emitter needed |
| Project slug | `quartzite` — project already exists on bencher.dev |
| Testbed | `ubuntu-latest` only — cross-OS comparison statistically incoherent |
| PR regression blocking | Yes — `--error-on-alert` fails the run when threshold exceeded |
| PR pattern | 3-workflow fork pattern — public repo, fork PRs cannot read secrets |
| Threshold | Student's t-test: `--threshold-max-sample-size 64`, `--threshold-upper-boundary 0.99` |

## Technical constraints

- `BENCHER_API_TOKEN` repository secret already configured (2026-05-07)
- Bencher project `quartzite` already created at bencher.dev
- Fork PRs cannot read repository secrets → bench run and secret-bearing upload must be separate workflows
- `workflow_run` context runs with base-branch permissions, enabling secret access for fork PRs
- `dawidd6/action-download-artifact@v6` required for cross-run artifact download in `workflow_run` context

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-core/benches/` exists with criterion benchmarks covering signal emit (typed + dynamic via `emit!`), property R/W (`read_property` / `write_property`), object-tree lookup (`find_by_path` / `find_by_name`) |
| AC2 | `cargo bench --workspace` runs clean locally and produces parseable criterion output |
| AC3 | `quartzite-core/Cargo.toml` has `criterion = "0.8"` as a dev-dep and a `[[bench]] harness = false` entry per bench file |
| AC4 | `.github/workflows/base_benchmarks.yml` exists, triggers on push to `master`, calls `bencher run` with Student's t-test threshold (`--threshold-test t_test`, `--threshold-max-sample-size 64`, `--threshold-upper-boundary 0.99`) |
| AC5 | `.github/workflows/fork_pr_benchmarks_run.yml` exists, triggers on `pull_request` events, runs benches, uploads `benchmark_results.txt` and `event.json` as artifacts without reading any secrets |
| AC6 | `.github/workflows/fork_pr_benchmarks_track.yml` exists, triggers via `workflow_run` on the above workflow completing, downloads both artifacts, calls `bencher run --file` to upload results and post PR comment |
| AC7 | All three workflow files reference `--project quartzite` |
| AC8 | `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check`, `cargo test`, `cargo build -p quartzite --no-default-features`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` all clean |

## Open questions

(none)
