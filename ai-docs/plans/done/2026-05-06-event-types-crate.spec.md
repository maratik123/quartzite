# Event-types subcrate extraction

**Source:** issue #95
**Date:** 2026-05-06
**Tracked in:** #95

## Scope

1. Create new `quartzite-event-types` subcrate (`no_std` + alloc, added to workspace).
2. Move `Event<T>` trait, `EventType<T>` enum, `EventFilter<T>` trait, `KeyEventKind`, `MouseEventKind` from `quartzite-events/src/event.rs` into the new crate.
3. Move `TimerEvent` from `quartzite-events/src/timer.rs` into the new crate; add `fire_count: usize` field to carry the fire count that the runtime currently passes as a bare `usize`.
4. `quartzite-events` re-exports all six types from `quartzite-event-types`; its `Cargo.toml` gains the new dependency.
5. `quartzite-runtime` adds `quartzite-event-types` to `Cargo.toml`; changes `Timer.tick` from `Signal<(usize,)>` to `Signal<(TimerEvent,)>`; updates `connect_tick`, `connect_tick_queued`, `connect_tick_auto`, `emit_tick`, `disconnect_tick` accordingly.
6. Update `quartzite` facade `Cargo.toml` if the facade re-exports event types through a path that changes.
7. Update `ai-docs/context.md` and `README.md` to reflect the new crate.

## Out of scope

- Concrete event types that are not `TimerEvent` (`KeyEvent`, `MouseEvent`, `ResizeEvent`, `CloseEvent`) — these remain in `quartzite-events`.
- `enumflags2`-backed `KeyModifiers` flags — stays in `quartzite-events`.
- Widget or graphics-stack events.
- A full event-dispatch loop using the unified types.

## Deferred

- Porting more runtime concepts (e.g. `Application` event queue) to the unified event type system | out of scope for this PR | no separate issue needed yet.

## Key decisions

| Question | Decision |
|---|---|
| Crate name | `quartzite-event-types` |
| `no_std` support | Yes — crate is `#![no_std]` + alloc (mirrors `quartzite-events`) |
| Timer signal type | Switch `Timer.tick` from `Signal<(usize,)>` to `Signal<(TimerEvent,)>` in this PR |
| `fire_count` in `TimerEvent` | Add `fire_count: usize` field so no information is lost vs. current `(usize,)` payload |

## Technical constraints

- `quartzite-event-types` depends only on `quartzite-core` (for `ObjectId` used by `EventFilter` and `TimerEvent`); no geometry, no std.
- `quartzite-events` continues to be `#![no_std]` after the change.
- `quartzite-runtime` must NOT depend on `quartzite-events` — only on `quartzite-event-types`.
- No duplicate `TimerEvent` definitions; the one in `quartzite-event-types` is the canonical type.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-event-types` crate exists in the workspace with `#![no_std]` + `extern crate alloc` and `#![deny(missing_docs)]`. |
| AC2 | Crate publicly exposes `Event<T>`, `EventType<T>`, `EventFilter<T>`, `KeyEventKind`, `MouseEventKind`, `TimerEvent`; every public item has a `///` doc line and a `# Examples` block. |
| AC3 | `TimerEvent` carries `timer_id: ObjectId` and `fire_count: usize`; both fields are accessible via `const fn` getters. |
| AC4 | `quartzite-events` re-exports all six types from `quartzite-event-types`; existing `use quartzite_events::{Event, EventType, EventFilter, KeyEventKind, MouseEventKind, TimerEvent}` compile without modification. |
| AC5 | `Timer.tick` signal type is `Signal<(TimerEvent,)>`; `connect_tick` slot closures receive `&(TimerEvent,)`; `connect_tick_queued` and `connect_tick_auto` receive `(TimerEvent,)`. |
| AC6 | `emit_tick` accepts a `TimerEvent` argument (not a bare `usize`). |
| AC7 | `quartzite-runtime/Cargo.toml` lists `quartzite-event-types` as a dependency; it does **not** list `quartzite-events`. |
| AC8 | `cargo build -p quartzite --no-default-features` compiles clean. |
| AC9 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` passes with no warnings or errors. |
| AC10 | All pre-existing timer tests pass; `quartzite-event-types` has a `#[cfg(test)] mod tests` block with at least 3 unit tests covering the moved types. |

## Open questions

_(none)_
