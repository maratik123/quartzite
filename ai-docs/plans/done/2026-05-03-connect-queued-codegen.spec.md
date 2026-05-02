# connect_<signal>_queued codegen

**Source:** issue #66
**Date:** 2026-05-03
**Tracked in:** #66

## Scope

1. Add `emit_connect_queued_wrappers` function in `quartzite-macros/src/object/codegen.rs`
2. Generate `connect_<signal>_queued(&mut self, receiver: &ObjectBase, f: F) -> ConnectionId` per signal
   - Internally calls `Arc::downgrade(receiver.receiver_guard())`; delegates to `Signal::connect_queued`
   - Gated `#[cfg(feature = "std")]`, `#[inline]`
   - `#[allow(unexpected_cfgs)]` on the outer impl block (same pattern as `connect_auto` wrappers)
3. Tests: generated method present, `receiver_guard` in output, wrapper lives outside hidden mod, absent when no signals

## Out of scope

- `connect_<signal>_direct` or any other connection type wrappers
- Changes to runtime or core crates

## Deferred

- none

## Key decisions

| Question | Decision |
|---|---|
| What to emit when struct has no signals? | Return `quote! {}` — omit the entire impl block, same as `emit_connect_auto_wrappers` |

## Technical constraints

- Mirror `emit_connect_auto_wrappers` exactly in structure; reuse all patterns from it
- `#[allow(unexpected_cfgs)]` on the generated `impl` block (not the fn) — same as auto wrappers
- Doc comment on each generated method; `# Examples` block required (use `no_run`)

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `#[derive(Object)]` on a struct with signals generates a `connect_<signal>_queued` method for each signal |
| AC2 | The generated method accepts `&mut self`, `receiver: &ObjectBase`, and a closure `f: F`; returns `ConnectionId` |
| AC3 | The generated method internally calls `Arc::downgrade(receiver.receiver_guard())` and delegates to `Signal::connect_queued` |
| AC4 | The generated method is gated `#[cfg(feature = "std")]` and carries `#[inline]` |
| AC5 | The generated `impl` block carries `#[allow(unexpected_cfgs)]` |
| AC6 | The generated method lives outside the `#[doc(hidden)]` mod (same as `connect_auto` wrappers) |
| AC7 | `#[derive(Object)]` on a struct with no signals emits no `connect_queued` impl block |
| AC8 | Each generated method has a `///` doc comment and a `# Examples` (`no_run`) block |

## Open questions

- none
