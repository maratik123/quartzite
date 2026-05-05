# Design: parent / children accessors on ObjectExt

**Issue:** #55
**Date:** 2026-05-05

## Approach

### Chosen solution

The implementation has three layers:

1. **Process-global tree accessor in `quartzite-runtime`** — a `static TREE_LIVE: AtomicBool`
   flag (in `quartzite-runtime/src/global_tree.rs`) and a public free function
   `try_with_tree<R>(f: impl FnOnce(&ObjectTree) -> R) -> Option<R>` (in
   `quartzite-runtime/src/application.rs`, which has access to both `TREE_LIVE` and
   the private `APP: OnceLock<Arc<ApplicationInner>>`).

   `Application::new()` calls `global_tree::register()` (sets `TREE_LIVE = true`);
   `impl Drop for Application` calls `global_tree::deregister()` (sets `TREE_LIVE = false`).

   `try_with_tree` implementation:
   ```rust
   pub fn try_with_tree<R>(f: impl FnOnce(&ObjectTree) -> R) -> Option<R> {
       if !crate::global_tree::is_live() { return None; }
       let guard = APP.get()?.object_tree.lock().ok()?;
       Some(f(&guard))
   }
   ```

   **Why `AtomicBool` instead of `AtomicPtr`:**
   An earlier draft stored a raw `*const Mutex<ObjectTree>` in `AtomicPtr`, requiring
   an `unsafe` dereference and a `// SAFETY:` block. This was rejected because
   `APP: OnceLock<Arc<ApplicationInner>>` already keeps the `Mutex<ObjectTree>` alive
   for the process lifetime — the pointer can be reached via the safe `APP.get()` path.
   `AtomicBool` expresses the "live/dead" state with zero `unsafe`.

   **`.lock().ok()?` instead of `.lock().unwrap()`:**
   Mutex poisoning (another thread panicking while holding the lock) is not a broken
   global invariant from the caller's perspective. Returning `None` is correct library
   behaviour; `.ok()?` achieves this with no panic.

   **Threading note for v1:** The project is single-threaded. The `AtomicBool` TOCTOU
   (check-then-use) is benign under this constraint. Multi-threaded use would require
   a more sophisticated locking strategy regardless.

2. **`ObjectExt` methods in `quartzite-core/src/traits.rs`** — four new default
   methods: `parent()`, `parent_in(tree)`, `children()`, `children_in(tree)`.
   The `_in` variants accept `&ObjectTree` explicitly; the unsuffixed ergonomic
   variants call `try_with_tree`. Because `quartzite-core` must not depend on
   `quartzite-runtime` (no circular dependency), the ergonomic variants cannot
   import `try_with_tree` directly.

   **Dependency break strategy:** introduce a thin indirection — a second global
   function pointer in `quartzite-core` (similar to how `set_queued_dispatcher`
   works in `quartzite-core/src/signal.rs`). Name it `set_tree_accessor` / `tree_accessor`
   and store it as a `static TREE_ACCESSOR: OnceLock<Box<dyn Fn(ObjectId) -> ...>>`.

   However, the `_in` variants need `&ObjectTree`, which is a `quartzite-runtime`
   type, so they cannot be defined in `quartzite-core` at all. They live in
   `quartzite-runtime` as methods on a separate extension trait, or as inherent
   methods on `ObjectRef` / helper types.

   **Revised approach (cleanest, no new indirection layer):**

   - `parent_in` / `children_in` are **not** part of `ObjectExt` in `quartzite-core`.
     They cannot be, because `ObjectTree` is defined in `quartzite-runtime` and
     `quartzite-core` must stay runtime-free.
   - Define a second extension trait `ObjectTreeExt` in
     `quartzite-runtime/src/object_tree.rs` (or a new file) with `parent_in` and
     `children_in` as default methods. Blanket impl: `impl<T: AsObject> ObjectTreeExt for T {}`.
   - The ergonomic `parent()` / `children()` methods also live in `ObjectTreeExt`
     (not in `quartzite-core`). They call `try_with_tree` directly.
   - `quartzite-runtime` re-exports `ObjectTreeExt` from its `lib.rs`.

   This keeps `quartzite-core` clean, avoids any global function-pointer gymnastics,
   and puts all tree-aware helpers in the crate that owns the tree.

   The spec says "New methods go here as default methods" pointing at `ObjectExt`, but
   notes this uses `self.object_id()` which is `self.id()` in the current code. That
   works only if `ObjectExt` can reach `try_with_tree` — which it cannot without a
   dependency on `quartzite-runtime`. The spec also acknowledges the global accessor
   "needs to be set by `Application` on construction". The cleanest resolution
   consistent with all constraints is the `ObjectTreeExt` trait in `quartzite-runtime`.

3. **`Application` wiring** — `Application::new()` calls
   `global_tree::register(Arc::clone(&inner))` (or equivalent); `impl Drop for Application`
   calls `global_tree::deregister()`.

### Rejected alternatives

- **`AtomicPtr<Mutex<ObjectTree>>`** (earlier draft): required an `unsafe` dereference of
  a raw pointer in `try_with_tree` and `.lock().unwrap()` which panics on mutex poisoning.
  Rejected because `APP: OnceLock<Arc<ApplicationInner>>` already holds the tree safely —
  no raw pointer needed. `AtomicBool` expresses the same "live/dead" semantic with zero unsafe.
- **Store closure in `quartzite-core` `OnceLock`** (like `set_queued_dispatcher`): would
  require erasing `&ObjectTree` behind a `dyn Fn` or similar, making the API awkward and
  adding an allocation. Rejected in favor of `ObjectTreeExt` in `quartzite-runtime`.
- **`Rc<RefCell<ObjectTree>>` thread-local**: deferred per spec (blocked on #51). Out of
  scope for this task.
- **`parent_in` / `children_in` as `ObjectTree` methods taking `&impl AsObject`**:
  would be fine for the explicit-param variants but forces callers to have both the
  object and the tree in scope. `ObjectTreeExt` is more ergonomic and mirrors the
  `ObjectExt` pattern.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `global_tree` module with `AtomicBool` flag + `register`/`deregister`; add `try_with_tree` to `application.rs` | `quartzite-runtime/src/global_tree.rs`, `quartzite-runtime/src/application.rs`, `quartzite-runtime/src/lib.rs` | — |
| 2 | Wire `Application::new()` to call `register` and `impl Drop for Application` to call `deregister` | `quartzite-runtime/src/application.rs` | 1 |
| 3 | Add `ObjectTreeExt` trait with `parent_in`, `children_in`, `parent`, `children` | `quartzite-runtime/src/object_tree_ext.rs`, `quartzite-runtime/src/lib.rs` | 1 |
| 4 | Unit tests for `global_tree` (register / deregister / try_with_tree) | `quartzite-runtime/src/global_tree.rs` `#[cfg(test)]` | 1, 2 |
| 5 | Integration tests for `ObjectTreeExt` (AC1–AC9) | `quartzite-runtime/tests/object_tree_ext.rs` | 2, 3 |

Five tasks — within the 7-task limit; no split needed.

## Risks

- **`AtomicBool` TOCTOU (check-then-use):** between `is_live()` returning `true` and
  `APP.get()` being called, another thread could drop `Application` (setting `TREE_LIVE =
  false`). In that window, `APP.get()` still returns `Some` (the `OnceLock` never clears),
  so `try_with_tree` would proceed and call `f`. The caller would receive results from a
  "just-dropped" tree. Under v1 (single-threaded) this race is impossible. Multi-threaded
  correctness would require a different locking strategy — accepted as future work.
- **`OnceLock` for `APP` never clears:** `Application::drop()` cannot remove the entry from
  `APP: OnceLock`. This is by design — `Application::global()` returning `Some` after the
  handle is dropped is a pre-existing behavior. `TREE_LIVE` is separate and *does* get
  cleared, so `try_with_tree` correctly returns `None` after drop (in the single-threaded case).
- **`ObjectTreeExt` name collision with `ObjectExt`:** both traits auto-impl for all
  `AsObject` types. Names `parent`, `children`, `parent_in`, `children_in` must not appear
  in `ObjectExt`. Currently they do not. Mitigation: verify at compile time (clippy).
- **Doc-gate CI (`missing-docs`):** `ObjectTreeExt` and all its methods must be fully
  documented. `#[inline]` on every simple method. Risk of clippy warnings if omitted.
  Mitigation: addressed in task 3.
- **Integration test isolation:** `Application` singleton tests must run in separate
  process binaries (see existing `tests/application.rs` pattern). The new integration
  test file `tests/object_tree_ext.rs` will create its own `Application` instance and
  must not share state with other test binaries. Each `tests/*.rs` file is a separate
  binary — this is already the convention.
  **AC3 / AC6 / AC9 require dropping `Application`** to reach the "no tree" state, which
  means any other `#[test]` fn that needs a live `Application` in the same binary would
  race against or fail after that drop. Mitigation: use a **single `#[test]` function**
  that runs all scenarios in sequence — "before `new()`" → create `Application` → live
  scenarios → drop handle → after-drop assertions. This mirrors the pattern already used
  in `tests/application.rs`. Risk: eliminated.
- **No `unsafe` code:** the `AtomicBool` + `APP.get()` approach is entirely safe Rust.
  The earlier `AtomicPtr` draft required an `unsafe` dereference; this approach eliminates
  that entirely. `#![warn(clippy::undocumented_unsafe_blocks)]` remains in `lib.rs` as a
  future safeguard.

## Test Design

### Task 4 — `global_tree` unit tests (in `#[cfg(test)]` module of `global_tree.rs`)

Not feasible to test registration/deregistration in isolation within a unit test because
`Application::new()` sets `APP: OnceLock` which cannot be reset. These cases are covered
by the integration tests instead (task 5). The unit module can test:

- `try_with_tree` returns `None` when not registered (`TREE_LIVE` is `false` at module init).

### Task 5 — `tests/object_tree_ext.rs` integration tests

**Single `#[test]` function** covering all ACs in sequence, matching the pattern in
`tests/application.rs`. This eliminates parallel-execution interference between
after-drop assertions (AC3, AC6, AC9) and live-Application scenarios (AC1, AC2, AC4–AC8).

**Execution sequence:**

```
[Phase 0 — before Application::new()]
  → try_with_tree returns None (AC9 pre)

[Phase 1 — create Application]
  let app = Application::new().unwrap();
  → try_with_tree returns Some (AC9 live)
  → insert objects into tree

  → parent of root is None (AC1)
  → parent of child is Some(parent_id) (AC2)
  → children in insertion order (AC4)
  → leaf has empty children (AC5)
  → parent_in matches parent (AC7)
  → children_in matches children (AC8)

[Phase 2 — drop Application handle]
  drop(app);
  → obj.parent() returns None (AC3)
  → obj.children() returns [] (AC6)
  → try_with_tree returns None (AC9 post)
```

**Fixtures / helpers:**

- Stub type implementing `Object` / `AsObject` with a name — duplicate from
  `tests/object_tree.rs` inline (small enough; avoids a `common/` module for now).
- ObjectIds obtained from `app.tree_mut()` insert calls; saved as locals for Phase 2
  use (objects still in tree via `APP: OnceLock` even after drop).

## Open questions

_None._
