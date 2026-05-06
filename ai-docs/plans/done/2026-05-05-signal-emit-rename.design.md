# Design: Signal::emit rename — make blocked-aware emit the ergonomic default

**Issue:** #93
**Date:** 2026-05-06

## Approach

The current API has two public emit methods on `Signal<Args>`:

- `emit(&mut self, args: &Args)` — unconditional; does not check `signals_blocked`.
- `emit_unless_blocked(&mut self, blocked: bool, args: &Args)` — guarded; the method all object code should use.

The naming creates a friction inversion: the safe, idiomatic call (`emit_unless_blocked`) has a longer, uglier name, while the more dangerous, low-level call (`emit`) has the short name. The rename corrects this by:

1. Replacing `pub fn emit(&mut self, args: &Args)` with `pub fn emit(&mut self, blocked: bool, args: &Args)` — the new signature subsumes `emit_unless_blocked`.
2. Inlining the old `emit` body directly into the `if !blocked { … }` branch of the new `emit`. No intermediate private helper is introduced.
3. Removing `emit_unless_blocked` entirely (no deprecation alias — API not yet published).
4. Updating all `sig.emit(args)` call sites to `sig.emit(false, args)` (tests, integration tests, examples).
5. Updating `quartzite-macros` codegen to call `emit` instead of `emit_unless_blocked`.
6. Fixing `timer.rs`: removing the redundant outer `signals_blocked` guards and passing the flag directly to `emit` at each call site.

### timer.rs fix detail

Issue #36 is resolved. `Timer` now has `pub base: ObjectBase` and `TimerState::signals_blocked: AtomicBool`.

Two current patterns need updating:

- **`emit_tick` (line 383–388):** outer guard `if self.base.signals_blocked() { return; }` followed by `self.tick.lock().emit(&(fire_count,))`. Replace with `self.tick.lock().emit(self.base.signals_blocked(), &(fire_count,))` and remove the outer guard.
- **Driver callback closure (lines 498–501):** outer `if !state.signals_blocked.load(Ordering::Relaxed) { ... sig.emit(&(count,)); }` block. Replace with `sig.emit(state.signals_blocked.load(Ordering::Relaxed), &(count,))` and remove the outer guard.
- **Test helper (line 675):** `timer.state.signal.lock().emit(&(41,))` — replace with `timer.state.signal.lock().emit(false, &(41,))` (test accesses the signal directly, no blocking needed).

### Rejected alternatives

- **Keep old `emit` as `pub(crate)` internal helper**: not useful — `quartzite-runtime` is a different crate, so `pub(crate)` does not bridge the crate boundary. Inlining is cleaner.
- **Add `emit_raw` as the unconditional variant**: adds naming complexity with no benefit at this stage; no external caller needs the unconditional path directly.
- **Deprecate with `#[deprecated]`**: unnecessary — the crate is unpublished.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Rename `Signal::emit` to the new `emit(&mut self, blocked: bool, args: &Args)`; inline old emit body into `if !blocked { … }` branch; remove `emit_unless_blocked` fn; update doc comments, `# Examples` blocks, and intra-doc links in the same file; update `ConnectionType` doc examples at lines 31 and 44 (`sig.emit(&())` → `sig.emit(false, &())`) | `quartzite-core/src/signal.rs` | — |
| 2 | Update the three `emit_unless_blocked` tests in `signal.rs` to call `sig.emit(true/false, args)` and rename them to drop the `emit_unless_blocked` prefix | `quartzite-core/src/signal.rs` | 1 |
| 3 | Update all remaining `sig.emit(args)` call sites in `signal.rs` tests to `sig.emit(false, args)` | `quartzite-core/src/signal.rs` | 1 |
| 4 | Update macros codegen: in `emit_signal_wrappers`, change `self.#field.emit_unless_blocked(blocked, &(…))` to `self.#field.emit(blocked, &(…))`; in `emit_write_property`, change `this.#sig_ident.emit_unless_blocked(blocked, &(…))` to `this.#sig_ident.emit(blocked, &(…))` | `quartzite-macros/src/object/codegen.rs` | 1 |
| 5 | Update macros codegen tests: replace all occurrences of `"emit_unless_blocked"` assertion strings with `"emit"` (search for all occurrences rather than relying on a hard-coded count); rename affected tests | `quartzite-macros/src/object/codegen.rs` | 4 |
| 6 | Fix `timer.rs`: remove outer `signals_blocked` guards at lines 384–386 and 498–501; pass the flag directly to `emit` at each site; update test helper at line 675 to `emit(false, &(41,))` | `quartzite-runtime/src/timer.rs` | 1 |
| 7 | Update integration test and example call sites | `quartzite-core/tests/auto_no_dispatcher.rs`, `quartzite-macros/tests/object.rs`, `examples/signals_slots.rs` | 1 |

Tasks 2–7 depend only on task 1 and can otherwise be applied in any order.

## Risks

- **Broad call-site churn in tests:** Every bare `sig.emit(args)` in `quartzite-core/src/signal.rs` tests needs a `false` first argument. Missing one is a compile error — the compiler catches all omissions immediately; no silent breakage.
- **Macros codegen test string-matching:** Tests in `codegen.rs` assert on the rendered `TokenStream` string. After the rename the generated token sequence changes from `"emit_unless_blocked"` to `"emit"`. The assertions must be updated. Risk: `"emit"` is a substring of many tokens (`"emit_tick"`, `"emit_value_changed"`, etc.) — assertions must be specific enough to distinguish the method call. Use patterns like `". emit ("` or `"field . emit ("` and verify the `"signals_blocked"` check is still asserted where relevant.
- **`ConnectionType` doc examples:** Two doc examples on `ConnectionType` (lines 31 and 44 of `signal.rs`) call `sig.emit(&())`. These are compiled doctests and will fail to compile after task 1 unless updated. Included explicitly in task 1 scope.
- **No API backward-compat shim needed:** Project is unpublished; AGENTS.md explicitly forbids compat wrappers.

## Test Design

### Tasks 1–3 — `quartzite-core/src/signal.rs`

- **Location:** `quartzite-core/src/signal.rs` `#[cfg(test)] mod tests`
- **Entry point:** `Signal::emit`
- **Scenarios to update (not new tests):**
  - `emit_unless_blocked_suppressed_when_blocked` → rename to `emit_suppressed_when_blocked`; change `sig.emit_unless_blocked(true, &())` → `sig.emit(true, &())`
  - `emit_unless_blocked_fires_when_not_blocked` → rename to `emit_fires_when_not_blocked`; update call
  - `emit_unless_blocked_single_shot_fires_once_when_not_blocked` → rename; update calls
  - All other `sig.emit(&args)` tests → add `false` as first arg
- **No new test fixtures required** — the same `TestDispatcher` and helpers are reused.

### Tasks 4–5 — `quartzite-macros/src/object/codegen.rs`

- **Location:** `#[cfg(test)] mod tests` inside `codegen.rs`
- **Entry point:** `emit_signal_wrappers`, `emit_write_property` (via `emit()` helper in tests)
- **Scenarios to update:**
  - `write_property_notify_emits_signal_call` — change assertion from `"changed . emit_unless_blocked"` to the new `emit` token pattern
  - `write_property_notify_uses_emit_unless_blocked` — rename to `write_property_notify_uses_emit`; update all `"emit_unless_blocked"` string checks; keep `"signals_blocked"` assertion
  - `emit_wrappers_generated_for_signal` — change `"emit_unless_blocked"` assertion; keep `"signals_blocked"` assertion
  - All other tests asserting on `"emit_unless_blocked"` — update to `"emit"` (search for all occurrences rather than relying on a hard-coded count)

### Task 6 — `quartzite-runtime/src/timer.rs`

- **Location:** `quartzite-runtime/src/timer.rs` `#[cfg(test)] mod tests`
- **Entry point:** `emit_tick`, driver closure, test helper
- **Scenarios:**
  - Existing `emit_tick_suppressed_when_blocked` / `emit_tick_fires_when_unblocked` tests remain valid — they exercise `Timer::emit_tick` which will now pass `self.base.signals_blocked()` directly to `Signal::emit`. No new test scenarios are needed; the existing tests cover both the blocked and unblocked paths.
  - `timer_state_signal_shared_with_tick` test: update `timer.state.signal.lock().emit(&(41,))` → `timer.state.signal.lock().emit(false, &(41,))`.

### Task 7 — Integration tests and examples

- **`quartzite-core/tests/auto_no_dispatcher.rs`:** `sig.emit(&(99,))` → `sig.emit(false, &(99,))` — no behavioural change.
- **`quartzite-macros/tests/object.rs`:** `c.count_changed.emit(&(7,))` → `c.count_changed.emit(false, &(7,))` — no behavioural change.
- **`examples/signals_slots.rs`:** `g.greeted.emit(&(String::from("world"),))` → `g.greeted.emit(false, &(String::from("world"),))` — examples directory; no `#[cfg(test)]` block required per AGENTS.md.

## Open questions

- None.
