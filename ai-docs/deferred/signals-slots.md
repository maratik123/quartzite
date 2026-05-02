# Signals & Slots

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status |
|------|--------|--------|
| `BlockingQueued` connection type \| threading model not yet decided | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | |
| Signal-to-signal connections \| needs runtime design first | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | |
| `BlockingQueued` connection type \| depends on per-thread loops | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | |
| Enforcing the signals_blocked check at `Signal` level rather than codegen level \| requires API redesign (#38) | [signals-blocked spec](../plans/done/2026-05-02-signals-blocked.spec.md) | |
| `auto_cross_thread_slot_not_posted_after_receiver_destroyed` test \| requires `Weak<ReceiverGuard>` in the auto slot entry; `AutoSlotInner` does not hold a guard in v1 | [auto-connection design](../plans/done/2026-05-01-auto-connection.design.md) | |
| `ReceiverGuard` for `Auto` connections \| `connect_auto` currently accepts no guard; cross-thread Auto slots will post even after the receiver is destroyed; requires `ConnectionTable` integration | [auto-connection design](../plans/done/2026-05-01-auto-connection.design.md) | |

## Out of scope

| Item | Source | Status |
|------|--------|--------|
| `BlockingQueued` — threading model not yet decided (already deferred in core-types spec) | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | |
| Signal-to-signal connections — blocked on runtime design (already deferred) | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | |
| Changes to `Signal::emit` itself (tracked in #38) | [signals-blocked spec](../plans/done/2026-05-02-signals-blocked.spec.md) | |
| Serialization of `signals_blocked` state (tracked in #39) | [signals-blocked spec](../plans/done/2026-05-02-signals-blocked.spec.md) | |

## Open questions

| Item | Source | Status |
|------|--------|--------|
| Should `Signal` be `Send + Sync`? (Needed if objects move across threads) | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | ✅ done |
| Should `Auto` on a same-thread emit respect `signals_blocked` the same way as `Direct`? (Almost certainly yes — confirm when implementing `signals_blocked` logic) | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | ✅ done |
| Should the `SlotEntry` for `Auto` store both a `Fn(&Args)` (direct path) and a `Fn(Args) + Send + Sync` (queued path), or a single `Fn(Args) + Send + Sync` called directly (with clone) on the same-thread path? Design doc to decide. | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | ✅ done |
| **`signals_blocked` interaction with `Auto`:** Deferred to the `signals_blocked` design. | [auto-connection design](../plans/done/2026-05-01-auto-connection.design.md) | ✅ done |
