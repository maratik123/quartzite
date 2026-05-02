# ReceiverGuard for Auto Connections

**Source:** issue #50
**Date:** 2026-05-03
**Tracked in:** #50

## Scope

1. Add `guard: Weak<ReceiverGuard>` field to `AutoSlotInner`.
2. In `DynAutoSlot::dispatch`: check guard on **both** same-thread (direct) and cross-thread (post) paths — silently skip if guard is dead.
3. Update `Signal::connect_auto` signature: add `guard: Weak<ReceiverGuard>` parameter (after `receiver_thread_id`, before `f`).
4. Update existing `connect_auto` call sites (tests in `signal.rs`, `auto_no_dispatcher.rs`) to pass a guard.
5. Generate typed `connect_<signal>_auto(&mut self, receiver: &ObjectBase, f: F) -> ConnectionId` convenience methods in `#[derive(Object)]` codegen (gated `#[cfg(feature = "std")]`, `#[inline]`). The generated wrapper extracts `thread_id` and `Weak<ReceiverGuard>` from `receiver` and delegates to `Signal::connect_auto`.
6. New tests:
   - `auto_cross_thread_slot_not_posted_after_receiver_destroyed` (deferred in auto-connection design)
   - `auto_same_thread_slot_not_called_after_receiver_destroyed` (same-thread sibling)
   - Codegen test: generated `connect_<signal>_auto` method is present in output.

## Out of scope

- `ConnectionTable` changes — guard check is local to `Signal::AutoSlotInner::dispatch`.
- `signals_blocked` integration — tracked in #38.
- `BlockingQueued` (#48), signal-to-signal connections (#49) — separate issues.

## Deferred

- `connect_<signal>_queued` typed codegen wrappers | out of issue scope; natural follow-up after this lands.

## Key decisions

| Question | Decision |
|---|---|
| Guard check on same-thread path? | Yes — consistent with cross-thread; both paths skip if guard dead |
| Generated wrapper signature | `connect_<signal>_auto(&mut self, receiver: &ObjectBase, f: F)` — takes `&ObjectBase`, extracts `thread_id` + `Weak<ReceiverGuard>` internally |
| Guard param position in `connect_auto` | After `receiver_thread_id`, before `f` |

## Technical constraints

- `connect_auto` and `AutoSlotInner` are `#[cfg(feature = "std")]`; the generated wrapper must carry the same cfg gate.
- `#[inline]` required on the generated `connect_<signal>_auto` — simple delegation, no branches or loops.
- `ObjectBase::thread_id` is `pub`; `ObjectBase::receiver_guard()` returns `&Arc<ReceiverGuard>` — both accessible in codegen-emitted code via `::quartzite::core`.
- Existing `connect_auto` callers in tests must be updated (5–6 in `signal.rs`, 1 in `auto_no_dispatcher.rs`); no production call sites exist.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | After the receiver `Arc<ReceiverGuard>` is dropped, a same-thread Auto slot is **not** called on the next emit. |
| AC2 | After the receiver `Arc<ReceiverGuard>` is dropped, a cross-thread Auto slot is **not** posted to the dispatcher on the next emit. |
| AC3 | Same-thread Auto slot is still called while the guard is alive (no regression). |
| AC4 | Cross-thread Auto slot is still posted while the guard is alive (no regression). |
| AC5 | `#[derive(Object)]` structs with signals expose a `connect_<signal>_auto` method gated `#[cfg(feature = "std")]`. |
| AC6 | `connect_<signal>_auto` accepts a `&ObjectBase` receiver and a closure; internally delegates to `Signal::connect_auto` using `receiver.thread_id` and `Arc::downgrade(receiver.receiver_guard())`. |
| AC7 | `Direct` and `Queued` connection semantics are unchanged. |
| AC8 | `cargo build --no-default-features` (no_std path) compiles without error. |

## Open questions

_(none)_
