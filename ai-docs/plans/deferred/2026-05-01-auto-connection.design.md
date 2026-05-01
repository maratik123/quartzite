# Design: AutoConnection (`ConnectionType::Auto`)

**Issue:** —
**Date:** 2026-05-01

## Prerequisites

This design cannot be implemented until Runtime Task 0 lands in quartzite-core:
- `ConnectionType::Queued` variant (`#[cfg(feature = "std")]`)
- `QueuedDispatcher` trait (`fn post(&self, f: Box<dyn FnOnce() + Send + 'static>)`)
- `QUEUED_DISPATCHER: OnceLock<Arc<dyn QueuedDispatcher>>` global
- `set_queued_dispatcher()` / `queued_dispatcher()` accessors

These are defined in `ai-docs/plans/deferred/2026-05-01-runtime.design.md` Task 0.
Do not begin Task 2 of this design until Runtime Task 0 is merged.

**Note:** The `'static` bound on `post` is required because the dispatcher may store the closure
past the emitter's stack frame. The runtime design's Task 0 should be amended to match this
signature if it does not already use `'static`.

## Approach

### Chosen approach

Extend `Signal<Args>` in `quartzite-core` to support `ConnectionType::Auto`. At emit time the
signal reads `thread::current().id()` once and compares it against the `ThreadId` captured per
slot at connect time. If the ids match the slot is called directly (`Direct` path). If they differ
the args are cloned and a closure is posted to the process-global `QueuedDispatcher` (`Queued`
path).

The open question from the spec ("store both `Fn(&Args)` and `Fn(Args) + Send + Sync`, or a
single `Fn(Args) + Send + Sync` called with a clone on the same-thread path?") is resolved as
follows: **one callback** — `Box<dyn Fn(Args) + Send + Sync>` with `Args: Clone + Send + 'static`
required. This is the same constraint as `Queued` slot entries. Dispatch is:

- **Same-thread path** — `callback(args.clone())` called directly (no posting).
- **Cross-thread path** — args cloned, closure posted to `QueuedDispatcher`.

This is simpler than the dual-callback approach and consistent with how `Queued` slots already
work. The same-thread path does incur one clone of `Args`, but this is the accepted tradeoff:
it avoids an extra heap allocation per `Auto` slot and keeps the `SlotCallback` enum uniform.
This supersedes the spec's `&Args` same-thread description; the spec will be updated after this design is approved.

### `SlotEntry` representation

The current `SlotEntry<Args>` is a flat struct with a single `callback: Box<dyn Fn(&Args)>`. To
support the new variants the callback field is replaced by a `SlotCallback<Args>` enum:

```rust
#[cfg(feature = "std")]
enum SlotCallback<Args: 'static> {
    Direct(Box<dyn Fn(&Args)>),
    Queued(Box<dyn Fn(Args) + Send + Sync>),
    Auto {
        thread_id: std::thread::ThreadId,
        callback:  Box<dyn Fn(Args) + Send + Sync>,
    },
}

#[cfg(not(feature = "std"))]
enum SlotCallback<Args: 'static> {
    Direct(Box<dyn Fn(&Args)>),
    // Queued and Auto variants are not present without std
}
```

`ConnectionType` gains the new `Auto` variant, gated on `#[cfg(feature = "std")]`.

`SlotEntry<Args>` is updated to hold a `SlotCallback<Args>` in place of the current
`callback: Box<dyn Fn(&Args)>`. The `conn_type: ConnectionType` field is retained for
`SingleShot` cleanup semantics (see Risks).

### Dispatcher interaction

The `QueuedDispatcher` trait and the `QUEUED_DISPATCHER` global (both specified in the runtime
design) are prerequisites for `Queued` connections. `Auto`'s cross-thread path uses the same
accessor (`quartzite_core::queued_dispatcher()`). No new global state is required for `Auto`.

When no dispatcher is installed, `queued_dispatcher()` returns `None`. Both the `Queued` path and
the `Auto` cross-thread path silently drop the invocation in that case (AC3, consistent with the
`Queued` silent-drop policy documented in the runtime design).

### Relationship to the runtime design's Task 0

The runtime design's Task 0 requires amending `quartzite-core` to:
- Gate `ConnectionType::Queued` behind `#[cfg(feature = "std")]`
- Add the `QueuedDispatcher` trait and `QUEUED_DISPATCHER` global
- Add `set_queued_dispatcher` / `queued_dispatcher` accessors

`Auto` depends on all three of those changes. The `Auto` work is therefore a continuation of Task 0
from the runtime design, not an independent parallel track. The decomposition below models this
with an explicit dependency.

### `signals_blocked` interaction

The spec open question ("should `Auto` on a same-thread emit respect `signals_blocked`?") remains
open. `Signal::emit` in the current codebase does not check `signals_blocked`; `Signal` has no
access to `ObjectBase`. No assertion is made in this design about `Auto` receiving `signals_blocked`
behavior for free. This question is deferred to the `signals_blocked` implementation design.

### Cross-thread emit scenario (AC2)

Thread A owns a `Signal`. A slot is connected with `Auto`, capturing the receiver's
`thread_id = Thread B's id` at connect time (passed explicitly as `receiver_thread_id` by the
caller). Thread A calls `emit`. `emit` reads `thread::current().id()` (Thread A's id) and
compares it against the stored `thread_id` (Thread B's id) — they differ → `emit` clones `args`
and posts the closure to the dispatcher. `Signal` is not moved; it remains on Thread A throughout.
`Signal` remains `!Send`.

### Rejected alternatives

| Alternative | Reason rejected |
|---|---|
| Dual-callback `Auto` (`Fn(&Args)` direct + `Fn(Args) + Send + Sync` queued) | More complex: two heap allocations per slot, two closures to construct at connect time; same-thread path still clones nothing but callers must supply two closures — poor ergonomics |
| Separate `connect_auto()` method instead of `ConnectionType::Auto` variant | Inconsistent with spec decision; `ConnectionType` is the canonical discriminator |
| Runtime-only `Auto` (no change to `quartzite-core`) | `Auto`'s same-thread path is a `Direct`-style call; the dispatch logic belongs in core alongside `Direct` and `Queued` |

---

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `ConnectionType::Auto` variant (gated `#[cfg(feature = "std")]`) | `quartzite-core/src/signal.rs` | — |
| 2 | Replace `SlotEntry.callback` with `SlotCallback<Args>` enum; add `Auto { thread_id, callback }` arm with `Box<dyn Fn(Args) + Send + Sync>` | `quartzite-core/src/signal.rs` | 1, Runtime Task 0 — `Queued` variant gating and `QueuedDispatcher` global must already exist |
| 3 | Extend `Signal::emit` to handle `SlotCallback::Auto`: read `thread::current().id()` once; same-thread → `callback(args.clone())`; cross-thread → clone args, post closure to dispatcher | `quartzite-core/src/signal.rs` | 2; Runtime Task 0 must use `fn post(&self, f: Box<dyn FnOnce() + Send + 'static>)` — amend runtime design before Task 0 is merged |
| 4 | Add `connect_auto` convenience method on `Signal<Args>` accepting `receiver_thread_id: ThreadId` and `F: Fn(Args) + Send + Sync + 'static` with `Args: Clone + Send + 'static` | `quartzite-core/src/signal.rs` | 2 |
| 5 | Unit tests for `Auto` in `signal.rs` `#[cfg(test)]` module (AC1, AC2, AC4, AC5) | `quartzite-core/src/signal.rs` | 3, 4 |
| 6 | Integration test for no-dispatcher silent drop (AC3) | `quartzite-core/tests/auto_no_dispatcher.rs` | 3 |

Total: 6 tasks. Primary changes are confined to `signal.rs`. One new test file is required.

**Implementation note for Task 3 — `emit` snapshot strategy for `Auto` slots:**
The snapshot inside `emit` currently captures `(ConnectionId, ConnectionType)` pairs. After the
`SlotCallback` refactor, the snapshot must capture `(ConnectionId, Arc<SlotCallback<Args>>)` — or
equivalently the full `SlotEntry` — so the dispatch loop can match on
`SlotCallback::Auto { thread_id, callback }` directly. `thread::current().id()` is read once
before the loop (thread id is stable during a single `emit` call). `SingleShot` removal after
calling an `Auto` slot on the same-thread path uses the same `retain`-after-snapshot mechanism as
`Direct` single-shot cleanup.

**Note on `connect_auto`:** The spec mandates `ConnectionType::Auto` as the API surface. The
`connect_auto` method in Task 4 is a convenience wrapper that constructs the `Auto` variant; it
is not a separate connection mechanism. The method is justified because `connect_typed` accepts a
single `Box<dyn Fn(&Args)>` and cannot directly accept the `Auto`-shaped callback — a dedicated
method avoids an awkward builder API while keeping the variant as the canonical discriminator.

---

## Risks

- **`Args: Clone + Send + 'static` constraint on `Auto` connections:** Callers who connect `Auto`
  without `Args: Clone + Send` get a compile error. This is the same constraint as `Queued`.
  Mitigation: document the constraint in the `connect_auto` rustdoc.

- **Same-thread path clone cost:** `Auto` clones `args` on the same-thread path. For
  `Args = ()` or cheap types this is negligible. For types containing `String` or `Vec` it is a
  real allocation. Callers who need zero-copy same-thread dispatch should use `Direct`.
  Mitigation: document in `connect_auto` rustdoc; no optimization in v1.

- **`thread_id` staleness:** The captured `ThreadId` goes stale if the receiver object migrates
  to a different thread after `connect`. This is explicitly deferred in the spec (requires an
  object-mobility API). Mitigation: document the invariant in `connect_auto` rustdoc; no runtime
  check added in v1 (AC5 confirms current behavior is correct by spec).

- **`no_std` compile error path:** `ConnectionType::Auto` must be absent in `no_std` builds.
  Mitigation: `Auto` arm of the enum and `connect_auto` method are both `#[cfg(feature = "std")]`.
  Additionally, a `#[cfg(not(feature = "std"))] compile_error!(...)` guard inside the `Auto`
  variant definition ensures a clear, actionable compiler message. AC4 is verified by the `no_std`
  CI job (`cargo build --no-default-features`), not by a `compile_fail` doc-test (which would run
  with `std` enabled and fail CI).

- **`SlotCallback` enum refactor breaks existing `Direct`/`SingleShot` handling in `emit`:**
  The current `emit` matches on `conn_type: ConnectionType`. After the refactor, dispatch logic
  moves into `SlotCallback`. Care is needed to preserve `SingleShot` cleanup semantics.
  Mitigation: `SingleShot` remains a `ConnectionType` discriminator alongside the callback type;
  `SlotEntry` retains `conn_type` for the `SingleShot` removal logic, while `SlotCallback`
  handles dispatch. Alternatively, `SingleShot` can be a wrapper variant in `SlotCallback` itself
  — the implementer decides, but the tests enforce the behavior.

- **`QUEUED_DISPATCHER` `OnceLock` test isolation:** The global cannot be reset between tests in
  the same process. AC2 and AC3 both interact with it. Mitigation: a shared `TestDispatcher` stub
  is installed once per test binary via `TEST_DISPATCHER.get_or_init(...)` (see Test Design for
  the coordination strategy). AC3 runs in `tests/auto_no_dispatcher.rs` — a separate test binary
  that starts with no dispatcher installed, ensuring isolation.

---

## Test Design

### Task 5 — unit tests in `quartzite-core/src/signal.rs` `#[cfg(test)]` module

All tests in this section are behind `#[cfg(feature = "std")]`.

#### `OnceLock` test coordination strategy

`QUEUED_DISPATCHER` is a process-wide `OnceLock` — it can only be set once per process. Unit
tests sharing the same binary that need a dispatcher must coordinate as follows:

```rust
// In test module:
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

`TEST_DISPATCHER.get_or_init(|| { let d = Arc::new(TestDispatcher { posted: Mutex::new(vec![]) }); set_queued_dispatcher(Arc::clone(&d)); d })` is
called once (typically via a shared helper). The `set_queued_dispatcher` call is placed INSIDE the
`get_or_init` closure so it executes at most once, even under parallel test execution. Because
`OnceLock` guarantees only the first call to `get_or_init` initialises the value, subsequent calls
across tests in the same binary return the same instance without re-registering the dispatcher.

Each test that needs to assert on posted closures drains the queue before asserting:

```rust
let posted: Vec<_> = dispatcher.posted.lock().unwrap().drain(..).collect();
assert_eq!(posted.len(), 1);
posted.into_iter().for_each(|f| f()); // execute to verify callback runs
```

This avoids any cross-test state leakage: each test starts with an empty `posted` queue by
draining it before use.

#### AC1 — same-thread `Auto` calls slot synchronously

- **Entry point:** `Signal::emit` with an `Auto`-connected slot, called from the same thread that
  connected it.
- **Scenario:** `auto_same_thread_calls_slot_synchronously`
  - Install `TEST_DISPATCHER` via the shared helper.
  - Connect a slot via `connect_auto`, passing `thread::current().id()` as `receiver_thread_id`;
    emit on the same thread.
  - Assert the callback was called before `emit` returns (use `Arc<AtomicBool>` shared with
    callback).
  - Assert the dispatcher's `posted` queue is empty (no closure was posted).

#### AC2 — cross-thread `Auto` posts to dispatcher

- **Entry point:** `Signal::emit` called from Thread A; slot connected with Thread B's `thread_id`.
- **Scenario:** `auto_cross_thread_posts_to_dispatcher`
  - Install `TEST_DISPATCHER` via the shared helper.
  - Create a `Signal` on Thread A. Call `connect_auto` passing a `ThreadId` that is guaranteed to
    differ from Thread A's — obtain it by spawning a helper thread that sends back its
    `thread::current().id()` over a channel, then join the thread.
  - Thread A calls `emit`.
  - Assert: the callback was NOT called directly on Thread A.
  - Assert: `TEST_DISPATCHER.posted` contains exactly one closure.
  - Drain the posted closures and execute them; assert the callback ran.

#### AC4 — `Auto` unavailable in `no_std`

AC4 is verified by the `no_std` CI job: `cargo build --no-default-features` must not compile code
that uses `ConnectionType::Auto`.

Add a `#[cfg(not(feature = "std"))] compile_error!(...)` guard inside the `Auto` variant
definition so the compiler reports a clear error message when `std` is absent. This is simpler
than a separate test file.

(The `compile_fail` doc-test approach is NOT used here: a `compile_fail` doc-test in `signal.rs`
runs with `std` enabled by default, so `ConnectionType::Auto` compiles fine and the
`compile_fail` expectation itself fails CI.)

#### AC5 — `thread_id` captured at connect time; changes to `ObjectBase::thread_id` after connect do not affect dispatch

- **Scenario A — same-thread path:** `auto_thread_id_same_thread_calls_directly`
  - Connect `Auto` via `connect_auto`, passing `thread::current().id()` as `receiver_thread_id`
    (same thread as the emitter).
  - Call `emit`.
  - Assert the slot IS called directly (use `Arc<AtomicBool>` set inside the callback and checked
    before `emit` returns).
  - Assert `TEST_DISPATCHER.posted` is empty (no closure was posted).

- **Scenario B — cross-thread path:** `auto_thread_id_foreign_thread_posts_to_dispatcher`
  - Obtain a foreign `ThreadId` via `other_thread_id()` (spawns a helper thread, captures its
    `thread::current().id()`, joins — guaranteed to differ from the current thread's id).
  - Connect `Auto` via `connect_auto`, passing the foreign `ThreadId` as `receiver_thread_id`.
  - Call `emit` from the current thread.
  - Assert the slot is NOT called directly (the `AtomicBool` is still `false` immediately after
    `emit` returns).
  - Assert `TEST_DISPATCHER.posted` contains exactly one closure; drain and execute it; assert
    the callback then runs (the `AtomicBool` becomes `true`).

These two scenarios together confirm that dispatch is governed by the `receiver_thread_id` value
supplied at connect time, without inspecting `SlotCallback::Auto.thread_id` or any other private
internal state.

### Task 6 — `tests/auto_no_dispatcher.rs` (AC3)

Separate integration test binary. No dispatcher is installed; no `Application` is created. Because
this binary never calls `set_queued_dispatcher`, the `QUEUED_DISPATCHER` `OnceLock` remains empty
for the lifetime of the process — providing guaranteed isolation from tests in other binaries.

#### AC3 — cross-thread `Auto` with no dispatcher installed: silent drop

- **Scenario:** `auto_cross_thread_no_dispatcher_silent_drop`
  - Do NOT call `set_queued_dispatcher` anywhere in this binary.
  - Create a `Signal<(i32,)>` on Thread A. Connect via `connect_auto` with a callback that panics
    if called. Pass a `receiver_thread_id` that differs from Thread A's — obtained by spawning a
    helper thread, receiving its `thread::current().id()`, then joining (same technique as AC2).
  - Call `emit` on Thread A.
  - Assert: no panic, no error. The `QueuedDispatcher` was never consulted (it returns `None`);
    the invocation is silently dropped.

### Fixtures / helpers needed

- `TestDispatcher` — hand-written stub in the test module; implements `QueuedDispatcher`.
  Stores posted closures in `Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>` for assertion
  and execution in AC1 and AC2. No `mockall` dependency required.
- `install_test_dispatcher() -> Arc<TestDispatcher>` — helper that calls
  `TEST_DISPATCHER.get_or_init(|| { let d = Arc::new(...); set_queued_dispatcher(Arc::clone(&d)); d })`.
  The `set_queued_dispatcher` call is inside the closure so it executes at most once even under
  parallel test runs. Returns the `Arc<TestDispatcher>` so callers can drain `posted`. Tests call
  this helper rather than setting up the dispatcher inline.
- `other_thread_id() -> ThreadId` — helper that spawns a thread, captures its
  `thread::current().id()`, and returns it after joining. Used in AC2 and AC3 to obtain a
  `ThreadId` that is guaranteed to differ from the current thread.

### Deferred tests

- `auto_cross_thread_slot_not_posted_after_receiver_destroyed` — requires `Weak<ReceiverGuard>`
  in `SlotEntry`, which is not present in the current `signal.rs` layer. This test cannot be
  implemented at the `signal.rs` layer without `ConnectionTable`. Deferred until `ConnectionTable`
  guard-check integration is designed (see runtime design Task 5 and the ReceiverGuard invariant
  in the runtime design).

---

## Open questions

- **`signals_blocked` interaction with `Auto`:** Should `Auto` on a same-thread emit respect
  `signals_blocked` the same way as `Direct`? Almost certainly yes, but `Signal::emit` does not
  currently check `signals_blocked` and has no access to `ObjectBase`. This remains open; to be
  resolved in the `signals_blocked` implementation design.

- **`SlotCallback` vs. keeping `conn_type` for `SingleShot`:** Should `SingleShot` become a
  variant of `SlotCallback` (wrapping an inner `Direct` callback), or should `SlotEntry` keep its
  `conn_type: ConnectionType` field alongside the new `SlotCallback` enum? Both work; the
  implementer should choose whichever keeps `emit` simplest. Not a design-level decision.
