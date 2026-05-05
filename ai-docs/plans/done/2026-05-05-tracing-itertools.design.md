# Design: feat(deps): integrate tracing instrumentation and itertools test helpers

**Issue:** #89
**Date:** 2026-05-05

## Approach

### tracing in `quartzite-core`

Add `tracing = { version = "0", default-features = false }` to `quartzite-core/Cargo.toml`. The
`default-features = false` flag is mandatory: the default `tracing` feature set pulls in `std`, which
breaks the no_std build path (`cargo build -p quartzite --no-default-features`). With default features
disabled, `tracing` macros (`trace!`, `debug!`, etc.) remain available as no-ops when no subscriber is
installed — there is nothing to enable at compile time in the library; the subscriber is the
application's responsibility.

The `std` feature of `quartzite-core` will **not** be wired into `tracing`'s feature flags for now
(deferred per spec). Adding `tracing/std` conditionally via the `std` feature gate is straightforward
but low-value before a subscriber is chosen.

Instrumented call sites in `quartzite-core/src/signal.rs`:
- `Signal::emit` — `trace!` at entry (slot count, `SingleShot` path)
- `Signal::connect` / `connect_typed` / `connect_queued` / `connect_auto` — `trace!` at entry
- `Signal::disconnect` — `trace!` at entry

### tracing in `quartzite-runtime`

Add `tracing = "0"` (full default features, std-only crate) to `quartzite-runtime/Cargo.toml`.

Instrumented call sites:
- `EventLoop::post` — `trace!` at entry
- `EventLoop::stop` — `debug!` at entry (stop is a higher-level lifecycle event)
- `ObjectTree::insert` — `debug!` (id, name, parent)
- `ObjectTree::destroy` — `debug!` (id)
- `ObjectTree::rename` — `debug!` (id, old name, new name)
- `ObjectTree::reparent` — `debug!` (id, new parent)

`ConnectionTable::post` (the `QueuedDispatcher` impl) is intentionally not instrumented — it is an
internal forwarding shim; the `EventLoop::post` call it delegates to already emits a trace event.

### itertools in `quartzite-core` (dev-dep)

Add `itertools = "0"` to `[dev-dependencies]` in `quartzite-core/Cargo.toml`.

Simplification target — `find_by_name_returns_all_with_same_name` in
`quartzite-runtime/src/object_tree.rs` (unit test, `#[cfg(test)]` module):

Current pattern:
```rust
assert!(ids.contains(&a), "missing a: {ids:?}");
assert!(ids.contains(&b), "missing b: {ids:?}");
assert_eq!(ids.len(), 2);
```

Replacement using `itertools::Itertools::sorted_unstable_by_key` + `assert_equal`:
```rust
use itertools::Itertools as _;
let key = |id: &ObjectId| id.raw();
itertools::assert_equal(
    ids.iter().copied().sorted_unstable_by_key(key),
    [a, b].into_iter().sorted_unstable_by_key(key),
);
```

`ObjectId` derives `Eq + Hash` but not `Ord`, so `sorted_unstable` is unavailable; `sorted_unstable_by_key`
on the underlying `u64` raw value is used instead. This replaces three assertions with one. The sort on
both sides makes the comparison order-independent, which was the original intent of the three-assertion
pattern (checking presence, not position).

### itertools in `quartzite-runtime` (dev-dep)

Add `itertools = "0"` to `[dev-dependencies]` in `quartzite-runtime/Cargo.toml`.

Simplification target — `destroy_is_depth_first_post_order` in
`quartzite-runtime/tests/object_tree.rs` (integration test):

Current pattern:
```rust
let gc_pos   = order.iter().position(|n| n == "gc").unwrap();
let c1_pos   = order.iter().position(|n| n == "c1").unwrap();
let root_pos = order.iter().position(|n| n == "root").unwrap();

assert!(gc_pos < c1_pos, "gc must be destroyed before its parent c1");
assert!(c1_pos < root_pos, "c1 must be destroyed before root");
assert_eq!(order.last().unwrap(), "root", "root must be destroyed last");
```

Replacement using `tuple_windows`:
```rust
use itertools::Itertools as _;
let positions: Vec<usize> = ["gc", "c1", "root"]
    .iter()
    .map(|name| order.iter().position(|n| n == name).unwrap())
    .collect();
assert!(positions.iter().copied().tuple_windows().all(|(a, b)| a < b),
    "destruction order must be gc < c1 < root; got {order:?}");
assert_eq!(order.last().unwrap(), "root", "root must be destroyed last");
```

The `tuple_windows` call verifies the strict-ascending-positions constraint in a single expression.
The final `assert_eq!` on `order.last()` is kept as a separate guard because it is more expressive
than inferring "last" from the position chain alone.

### `ai-docs/context.md` update

Add an entry under **Open Questions** documenting the `futures-util` deferred decision as specified
in AC10.

### Rejected alternatives

- **`#[instrument]` proc-macro on methods instead of inline `trace!`/`debug!`** — too coarse; wraps
  the entire function body in a span even when there is no meaningful async context. Inline macro calls
  are sufficient and cheaper for synchronous hot-path functions like `emit`.
- **Adding `tracing/std` to `quartzite-core`'s `std` feature** — deferred per spec; low value before a
  concrete subscriber strategy is chosen.
- **`itertools` as a production dep** — out of scope per spec; no iterator chains in production code
  currently require it.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `tracing = { version = "0", default-features = false }` to `quartzite-core` deps; run `cargo update` | `quartzite-core/Cargo.toml`, `Cargo.lock` | — |
| 2 | Add `tracing = "0"` to `quartzite-runtime` deps; run `cargo update` | `quartzite-runtime/Cargo.toml`, `Cargo.lock` | — |
| 3 | Add `itertools = "0"` to `quartzite-core` dev-deps | `quartzite-core/Cargo.toml` | — |
| 4 | Add `itertools = "0"` to `quartzite-runtime` dev-deps | `quartzite-runtime/Cargo.toml` | — |
| 5 | Instrument `Signal::emit`, `connect`, `connect_typed`, `connect_queued`, `connect_auto`, `disconnect` with `tracing::trace!` | `quartzite-core/src/signal.rs` | 1 |
| 6 | Instrument `EventLoop::post` (trace) and `EventLoop::stop` (debug) | `quartzite-runtime/src/event_loop.rs` | 2 |
| 7 | Instrument `ObjectTree::insert`, `destroy`, `rename`, `reparent` with `tracing::debug!` | `quartzite-runtime/src/object_tree.rs` | 2 |
| 8 | Simplify `find_by_name_returns_all_with_same_name` unit test using `itertools::assert_equal` + `sorted_unstable` | `quartzite-runtime/src/object_tree.rs` | 3, 4 |
| 9 | Simplify `destroy_is_depth_first_post_order` integration test using `tuple_windows` | `quartzite-runtime/tests/object_tree.rs` | 4 |
| 10 | Update `ai-docs/context.md` Open Questions section with `futures-util` deferred decision | `ai-docs/context.md` | — |
| 11 | Run full CI gate: `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`, `cargo build -p quartzite --no-default-features` | — | 1–10 |

Tasks 1–4 are independent of each other and can be done in one commit. Tasks 5–9 depend on the
corresponding Cargo.toml changes (1–4). Task 10 is independent. Task 11 validates everything.

## Risks

- **no_std build break**: `tracing` with default features enabled in `quartzite-core` would pull in
  `std` and break `cargo build -p quartzite --no-default-features`. Mitigation: always use
  `default-features = false` in `quartzite-core/Cargo.toml`; AC9 + task 11 will catch regressions.
- **`tracing` macro call sites in `#[cfg(not(feature = "std"))]` paths**: macros from `tracing`
  (with default features off) are still usable in no_std contexts as no-ops. No conditional
  compilation is needed around the `trace!` calls. Confirmed by tracing's own documentation for
  no_std support.
- **`itertools::assert_equal` import conflicts with `assert_eq!`**: `assert_equal` is a free function
  in `itertools`, not a macro — no name clash. `use itertools::Itertools as _` glob-imports the
  extension trait methods without polluting the namespace.
- **`sorted_unstable` on `ObjectId`**: `ObjectId` derives `Eq + Hash` but not `Ord` (confirmed by
  reading `quartzite-core/src/id.rs`). The design uses `sorted_unstable_by_key(|id| id.raw())` on
  both sides of `assert_equal` to avoid this limitation.
- **Clippy `missing_docs` on tracing instrumentation**: `trace!`/`debug!` are statements inside
  existing documented functions — no new public items are added, no doc-gate risk.
- **`tuple_windows` produces wrong result if `positions` is empty or has only one element**: the test
  builds a tree of 4 nodes and checks exactly 3 positions; the slice is always length 3. No edge-case
  risk here.

## Test Design

Tasks 5–9 are modifications to existing tests or production code with existing tests; no new test
infrastructure is needed.

### Task 5 — signal tracing (`quartzite-core/src/signal.rs`)

The `tracing` calls are fire-and-forget with no subscriber installed in tests. Existing tests
continue to pass unchanged — they do not assert on tracing output. No new tests required.

### Task 6 — EventLoop tracing (`quartzite-runtime/src/event_loop.rs`)

Same rationale as task 5. The existing `post_from_other_thread_executes_on_loop_thread`,
`post_multiple_preserves_order`, and `stop_terminates_run` tests remain the correctness validators.

### Task 7 — ObjectTree tracing (`quartzite-runtime/src/object_tree.rs`)

Same rationale. Existing unit and integration tests remain the correctness validators.

### Task 8 — itertools simplification of `find_by_name_returns_all_with_same_name`

- **Location:** `quartzite-runtime/src/object_tree.rs` `#[cfg(test)]` module
- **Entry point:** `find_by_name_returns_all_with_same_name`
- **Scenario:** Two objects with the same name "dup" inserted; `find_by_name("dup")` returns both ids.
- **Before:** three separate `contains` + `len` assertions
- **After:** single `itertools::assert_equal` on iterators sorted by `id.raw()` — semantically equivalent,
  order-independent; `sorted_unstable_by_key` used because `ObjectId` does not implement `Ord`
- **No new fixture needed** — existing `StubObject::named` helper is reused.

### Task 9 — itertools simplification of `destroy_is_depth_first_post_order`

- **Location:** `quartzite-runtime/tests/object_tree.rs`
- **Entry point:** `destroy_is_depth_first_post_order`
- **Scenario:** root → c1 → gc / c2 tree; `destroy(root)` must fire `Drop` in post-order (leaves first).
- **Before:** three `position`-based variables + three assertions
- **After:** `tuple_windows().all(|(a, b)| a < b)` over a collected positions vec + one
  `assert_eq!(order.last(), "root")` guard
- **No new fixture needed** — existing `LogObj` helper is reused.

### Task 10 — `ai-docs/context.md`

Prose-only change; no test impact.

## Open questions

_None._
