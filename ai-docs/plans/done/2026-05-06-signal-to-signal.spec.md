# Signal-to-signal connections

**Source:** issue #49
**Date:** 2026-05-06
**Tracked in:** #49

## Scope

1. Add `fn emit_signal(&mut self, signal: &str, args: &[Value]) -> Option<()>` to the `Object` trait — dynamic emit-by-name; `None` if the signal name is unknown.
2. `quartzite-macros` codegen implements `emit_signal` for all `#[derive(Object)]` types by matching the signal name and converting `&[Value]` → typed args before calling `Signal::emit`.
3. Free function `connect_signal_to_signal` (dynamic path) in `quartzite-core` (`std` feature): `(from: &mut dyn Object, from_signal: &str, to: Arc<Mutex<dyn Object + Send>>, to_signal: &str, conn_type: ConnectionType) -> Result<ConnectionId, SignalConnectionError>`.
4. Typed API (exact form settled in design — a free generic function or `macro_rules!`) for connecting typed signal fields directly, also taking an explicit `ConnectionType` and `Arc<Mutex<T>>` for the target.
5. Arity and type-name validation at connection time: compare `SignalMeta::params` length and each `ParamMeta::type_name` string → `Err(SignalConnectionError)` on mismatch.
6. Liveness: forwarding callback holds a `Weak` reference to `to` so the connection silently breaks when `to` is dropped (all strong `Arc` holders released).
7. Returns `ConnectionId`; disconnect via the same `Signal::disconnect(id)` used for regular slot connections.
8. Re-export through the `quartzite` facade prelude.
9. Integration tests in `quartzite-runtime` or `quartzite/tests`: same-thread Direct chain, cross-thread Auto chain (A→B→C verifying chains > 2 work naturally), disconnect mid-chain.

## Out of scope

- `BlockingQueued` connection type (blocked on #48).
- Serialization of signal-to-signal connections.

## Deferred

- None.

## Key decisions

| Question | Decision |
|---|---|
| Does `Object` trait need `emit_signal`? | Yes — required for the dynamic forwarding callback to invoke the target signal via `dyn Object` |
| Ownership of `to_obj` in the callback | `Arc<Mutex<dyn Object + Send>>` (dynamic path); typed path also uses `Arc<Mutex<T>>` |
| Dynamic vs typed API | Both: dynamic string-based function + typed generic/macro API |
| Connection type parameter | Explicit `ConnectionType` on both APIs |
| Type compatibility check | Compare `SignalMeta::params` length and `ParamMeta::type_name` strings at connection time |
| Liveness mechanism | Callback holds `Weak<Arc<Mutex<...>>>` (or `Weak<Mutex<...>>`); silently skips if upgrade returns `None` |
| Chains > 2 objects | Work naturally — no special handling needed |

## Technical constraints

- `connect_signal_to_signal` and `Object::emit_signal` are `std`-only; guard with `#[cfg(feature = "std")]`.
- The forwarding callback must be `Fn + Send + Sync + 'static` — `Arc<Mutex<...>>` is the only ownership model that satisfies this.
- `Signal::emit` takes `&mut self`; the callback must lock `to` before calling `emit_signal`.
- For `ConnectionType::Auto`: the forwarding closure must determine at emit time whether the emitting thread matches `to`'s owner thread, posting to `QueuedDispatcher` if not. `quartzite-core` already exposes `get_queued_dispatcher()` for this.
- `Object::emit_signal` codegen must handle arity mismatch gracefully (return `None`) — the `&[Value]` slice may have the wrong length if called incorrectly.
- `emit_signal` methods inside `impl Trait for Type` blocks are exempt from `# Parameters` and `# Examples` doc requirements (trait definition documents the contract).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Object::emit_signal(signal, args)` returns `Some(())` and fires all connected slots when `signal` exists and `args` arity matches |
| AC2 | `Object::emit_signal(signal, args)` returns `None` when `signal` name is unknown |
| AC3 | `connect_signal_to_signal` returns `Ok(ConnectionId)` when both signal names exist and their `params` arity and all `type_name` strings match |
| AC4 | `connect_signal_to_signal` returns `Err(SignalConnectionError)` when signal names are unknown, arity differs, or any `type_name` pair mismatches |
| AC5 | Emitting `from_signal` fires `to_signal` on the target object (Direct connection, same thread) |
| AC6 | Emitting `from_signal` fires `to_signal` on the target object (Auto connection, cross-thread — posted to event loop) |
| AC7 | After the last strong `Arc` to `to` is released, subsequent `from_signal` emissions silently skip the forwarding callback (no panic, no error) |
| AC8 | `disconnect(id)` on the returned `ConnectionId` stops forwarding; subsequent `from_signal` emissions do not invoke `to_signal` |
| AC9 | A three-object chain (A→B→C) works without special configuration: emitting A's signal propagates to C |
| AC10 | The typed API connects two typed signals and satisfies AC5–AC8 without string names |
| AC11 | `connect_signal_to_signal` and the typed API are accessible via `use quartzite::prelude::*` |

## Open questions

- None.
