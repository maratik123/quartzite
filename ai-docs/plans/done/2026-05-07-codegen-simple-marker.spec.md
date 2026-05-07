# Codegen: drop `#[inline]` from generated trait-impl methods

**Source:** issue #117
**Date:** 2026-05-07
**Tracked in:** #117

## Scope

1. `quartzite-macros/src/extend/codegen.rs` — drop `#[inline]` from generated trait-impl methods: `AsObject::{object_base, object_base_mut, as_any, as_any_mut}`, `As<Self>` self-ref accessor pair, parent-chain delegation pair, mixin leaf accessor pair.
2. `quartzite-macros/src/object_impl/codegen.rs` — drop `#[inline]` from generated `Object::{meta_object, connect_signal, emit_signal}`; keep `#[inline]` on `__meta_init_<Foo>()` (concrete free fn, correct marker).
3. `quartzite-macros/src/meta_enum/codegen.rs` — drop `#[inline]` from generated `IntoValue::into_value`. No marker on `FromValue::from_value` (already absent, non-simple body).
4. Codegen tests (token-stream-contains style) for each file verifying: trait-impl methods have no `#[inline]`, `__meta_init_<Foo>()` retains `#[inline]`, non-simple methods have neither marker.

## Out of scope

- Part 1 (hand-written trait declarations) — closed by PR #120.
- Hand-written `Signal<Args>` / `ObjectRef<T>` / `WeakRef<T>` — issue #116.
- Adding any `_Simple._` marker form to codegen — the trait-declaration `/// _Simple._` tag (PR #120) is the canonical signal; generated impls inherit it by Rust's rustdoc inheritance rules.

## Deferred

- None.

## Key decisions

| Question | Decision |
|---|---|
| Marker form for generated trait-impl methods (options a/b/c) | Option (c): emit no marker on generated trait-impl methods. The trait declaration carries `/// _Simple._` (Part 1/PR #120); generated impls satisfy the contract by construction. Avoids fighting `quote!` macro mechanics. |
| Keep `#[inline]` on `__meta_init_<Foo>()`? | Yes — it is a concrete free fn, not a trait-impl method; `#[inline]` is the correct marker for its position. |
| Test style | `token-stream-contains` (`.contains("# [inline]")` / `.matches(...).count() == N`), mirroring the 2026-05-02 inline-simple-fns codegen tests. Snapshot tests would couple too tightly to whitespace. |

## Technical constraints

- Codegen uses `quote!` macro blocks; `// _Simple._` is not a syntactic token and cannot be emitted via `quote!`. Option (c) avoids this entirely.
- The marker-form decision tree (from PR #121): trait-impl methods → `// _Simple._`; concrete free fn → `#[inline]`. Generated impls are always in the trait-impl position even when the user struct is concrete (`impl AsObject for Foo`).
- `cargo build -p quartzite --no-default-features` must remain clean (derive-free / no_std path).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `extend/codegen.rs` generated trait-impl methods (`AsObject::*`, `As<Self>` accessors, parent-chain delegation, mixin leaf accessors) do not contain `#[inline]` in the emitted token stream. |
| AC2 | `object_impl/codegen.rs` generated `Object::{meta_object, connect_signal, emit_signal}` do not contain `#[inline]` in the emitted token stream. |
| AC3 | `object_impl/codegen.rs` generated `__meta_init_<Foo>()` continues to contain `#[inline]` in the emitted token stream. |
| AC4 | `meta_enum/codegen.rs` generated `IntoValue::into_value` does not contain `#[inline]` in the emitted token stream. |
| AC5 | Codegen tests assert the above token-stream contents (present/absent) for each of the three files. |
| AC6 | `cargo build && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check && cargo test` all pass clean. |
| AC7 | `cargo build -p quartzite --no-default-features` passes clean. |
| AC8 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` passes clean. |

## Open questions

None.
