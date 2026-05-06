# emit! macro — ergonomic blocked-aware signal emission

**Source:** user description
**Date:** 2026-05-06
**Tracked in:** #93

## Scope

1. Remove `blocked: bool` from `Signal::emit` — revert to unconditional `emit(&mut self, args: &Args)`
2. Add `macro_rules! emit` in `quartzite-core`: `emit!(receiver.signal_field, &args)` — binds `__blocked` via `let` to release the immutable borrow, then calls `receiver.signal_field.emit(&args)` inside `if !__blocked { … }`
3. Re-export `emit!` from the `quartzite` prelude
4. Update `emit_signal_wrappers` and `emit_write_property` codegen in `quartzite-macros` to use `emit!` macro instead of `.emit(signals_blocked(), …)`
5. Update `timer.rs`: `emit_tick` and the driver closure revert to an explicit `if !signals_blocked { … }` guard — the macro does not apply to `Arc<Mutex<Signal>>` fields
6. All changes land in the existing branch `feat/2026-05-05-signal-emit-rename` / PR #100 — no new branch or PR

## Out of scope

- Alternate macro form for standalone `Signal` with no owning object (tests keep using `sig.emit(&args)` directly — unconditional is fine there)
- Proc-macro variant of `emit!`
- Making `Timer` use `emit!` (its `tick` is `Arc<Mutex<Signal>>`, not a bare field on an `AsObject`)

## Deferred

- None.

## Key decisions

| Question | Decision |
|---|---|
| Should `Signal::emit` stay unconditional or keep `blocked`? | Unconditional — `emit!` macro owns the guard; `Signal::emit` is a pure emitter |
| Macro syntax | `emit!(receiver.signal_field, &args)` — single-level `$r:ident . $f:ident` only |
| How to avoid the borrow conflict in the macro? | `let __blocked = AsObject::object_base(&receiver).signals_blocked();` before the `if` — immutable borrow released before `&mut receiver.field` is taken |
| Should codegen use `emit!` or inline the equivalent? | Use `emit!` macro directly in generated code |
| Receiver depth | Single level only (`self.signal`); deeper paths not needed |
| Export | `quartzite_core` (defined) + `quartzite` prelude (re-exported) |

## Technical constraints

- `macro_rules!` in `quartzite-core` must reference `AsObject` via `$crate::AsObject` so the macro works without an explicit `use` by the caller
- The generated `emit_<signal>` wrappers in `quartzite-macros` emit token streams, not Rust source — the codegen must emit `emit!` as a macro call token sequence referencing `#cr::emit`
- `Timer::emit_tick` takes `&self` (not `&mut self`) so it cannot call the macro directly on its own signal field; it retains the explicit guard
- `no_std`: the macro must compile on the no_std path — `AsObject` and `Signal` are both available there; `#[macro_export]` + `$crate::` paths are no_std-compatible

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Signal::emit(&mut self, args: &Args)` takes no `blocked` parameter; it unconditionally invokes all connected slots |
| AC2 | `emit!(self.my_signal, &args)` on an `AsObject` implementor suppresses slot invocations when `signals_blocked()` is `true` |
| AC3 | `emit!(self.my_signal, &args)` invokes all connected slots when `signals_blocked()` is `false` |
| AC4 | `emit!` is accessible via `use quartzite::prelude::*` |
| AC5 | The macro-generated `emit_<signal>` wrappers use `emit!` and behave correctly (blocked suppresses, unblocked fires) |
| AC6 | `sig.emit(&args)` on a standalone `Signal` with no owning object fires unconditionally (tests and `Arc<Mutex<Signal>>` paths) |
| AC7 | `cargo doc` produces no warnings related to the macro or `Signal::emit` |

## Open questions

- None.
