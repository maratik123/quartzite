# Design: Criterion benchmarks with Bencher CI integration

**Issue:** #135
**Date:** 2026-05-07

## Approach

### Benchmark placement

The spec lists `quartzite-core/benches/` as the home for all three benchmark
categories, but the object-tree benchmarks (`find_by_name` / `find_by_name_in`)
require `quartzite-runtime::ObjectTree`. Adding `quartzite-runtime` as a dev-dep
to `quartzite-core` would create a dependency cycle (`quartzite-runtime` → (dep)
→ `quartzite-core` → (dev-dep) → `quartzite-runtime`), which Cargo rejects in a
workspace.

**Resolution:** split into two bench files across two crates:

| Crate | Bench file | Coverage |
|---|---|---|
| `quartzite-core` | `benches/signal_property.rs` | signal emit typed + `emit!`, `read_property`, `write_property` |
| `quartzite-runtime` | `benches/object_tree.rs` | `find_by_name` (hash lookup), `find_by_name_in` (BFS subtree) |

Both crates get `criterion = "0.8"` as a dev-dep and a `[[bench]]` entry with
`harness = false`.

### `find_by_path` clarification

`find_by_path` does not exist in the codebase. The spec's AC1 phrase
"find_by_path / find_by_name" maps to the two existing tree-lookup methods:

- `ObjectTree::find_by_name` — O(1) hash lookup returning `&[ObjectId]`
- `ObjectTree::find_by_name_in` — BFS subtree scan returning `Vec<ObjectId>`

The bench group is named `object_tree_lookup`; the two functions serve as
`find_by_name` (global) and `find_by_name_in` (subtree / path-scoped), which is
what the spec intends.

### Benchmark fixture design

**Signal + property bench** (`quartzite-core`):

A minimal struct `BenchObject` implementing `AsObject` + `Object` is defined
inside `signal_property.rs` (same pattern as the unit-test `StubObject` in
`object_tree.rs`). It holds:

- One `Signal<(i32,)>` field for typed emit.
- One `i32` property `"count"` accessible via `read_property` / `write_property`.
- A `MetaObject` static with one `PropertyMeta` entry and one `SignalMeta` entry,
  using fn-pointer dispatch (matching the existing `StubObject` pattern but
  extended with working property/signal lookup fns).

No proc-macro (`#[derive(Object)]`) is used in the bench to avoid a dep on
`quartzite-macros` from `quartzite-core`. The lookups and dispatch are
hand-written (equivalent to what the macro would generate), keeping the bench
file self-contained with only the existing `quartzite-core` dev-dep set.

**Tree bench** (`quartzite-runtime`):

`BenchTree` setup helper builds a tree of depth-3 with a known branching factor
(e.g. 4 children per node → 1 + 4 + 16 = 21 nodes). Objects are named
deterministically so a target can always be found. Setup is done in the
`Criterion` setup closure, not benchmarked itself.

### Benchmark groups

`signal_property.rs`:

| Group | Bench | Entry point |
|---|---|---|
| `signal_emit` | `typed_no_slots` | `Signal::emit_unconditionally` (0 slots) |
| `signal_emit` | `typed_one_slot` | `Signal::emit_unconditionally` (1 direct slot) |
| `signal_emit` | `emit_macro_one_slot` | `emit!(obj.sig, &(1,))` |
| `signal_emit` | `dynamic_emit_signal` | `Object::emit_signal("sig", &[Value::Int(1)])` |
| `property_rw` | `read` | `Object::read_property("count")` |
| `property_rw` | `write` | `Object::write_property("count", Value::Int(42))` |

`object_tree.rs`:

| Group | Bench | Entry point |
|---|---|---|
| `object_tree_lookup` | `find_by_name_hit` | `tree.find_by_name("leaf-3-3")` (present) |
| `object_tree_lookup` | `find_by_name_miss` | `tree.find_by_name("absent")` |
| `object_tree_lookup` | `find_by_name_in_hit` | `tree.find_by_name_in(root, "leaf-3-3")` |
| `object_tree_lookup` | `find_by_name_in_miss` | `tree.find_by_name_in(root, "absent")` |

### GitHub Actions — three-workflow Bencher fork-PR pattern

The three workflow files follow the official Bencher pattern for public repos
where fork PRs cannot access repository secrets:

1. **`base_benchmarks.yml`** — triggers on `push` to `master`; installs Bencher,
   runs `cargo bench --workspace`, pipes output through `bencher run` with
   Student's t-test threshold and `--error-on-alert`.

2. **`fork_pr_benchmarks_run.yml`** — triggers on `pull_request` events; runs
   `cargo bench --workspace 2>&1 | tee benchmark_results.txt`; uploads
   `benchmark_results.txt` and `event.json` (GitHub event payload) as artifacts.
   No secrets read.

3. **`fork_pr_benchmarks_track.yml`** — triggers on `workflow_run` when
   `fork_pr_benchmarks_run` completes; uses `dawidd6/action-download-artifact@v6`
   to fetch both artifacts; calls `bencher run --file benchmark_results.txt`
   with `BENCHER_API_TOKEN` secret; reads `event.json` to extract the PR number
   for the comment.

All three reference `--project quartzite` and `--adapter rust_criterion`.

### Rejected alternatives

- **`quartzite-core/benches/` for all three**: impossible due to circular
  dependency constraint described above.
- **`quartzite-runtime/benches/` for all**: would satisfy AC1 by treating the
  spec's "quartzite-core/benches/" as an imprecise label for "the hot-path
  benchmarks". Rejected in favour of the split: signal and property are
  `quartzite-core` concerns; tree lookup belongs in `quartzite-runtime`.
- **Custom Bencher emitter**: not needed; `rust_criterion` adapter parses
  criterion stdout.
- **`bencherdev/bencher@latest` pin**: spec explicitly says `@main`; no change.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `criterion = "0.8"` dev-dep to `quartzite-core/Cargo.toml` and a `[[bench]]` entry for `signal_property` with `harness = false` | `quartzite-core/Cargo.toml` | — |
| 2 | Write `quartzite-core/benches/signal_property.rs`: `BenchObject` fixture, `signal_emit` group (4 benches), `property_rw` group (2 benches) | `quartzite-core/benches/signal_property.rs` | 1 |
| 3 | Add `criterion = "0.8"` dev-dep to `quartzite-runtime/Cargo.toml` and a `[[bench]]` entry for `object_tree` with `harness = false` | `quartzite-runtime/Cargo.toml` | — |
| 4 | Write `quartzite-runtime/benches/object_tree.rs`: `build_bench_tree` helper, `object_tree_lookup` group (4 benches) | `quartzite-runtime/benches/object_tree.rs` | 3 |
| 5 | Write `.github/workflows/base_benchmarks.yml` | `.github/workflows/base_benchmarks.yml` | 2, 4 |
| 6 | Write `.github/workflows/fork_pr_benchmarks_run.yml` | `.github/workflows/fork_pr_benchmarks_run.yml` | — |
| 7 | Write `.github/workflows/fork_pr_benchmarks_track.yml` | `.github/workflows/fork_pr_benchmarks_track.yml` | 6 |

## Risks

- **`BenchObject` property/signal dispatch must be correct**: if the hand-written
  `read_property` / `write_property` / `emit_signal` implementations do not match
  what the macro would generate, the bench measures the wrong code path. Mitigation:
  keep the implementation as close to the `combined.rs` example as possible; verify
  `cargo bench` produces non-zero timing for both the hit and miss cases.
- **AC8 clippy `--all-targets`**: bench files are included in `--all-targets`;
  clippy must pass. Mitigation: run `cargo clippy --all-targets -- -D warnings`
  locally before committing.
- **`missing_docs` gate on bench files**: `quartzite-core` and `quartzite-runtime`
  have `#![deny(missing_docs)]`. Bench files are not library source so the gate
  does not apply — but any pub items declared in the bench file must have doc
  comments. Mitigation: make all helpers inside the bench file non-pub (use
  `fn`, not `pub fn`).
- **Bencher `--error-on-alert` blocks master merges on regression**: intentional
  per spec, but a noisy timer baseline could false-positive. Mitigation: the
  Student's t-test with `--threshold-max-sample-size 64` and
  `--threshold-upper-boundary 0.99` is conservative enough for a shared CI runner.
- **`workflow_run` context knows nothing about the PR head branch**: the track
  workflow must reconstruct PR number from the downloaded `event.json`. Mitigation:
  use the official Bencher pattern exactly — parse
  `github.event.workflow_run.pull_requests[0].number` or fall back to the
  `event.json` file.

## Test Design

Benchmarks are not unit-tested (criterion is the test harness). The correctness
of the bench fixture is validated by:

- **Smoke run**: `cargo bench --workspace` must complete without panic or compile
  error (AC2).
- **Non-zero output**: criterion must emit at least one timing line per bench
  function. If `BenchObject::read_property("count")` returns `None` the bench
  still compiles but measures nothing meaningful — the implementation must return
  `Some(Value::Int(...))`.
- **`cargo test --workspace`**: existing test suites must not regress; the bench
  file adds no `#[test]` items.

## Open questions

(none — both resolved by product owner 2026-05-07)

- **Bench split**: accepted — tree-lookup benches live in `quartzite-runtime/benches/` to avoid circular dep.
- **`find_by_path`**: bench `find_by_name` + `find_by_name_in` as-is; no new API needed.
