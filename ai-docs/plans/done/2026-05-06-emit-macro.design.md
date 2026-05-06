# Design: emit! macro — ergonomic blocked-aware signal emission

**Issue:** #93
**Date:** 2026-05-06

## Approach

### Chosen solution

Make `Signal::emit` unconditional — it takes `&mut self` and `args: &Args`, invokes
all connected slots with no guard. The blocked-signal check moves exclusively into:

1. A `macro_rules! emit` in `quartzite-core` that does the check at the call site.
2. Explicit `if !blocked { … }` guards in the two `Timer` paths that cannot use the
   macro (`emit_tick` takes `&self`; the driver closure captures `Arc<TimerState>`).

The macro pattern:

```text
emit!($r:ident . $f:ident, $args:expr)
expands to:
{
    let __blocked = $crate::AsObject::object_base(&$r).signals_blocked();
    if !__blocked {
        $r.$f.emit($args);
    }
}
```

`let __blocked` releases the immutable borrow of `self` (via `AsObject::object_base`)
before the `&mut self.$f` borrow needed by `Signal::emit` is taken. This is the only
pattern that satisfies the borrow checker without interior mutability or splitting borrows.

The macro is placed in `quartzite-core/src/signal.rs` with `#[macro_export]` so it
lives at `quartzite_core::emit`. The `quartzite` prelude re-exports it.

The `quartzite-macros` codegen (`emit_signal_wrappers` and `emit_write_property`)
replaces each `.emit(signals_blocked(), …)` call with `#cr::emit!(self.#field, &(…))`.
Because `#cr` expands to `::quartzite::core` or `::quartzite_core`, the macro
invocation in generated token streams is `#cr :: emit ! (self . #field , &(#(#arg_idents ,)*))`.

### Why not inline the guard in codegen instead of using the macro?

The macro makes the pattern visible and testable in isolation, gives users a single
canonical form for their own signal fields not covered by `#[derive(Object)]`, and
keeps the codegen smaller. No YAGNI concern: the macro is small and directly
motivated by AC2/AC3/AC5.

### Rejected alternative: keep `blocked: bool` on `Signal::emit`

Rejected because `Signal` is a generic data structure with no knowledge of the owning
object; threading `blocked` through it couples signal semantics to the object model.
Moving the guard into a macro at the caller's level is the correct separation.

### Rejected alternative: proc-macro variant of `emit!`

Out of scope per the spec. `macro_rules!` is sufficient for the single-level
`receiver.field` pattern.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Remove `blocked: bool` from `Signal::emit`; emit unconditionally; update all doc-comments and tests that call `sig.emit(bool, args)` | `quartzite-core/src/signal.rs` | — |
| 2 | Define `macro_rules! emit` in `quartzite-core`; re-export from `quartzite` prelude | `quartzite-core/src/signal.rs`, `src/lib.rs` | 1 |
| 3 | Update `emit_signal_wrappers` codegen to emit `#cr::emit!(self.#field, &(args))` | `quartzite-macros/src/object/codegen.rs` | 2 |
| 4 | Update `emit_write_property` codegen (notify path) to emit `#cr::emit!(self.#sig_ident, &(__notify_val,))` | `quartzite-macros/src/object/codegen.rs` | 2 |
| 5 | Update `Timer::emit_tick` and the driver closure in `Timer::start` to use explicit `if !signals_blocked { … }` guards | `quartzite-runtime/src/timer.rs` | 1 |
| 6 | Update all call sites using the old `sig.emit(blocked, args)` signature in tests, examples, and integration tests | `quartzite-core/src/signal.rs` tests, `quartzite-macros/tests/`, `quartzite-runtime/src/timer.rs` tests | 1 |
| 7 | Verify `cargo build -p quartzite --no-default-features` compiles (no_std path) | — | 2, 3, 4, 5 |

### Notes on task dependencies

Tasks 3 and 4 are independent and can be done in a single editing pass (same file).
Task 6 overlaps with tasks 1 and 5 — update tests in the same edit that changes the
production code rather than treating it as a separate pass.

## Risks

- **Borrow-checker constraint in macro body:** `AsObject::object_base(&$r).signals_blocked()` borrows `$r` immutably. If `$r.$f.emit(…)` is reached in the same statement the compiler sees overlapping borrows. Mitigated by the `let __blocked = …;` binding — it ends the immutable borrow before the `if !__blocked { $r.$f.emit($args); }` line, where only a mutable borrow of `$r.$f` is needed. This pattern is the entire reason for the `let` binding; must not be collapsed into a one-liner.
- **Codegen token-stream quoting:** `emit!` is a macro invocation token, not a function call. In `quote!`, it must be written as `#cr :: emit ! (self . #field , &(#(#arg_idents ,)*))`. A common mistake is writing it as a function call path, which would fail to compile. The codegen tests catch this because they check the string representation of the token stream.
- **no_std path:** `#[macro_export]` macros are always unconditionally exported regardless of feature flags. The macro body uses only `$crate::AsObject` and `Signal::emit` — both available in `no_std`. No issue.
- **Existing test call sites:** All tests in `signal.rs` currently call `sig.emit(false, &args)`. After task 1 the signature changes to `sig.emit(&args)`. Failing to update them produces compile errors, not silent behavior changes. The failing build makes the omission obvious.
- **`Timer` tests that call `state.signal.lock().emit(false, &(41,))`:** This test calls `emit` directly on the `Arc<Mutex<Signal>>` path (task 6 scope). The new signature drops the `bool` argument; must update the test too.
- **`#[macro_export]` pollutes the top-level namespace of `quartzite_core`:** Accepted — the macro is intentionally public API. It will appear as `quartzite_core::emit` after export. The prelude re-export aliases it in the `quartzite` facade. Name collision with a potential future `fn emit` is not a concern because `macro_rules!` and function names occupy different namespaces.

## Test Design

### Task 1 — `Signal::emit` unconditional

- **Location:** `quartzite-core/src/signal.rs` `#[cfg(test)]`
- **Entry point:** `Signal::emit`
- **Scenarios:**
  - Existing tests that previously passed `false` as first arg: update to omit the arg; behavior unchanged.
  - Existing tests that passed `true` (suppression tests `emit_suppressed_when_blocked`, `emit_fires_when_not_blocked`): these tests verified the guard *inside* `emit`; after the change the guard is gone from `emit`. **Remove** these two tests from `signal.rs` — the guard semantics are now tested at the macro level (task 2).
  - `emit_with_no_slots_does_not_panic` and all other direct-call tests: update call sites only.

### Task 2 — `emit!` macro

- **Location:** `quartzite-core/src/signal.rs` `#[cfg(test)]` (new test block for the macro)
- **Entry point:** `emit!` macro
- **Scenarios (matching AC2, AC3):**
  - `emit_macro_suppressed_when_signals_blocked`: construct a minimal `AsObject` implementor, block signals, call `emit!`, assert slot not invoked.
  - `emit_macro_fires_when_not_blocked`: same object unblocked, assert slot invoked.
  - `emit_macro_releases_borrow`: compile-time test — if the borrow splitting is wrong the test does not compile; no runtime assertion needed beyond the macro compiling successfully.
- **Fixtures:** A minimal inline struct implementing `AsObject` (similar to `DummyObject` in `traits.rs`) with one `Signal` field.

### Task 3/4 — Codegen tests

- **Location:** `quartzite-macros/src/object/codegen.rs` `#[cfg(test)]`
- **Entry point:** `emit_signal_wrappers`, `emit_write_property`
- **Scenarios:**
  - Rename existing test `emit_wrappers_generated_for_signal` assertion: change `signals_blocked` check to verify `emit !` appears in the token stream and `signals_blocked` does NOT appear (guard moved out of the generated code into the macro).
  - Update `write_property_notify_uses_emit` similarly.
  - Add `emit_wrappers_use_emit_macro`: verify the output string contains `emit !` (the macro call token) rather than `.emit (signals_blocked ()`.
  - Add `write_property_notify_uses_emit_macro`: same for the notify path.

### Task 5 — Timer explicit guards

- **Location:** `quartzite-runtime/src/timer.rs` `#[cfg(test)]`
- **Scenarios:**
  - Existing tests `emit_tick_suppressed_when_blocked`, `emit_tick_fires_when_unblocked`, `block_unblock_restores_emission`, `timer_state_signal_shared_with_tick`: update call sites that call `emit(bool, …)` directly; behavioral coverage is preserved.
  - No new tests needed — the explicit guard is trivial and already tested indirectly.

### Task 6 — Integration / doc tests

- **Location:** `quartzite-macros/tests/object.rs`, any `examples/`
- **Scenarios:** Update each `sig.emit(false, …)` or `sig.emit(true, …)` call site to match new signature. No new test logic.

## Open questions

- None.
