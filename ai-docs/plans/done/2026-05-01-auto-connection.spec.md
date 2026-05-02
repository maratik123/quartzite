# AutoConnection

**Source:** User task — derived from signal-slot chat log analysis (tmp/signal-slot.impr.chat.txt)
**Date:** 2026-05-01

## Scope

Add `ConnectionType::Auto` to `quartzite-core`. At emit time, compare the emitting thread against
the receiver's `thread_id` (captured at connect time). If same thread → behave as `Direct`
(call slot immediately). If different thread → behave as `Queued` (post to `QueuedDispatcher`).

- `ConnectionType::Auto` variant in `quartzite-core`
- `SlotEntry` stores receiver `ThreadId` for `Auto` connections (captured at connect time)
- `Signal::emit` thread check: `thread::current().id() == stored_thread_id`
- Same-thread path: call slot with borrowed `&Args` (identical to Direct)
- Cross-thread path: clone `Args`, post owned closure to `QueuedDispatcher`
- No dispatcher installed: silently drop (no panic, no log)
- `Auto` variant gated behind `#[cfg(feature = "std")]` (same as `Queued`)

## Out of scope

- `BlockingQueued` — threading model not yet decided (already deferred in core-types spec)
- Thread migration — if an object moves to another thread after a connection is established,
  the captured `thread_id` will be stale; this requires a separate object-mobility design
- Signal-to-signal connections — blocked on runtime design (already deferred)

## Deferred

- Stale `thread_id` invalidation | needs object-mobility / thread-affinity-change API first
- `AutoConnection` in no_std | same gating as `Queued`; unblocked when std feature is defined

## Key decisions

| Question | Decision |
|---|---|
| API surface | `ConnectionType::Auto` variant, not a separate `connect_auto()` method |
| Same-thread behavior | Direct — slot called synchronously with `&Args` |
| Cross-thread behavior | Queued — `Args` cloned, posted to `QueuedDispatcher` |
| No dispatcher | Silent drop — consistent with existing `Queued` behavior |
| `thread_id` timing | Captured at connect time from `receiver.object_base().thread_id` |
| `no_std` policy | Compile error — `Auto` variant `#[cfg(feature = "std")]` only |

## Technical constraints

- `Args: Clone` required for `Auto` connections (cross-thread path must own the args)
- `Auto` slot closure must be `Box<dyn Fn(Args) + Send + Sync>` (owned args, cross-thread safe)
  — same constraint as `Queued` slot entries
- At emit, `Signal` reads `thread::current().id()` once; compares against each `Auto` slot's
  stored `ThreadId`
- `Signal<Args>` itself remains `!Send + !Sync` in this iteration (unchanged from core-types spec)

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ConnectionType::Auto` connected on the same thread: `emit` calls the slot synchronously before returning |
| AC2 | `ConnectionType::Auto` connected from a different thread: `emit` posts the invocation to `QueuedDispatcher` without calling the slot directly |
| AC3 | `ConnectionType::Auto` with no `QueuedDispatcher` installed and cross-thread emit: slot is not called, no panic, no error returned |
| AC4 | `ConnectionType::Auto` variant is unavailable in `#![no_std]` builds (compile error) |
| AC5 | Receiver's `thread_id` is captured at `connect` time; changing `ObjectBase::thread_id` after connect does not affect which path is taken on the next emit |

## Open questions

- Should `Auto` on a same-thread emit respect `signals_blocked` the same way as `Direct`?
  (Almost certainly yes — confirm when implementing `signals_blocked` logic)
- Should the `SlotEntry` for `Auto` store both a `Fn(&Args)` (direct path) and a
  `Fn(Args) + Send + Sync` (queued path), or a single `Fn(Args) + Send + Sync` called
  directly (with clone) on the same-thread path? Design doc to decide.
