# Signals & Slots

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| `BlockingQueued` connection type \| threading model not yet decided | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | | #48 |
| Signal-to-signal connections \| needs runtime design first | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | | #49 (closed) |
| `BlockingQueued` connection type \| depends on per-thread loops | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | | #48 |
| Enforcing the signals_blocked check at `Signal` level rather than codegen level \| requires API redesign (#38) | [signals-blocked spec](../plans/done/2026-05-02-signals-blocked.spec.md) | | #38 (closed) |
| `auto_cross_thread_slot_not_posted_after_receiver_destroyed` test \| requires `Weak<ReceiverGuard>` in the auto slot entry; `AutoSlotInner` does not hold a guard in v1 | [auto-connection design](../plans/done/2026-05-01-auto-connection.design.md) | | #50 (closed) |
| `ReceiverGuard` for `Auto` connections \| `connect_auto` currently accepts no guard; cross-thread Auto slots will post even after the receiver is destroyed; requires `ConnectionTable` integration | [auto-connection design](../plans/done/2026-05-01-auto-connection.design.md) | | #50 (closed) |
| `connect_<signal>_queued` typed codegen wrappers — out of issue scope; natural follow-up after this lands. | [receiver-guard-auto spec](../plans/done/2026-05-03-receiver-guard-auto.spec.md) |  | untracked |
| Future `CURRENT_SCHEMA_VERSION` bump policy — `#[serde(default)]` covers additive evolution within v1; a bump is reserved for non-additive shape changes (rename, type change, removal). Open a new issue when that need arises. | [persist-signals-blocked-serialization spec](../plans/done/2026-05-22-persist-signals-blocked-serialization.spec.md) | — |
| `(a, b, c) → (a, c)` projection adapters for signal connections — separate issue; not in scope for arity relaxation | [signal-arity-relaxation spec](../plans/done/2026-05-25-signal-arity-relaxation.spec.md) | — |
| Return-type checking for slots — slots are fire-and-forget today; `invoke_method` return value is discarded | [signal-arity-relaxation spec](../plans/done/2026-05-25-signal-arity-relaxation.spec.md) | — |

## Out of scope

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| `BlockingQueued` — threading model not yet decided (already deferred in core-types spec) | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | | #48 |
| Signal-to-signal connections — blocked on runtime design (already deferred) | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | | #49 (closed) |
| Changes to `Signal::emit` itself (tracked in #38) | [signals-blocked spec](../plans/done/2026-05-02-signals-blocked.spec.md) | | #38 (closed) |
| Serialization of `signals_blocked` state (tracked in #39) | [signals-blocked spec](../plans/done/2026-05-02-signals-blocked.spec.md) | | #39 (closed) |
| `connect_<signal>_direct` or any other connection type wrappers | [connect-queued-codegen spec](../plans/done/2026-05-03-connect-queued-codegen.spec.md) |  | #246 |
| Changes to runtime or core crates | [connect-queued-codegen spec](../plans/done/2026-05-03-connect-queued-codegen.spec.md) |  | untracked |
| `ConnectionTable` changes — guard check is local to `Signal::AutoSlotInner::dispatch`. | [receiver-guard-auto spec](../plans/done/2026-05-03-receiver-guard-auto.spec.md) |  | untracked |
| Wiring `Signal` directly to `ObjectBase` (coupling two independent types) | [signal-emit-checked spec](../plans/done/2026-05-03-signal-emit-checked.spec.md) |  | untracked |
| Backward-compat alias for old `emit` name (project not yet on crates.io) | [signal-emit-checked spec](../plans/done/2026-05-03-signal-emit-checked.spec.md) |  | untracked |
| Changing any other `Signal` methods (connect, disconnect, etc.) | [signal-emit-rename spec](../plans/done/2026-05-05-signal-emit-rename.spec.md) |  | untracked |
| Changing the generated per-signal `emit_<name>` wrappers' public signatures (those already hide the `blocked` parameter) | [signal-emit-rename spec](../plans/done/2026-05-05-signal-emit-rename.spec.md) |  | untracked |
| Any changes to `emit_checked` (separate method, independent concern) | [signal-emit-rename spec](../plans/done/2026-05-05-signal-emit-rename.spec.md) |  | untracked |
| Alternate macro form for standalone `Signal` with no owning object (tests keep using `sig.emit(&args)` directly — unconditional is fine there) | [emit-macro spec](../plans/done/2026-05-06-emit-macro.spec.md) |  | #247 |
| Proc-macro variant of `emit!` | [emit-macro spec](../plans/done/2026-05-06-emit-macro.spec.md) |  | #248 |
| Making `Timer` use `emit!` (its `tick` is `Arc<Mutex<Signal>>`, not a bare field on an `AsObject`) | [emit-macro spec](../plans/done/2026-05-06-emit-macro.spec.md) |  | untracked |
| Serialization of signal-to-signal connections. | [signal-to-signal spec](../plans/done/2026-05-06-signal-to-signal.spec.md) |  | #249 |

## Open questions

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Should `Signal` be `Send + Sync`? (Needed if objects move across threads) | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | ✅ done | |
| Should `Auto` on a same-thread emit respect `signals_blocked` the same way as `Direct`? (Almost certainly yes — confirm when implementing `signals_blocked` logic) | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | ✅ done | |
| Should the `SlotEntry` for `Auto` store both a `Fn(&Args)` (direct path) and a `Fn(Args) + Send + Sync` (queued path), or a single `Fn(Args) + Send + Sync` called directly (with clone) on the same-thread path? Design doc to decide. | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | ✅ done | |
| **`signals_blocked` interaction with `Auto`:** Deferred to the `signals_blocked` design. | [auto-connection design](../plans/done/2026-05-01-auto-connection.design.md) | ✅ done | |
