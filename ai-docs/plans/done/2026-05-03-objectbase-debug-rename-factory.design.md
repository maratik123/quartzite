# Design: ObjectBase Debug, rename no-op, ObjectFactory singleton

**Issue:** #61
**Date:** 2026-05-03

## Approach

### 1. `#[derive(Debug)]` for `ObjectBase`

All four fields in `ObjectBase` already implement `Debug`:
- `id: ObjectId` — `#[derive(Debug)]` present in `quartzite-core/src/id.rs`
- `name: Option<String>` — stdlib type, `Debug` is standard
- `receiver_guard: Arc<ReceiverGuard>` — `Arc<T>` is `Debug` when `T: Debug`; `ReceiverGuard` does not derive `Debug`, but `Arc<T>` prints the pointer address even when `T` is not `Debug` (since Rust 1.x, `Arc<T>: Debug` requires `T: Debug`)

Wait — `Arc<T>: Debug` requires `T: Debug`. `ReceiverGuard` does not currently derive or impl `Debug`. Therefore `ObjectBase` cannot auto-derive `Debug` without also adding `Debug` to `ReceiverGuard`. The fix is to add `#[derive(Debug)]` to `ReceiverGuard` first (it is a zero-sized struct, so the derived impl is trivial), then add `#[derive(Debug)]` to `ObjectBase`.

- `signals_blocked: bool` — `Debug` is standard
- `thread_id: std::thread::ThreadId` — implements `Debug` in std

**`#[cfg]`-gated field:** `thread_id: std::thread::ThreadId` is present only when `feature = "std"` is enabled. Rust's `#[derive(Debug)]` respects `#[cfg]` attributes on individual fields — the derived impl only references `self.thread_id` in the `std` branch — so `cargo build -p quartzite-core --no-default-features` must be verified to compile cleanly after adding the derive (added as an explicit step in Task 1/2).

**Approach:** Add `#[derive(Debug)]` to `ReceiverGuard` in `quartzite-core/src/receiver_guard.rs`, then add `#[derive(Debug)]` to `ObjectBase` in `quartzite-core/src/object_base.rs`. No manual impl needed; the auto-derive is sufficient as noted in the spec.

**Doctest:** The `format!("{:?}", ObjectBase::new())` doctest in Task 2 calls `std::thread::current().id()` internally and therefore requires the `std` feature. The doctest must be marked `no_run` (or placed in a `#[cfg(feature = "std")]`-gated block) so the no-std CI path does not fail.

### 2. `ObjectTree::rename` no-op when name unchanged

Current implementation (`quartzite-runtime/src/object_tree.rs`, `rename` method, lines 222–238) always:
1. Removes from the old name bucket,
2. Calls `set_name_raw` on the object,
3. Inserts into the new name bucket.

When `new_name == current_name`, this is wasted work. More importantly, spec AC2 requires no event be fired (the rename event infrastructure is not yet present, but the no-op guard future-proofs the semantics).

**Approach:** After resolving `new_name: String` (line 223), retrieve the current name. If `current_name == Some(&new_name)`, return immediately. This short-circuits before any index mutation or `set_name_raw` call.

**Return type stays `()`:** The spec's "return `Ok(())`" is semantic shorthand for "succeed silently with no mutation" — it is *not* a signature change. `rename` continues to return `()`. Adding a `Result` return type would be a breaking API change with no AC requiring it. The no-op short-circuit is a plain `return;`.

**Rejected alternative:** Changing `rename` to return `Result<(), SomeError>` — not required by any AC, adds complexity, breaks call sites.

### 3. `ObjectFactory` as process-global singleton

Currently `ObjectFactory` is a plain struct with no global accessor. The spec requires it be accessible process-wide without an `Application` reference, using the same pattern as `ConnectionTable`.

`ConnectionTable` uses `OnceLock<Arc<ApplicationInner>>` inside `Application`. `ObjectFactory`, however, needs to be accessible *independently* of `Application` (AC4: "no `Application` reference required to obtain it"). The canonical pattern for this in the codebase is the `QUEUED_DISPATCHER: OnceLock<Arc<dyn ...>>` in `quartzite-core/src/signal.rs`, and the `APP: OnceLock<Arc<ApplicationInner>>` in `application.rs`.

**Approach:** Add a module-level `static FACTORY: OnceLock<Arc<RwLock<ObjectFactory>>>` to `quartzite-runtime/src/factory.rs`. Expose two functions:
- `ObjectFactory::install(factory: ObjectFactory) -> Result<(), FactoryAlreadySet>` — sets the global once; returns error if already set (mirrors `DispatcherAlreadySet` / `install_as_dispatcher`).
- `ObjectFactory::global() -> Option<Arc<RwLock<ObjectFactory>>>` — returns a clone of the `Arc` if installed.

`RwLock` is needed because `register` takes `&mut self` and must be callable after installation. `Arc<RwLock<ObjectFactory>>` satisfies `Send + Sync` for cross-thread access.

**Rejected alternative:** Embed `ObjectFactory` inside `Application` (same as `ConnectionTable`) — violates AC4, which requires access without an `Application` reference.

**Rejected alternative:** Use `Mutex` instead of `RwLock` — `create` is read-only (`&self`), so concurrent creation would block unnecessarily. `RwLock` is the natural fit.

A `FactoryAlreadySet` error type (unit struct or zero-field enum, implementing `Debug + Display + Error`) must be added, mirroring `DispatcherAlreadySet` and `ApplicationError::AlreadyExists`.

`Application::new` should call `ObjectFactory::install(ObjectFactory::new())` and ignore `FactoryAlreadySet` (same as it ignores the dispatcher-already-set case) so that the global factory is available whenever an `Application` exists. This wiring is a convenience registration only — subsequent `Application::new()` calls in the same process share the first factory, matching `OnceLock` semantics. `Application` does not own the factory.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `#[derive(Debug)]` to `ReceiverGuard` | `quartzite-core/src/receiver_guard.rs` | — |
| 2 | Add `#[derive(Debug)]` to `ObjectBase`; add doctest verifying `format!("{:?}", base)` compiles | `quartzite-core/src/object_base.rs` | 1 |
| 3 | Short-circuit `ObjectTree::rename` when new name equals current name | `quartzite-runtime/src/object_tree.rs` | — |
| 4 | Add tests for no-op `rename` (same name, state unchanged, no index corruption) | `quartzite-runtime/src/object_tree.rs` `#[cfg(test)]` and/or `quartzite-runtime/tests/object_tree.rs` | 3 |
| 5 | Add `FactoryAlreadySet` error type and `static FACTORY: OnceLock<…>` with `install`/`global` | `quartzite-runtime/src/factory.rs` | — |
| 6 | Wire `ObjectFactory::install` into `Application::new` (ignore `FactoryAlreadySet`) | `quartzite-runtime/src/application.rs` | 5 |
| 7 | Add tests for singleton install/global accessor | `quartzite-runtime/tests/factory.rs` | 5 |

Total: 7 tasks — within the 7-task limit, no split needed.

## Risks

- **`ReceiverGuard: Debug`:** `ReceiverGuard` is a zero-sized struct; `#[derive(Debug)]` emits `ReceiverGuard`. This is benign but now part of the public `Debug` output of `ObjectBase`. If `ReceiverGuard` is ever non-trivial, a custom `Debug` impl can replace the derive. Mitigation: no action needed now; the spec defers manual `Debug` to a future issue.
- **`OnceLock` in integration tests:** `OnceLock::set` is irrevocable per process. Factory singleton tests must run in isolated processes (separate integration test binary files) or accept that `install` can only succeed once per process — tests after the first call will get `Err(FactoryAlreadySet)`. Mitigation: design `global()` tests so they do not depend on a fresh `OnceLock`; use the `Err` path as a legitimate test scenario. Structure tests as: one test asserts first `install` succeeds; another asserts second `install` returns `Err`; both share the same process-level state (integration tests in Rust share per-binary state).
- **`RwLock` poisoning:** If a thread panics while holding the write lock, subsequent `write()` calls return `Err`. Mitigation: `register` is an infallible operation on a `HashMap`; panics inside the lock body are extremely unlikely. Document that callers should `expect("factory lock poisoned")` — consistent with the rest of the codebase (e.g., `Application::object_tree().lock().unwrap()`).
- **API surface of `ObjectFactory`:** Adding `install`/`global` is additive. Existing call sites (`factory.create(...)`, `factory.register(...)`) are unaffected because `ObjectFactory` struct itself is unchanged. No backward-compat concern (project not yet published, per AGENTS.md).

## Test Design

### Task 2 — `ObjectBase: Debug`
- Location: `quartzite-core/src/object_base.rs` doctest in `impl ObjectBase` or in the existing `#[cfg(test)]` module
- Entry point: `format!("{:?}", ObjectBase::new())`
- Scenarios:
  - Compiles and does not panic (AC1 literal check)
  - Named base includes the name string in debug output (sanity, not a strict format contract)

### Task 4 — `rename` no-op
- Location: `quartzite-runtime/src/object_tree.rs` `#[cfg(test)]` module (unit) and `quartzite-runtime/tests/object_tree.rs` (integration)
- Entry point: `ObjectTree::rename`
- Scenarios:
  - `rename(id, same_name)` → object still found by that name (AC3)
  - `rename(id, same_name)` → `find_by_name` returns exactly one entry (no duplicate insertion)
  - `rename(id, "")` on an object whose current name is `Some("")` — genuinely same name, must be a no-op (note: an anonymous object has `name = None`, not `Some("")`; renaming `None → ""` is a real rename and is *not* a no-op)
  - `rename(id, different_name)` still works as before (regression guard, AC5)
  - `rename(unknown_id, any_name)` is a no-op (existing behaviour unchanged)

### Task 7 — `ObjectFactory` singleton
- Location: `quartzite-runtime/tests/factory.rs` (integration, isolated binary)
- Entry point: `ObjectFactory::install`, `ObjectFactory::global`
- Scenarios:
  - First `install` returns `Ok(())`
  - Second `install` in the same process returns `Err(FactoryAlreadySet)`
  - `global()` before any `install` returns `None` — **cannot be tested in the same binary as the success test** because `OnceLock` is per-process; must be a separate integration test binary or accepted as untestable in isolation. Use `#[ignore]` on any test that requires a clean slate, with a comment explaining.
  - After `install`, `global()` returns `Some(_)`; `create`/`register` work through the lock.

## Open questions

_(none)_
