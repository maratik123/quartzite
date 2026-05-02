# Design: Public API Documentation

**Issue:** user description
**Date:** 2026-05-02

## Approach

### Summary

Add `///` doc comments and `# Examples` blocks to all undocumented public items across the four crates, add `#![deny(missing_docs)]` to each `lib.rs`, update CI to enforce it, and update the self-review agent checklist to catch regressions.

The work is purely additive doc text — no logic changes, no refactoring. The only structural changes are:

- `#![deny(missing_docs)]` inner attribute in four `lib.rs` files
- One new `env:` variable (`RUSTDOCFLAGS`) in `.github/workflows/ci.yml`
- Two new checklist lines in `.claude/agents/self-review.md`
- One new rule paragraph in the Code Style section of `AGENTS.md`

### Current state (measured with `RUSTDOCFLAGS="-D missing-docs"`)

120 missing-doc errors across the workspace:

| Category | Count |
|---|---|
| struct fields | 40 |
| associated functions | 22 |
| methods | 22 |
| modules | 15 |
| enum variants | 12 |
| free functions | 4 |
| structs | 3 |
| crate root | 1 |
| enum | 1 |

Affected files (by crate):
- `quartzite-macros/src/lib.rs` — 4 proc-macro functions have no `///` doc
- `quartzite-core/src/id.rs` — `ObjectId` struct + `new()`, `raw()`, `ConnectionId` struct + `new()`, `raw()`
- `quartzite-core/src/object_base.rs` — `ObjectBase::new()`, `ObjectBase::named()`, public fields (`name`, `outgoing_connections`, `dynamic_properties`, `signals_blocked`, `thread_id`)
- `quartzite-core/src/signal.rs` — `Signal::new()`
- `quartzite-core/src/receiver_guard.rs` — `ReceiverGuard` struct, `ReceiverGuard::new_pair()`
- `quartzite-core/src/meta.rs` — all `pub` struct fields in `PropertyFlags`, `PropertyMeta`, `ParamMeta`, `SignalMeta`, `MethodMeta`, `EnumEntry`, `EnumMeta`, `MetaObject`; `new()` constructors; `PropertyMeta::new()`, `ParamMeta::new()`, `SignalMeta::new()`, `MethodMeta::new()`, `EnumEntry::new()`, `EnumMeta::new()`, `MetaObject::new()`
- `quartzite-core/src/value.rs` — `CustomValue` trait methods (`type_name`, `clone_box`, `as_any`), `Value` enum variants, `TypeError` fields, `FromValue::from_value`, `IntoValue::into_value`
- `quartzite-core/src/traits.rs` — `AsObject` trait methods, `Object::meta_object`
- `quartzite-core/src/lib.rs` — module re-exports `id`, `receiver_guard` missing crate-level doc
- `quartzite-runtime/src/lib.rs` — crate root doc, all 8 module declarations
- `quartzite-runtime/src/application.rs` — `ApplicationError` enum + `AlreadyExists` variant, `Application::object_tree()`, `Application::connection_table()`, `Application::event_loop()`
- `quartzite-runtime/src/object_ref.rs` — `ObjectRef::new()`, `ObjectRef::id()`, `ObjectRef::downgrade()`, `WeakRef::new()`, `WeakRef::id()`, `WeakRef::is_valid()`
- `quartzite-runtime/src/factory.rs` — `ObjectFactory::new()`
- `quartzite-runtime/src/event_loop.rs` — `EventLoop::new()`
- `quartzite-runtime/src/connection_table.rs` — `ConnectionTable::new()`, `ConnectionRecord` fields, `SlotKind` variants
- `quartzite-runtime/src/thread_pool.rs` — `ThreadPool::new()`
- `quartzite-runtime/src/timer.rs` — `Timer::stop()`, `Timer::is_running()`
- `quartzite-runtime/src/object_tree.rs` — (all methods already documented)

### Example strategy

**Proc-macros (`quartzite-macros`):** all examples use `no_run` because proc-macro crates cannot use their own macros in doctests.

**Runtime items requiring an event loop:** `Application::new`, `EventLoop::run`, `Timer::start`, `ConnectionTable` — use `no_run`.

**Simple constructors and ID types:** use compiling doctests (no event loop needed).

**`ObjectBase`, `Signal`, `ReceiverGuard`:** use compiling doctests; these are pure library types with no external dependencies.

### Rejected alternatives

- **`#[allow(missing_docs)]` per-item:** defeats the purpose of the lint and creates maintenance debt.
- **`#![warn(missing_docs)]`:** spec mandates `deny`; `warn` would pass CI silently when new undocumented items are added.
- **Writing docs only for the spec-listed items and skipping the rest:** `#![deny(missing_docs)]` requires 100% coverage; all 120 currently-flagged items must be addressed.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Document `quartzite-macros` proc-macro functions (`derive_extend`, `derive_object`, `object_impl`, `derive_meta_enum`) with attribute syntax, expansion description, and `no_run` `# Examples` block | `quartzite-macros/src/lib.rs` | — |
| 2 | Document `quartzite-core` ID types: `ObjectId` struct + `new()` + `raw()`, `ConnectionId` struct + `new()` + `raw()` | `quartzite-core/src/id.rs` | — |
| 3 | Document `quartzite-core` `ObjectBase`: struct-level doc, `new()`, `named()`, all public fields (`name`, `outgoing_connections`, `dynamic_properties`, `signals_blocked`, `thread_id`) | `quartzite-core/src/object_base.rs` | — |
| 4 | Document `quartzite-core` `Signal::new()` and `ReceiverGuard` struct + `new_pair()` | `quartzite-core/src/signal.rs`, `quartzite-core/src/receiver_guard.rs` | — |
| 5 | Document `quartzite-core` meta types: all `pub` struct fields and `new()` constructors in `PropertyFlags`, `PropertyMeta`, `ParamMeta`, `SignalMeta`, `MethodMeta`, `EnumEntry`, `EnumMeta`, `MetaObject` | `quartzite-core/src/meta.rs` | — |
| 6 | Document `quartzite-core` value types: `CustomValue` methods, `Value` enum variants, `TypeError` fields, `FromValue::from_value`, `IntoValue::into_value`; also `AsObject` and `Object::meta_object` trait methods | `quartzite-core/src/value.rs`, `quartzite-core/src/traits.rs` | — |
| 7 | Add `#![deny(missing_docs)]` and crate-level doc to `quartzite-core/src/lib.rs`; add doc to the `id` and `receiver_guard` module re-exports if still needed | `quartzite-core/src/lib.rs` | 2, 3, 4, 5, 6 |
| 8 | Document `quartzite-runtime` application items: `ApplicationError` + `AlreadyExists` variant, `Application::object_tree()`, `Application::connection_table()`, `Application::event_loop()` | `quartzite-runtime/src/application.rs` | — |
| 9 | Document `quartzite-runtime` `ObjectRef<T>` and `WeakRef<T>`: structs + `new()` + `id()` + `downgrade()` / `is_valid()` | `quartzite-runtime/src/object_ref.rs` | — |
| 10 | Document `quartzite-runtime` factory, event loop, connection table, thread pool, and timer undocumented items: `ObjectFactory::new()`, `EventLoop::new()`, `ConnectionTable::new()` + `ConnectionRecord` fields + `SlotKind` variants, `ThreadPool::new()`, `Timer::stop()`, `Timer::is_running()` | `quartzite-runtime/src/factory.rs`, `quartzite-runtime/src/event_loop.rs`, `quartzite-runtime/src/connection_table.rs`, `quartzite-runtime/src/thread_pool.rs`, `quartzite-runtime/src/timer.rs` | — |
| 11 | Add `#![deny(missing_docs)]`, crate-level doc, and module-level docs to `quartzite-runtime/src/lib.rs` | `quartzite-runtime/src/lib.rs` | 8, 9, 10 |
| 12 | Add `#![deny(missing_docs)]` to `quartzite-macros/src/lib.rs` | `quartzite-macros/src/lib.rs` | 1 |
| 13 | Add `#![deny(missing_docs)]` to `quartzite/src/lib.rs` | `quartzite/src/lib.rs` | — |
| 14 | Add `# Examples` blocks to all single-line-only public doc items that currently have none (across all crates — approximately 40 items covering `ObjectExt` methods, `Signal::connect*`, `disconnect`, `emit`, `ConnectionType` variants, `PropertyFlags` constructors, `MetaObject` lookup methods, `Value::type_name`, `ObjectTree` methods, `Timer::connect_timeout`, etc.) | Multiple files across all crates | 2–11 |
| 15 | Update CI: add `-D missing-docs` to `RUSTDOCFLAGS` in `.github/workflows/ci.yml` | `.github/workflows/ci.yml` | 7, 11, 12, 13 |
| 16 | Update self-review agent: add two checklist items for `#![deny(missing_docs)]` and `# Examples` | `.claude/agents/self-review.md` | — |
| 17 | Add documentation rule to `AGENTS.md` Code Style section | `AGENTS.md` | — |

Tasks 1–6 and 8–10 are independent and can be done in any order or in parallel. Tasks 7 and 11 each depend on their respective crate's doc tasks being complete. Tasks 15–17 can be done alongside or after any order.

## Risks

- **`#![deny(missing_docs)]` on modules re-exported via `pub mod`:** `quartzite-core` and `quartzite-runtime` both use `pub mod` declarations that are also module-level re-exports. With `deny(missing_docs)`, every re-exported module must have a crate-level `//!` comment. Add these before enabling the lint. Mitigation: resolve all 120 errors first (tasks 1–14), then add the `#![deny]` attribute (tasks 7, 11–13).
- **`ConnectionRecord` and `SlotKind` are public but internal:** `ConnectionRecord` and `SlotKind` in `quartzite-runtime/src/connection_table.rs` are `pub` struct and enum at crate level and will trigger `missing_docs`. Their fields and variants need brief docs even if the items are intended as semi-internal. Mitigation: document them with short descriptions.
- **Proc-macro examples:** proc-macro crates cannot use their own macros in doctests. All examples for `quartzite-macros` use `no_run`. Mark carefully to avoid doctest failures. Mitigation: annotate all macro `# Examples` blocks with `# ```no_run`.
- **`cargo test --doc --workspace` must pass:** non-`no_run` / non-`ignore` doctests in `quartzite-core` (the `no_std`-compatible crate) must compile. Items that require `std` must either use `no_run` or be wrapped in `#[cfg(feature = "std")]`. The crate's default feature enables `std`, so most doctests can compile without `no_run`. Mitigation: run `cargo test --doc --workspace` as part of the verify step.
- **`quartzite-core` `no_std` doctests:** the crate supports `no_std + alloc`. Doctests run with the default `std` feature, so they compile normally. Only runtime items that need a running event loop need `no_run`.
- **`Value` enum variant docs:** variant-level `///` comments must be placed directly above each variant. `Value::Null`, `Bool(bool)`, etc. each need a one-liner. `Value::Custom` can reference `CustomValue`. Mitigation: straightforward — no architecture impact.

## Test Design

This task produces only documentation, not logic. No new tests are required under AGENTS.md rules. The existing test suite continues to provide behavioral coverage.

**Verification (as part of the verify step, not new tests):**

- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` exits 0 with zero errors
- `cargo test --doc --workspace` passes (all non-`no_run`/non-`ignore` doctests compile and run)
- `cargo clippy -- -D warnings` passes (no regression)
- `cargo fmt -- --check` passes

## Open questions

- None.
