# Signal emit_checked / emit_unchecked split

**Source:** issue #38
**Date:** 2026-05-03
**Tracked in:** #38

## Scope

1. Restore `Signal::emit` as the unconditional primitive (raw, fires unconditionally)
2. Add `Signal::emit_unless_blocked(blocked: bool, args: &Args)` — returns early without firing when `blocked == true`; calls `emit` otherwise
3. Update generated `emit_<signal>` wrappers to call `self.field.emit_unless_blocked(base.signals_blocked(), &args)` and remove the external `if !signals_blocked()` guard
4. Update generated `write_property` notify path to use `emit_unless_blocked` the same way
5. Add doc comments clarifying the contract of both methods
6. Add tests for `emit_unless_blocked` (fires when unblocked, suppressed when blocked)

## Out of scope

- Wiring `Signal` directly to `ObjectBase` (coupling two independent types)
- Backward-compat alias for old `emit` name (project not yet on crates.io)

## Deferred

- none

## Key decisions

| Question | Decision |
|---|---|
| Return type of `emit_checked` | `()` — consistent with `emit_unchecked`; callers don't need a "did it fire" signal |
| Name for raw method | `emit` — the safe, unconditional primitive; `_unchecked` is reserved for `unsafe` fns per project naming rules |
| Where checked logic lives | Inside `Signal::emit_checked`, not in generated wrapper — keeps codegen clean |

## Technical constraints

- `quartzite-core` is `no_std + alloc`; no std-only APIs in `Signal`
- Generated code in `quartzite-macros` must use the `::quartzite::core::` path prefix as before

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Signal::emit` exists and fires all connected slots unconditionally |
| AC2 | `Signal::emit_unless_blocked` exists; `Signal::emit_unless_blocked(true, args)` does not invoke any connected slot |
| AC3 | `Signal::emit_unless_blocked(false, args)` invokes all connected slots |
| AC4 | Generated `emit_<signal>` wrappers call `emit_unless_blocked` (no external `if !signals_blocked` guard in generated output) |
| AC5 | Generated `write_property` notify path calls `emit_unless_blocked` (no external `if !signals_blocked` guard) |

## Open questions

- none
