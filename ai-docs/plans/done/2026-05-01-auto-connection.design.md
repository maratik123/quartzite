# Design: AutoConnection (`ConnectionType::Auto`)

**Issue:** —
**Date:** 2026-05-01
**Revised:** 2026-05-02 — updated to reflect actual implementation (separate `auto_slots` vector, `DynAutoSlot` trait)

## Prerequisites

This design cannot be implemented until Runtime Task 0 lands in quartzite-core:
- `ConnectionType::Queued` variant (`#[cfg(feature = "std")]`)
- `QueuedDispatcher` trait (`fn post(&self, f: Box<dyn FnOnce() + Send + 'static>)`)
- `QUEUED_DISPATCHER: OnceLock<Arc<dyn QueuedDispatcher>>` global
- `set_queued_dispatcher()` / `queued_dispatcher()` accessors

These are defined in `ai-docs/plans/done/2026-05-01-runtime.design.md` Task 0 and are
already present in `quartzite-core/src/signal.rs`.

## Approach

### Chosen approach

Extend `Signal<Args>` in `quartzite-core` to support `ConnectionType::Auto`. At emit time the
signal reads `thread::current().id()` once and compares it against the `ThreadId` captured per
slot at connect time. If the ids match the slot is called directly (`Direct` path). If they differ
the args are cloned and a closure is posted to the process-global `QueuedDispatcher` (`Queued`
path).

**Callback representation:** one `Box<dyn Fn(Args) + Send + Sync>` per Auto slot with
`Args: Clone + Send + 'static` required. Dispatch is:

- **Same-thread path** — `callback(args.clone())` called directly (no posting).
- **Cross-thread path** — args cloned, closure posted to `QueuedDispatcher`.

The same-thread path incurs one clone of `Args`, but avoids a heap allocation and keeps slot
storage uniform. This supersedes the spec's `&Args` same-thread description.

### Storage strategy — separate `auto_slots` vector

`Signal<Args>` requires only `Args: 'static` on its generic parameter. `Auto` connections require
the stricter `Args: Clone + Send + 'static` on the concrete callback. This mismatch makes it
impossible to store Auto callbacks inside the existing `slots: Vec<SlotEntry<Args>>` without
widening `Signal`'s bounds (which would break existing Direct/SingleShot callers).

The same constraint applies to `Queued` connections, which are already stored in a separate
`queued_slots: Vec<Box<dyn DynQueuedSlot<Args>>>` using a trait-object approach. `Auto` follows
the identical pattern:

```
Signal<Args: 'static>
  slots:        Vec<SlotEntry<Args>>                  ← Direct + SingleShot
  queued_slots: Vec<Box<dyn DynQueuedSlot<Args>>>     ← Queued
  auto_slots:   Vec<Box<dyn DynAutoSlot<Args>>>       ← Auto  (new)
```

`DynAutoSlot<Args>` is an object-safe trait (`Send + Sync`) with two methods:
- `fn id(&self) -> ConnectionId`
- `fn dispatch(&self, emit_thread_id: ThreadId, args: &Args)`

`AutoSlotInner<Args: Clone + Send + 'static>` implements `DynAutoSlot<Args>` and holds:
- `id: ConnectionId`
- `receiver_thread_id: ThreadId`
- `callback: Arc<dyn Fn(Args) + Send + Sync>`

`dispatch` reads `emit_thread_id` (supplied by `emit`) and either calls `callback(args.clone())`
directly or posts a closure to the dispatcher. If no dispatcher is installed the cross-thread path
silently drops the invocation (AC3).

The early design drafted a `SlotCallback<Args>` enum to unify all three callback kinds in
`SlotEntry`. That approach was rejected during implementation because it would require widening
`Signal<Args>`'s bounds or introducing unsafe code; the separate-vector / trait-object pattern is
the established solution already used by `Queued`.

### Dispatcher interaction

`Auto`'s cross-thread path uses the same `queued_dispatcher()` accessor as `Queued`. No new
global state is required.

When no dispatcher is installed, `queued_dispatcher()` returns `None`. Both the `Queued` path and
the `Auto` cross-thread path silently drop the invocation in that case (AC3, consistent with
existing `Queued` silent-drop policy).

### `signals_blocked` interaction

`Signal::emit` does not currently check `signals_blocked` and has no access to `ObjectBase`.
`Auto` inherits the same absence. This is deferred to the `signals_blocked` implementation design.

### Cross-thread emit scenario (AC2)

Thread A owns a `Signal`. A slot is connected with `Auto`, capturing
`receiver_thread_id = Thread B's id` at connect time. Thread A calls `emit`. `emit` reads
`thread::current().id()` once (Thread A's id) and passes it to each `DynAutoSlot::dispatch`. For
the `Auto` slot, Thread A ≠ Thread B → `emit` clones `args` and posts the closure to the
dispatcher. `Signal` is not moved; it remains on Thread A throughout. `Signal` remains `!Send`.

---

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `ConnectionType::Auto` variant (gated `#[cfg(feature = "std")]`) | `quartzite-core/src/signal.rs` | — |
| 2 | Add `DynAutoSlot<Args>` trait and `AutoSlotInner<Args>` struct; add `auto_slots: Vec<Box<dyn DynAutoSlot<Args>>>` to `Signal`; update `new()`, `disconnect()` | `quartzite-core/src/signal.rs` | 1, `QueuedDispatcher` global already present |
| 3 | Extend `Signal::emit`: read `thread::current().id()` once; iterate `auto_slots` calling `slot.dispatch(emit_thread_id, args)` | `quartzite-core/src/signal.rs` | 2 |
| 4 | Add `connect_auto` convenience method on `Signal<Args>` accepting `receiver_thread_id: ThreadId` and `F: Fn(Args) + Send + Sync + 'static` with `Args: Clone + Send + 'static` | `quartzite-core/src/signal.rs` | 2 |
| 5 | Unit tests for `Auto` in `signal.rs` `#[cfg(test)]` module (AC1, AC2, AC4, AC5) | `quartzite-core/src/signal.rs` | 3, 4 |
| 6 | Integration test for no-dispatcher silent drop (AC3) | `quartzite-core/tests/auto_no_dispatcher.rs` | 3 |

Total: 6 tasks. Primary changes confined to `signal.rs`. One new test file required.

**Implementation note for Task 3 — `emit` dispatch loop:**
`emit` reads `thread::current().id()` exactly once before the auto_slots loop. The id is stable
for the duration of a single `emit` call. Each `DynAutoSlot::dispatch` receives this id and
decides same-thread vs cross-thread locally. No snapshot of the auto_slots list is needed: `emit`
takes `&mut self`, so no concurrent modification can occur during the loop.

**Note on `connect_auto` and `connect_typed`:** The spec mandates `ConnectionType::Auto` as the
API surface. `connect_auto` is the sole correct entry point for `Auto` connections — it constructs
an `AutoSlotInner` (with the required `receiver_thread_id`) and pushes it into `auto_slots`.

`connect_typed` only supports `Direct` and `SingleShot`. Passing `ConnectionType::Auto` or
`ConnectionType::Queued` to `connect_typed` is a misuse: the slot would be stored in `slots` and
dispatched unconditionally without thread comparison or guard check, silently violating the
connection semantics. `connect_typed` guards against this with a `debug_assert!` (in std builds)
and documents the restriction in its rustdoc. The same restriction applies to `Queued` (which has
always required `connect_queued`).

---

## Risks

- **`Args: Clone + Send + 'static` constraint on `Auto` connections:** Callers who connect `Auto`
  without `Args: Clone + Send` get a compile error. Same constraint as `Queued`.
  Mitigation: document in `connect_auto` rustdoc.

- **Same-thread path clone cost:** `Auto` clones `args` on the same-thread path. For cheap types
  (`()`, `i32`) this is negligible. For types containing `String` or `Vec` it is a real allocation.
  Callers who need zero-copy same-thread dispatch should use `Direct`.
  Mitigation: document in `connect_auto` rustdoc; no optimization in v1.

- **`thread_id` staleness:** The captured `ThreadId` goes stale if the receiver migrates to a
  different thread after `connect`. Explicitly deferred in the spec (requires object-mobility API).
  Mitigation: document in `connect_auto` rustdoc; no runtime check in v1 (AC5 confirms this).

- **`no_std` compile error path:** `ConnectionType::Auto`, `DynAutoSlot`, `AutoSlotInner`, and
  `connect_auto` are all `#[cfg(feature = "std")]`. No_std builds get "cannot find variant `Auto`"
  from the compiler, which is sufficiently actionable. AC4 is verified structurally by the cfg gate.

- **`QUEUED_DISPATCHER` `OnceLock` test isolation:** The global cannot be reset between tests in
  the same process. AC2 and AC3 both interact with it. Mitigation: a shared `TestDispatcher` stub
  is installed once per test binary via `TEST_DISPATCHER.get_or_init(...)`. AC3 runs in
  `tests/auto_no_dispatcher.rs` — a separate binary that starts with no dispatcher installed.

---

## Test Design

### Task 5 — unit tests in `quartzite-core/src/signal.rs` `#[cfg(test)]` module

All tests in this section are behind `#[cfg(feature = "std")]`.

#### `OnceLock` test coordination strategy

`QUEUED_DISPATCHER` is a process-wide `OnceLock` — it can only be set once per process. Unit
tests sharing the same binary that need a dispatcher must coordinate as follows:

```rust
static TEST_DISPATCHER: OnceLock<Arc<TestDispatcher>> = OnceLock::new();

struct TestDispatcher {
    posted: Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>,
}

impl QueuedDispatcher for TestDispatcher {
    fn post(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        self.posted.lock().unwrap().push(f);
    }
}
```

`install_test_dispatcher()` calls `TEST_DISPATCHER.get_or_init(|| { ... set_queued_dispatcher(...) ... })`.
The `set_queued_dispatcher` call is inside the closure so it executes at most once even under
parallel test execution. Subsequent calls return the same instance without re-registering.

Each test that needs to assert on posted closures drains the queue before asserting:

```rust
dispatcher.posted.lock().unwrap().drain(..).for_each(drop); // clear before test
// ... emit ...
let posted: Vec<_> = dispatcher.posted.lock().unwrap().drain(..).collect();
assert_eq!(posted.len(), 1);
posted.into_iter().for_each(|f| f()); // execute to verify callback runs
```

#### AC1 — same-thread `Auto` calls slot synchronously (`auto_same_thread_calls_slot_synchronously`)

- Connect via `connect_auto` passing `thread::current().id()` as `receiver_thread_id`; emit on same thread.
- Assert `called` `AtomicBool` is `true` before emit returns.
- Assert dispatcher's `posted` queue is empty.

#### AC2 — cross-thread `Auto` posts to dispatcher (`auto_cross_thread_posts_to_dispatcher`)

- Obtain foreign `ThreadId` via `other_thread_id()` helper (spawns thread, captures id, joins).
- Connect via `connect_auto` with the foreign id; emit from current thread.
- Assert `called` is `false` immediately after emit.
- Drain `posted`; assert exactly one closure; execute it; assert `called` is now `true`.

#### AC4 — `Auto` unavailable in `no_std`

Verified structurally: `ConnectionType::Auto`, `connect_auto`, `DynAutoSlot`, and `AutoSlotInner`
are all `#[cfg(feature = "std")]`. The `no_std` CI job (`cargo build --no-default-features`)
confirms the variant does not exist without `std`.

#### AC5 — `thread_id` captured at connect time

- **Scenario A** (`auto_thread_id_same_thread_calls_directly`): pass `thread::current().id()` →
  assert direct call, no posting.
- **Scenario B** (`auto_thread_id_foreign_thread_posts_to_dispatcher`): pass foreign id →
  assert no direct call, exactly one closure posted, runs correctly.

### Task 6 — `tests/auto_no_dispatcher.rs` (AC3)

Separate integration test binary. No dispatcher is installed; `QUEUED_DISPATCHER` `OnceLock`
remains empty for the lifetime of the process.

#### AC3 — cross-thread `Auto` with no dispatcher installed: silent drop (`auto_cross_thread_no_dispatcher_silent_drop`)

- Do NOT call `set_queued_dispatcher` anywhere in this binary.
- Connect via `connect_auto` with a foreign `ThreadId` (via `other_thread_id()`) and a callback
  that panics if called.
- Call `emit`; assert no panic and no error.

### Fixtures / helpers

- `TestDispatcher` — stores posted closures in `Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>`.
- `install_test_dispatcher() -> Arc<TestDispatcher>` — installs via `OnceLock`, safe to call N times.
- `other_thread_id() -> ThreadId` — spawns helper thread, captures its id, joins.

### Deferred tests

- `auto_cross_thread_slot_not_posted_after_receiver_destroyed` — requires `Weak<ReceiverGuard>`
  in the auto slot entry. `AutoSlotInner` does not hold a guard in v1 (no guard parameter in
  `connect_auto`). Deferred until `ConnectionTable` guard-check integration is designed.

---

## Open questions

- **`signals_blocked` interaction with `Auto`:** Deferred to the `signals_blocked` design.
- **`ReceiverGuard` for `Auto`:** `connect_auto` currently accepts no guard. Cross-thread Auto
  slots will post even after the receiver is destroyed. Deferred; requires `ConnectionTable`
  integration (same as Queued slots had before `connect_queued` added the guard parameter).
