# bench: criterion benchmarks for macro-derived objects

**Source:** issue #143
**Date:** 2026-05-07
**Tracked in:** #143

## Scope

1. New bench file `./benches/macro_object.rs` (root `quartzite` facade package — has `quartzite-macros` available, avoids circular deps)
2. Macro-derived `BenchObject` fixture using `#[derive(Extend, DeriveObject)]` + `#[root]` + `#[base]` + `#[prop(notify = count_changed)]` + `#[signal]` + `#[object_impl]`
3. Same 6 bench functions as `quartzite-core/benches/signal_property.rs` in two groups:
   - `signal_emit`: `typed_no_slots`, `typed_one_slot`, `emit_macro_one_slot`, `dynamic_emit_signal`
   - `property_rw`: `read`, `write`
4. Root `./Cargo.toml`: add `criterion = "0.8"` dev-dep + `[[bench]] name = "macro_object" harness = false`

## Out of scope

- Object-tree lookup benchmarks (already in `quartzite-runtime/benches/object_tree.rs`)
- Benchmarking the proc-macro compilation step itself
- Workflow file changes (Bencher CI picks up `cargo bench --workspace` automatically)

## Deferred

- None

## Key decisions

| Question | Decision |
|---|---|
| Which crate hosts the bench? | Root `quartzite` facade crate — only place where both `quartzite-core` types and `quartzite-macros` derive macros are available without circular deps |
| Signal type for emit benches? | `Signal<(i64,)>` matching `count: i64` field; dynamic emit uses signal name `"count_changed"` |
| criterion version? | `0.8` (pinned as `0.x` per AGENTS.md; matches existing `quartzite-core` dev-dep; verified current at 0.8.2 on 2026-05-07) |
| `required-features` on bench entry? | `required-features = ["derive"]` for explicitness, consistent with examples in same Cargo.toml |

## Technical constraints

- Macros generate paths via `::quartzite::core`; bench must live in the `quartzite` crate so generated code resolves correctly
- `quartzite-core` cannot depend on `quartzite-macros` (would be circular); only the facade can combine both
- All quality gates (build, test, clippy, fmt, doc) must pass per AC5

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `./benches/macro_object.rs` exists with a `#[derive(Extend, DeriveObject)]` + `#[object_impl]` fixture and the same 6 bench functions (`typed_no_slots`, `typed_one_slot`, `emit_macro_one_slot`, `dynamic_emit_signal`, `read`, `write`) as `quartzite-core/benches/signal_property.rs` |
| AC2 | `cargo bench --workspace` runs clean and produces parseable criterion output for all bench groups (`signal_emit` and `property_rw`) |
| AC3 | Root `./Cargo.toml` has `criterion = "0.8"` in `[dev-dependencies]` and a `[[bench]]` entry with `name = "macro_object"` and `harness = false` |
| AC4 | All three Bencher CI workflows pick up the new bench binary automatically (no workflow changes needed) |
| AC5 | `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check`, `cargo test`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` all pass clean |

## Open questions

- None
