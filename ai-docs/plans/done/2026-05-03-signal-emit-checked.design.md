# Design: Signal emit_unless_blocked / emit split

**Issue:** #38
**Date:** 2026-05-03

## Approach

Keep `Signal::emit` as the unconditional primitive (safe fn, no suffix), then add
`Signal::emit_unless_blocked(blocked: bool, args: &Args)` that returns early when `blocked == true` and
delegates to `emit` otherwise.

Both generated call sites — `emit_<signal>` wrappers and the `write_property` notify path — are
updated to call `emit_unless_blocked(base.signals_blocked(), ...)` directly, removing their external
`if !signals_blocked()` guard.

**`emit` is the unsuffixed unconditional primitive**
`_unchecked` is reserved for `unsafe` fns per AGENTS.md naming rules. The plain `emit` name is
the safe default. `emit_unless_blocked` is the descriptive name for the blocked-aware variant —
it says exactly what it does without co-opting `_checked` (which implies `Result`/`Option` return).

**Chosen over alternative: put the guard inside `emit_unless_blocked` at the call site vs. inside `Signal`**
The spec says the `blocked` bool is passed in from the caller. This is deliberate: `Signal` stays
decoupled from `ObjectBase`. The caller (generated wrapper or `write_property`) owns the
`signals_blocked()` query; `Signal` only sees a plain `bool`. This matches the spec's "Technical
constraints" note that `Signal` must stay `no_std` and independent.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Rename `Signal::emit` → `Signal::emit`; update its doc comment | `quartzite-core/src/signal.rs` | — |
| 2 | Add `Signal::emit_unless_blocked(blocked: bool, args: &Args)`; add doc comments for both methods | `quartzite-core/src/signal.rs` | 1 |
| 3 | Update all direct `sig.emit(…)` call sites in non-macro code that bypass the object-level guard | `quartzite-runtime/src/timer.rs`, `examples/signals_slots.rs`, `quartzite-macros/tests/object.rs` | 1 |
| 4 | Update codegen: `emit_signal_wrappers` — replace external `if !signals_blocked` guard + `self.#field.emit(…)` with `self.#field.emit_unless_blocked(…, &(…))` | `quartzite-macros/src/object/codegen.rs` | 2 |
| 5 | Update codegen: `emit_write_property` notify path — replace external `if !signals_blocked` guard + `this.#sig_ident.emit(…)` with `this.#sig_ident.emit_unless_blocked(…, &(…))` | `quartzite-macros/src/object/codegen.rs` | 2 |
| 6 | Update all `sig.emit(…)` calls in `quartzite-core/src/signal.rs` tests to `sig.emit(…)` | `quartzite-core/src/signal.rs` | 1 |
| 7 | Update integration test (`quartzite-core/tests/auto_no_dispatcher.rs`) `sig.emit(…)` → `sig.emit(…)` | `quartzite-core/tests/auto_no_dispatcher.rs` | 1 |
| 8 | Add `emit_unless_blocked` unit tests: fires when unblocked, suppressed when blocked | `quartzite-core/src/signal.rs` | 2 |
| 9 | Update codegen tests to assert `emit_unless_blocked` (not `signals_blocked` guard) in generated output | `quartzite-macros/src/object/codegen.rs` | 4, 5 |

Tasks 4 and 5 can be done together in one edit of `codegen.rs`.
Tasks 6 and 7 are mechanical renames that can be batched.

## Risks

- **All existing `.emit(…)` call sites must be found and renamed:** Missing any call site causes a
  compile error, which is the desired outcome — the rename is intentional and the compiler enforces
  completeness. Mitigation: `cargo build` after step 1 surfaces every remaining site.
- **`timer.rs` calls `emit` directly (bypasses signal-blocking):** This is intentional.
  The timer's `timeout` signal is a raw `Arc<Mutex<Signal<()>>>` with no owning `ObjectBase` in
  scope; there is no `signals_blocked` flag to query. Calling `emit` here is semantically
  correct and documents the bypass explicitly. No change in behaviour required.
- **`examples/signals_slots.rs` calls `emit` directly on the signal field:** This is a raw call
  that already bypasses the `emit_count_changed` wrapper (intentional in the example). Rename to
  `emit`. No semantic change.
- **`quartzite-macros/tests/object.rs` line 82 calls `c.count_changed.emit(&(7,))`:** This is an
  integration test verifying raw signal delivery independent of blocking. Rename to
  `emit`; the test intent is preserved.
- **Generated doc comment on `emit_<signal>` wrappers currently mentions `signals_blocked`:**
  The doc comment must be updated to reflect that the guard is now inside `emit_unless_blocked`, but the
  observable behaviour is identical. Update the doc string in `emit_signal_wrappers`.
- **`#[inline]` requirement:** `emit` has the same body as the current `emit` — not
  `#[inline]` eligible (has loops/branches). `emit_unless_blocked` is a simple two-branch function that
  either returns early or calls one function; add `#[inline]` to `emit_unless_blocked`.

## Test Design

### Task 8 — new `emit_unless_blocked` unit tests in `quartzite-core/src/signal.rs`

Location: `quartzite-core/src/signal.rs` `#[cfg(test)]` module

**Scenario 1 — suppressed when blocked:**
- Entry point: `Signal::emit_unless_blocked(true, args)`
- Setup: connect one Direct slot that sets an `AtomicBool`; call `emit_unless_blocked(true, &(42,))`
- Assert: slot not called (`AtomicBool` still `false`)

**Scenario 2 — fires when unblocked:**
- Entry point: `Signal::emit_unless_blocked(false, args)`
- Setup: connect one Direct slot; call `emit_unless_blocked(false, &(42,))`
- Assert: slot called with correct args

**Scenario 3 — `emit_unless_blocked(false, …)` is equivalent to `emit` for SingleShot:**
- Setup: connect one `SingleShot` slot; call `emit_unless_blocked(false, …)` twice
- Assert: slot fired exactly once (retain still works through `emit`)

### Task 9 — update codegen tests in `quartzite-macros/src/object/codegen.rs`

**`emit_wrappers_generated_for_signal` (existing test):**
- Remove assertion `out.contains("signals_blocked")` (external guard gone)
- Add assertion `out.contains("emit_unless_blocked")` in its place
- The test `emit_wrappers_no_signals_no_block` is unaffected

**`write_property_notify_guarded_by_signals_blocked` (existing test):**
- Rename to `write_property_notify_uses_emit_unless_blocked`
- Remove assertion `out.contains("signals_blocked")`
- Add assertion `out.contains("emit_unless_blocked")` and assert `!out.contains("signals_blocked")`

**`write_property_no_notify_no_guard` (existing test):**
- Keep as-is; still valid (no `emit_unless_blocked` call when no notify)

## Open questions

- none
