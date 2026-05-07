# Design: macro-derived object bench

**Issue:** #143
**Date:** 2026-05-07

## Approach

Add `benches/macro_object.rs` to the root `quartzite` crate. The bench mirrors the 6 functions in `quartzite-core/benches/signal_property.rs` but drives them through a macro-derived struct instead of the hand-rolled one.

### Fixture struct shape

```rust
#[derive(Extend, DeriveObject)]
#[root]
struct BenchObject {
    #[base]
    object_base: ObjectBase,
    #[prop(notify = count_changed)]
    pub count: i64,
    #[signal]
    pub count_changed: Signal<(i64,)>,
}

#[object_impl]
impl BenchObject {}
```

Key decisions from investigation:

- **`#[root]` is required** for a standalone object (no parent type). Without it, `#[derive(Extend)]` requires either a `#[base]` field pointing to another derived type or a `#[mixin]` — there is no way to satisfy `AsObject` for a pure `ObjectBase` root without `#[root]`.
- **`#[base] object_base: ObjectBase`** makes `Extend` emit `impl AsObject for BenchObject` with direct field access, which is the same pattern as every example in the codebase.
- **`count: i64`** matches the existing bench. `Value::Int` wraps `i64` natively; `i32` goes through a checked `TryFrom<i64>` conversion which adds overhead not present in the reference bench. Using `i64` keeps the two benches comparable.
- **`count_changed: Signal<(i64,)`** is the notify signal for the property. The `write_property` codegen emits `emit!(self.count_changed, &(v.clone(),))` — so the notify signal type must match the property type. The `dynamic_emit_signal` bench uses signal name `"count_changed"` with `Value::Int(1)` as the arg.
- **`#[object_impl] impl BenchObject {}`** — an empty impl block is sufficient. It generates: the `Object` trait impl (delegating to hidden-mod functions from `DeriveObject`), the `META_BenchObject` static, and lookup/invoke dispatch fns.

### Why the facade crate

The macros expand generated paths as `::quartzite::core::...`. The bench must live in a crate where `quartzite` resolves to the facade itself. `quartzite-core` cannot host this bench without circular deps. The root `quartzite` Cargo.toml is the only correct host.

### Signal for emit benches

The reference bench uses `sig: Signal<(i32,)>` and a separate signal from the property. For the macro bench, using `count_changed: Signal<(i64,)>` (the notify signal) keeps the fixture minimal — there is no need for an independent unreferenced `sig` field, and it means the bench struct is a realistic production-like usage rather than a synthetic one.

The four emit bench functions map as follows:
- `typed_no_slots` — `obj.count_changed.emit_unconditionally(&(black_box(1i64),))`
- `typed_one_slot` — connect a typed closure to `count_changed`, then `emit_unconditionally`
- `emit_macro_one_slot` — connect a typed closure to `count_changed`, then `emit!(obj.count_changed, &(black_box(1i64),))`
- `dynamic_emit_signal` — **connect a typed closure to `count_changed` before the timed loop** (mirroring the reference bench which connects a slot before timing `emit_signal`), then `obj.emit_signal(black_box("count_changed"), black_box(args.as_slice()))` with `args = [Value::Int(1)]`

The slot connection in `dynamic_emit_signal` is required to make the bench comparable to the reference: both measure the full dispatch path including slot invocation overhead, not an empty-list fast path.

### `property_rw::write` divergence from reference

The macro-generated `write_property` for `#[prop(notify = count_changed)]` calls `emit!(self.count_changed, &(v,))` on every successful write. The reference bench's hand-rolled `write_property` does not — no notify signal exists on that struct. This means the macro bench's `property_rw::write` bench includes notify-signal emit overhead absent in the reference.

This divergence is **intentional**: the bench exercises realistic macro-generated code, not a stripped-down version. The overhead reflects actual production behaviour of `#[prop(notify = ...)]`. Any user comparing the two benches should be aware that the macro bench's `write` path does more work by design.

No structural change is needed — the bench is correct as specified.

### Alternatives rejected

- **Adding a separate `sig: Signal<(i32,)>` field**: adds a field with no `#[signal]` annotation that is not wired to anything — untidy and diverges from typical macro usage. Using `count_changed` exercises the same code paths.
- **Hosting the bench in `quartzite-core`**: circular dep; proc-macro paths would not resolve.
- **Using `i32` for the property**: introduces a `TryFrom<i64>` overhead in `write_property` / `emit_signal` dynamic dispatch that the reference bench does not have, making apples-to-apples comparison harder.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `criterion = "0.8"` dev-dependency and `[[bench]]` entry to root `Cargo.toml` | `Cargo.toml` | — |
| 2 | Create `benches/` directory and write `benches/macro_object.rs` with the `BenchObject` fixture and 6 bench functions | `benches/macro_object.rs` | 1 |
| 3 | Run `cargo build` (refreshes `Cargo.lock`), then verify all quality gates: `cargo bench --workspace --no-run`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check`, `cargo test`, doc gate | `Cargo.lock` | 2 |

Three tasks — well within the 7-task budget for a single issue.

## Risks

- **`emit!` macro path resolution**: the `emit!` macro is re-exported from `quartzite_core::emit` into `quartzite::prelude`. In the bench file, it is imported via `use quartzite::prelude::*`. The `emit!` macro internally uses `::quartzite::core::AsObject` — this resolves correctly only when the bench crate *is* the `quartzite` facade crate. Confirmed: root `Cargo.toml` has `name = "quartzite"`, so `::quartzite` resolves to `self` and all generated paths compile. **Mitigation:** build verification in Task 3 catches any resolution failure.
- **`missing_docs` lint on the bench file**: the root `src/lib.rs` has `#![deny(missing_docs)]`, but bench binaries are not subject to `lib.rs` lint attributes. However, `cargo clippy --all-targets` may surface warnings from bench targets. The bench items (struct, `impl` block, bench functions) are private to the binary and need no doc comments — standard criterion bench pattern. **Mitigation:** confirm via clippy gate in Task 3; add `#[allow(missing_docs)]` to the bench file only if clippy surfaces the lint.
- **`required-features = ["derive"]` on `[[bench]]`**: the `derive` feature is in `default` features, so `cargo bench` works without explicit `--features`. Adding `required-features = ["derive"]` is stated in the spec for consistency with examples. If a future `cargo bench --no-default-features` is run, the bench is skipped rather than failing to compile. **Mitigation:** include `required-features = ["derive"]` exactly as the spec requires.
- **Signal arg type mismatch at runtime**: `dynamic_emit_signal` passes `[Value::Int(1)]` to `emit_signal("count_changed", ...)`. The generated `__emit_signal_BenchObject` calls `FromValue::from_value` on the first element expecting `i64` — `Value::Int(i64)` matches directly (no `TryFrom`). **No mitigation needed.**

## Test Design

This task adds a benchmark binary, not a library. Criterion benchmarks are self-validating at the structural level: if the bench compiles and runs under `cargo bench`, the fixture and all 6 functions are correct. There is no non-trivial logic in the bench itself that requires a separate `#[cfg(test)]` module.

The correctness proof is:
- `cargo bench --workspace --no-run` — confirms all 6 bench functions link without errors.
- `cargo test --workspace` — exercises the macro expansion paths via existing `quartzite-macros` unit tests and `quartzite` integration tests.

No additional test file is required for this task.

## Open questions

- None.
