# Signal::emit rename — make blocked-aware emit the ergonomic default

**Source:** user description
**Date:** 2026-05-05
**Tracked in:** #93

## Scope

1. Rename `pub fn emit_unless_blocked(&mut self, blocked: bool, args: &Args)` → `pub fn emit` on `Signal<Args>` in `quartzite-core`
2. Inline the old `emit` body directly into the new `emit`'s `if !blocked { … }` branch; remove the standalone raw `emit` fn entirely
3. Update all `sig.emit(args)` call sites in tests → `sig.emit(false, args)`
4. Update `emit_signal_wrappers` codegen in `quartzite-macros` to call `emit` instead of `emit_unless_blocked`
5. Fix `quartzite-runtime/src/timer.rs` to pass the Timer's `signals_blocked()` state to `emit` — **blocked on #36** (Timer needs a proper `ObjectBase` before the flag is cleanly accessible in the thread closure)

## Out of scope

- Changing any other `Signal` methods (connect, disconnect, etc.)
- Changing the generated per-signal `emit_<name>` wrappers' public signatures (those already hide the `blocked` parameter)
- Any changes to `emit_checked` (separate method, independent concern)

## Deferred

- Timer fix (`timer.rs:167`) | blocked on #36 — Timer needs a proper `ObjectBase` so `signals_blocked()` is accessible inside the thread closure | tracked in #93, depends on #36

## Key decisions

| Question | Decision |
|---|---|
| What happens to the old raw `emit` body? | Inlined into new `emit`'s `if !blocked { … }` branch — no internal helper function |
| Should old `emit` be kept as `pub(crate)`? | No — inlining removes the need entirely |
| Timer.rs interim approach | Blocked on #36; do not ship a `emit(false, &())` workaround |

## Technical constraints

- `quartzite-runtime` is a different crate from `quartzite-core`; `pub(crate)` on any internal emit helper would not be visible to `timer.rs` — hence the block on #36
- The generated `emit_signal_wrappers` in `quartzite-macros` call `emit_unless_blocked`; these must be updated in the same PR
- All existing tests use bare `sig.emit(args)` on `Signal` directly; they must become `sig.emit(false, args)` after the rename

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Signal::emit(&mut self, blocked: bool, args: &Args)` is the sole public emit entry point; `emit_unless_blocked` no longer exists |
| AC2 | When `blocked` is `true`, no connected slot is invoked |
| AC3 | When `blocked` is `false`, all connected slots are invoked with the correct args |
| AC4 | `SingleShot`, `Queued`, and `Auto` connection types behave identically to before the rename |
| AC5 | The macro-generated `emit_<signal>` wrappers compile and behave correctly after the codegen update |
| AC6 | `cargo doc` produces no warnings about missing or broken links related to the renamed method |

## Open questions

- None — timer.rs fix explicitly deferred to #36.
