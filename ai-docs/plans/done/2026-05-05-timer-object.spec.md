# feat: promote Timer to a first-class quartzite object

**Source:** issue #36
**Date:** 2026-05-05
**Tracked in:** #36

## Scope

1. Derive `PartialOrd` and `Ord` on `ObjectId` and `ConnectionId` in `quartzite-core` (both wrap `u64`; ordering follows allocation order)
2. Add `Value::Duration(core::time::Duration)` variant to `quartzite_core::value::Value`; implement `FromValue` and `IntoValue` for `core::time::Duration`
3. Apply `#[derive(Object)]` to `Timer` — adds `base: ObjectBase`, implements `AsObject`, makes `Timer` insertable into `ObjectTree`
4. Declare `tick` as a signal carrying `usize` (monotonically increasing fire count, 0-indexed from the first fire)
5. Declare `interval` as a read/write `Duration` property; `single_shot` as a read/write `bool` property; changes take effect on the next `start()` call
6. `signals_blocked` on the `Timer` object (via `timer.block_signals()`) suppresses `tick` emissions
7. Define public `TimerDriver` trait in `quartzite-runtime` — pluggable timer backend
8. `ThreadDriver` — one dedicated background thread per timer
9. `AppDriver` — executes tick callbacks on the application event-loop thread via `Application::global()`; gracefully skips if `Application` is not live
10. `PoolDriver` — one shared background thread using a min-heap of `(Instant, ObjectId)` deadlines, shared across multiple timers
11. `connect_auto` on `tick` uses `base.thread_id` for correct cross-thread delivery
12. Tests: `Timer` in `ObjectTree`; named-timer lookup; `signals_blocked` suppresses tick; all three driver backends fire correctly; `single_shot` fires exactly once

## Out of scope

- `quartzite-events` restructuring / shared event-types subcrate (→ #95)
- Timer-wheel implementation for `PoolDriver` (min-heap single thread is sufficient)
- Async timer support
- Explicit `mpsc::Sender` in `start()` (superseded by `TimerDriver`)

## Deferred

- Unified user event payload subcrate | needs cross-crate design | → #95

## Key decisions

| Question | Decision |
|---|---|
| `ObjectId`/`ConnectionId` ordering | Add `PartialOrd, Ord` (wraps `u64`; allocation order) |
| `#[derive(Object)]` vs manual `AsObject`? | `#[derive(Object)]` — consistent with all other object types |
| `tick` payload type | `usize` fire count (0-indexed; idiomatic for counts; enables exponential backoff as `2usize.pow(n)`) |
| `interval` property representation in `Value` | `Value::Duration` — first-class variant added to `quartzite_core::value::Value` |
| `interval` / `single_shot` live-settable? | Yes (r/w properties); changes apply on next `start()` |
| Timer backend model | Pluggable `TimerDriver` trait; three built-in implementations |
| `PoolDriver` internals | Single background thread + min-heap of `(Instant, ObjectId)` |
| `start()` API | Takes `Arc<dyn TimerDriver>` — no explicit `mpsc::Sender` |
| `signals_blocked` public API | `Timer::block_signals()` / `unblock_signals()` wrappers sync both `base` and driver state; calling `object_base_mut().block_signals()` directly bypasses driver sync (documented limitation) |

## Technical constraints

- `quartzite-core` must remain `no_std`-compatible; `core::time::Duration` is available without `std`
- `Value::Duration` addition must be exhaustive across all `match` arms in `value.rs`
- `TimerDriver` must be `Send + Sync + 'static` to allow `Arc`-sharing across threads (`PoolDriver` requires this)
- `PoolDriver` must handle timer cancellation (call to `stop()` while the pool sleeps) without data races
- `AppDriver` calls `Application::global()` from a background context; must handle `None` (application already dropped) gracefully — skip the tick, do not panic

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `core::time::Duration::from_secs(1).into_value()` returns `Value::Duration(_)`; `Duration::from_value(Value::Duration(d))` round-trips to the original value |
| AC2 | `Timer` implements `AsObject`; a `Timer` can be inserted into `ObjectTree` and retrieved by its registered name |
| AC3 | The first `tick` fire passes `0` to connected slots; the second fire passes `1`; count increments on every fire |
| AC4 | `timer.interval` and `timer.single_shot` are readable and writable via the declared property system |
| AC5 | Calling `timer.block_signals()` before a tick causes no `tick` emission; calling `timer.unblock_signals()` restores normal firing |
| AC6 | `TimerDriver` trait is public and stable; `ThreadDriver`, `AppDriver`, and `PoolDriver` each implement it and are re-exported from `quartzite-runtime` |
| AC7 | A timer started with `ThreadDriver` fires its `tick` signal at approximately the configured `interval` |
| AC8 | A timer started with `AppDriver` executes slots on the application event-loop thread |
| AC9 | Two or more timers sharing one `PoolDriver` each fire at approximately their individual configured intervals |
| AC10 | `connect_auto` on `tick` uses `base.thread_id`; a slot connected from thread A on a `Timer` created on thread B is invoked on thread A |
| AC11 | A `single_shot` timer fires exactly once regardless of which driver backend is used |
| AC12 | `ObjectId` and `ConnectionId` implement `PartialOrd` and `Ord`; a later-allocated id compares greater than an earlier-allocated one |

## Open questions

(none)
