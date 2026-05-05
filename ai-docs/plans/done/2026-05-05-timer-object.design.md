# Design: promote Timer to a first-class quartzite object

**Issue:** #36
**Date:** 2026-05-05

## Approach

The feature touches two crates — `quartzite-core` (id ordering + value type extension) and
`quartzite-runtime` (Timer refactor + three driver backends). The changes are additive everywhere
except the Timer struct itself, which is replaced in-place (no downstream crates exist yet; see
API Stability rule).

### ObjectId / ConnectionId ordering

Add `#[derive(PartialOrd, Ord)]` to both `ObjectId(u64)` and `ConnectionId(u64)` in
`quartzite-core/src/id.rs`. Both wrap `u64` which already implements `Ord`; the derived ordering
follows allocation order (monotonically increasing counter). No semantic ambiguity: a
later-allocated id is always greater. This is also required by `PoolDriver` (see below).

### Value::Duration

`core::time::Duration` is available in `no_std`, so adding `Value::Duration(core::time::Duration)`
requires only adding one arm to every existing `match self` in `value.rs` plus
`FromValue`/`IntoValue` impls. No feature flag needed. `usize` needs a `FromValue`/`IntoValue`
impl for the fire-count payload; the existing `impl_int_checked!` macro covers it via
`usize as i64` (consistent with the existing `u32`/`u64` pattern; `usize as i64` is lossless for
counts that won't approach `i64::MAX`; `TryFrom<i64>` handles the reverse direction safely on
32-bit targets).

### Timer as an Object

The current `Timer` struct holds an `Arc<Mutex<Signal<()>>>` and exposes a direct `mpsc::Sender`
in `start()`. The redesign replaces this with:

- `#[derive(Extend, Object)]` with `#[base] base: ObjectBase`
- `#[prop] pub interval: Duration` — backed by `Value::Duration`
- `#[prop] pub single_shot: bool`
- `tick: Arc<Mutex<Signal<(usize,)>>>` — **not** macro-managed (see below)
- `start(driver: Arc<dyn TimerDriver>)` and `stop()` — no `mpsc::Sender`
- `block_signals()` / `unblock_signals()` / `signals_blocked()` public methods on `Timer`

The `#[derive(Object)]` macro generates property read/write dispatch for `interval` and
`single_shot`. The `tick` signal is **not** declared with `#[signal]` because the driver
callback runs on a background thread and must emit via a shared `Arc<Mutex<Signal<...>>>`.
Declaring `tick` as a plain `Signal` field and emitting from `Arc<TimerState>` would create
two separate `Signal` instances; slots connected via the public API would never fire.

Instead, `Timer` holds `tick: Arc<Mutex<Signal<(usize,)>>>` (the same `Arc` as in
`TimerState`), and the following wrappers are written manually:

- `connect_tick<F: Fn(&(usize,)) + Send + 'static>(&self, f: F) -> ConnectionId`
- `connect_tick_auto<F: Fn(&(usize,)) + Send + 'static>(&self, f: F) -> ConnectionId`
- `connect_tick_queued<F: Fn(&(usize,)) + Send + 'static>(&self, f: F) -> ConnectionId`
- `disconnect_tick(&self, id: ConnectionId)`
- `emit_tick(&self, fire_count: usize)` — checks `base.signals_blocked()`; emits if clear

This mirrors the current `connect_tick` / `disconnect_tick` pattern in the existing `Timer`,
generalized to `usize` payload and expanded with auto/queued variants. The `Arc` is shared between
`Timer` and `TimerState` so the same `Signal` backing is used in both paths.

#### signals_blocked synchronisation

The driver callback runs on a background thread and does not have direct access to `&mut Timer`.
It instead reads a `signals_blocked: AtomicBool` inside the shared `Arc<TimerState>`. To keep
this in sync with `base.signals_blocked()`:

- `Timer::block_signals()` calls `self.base.block_signals()` **and** `self.state.signals_blocked.store(true, Relaxed)`
- `Timer::unblock_signals()` calls `self.base.unblock_signals()` **and** `self.state.signals_blocked.store(false, Relaxed)`
- `Timer::signals_blocked()` delegates to `self.base.signals_blocked()`

These are the correct public API. `AsObject::object_base_mut()` still exposes `&mut ObjectBase`
via the trait, but calling `object_base_mut().block_signals()` directly bypasses the
`TimerState` sync — this is a documented limitation. Callers must use `Timer::block_signals()`.

Since `base` is a public field (required by `#[derive(Object)]` codegen), this limitation cannot
be enforced at the type level. A doc comment on `base` will state: "Use `Timer::block_signals()`
/ `Timer::unblock_signals()` to correctly gate driver-initiated emissions."

Since `base` is a public field (required by `#[derive(Object)]` codegen), this limitation cannot
be enforced at the type level. A doc comment on `base` will state: "Use `Timer::block_signals()`
/ `Timer::unblock_signals()` to correctly gate driver-initiated emissions."

### TimerDriver trait

```rust
pub trait TimerDriver: Send + Sync + 'static {
    fn start(&self, config: TimerConfig, callback: Box<dyn Fn() + Send + Sync + 'static>);
    fn stop(&self, id: ObjectId);
}
```

`TimerConfig` carries `timer_id: ObjectId`, `interval: Duration`, and `single_shot: bool` —
snapshotted at `start()` time so live property changes only take effect on the next `start()`.
`timer_id` is used by `PoolDriver` to key into its per-timer maps; `ThreadDriver` and `AppDriver`
(one-timer-per-instance) ignore it. `stop(id)` follows the same convention: `PoolDriver` uses the
id to remove its entry; single-timer drivers ignore it.

`Timer::start()` calls `driver.start(config, callback)` where `callback` is a closure that:
1. Checks `TimerState::signals_blocked`; if set, skips emission
2. Checks `TimerState::running`; if clear, exits (for single-shot / stop race)
3. Emits the `tick` signal by calling `self.state.signal.lock().ok()?.emit(&fire_count)`
   after incrementing `fire_count` from the `AtomicUsize`

### TimerState

```rust
struct TimerState {
    signal: Mutex<Signal<(usize,)>>,
    fire_count: AtomicUsize,
    running: AtomicBool,
    signals_blocked: AtomicBool,   // mirrors base.signals_blocked(); kept in sync by Timer wrappers
}
```

`Timer` owns `Arc<TimerState>`; the driver callback captures `Arc<TimerState>`.

### Three driver implementations

**ThreadDriver** — one thread per timer, sleeping for `interval` in a loop.

To support `stop(&self, _id: ObjectId)` while still joining the thread, `ThreadDriver` uses
interior mutability:

```rust
pub struct ThreadDriver {
    running: Arc<AtomicBool>,
    // Stores (Thread-for-unpark, JoinHandle-for-join) together so stop() can do both atomically.
    handle: Mutex<Option<(Thread, JoinHandle<()>)>>,
}
```

On `start()`: spawn the thread, capture `handle.thread().clone()` before storing the `JoinHandle`,
store both in `handle`. The background loop uses `thread::park_tick` for the sleep so `stop()`
can call `unpark()` for immediate wakeup. `stop()` sets `running = false`, then locks `handle`,
calls `.take()` to get `(thread, join)`, calls `thread.unpark()`, and finally `join.join()`.

**AppDriver** — a background thread identical to `ThreadDriver` in loop structure, but instead of
calling the callback directly, wraps it and posts via
`Application::global()?.post_event(Box::new(move || cb()))` where `cb: Arc<dyn Fn() + Send + Sync>`
is cloned each tick to produce a `FnOnce` closure. If `Application::global()` returns `None`
(app dropped while timer was running), the tick is silently skipped. Same
`Mutex<Option<(Thread, JoinHandle<()>)>>` + `AtomicBool` interior-mutability pattern as
`ThreadDriver`.

**PoolDriver** — single shared background thread + min-heap of deadlines, shared across multiple
timers. All mutable pool state lives behind **one** `Mutex<PoolState>` to eliminate lock-ordering
questions:

```rust
pub struct PoolDriver {
    inner: Arc<PoolInner>,
}

struct PoolState {
    heap:      BinaryHeap<Reverse<(Instant, ObjectId)>>,
    tasks:     HashMap<ObjectId, Arc<TimerState>>,
    callbacks: HashMap<ObjectId, Arc<dyn Fn() + Send + Sync>>,
}

struct PoolInner {
    state:   Mutex<PoolState>,
    condvar: Condvar,
    running: AtomicBool,
}
```

Heap element is `Reverse<(Instant, ObjectId)>` — both `Instant: Ord` and (after Task 1)
`ObjectId: Ord`, so the tuple is `Ord`. `TimerState` and the callback are co-located in the same
`PoolState` struct, accessed under one lock.

`start(config, callback)` locks `state`, pushes `(deadline, config.timer_id)` into `heap`, inserts
into `tasks` and `callbacks`, drops the lock, then notifies `condvar`.

`stop(_, id)` locks `state`, clones `Arc<TimerState>` from `tasks` (before removing), removes `id`
from `tasks` and `callbacks`, drops the lock, marks `state_clone.running.store(false, Relaxed)`,
and notifies `condvar`.

The pool thread loop (all state accesses hold the single `state` mutex):
1. Lock `state`.
2. If `heap` empty: `condvar.wait(guard)` (releases lock; re-acquires on return); restart from step 2.
3. Peek at earliest `Reverse((deadline, _))`. If `deadline > Instant::now()`: `condvar.wait_tick(guard, deadline - now)` (releases and re-acquires); restart from step 2 to re-check emptiness and deadline.
4. Pop the entry `(_, id)`. If `id` absent from `tasks` (cancelled): loop back to step 2.
5. Clone the `Arc` callback from `callbacks`. Also clone `Arc<TimerState>` from `tasks` if repeating. Drop the lock.
6. Call `callback()`.
7. If repeating (`!tasks[id].single_shot`): re-lock, re-push `(Instant::now() + interval, id)` with new deadline; notify condvar.

### Rejected alternatives

- **Async timer**: out of scope per spec.
- **Timer wheel**: out of scope; min-heap is sufficient for the pool driver.
- **`mpsc::Sender` in `start()`**: replaced by `Arc<dyn TimerDriver>`.
- **`Arc<Mutex<Timer>>` as the shared state**: too coarse; locks the whole object per tick.
- **`&mut self` on `TimerDriver::stop`**: would require `Arc<Mutex<dyn TimerDriver>>` instead of
  `Arc<dyn TimerDriver>`, complicating call sites. Interior mutability in each driver is cleaner.
- **Tree-lock approach for `signals_blocked`** (locking `ObjectTree` in callback then calling
  `emit_tick()`): introduces reentrant lock risk if slots call `try_with_tree`; tightly couples
  all three drivers to `Application`. Rejected in favour of the `TimerState` mirror + wrapper API.
- **Multiple `Mutex` fields in `PoolDriver`** (`heap`, `tasks`, `callbacks` each separately):
  creates lock-ordering requirements and potential deadlock. Collapsed into one `Mutex<PoolState>`.
- **`#[signal] tick` on `Timer`** (macro-managed signal field): the driver callback runs on a
  background thread and must emit via `Arc<Mutex<Signal<...>>>`. Using a plain `Signal` field and
  emitting from `Arc<TimerState>` would be two separate `Signal` instances — connected slots would
  never fire. Solved by sharing `Arc<Mutex<Signal<(usize,)>>>` between `Timer` and `TimerState`,
  with manually written `connect_tick*`/`emit_tick` wrappers.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `PartialOrd, Ord` to `ObjectId` and `ConnectionId` | `quartzite-core/src/id.rs` | — |
| 2 | Add `Value::Duration` variant; add `usize` `FromValue`/`IntoValue` | `quartzite-core/src/value.rs` | — |
| 3 | Define `TimerDriver` trait, `TimerConfig` struct, `TimerState` inner struct | `quartzite-runtime/src/timer.rs` | 2 |
| 4 | Refactor `Timer` struct: `#[derive(Extend, Object)]`, declare `tick` signal and properties, embed `Arc<TimerState>`, add `block_signals`/`unblock_signals` wrappers | `quartzite-runtime/src/timer.rs` | 3 |
| 5 | Implement `Timer::start(driver)`, `stop()`, `is_running()` | `quartzite-runtime/src/timer.rs` | 4 |
| 6 | Implement `ThreadDriver` | `quartzite-runtime/src/timer.rs` (or `timer/thread_driver.rs`) | 3 |
| 7 | Implement `AppDriver` | `quartzite-runtime/src/timer.rs` (or `timer/app_driver.rs`) | 3 |
| 8 | Implement `PoolDriver` | `quartzite-runtime/src/timer.rs` (or `timer/pool_driver.rs`) | 1, 3 |
| 9 | Re-export `TimerDriver`, `TimerConfig`, `ThreadDriver`, `AppDriver`, `PoolDriver` from `quartzite-runtime::lib` | `quartzite-runtime/src/lib.rs` | 6, 7, 8 |
| 10 | Tests: `Value::Duration` and `usize` round-trips; `ObjectId`/`ConnectionId` ordering | `quartzite-core/src/value.rs` `#[cfg(test)]`, `quartzite-core/src/id.rs` `#[cfg(test)]` | 1, 2 |
| 11 | Tests: Timer in ObjectTree, named lookup, `signals_blocked` suppression, all three drivers, `single_shot` fires once | `quartzite-runtime/tests/timer.rs` | 5, 6, 7, 8 |

9 production tasks + 2 test tasks = 11 total.

### Module layout decision

Keep everything in `quartzite-runtime/src/timer.rs` if it stays under ~300 lines; split into
`quartzite-runtime/src/timer/` submodule (with `mod.rs`, `driver.rs`, `thread_driver.rs`,
`app_driver.rs`, `pool_driver.rs`) if the combined file would exceed 300 lines. The split is a
pure refactor and does not affect the public API.

## Risks

- **`Value::Duration` match exhaustiveness**: every existing `match val` / `match self` in
  `value.rs` and downstream tests must grow a `Duration` arm. Compiler enforces this.
- **`usize` width on 32-bit targets**: `usize` is 32 bits; `impl_int_checked!` casts to `i64`
  (always wider), so no truncation on `IntoValue`. `from_value` uses `TryFrom<i64>` so values
  above `u32::MAX` on 32-bit hosts produce `TypeError` — acceptable and consistent with existing
  `u64` pattern.
- **`signals_blocked` bypass via `object_base_mut()`**: documented limitation; callers who use
  `AsObject::object_base_mut().block_signals()` directly may see up to one spurious tick.
  Mitigated by doc comments; not exploitable at a library level.
- **`PoolDriver` cancellation**: stale heap entries (id absent from `tasks` map) are discarded at
  pop-time. `stop()` notifies the condvar so the thread wakes promptly. No data race because
  `heap`, `tasks`, and `callbacks` are each behind a `Mutex`.
- **`AppDriver`/`ThreadDriver` stop latency**: `park_tick` + `unpark()` limits worst-case
  wait; immediate wakeup on `stop()` in the common case.
- **`AppDriver` `None` from `Application::global()`**: explicitly handled with `if let Some` —
  skips the tick, does not panic.
- **Macro codegen for `Duration` property**: `IntoValue`/`FromValue` for `core::time::Duration`
  implemented in task 2; macro uses them generically — no macro changes needed.
- **Old `start(mpsc::Sender)` API is removed**: update all existing doctests and examples.
- **Tests using key-surrogate ordering on `*Id`**: any test that works around the missing `Ord`
  by sorting via `.raw()` or similar must be updated to use the native `<`/`>`/`cmp` once
  `PartialOrd, Ord` are derived. Grep for `.raw()` in sort/compare contexts during Task 1.

## Test Design

### Task 10 — unit tests in `quartzite-core`

**`value.rs` `#[cfg(test)]` — `Value::Duration` and `usize`:**
- `Duration::from_secs(1).into_value()` returns `Value::Duration(Duration::from_secs(1))`
- `Duration::from_value(Value::Duration(d))` round-trips
- `Duration::from_value(Value::Int(0))` returns `Err(TypeError { expected: "Duration", got: "Int" })`
- `usize::from_value(Value::Int(0))` returns `Ok(0usize)`
- `usize::from_value(Value::Int(-1))` returns `Err`
- `42usize.into_value()` equals `Value::Int(42)`
- `Value::Duration(_).type_name()` returns `"Duration"`

**`id.rs` `#[cfg(test)]` — ordering:**
- Two sequentially allocated `ObjectId`s: the first `<` the second
- Same for `ConnectionId`
- `ObjectId::new() != ObjectId::new()` still holds (existing test, verify not broken)

### Task 11 — `quartzite-runtime/tests/timer.rs` integration tests

- **Fixtures**: `Application::new()` in isolation (reuse existing serial-test pattern).

- **AC2 — Timer in ObjectTree / named lookup**:
  - Insert `Timer`, retrieve by `ObjectId`; insert with `ObjectBase::named("my-timer")`, look up by name

- **AC3 — fire count increments**:
  - Collect counts over 3 ticks via `ThreadDriver`; assert values are `[0, 1, 2]`

- **AC5 — `signals_blocked` suppresses tick**:
  - `timer.block_signals()` before start; run 2 ticks; assert slot never called

- **AC7 — ThreadDriver fires at interval** (±50% timing tolerance):
  - 50 ms interval; run ~150 ms; assert ≥1 fire

- **AC8 — AppDriver executes on event-loop thread**:
  - Record `thread::current().id()` in slot; compare with event-loop thread id

- **AC9 — PoolDriver shared across two timers**:
  - Two timers sharing one `Arc<PoolDriver>`; assert both fire ≥1 time

- **AC11 — single_shot fires exactly once** (parameterised over all three drivers):
  - Run for 3× interval; assert count == 1

- **Helpers**: `make_timer(interval_ms)`; `wait_for(counter, n, tick)` using `Condvar`.

## Open questions

(none)
