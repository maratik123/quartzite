# Design: parent / children accessors on ObjectExt

**Issue:** #55
**Date:** 2026-05-05

## Approach

### Chosen solution

The implementation has three layers:

1. **Process-global tree accessor in `quartzite-runtime`** — a new free function
   `try_with_tree<R, F: FnOnce(&ObjectTree) -> R>(f: F) -> Option<R>` in a new
   module `quartzite-runtime/src/global_tree.rs`. The global is a
   `static GLOBAL_TREE: OnceLock<Weak<ApplicationInner>>` — no, that couples
   `global_tree` to `ApplicationInner` internals.

   Better: store a raw `*const Mutex<ObjectTree>` in a process-global
   `static TREE_PTR: AtomicPtr<Mutex<ObjectTree>>`. `Application::new()` stores the
   address of `inner.object_tree` (which lives for the lifetime of `ApplicationInner`,
   pinned behind `Arc`); `Application::drop()` CAS-clears the pointer.

   Wait — `Application` currently has no `Drop` impl. The spec requires clearing on
   drop. Additionally the existing `APP: OnceLock<Arc<ApplicationInner>>` never clears,
   so `Application::drop()` would run but the `OnceLock` stays populated forever.
   Clearing the tree pointer from `Drop` is safe regardless: once the `Application`
   handle is dropped, using the tree accessor from `ObjectExt` methods would yield
   `None`, matching the spec requirement.

   `AtomicPtr` is simpler than a second `OnceLock` and does not require `unsafe` at
   the call site of `try_with_tree`. The raw pointer is obtained from the
   `Arc<ApplicationInner>` on construction and guaranteed valid for the duration
   the `Arc` is alive (i.e., while the `Application` value exists). On drop the
   pointer is zeroed with `store(ptr::null_mut(), Ordering::Release)`.

   `try_with_tree` loads the pointer with `Ordering::Acquire`, checks for null,
   and if non-null locks the mutex. Because the mutex lives inside the `Arc` which
   is kept alive by `APP: OnceLock<Arc<ApplicationInner>>` until the process ends,
   dereferencing is safe as long as the pointer is non-null after the load.

   **Threading note for v1:** The project is single-threaded in its first iteration.
   The `AtomicPtr` approach is documented as safe under this constraint and is already
   the correct multi-threaded-safe form for the future.

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
| 1 | Add `global_tree` module with `AtomicPtr`-based global and `try_with_tree` | `quartzite-runtime/src/global_tree.rs`, `quartzite-runtime/src/lib.rs` | — |
| 2 | Wire `Application::new()` to register and `impl Drop for Application` to deregister | `quartzite-runtime/src/application.rs` | 1 |
| 3 | Add `ObjectTreeExt` trait with `parent_in`, `children_in`, `parent`, `children` | `quartzite-runtime/src/object_tree_ext.rs`, `quartzite-runtime/src/lib.rs` | 1 |
| 4 | Unit tests for `global_tree` (register / deregister / try_with_tree) | `quartzite-runtime/src/global_tree.rs` `#[cfg(test)]` | 1, 2 |
| 5 | Integration tests for `ObjectTreeExt` (AC1–AC9) | `quartzite-runtime/tests/object_tree_ext.rs` | 2, 3 |

Five tasks — within the 7-task limit; no split needed.

## Risks

- **`AtomicPtr` aliasing:** the raw pointer to `Mutex<ObjectTree>` is kept live by `APP:
  OnceLock<Arc<ApplicationInner>>`, which never drops during the process. The `Application`
  handle's `Drop` impl zeros the pointer before any other cleanup. Locking after zeroing
  yields `None` — safe. The only hazard is a use-after-pointer-clear if a thread races
  between load and lock. Mitigation: `try_with_tree` loads with `Acquire`, checks for null,
  then locks. If the pointer races to null between load and lock the pointer value already
  held is still valid (the `Arc` is still alive), so the lock succeeds and the caller gets
  a `None`-returning closure from `Drop`-time perspective only after the lock is released.
  Actually, the `Drop` impl only needs to zero the `AtomicPtr`; objects inside the tree are
  still reachable via `APP` until process exit. The risk is low.
- **`OnceLock` for `APP` never clears:** `Application::drop()` cannot remove the entry from
  `APP: OnceLock`. This is by design — `Application::global()` returning `Some` after the
  handle is dropped is a pre-existing behavior. The tree accessor pointer is separate and
  *does* get cleared, so `try_with_tree` correctly returns `None` after drop.
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
- **`unsafe` block in `try_with_tree`:** dereferencing the raw `*const Mutex<ObjectTree>`
  loaded from `TREE_PTR: AtomicPtr` requires an `unsafe` block. The implementation body
  must carry a `// SAFETY:` comment: "The pointer, when non-null, was stored by
  `Application::new()` from `&inner.object_tree` where `inner: Arc<ApplicationInner>`
  held alive by `APP: OnceLock`. The pointer is valid for the process lifetime. v1 is
  single-threaded; the `AtomicPtr` clear in `Drop` prevents TOCTOU in the single-thread
  case." Omitting the safety comment will fail `clippy::undocumented_unsafe_blocks`.

## Test Design

### Task 4 — `global_tree` unit tests (in `#[cfg(test)]` module of `global_tree.rs`)

Not feasible to test registration/deregistration in isolation within a unit test because
`Application::new()` sets `APP: OnceLock` which cannot be reset. These cases are covered
by the integration tests instead (task 5). The unit module can test:

- `try_with_tree` returns `None` when not registered (pointer is null at module init).

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
