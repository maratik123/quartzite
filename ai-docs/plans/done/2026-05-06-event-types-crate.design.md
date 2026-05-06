# Design: Event-types subcrate extraction

**Issue:** #95
**Date:** 2026-05-06

## Approach

Create a new `quartzite-event-types` crate that holds the five types from
`quartzite-events/src/event.rs` (`Event<T>`, `EventType<T>`, `EventFilter<T>`,
`KeyEventKind`, `MouseEventKind`) and the one type from
`quartzite-events/src/timer.rs` (`TimerEvent`, extended with `fire_count`).
`quartzite-events` re-exports all six so existing `use` paths compile unchanged.
`quartzite-runtime` replaces its `Signal<(usize,)>` tick signal with
`Signal<(TimerEvent,)>` and drops any direct dependency on `quartzite-events`.

### Why this split

`quartzite-runtime` needs `TimerEvent` to build a typed tick signal, but it must
not pull in `quartzite-events` (which depends on `quartzite-geometry`, `enumflags2`,
`alloc::String` — unnecessary weight for the runtime layer). Separating the
stable, no-alloc event vocabulary (`Event<T>`, `TimerEvent`, …) into an
intermediate crate satisfies both the runtime's need and the no-`std`/minimal-dep
constraint.

### Rejected alternative: move types directly into `quartzite-core`

`quartzite-core` is deliberately free of event semantics. Adding `Event<T>` and
`TimerEvent` there would conflate the "object model" crate with the "event
vocabulary" crate and break the planned `quartzite-widgets` / `quartzite-paint`
layering. A dedicated intermediate crate keeps concerns separated.

### `fire_count` addition

`Timer` currently stores the fire count in `TimerState::fire_count: AtomicUsize`
and emits it as a bare `usize`. Moving this value into `TimerEvent` is
semantically lossless: `emit_tick(fire_count)` becomes
`emit_tick(TimerEvent::new(timer_id, fire_count))`. The `TimerState.fire_count`
atomic remains — it is the source of truth; `TimerEvent.fire_count` is its
point-in-time snapshot passed to slots.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `quartzite-event-types` crate skeleton: `Cargo.toml`, `src/lib.rs` (`#![no_std]` + alloc, `#![deny(missing_docs)]`, crate-level doc) | `quartzite-event-types/Cargo.toml`, `quartzite-event-types/src/lib.rs` | — |
| 2 | Add workspace member and facade dep: add `quartzite-event-types` to `[workspace] members` in root `Cargo.toml`; add it as a dep to the `quartzite` facade | root `Cargo.toml` | 1 |
| 3 | Copy event vocabulary into new crate: move `Event<T>`, `EventType<T>`, `EventFilter<T>`, `KeyEventKind`, `MouseEventKind` into `quartzite-event-types/src/event.rs`; update imports (swap `use quartzite_core::ObjectId` for the already-present dep); keep all existing doc-comments, `# Examples` blocks, and unit tests | `quartzite-event-types/src/event.rs` | 1 |
| 4 | Extend and move `TimerEvent` into new crate: copy `TimerEvent` into `quartzite-event-types/src/timer.rs`; add `fire_count: usize` field; add `TimerEvent::new(timer_id, fire_count)` and `fire_count()` getter with doc+examples; update existing `timer_id()` getter doc/examples; update `Event<T> for TimerEvent` impl (unchanged body); keep existing tests, add `fire_count` tests | `quartzite-event-types/src/timer.rs` | 1, 3 |
| 5 | Wire up `quartzite-event-types/src/lib.rs`: `pub mod event; pub mod timer;` + `pub use` for all six types; `pub use quartzite_core::ObjectId` (mirror what `quartzite-events` exposes) | `quartzite-event-types/src/lib.rs` | 3, 4 |
| 6 | Update `quartzite-events`: add `quartzite-event-types` dep to `quartzite-events/Cargo.toml`; replace bodies of `event.rs` and `timer.rs` with re-exports from `quartzite_event_types`; keep crate-level `pub use` lines in `lib.rs` unchanged (still re-export the same six names) | `quartzite-events/Cargo.toml`, `quartzite-events/src/event.rs`, `quartzite-events/src/timer.rs`, `quartzite-events/src/lib.rs` | 5 |
| 7 | Update `quartzite-runtime/Cargo.toml`: add `quartzite-event-types`; confirm `quartzite-events` is absent | `quartzite-runtime/Cargo.toml` | 5 |
| 8 | Migrate `quartzite-runtime/src/timer.rs` to `Signal<(TimerEvent,)>`: change `TimerState::signal` type; update `TimerState::new`; change `Timer.tick` field type; update `connect_tick` / `connect_tick_queued` / `connect_tick_auto` / `disconnect_tick` signatures and bodies; change `emit_tick(fire_count: usize)` → `emit_tick(event: TimerEvent)`; update `start` callback to construct `TimerEvent::new(config.timer_id, count)` and emit it; update all doc-comments and `# Examples` blocks | `quartzite-runtime/src/timer.rs` | 7 |
| 9 | Update unit tests in `quartzite-runtime/src/timer.rs`: fix slots that match on `args.0: usize` to destructure `args.0: TimerEvent`; update `emit_tick(N)` call-sites to `emit_tick(TimerEvent::new(id, N))` | `quartzite-runtime/src/timer.rs` | 8 |
| 10 | Update `ai-docs/context.md` and `README.md`: add `quartzite-event-types` row to the crate layout table; update Timer-related design decision notes | `ai-docs/context.md`, `README.md` | 6, 8 |

## Risks

- **Backward-compat of `TimerEvent::new` signature change:** existing callers pass one arg (`timer_id`); after this PR they must pass two (`timer_id, fire_count`). Only `quartzite-runtime` and `quartzite-events` tests call `TimerEvent::new` — both are updated in tasks 8–9 and 6 respectively. No external downstream (crate not yet published).
- **`Event<T> for TimerEvent` blanket impl lives in new crate:** This is fine because `Event<T>` is also defined in the same new crate; no orphan-rule issue.
- **`quartzite-events/src/event.rs` and `timer.rs` become thin re-export wrappers:** tests in those modules will be removed (moved to the new crate); `quartzite-events` integration tests in `tests/` (if any) are not affected because the public API is unchanged.
- **`no_std` verification:** `quartzite-event-types` must not accidentally pull in `std`. Covered by task 1 (`#![no_std]`) and verified by `cargo build -p quartzite --no-default-features` (AC8).
- **`Signal<(TimerEvent,)>` requires `TimerEvent: Clone + Send + 'static` for queued/auto connections.** `TimerEvent` derives `Copy + Clone` and all fields (`ObjectId`, `usize`) are `Send + Sync + 'static`. No issue.

## Test Design

### Task 3 — event vocabulary (moved to `quartzite-event-types/src/event.rs`)

- Location: `quartzite-event-types/src/event.rs` `#[cfg(test)] mod tests`
- Entry points: `EventType`, `KeyEventKind`, `MouseEventKind`, `Event` trait, `EventFilter` trait
- Scenarios: copy all five existing tests from `quartzite-events/src/event.rs` verbatim (they compile in the new crate unchanged because `quartzite-core` is the only dep)
- Fixtures: `alloc::boxed::Box` for the dyn-Event test (already used)

### Task 4 — `TimerEvent` (moved to `quartzite-event-types/src/timer.rs`)

- Location: `quartzite-event-types/src/timer.rs` `#[cfg(test)] mod tests`
- Entry points: `TimerEvent::new`, `timer_id()`, `fire_count()`, `event_type()`
- Scenarios (≥ 3 new tests as required by AC10):
  1. `timer_event_stores_id` — round-trip `timer_id` (ported from `quartzite-events`)
  2. `timer_event_stores_fire_count` — `TimerEvent::new(id, 7).fire_count() == 7` (new)
  3. `timer_event_fire_count_zero` — `TimerEvent::new(id, 0).fire_count() == 0` (edge)
  4. `timer_event_type` — `event_type() == EventType::<()>::Timer` (ported)
- Fixtures: `ObjectId::default()` for a cheap id

### Task 8–9 — `Timer` signal migration (`quartzite-runtime/src/timer.rs`)

- Location: existing `#[cfg(test)] mod tests`
- Entry points: `emit_tick`, `connect_tick`, `connect_tick_auto`, `connect_tick_queued`
- Scenarios: update the four existing tests that touch `args.0: usize` or `emit_tick(N)`:
  - `emit_tick_fires_when_unblocked` — change `args.0 (usize)` to `args.0.fire_count()`; pass a constructed `TimerEvent`
  - `emit_tick_suppressed_when_blocked` — `emit_tick(TimerEvent::new(id, 0))`
  - `block_unblock_restores_emission` — same pattern
  - `timer_state_signal_shared_with_tick` — emit `&(TimerEvent::new(id, 41),)` directly
  - `connect_tick_auto_same_thread_direct_delivery` — slot arg type `(TimerEvent,)`, check `fire_count`
  - `connect_tick_auto_guard_dropped_skips_slot` — slot arg type `(TimerEvent,)`

## Open questions

_(none)_
