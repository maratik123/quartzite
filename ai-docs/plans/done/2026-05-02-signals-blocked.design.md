# Design: signals-blocked — typed emit wrappers with signals_blocked guard

**Issue:** #34
**Date:** 2026-05-02

## Approach

### Chosen solution

Two mostly-independent changes with one shared mechanism:

1. **`ObjectBase` gets a `signals_blocked: bool` field** with three `#[inline]` methods —
   `block_signals(&mut self)`, `unblock_signals(&mut self)`, `signals_blocked(&self) -> bool`.
   The field defaults to `false` (unblocked), matching Qt's convention.

2. **`#[derive(Object)]` codegen emits an `impl TypeName { pub fn emit_<signal>(...) }` block**
   (one wrapper per `#[signal]` field) in the outer scope — outside the hidden `__quartzite_<TypeName>`
   module. Each wrapper:
   - Has `#[inline]` (it contains exactly one branch and one call — qualifies per AGENTS.md rule).
   - Checks `self.object_base().signals_blocked()` via the `AsObject` trait already required by every
     `#[derive(Object)]` user; if true, returns immediately.
   - Otherwise calls `self.<signal_field>.emit(&(arg0, arg1, ...))`.
   - Arguments are flattened: a `Signal<(i32, bool)>` produces
     `pub fn emit_moved(&mut self, arg0: i32, arg1: bool)`.

3. **`emit_write_property` wraps notify emit in the same guard**: before calling
   `this.<signal>.emit(&...)`, the generated code checks `::quartzite::core::AsObject::object_base(this).signals_blocked()`.
   If true, the emit is skipped (the property value is still written).

### Why this shape

- **No new parse IR**: the `SignalField` already records `ident` and `args_ty`. The flattening of
  tuple args into individual parameters reuses the existing `tuple_elems` helper.
- **Placement of `emit_<signal>` in outer scope** (not inside the hidden module): these are public
  user-facing methods that need `pub` visibility and must be addressable as `obj.emit_value_changed(v)`.
  Putting them in the hidden module would require re-exporting or glob-importing, which is noise.
- **`AsObject::object_base()` rather than a direct field reference**: the generated `emit_<signal>`
  methods and the write_property function both work for root types and derived types equally — they
  only need the `AsObject` bound that `#[derive(Object)]` already depends on.
- **The property value is always written, guard only suppresses the notify signal**: consistent with
  Qt semantics — `signals_blocked()` blocks emission, not mutation. If the caller wants to block
  writes too, that is a separate concern.

### Rejected alternatives

- **Guard inside `Signal::emit` itself** (issue #38): deferred per spec — would require API redesign
  and a reference back to `ObjectBase` inside `Signal`.
- **RAII guard type (`SignalBlocker`)**: unnecessary complexity for the scope of this issue.
  A pair of methods is sufficient and simpler to use in tests.
- **Storing the flag in a separate wrapping type**: over-engineering; the flag is fundamentally
  per-object state, `ObjectBase` is the correct home.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `signals_blocked: bool` field to `ObjectBase`; add `block_signals`, `unblock_signals`, `signals_blocked` methods with `#[inline]`, doc comments, and `# Examples` blocks; update `ObjectBase::new` to initialise the field to `false` | `quartzite-core/src/object_base.rs` | — |
| 2 | Add unit tests for the three new `ObjectBase` methods to the existing `#[cfg(test)] mod tests` | `quartzite-core/src/object_base.rs` | 1 |
| 3 | Add `emit_emit_wrappers` codegen function in `quartzite-macros/src/object/codegen.rs`; wire it into `codegen()` to emit `impl TypeName { pub fn emit_<signal>(...) }` outside the hidden module | `quartzite-macros/src/object/codegen.rs` | 1 |
| 4 | Add codegen unit tests for `emit_emit_wrappers` inside `codegen.rs` `#[cfg(test)]` block | `quartzite-macros/src/object/codegen.rs` | 3 |
| 5 | Update `emit_write_property` to wrap the notify `emit` call in a `signals_blocked()` guard | `quartzite-macros/src/object/codegen.rs` | 1 |
| 6 | Add codegen unit test verifying the guard appears in `write_property` output | `quartzite-macros/src/object/codegen.rs` | 5 |
| 7 | Add integration tests to `quartzite-macros/tests/object.rs` covering all four scenarios (AC3, AC4, AC5 from spec) | `quartzite-macros/tests/object.rs` | 3, 5 |

Seven tasks — at the boundary; they are all tightly coupled to one feature so no split is warranted.

## Risks

- **`#[inline]` on `emit_<signal>` wrappers**: each wrapper has one branch and one function call
  (the `emit`). The AGENTS.md rule says "at most one function call, no binary bloat" — one branch
  (`if`) plus one call still qualifies under the spirit of the rule (it is a trivial guard pattern).
  Apply `#[inline]` and let clippy/CI validate.
- **No-std path**: `signals_blocked: bool` has no std dependency. The three new methods are
  unconditional (no `#[cfg(feature = "std")]` needed). `cargo build -p quartzite --no-default-features`
  must be run before committing.
- **Breaking API**: adding a private field to `ObjectBase` changes its layout. Because `ObjectBase`
  is constructed directly in user code (`ObjectBase::new()`, `ObjectBase::named(...)`, and struct
  literals in tests), test files that use struct-literal construction will continue to compile only
  because the new field is **private** — the borrow-checker prevents struct-literal construction of
  types with private fields from outside their defining module. All existing test construction
  already goes through `ObjectBase::new()` or `ObjectBase::named(...)`, so this is safe.
- **`signals_blocked` not `#[doc(hidden)]`**: it is a private field, invisible to the public API.
  The three public methods are part of the public API and must have doc comments + `# Examples`.
- **`emit_<signal>` naming collision**: if a user manually defines `emit_foo` on their type, the
  macro-generated impl will collide. This is acceptable for now (same risk exists for `write_foo`
  in future property accessors). The spec does not ask for collision detection.

## Test Design

### Task 2 — `ObjectBase` unit tests (`quartzite-core/src/object_base.rs`)

Location: existing `#[cfg(test)] mod tests` at the bottom of `object_base.rs`.

Entry points: `block_signals`, `unblock_signals`, `signals_blocked`.

Scenarios:
- `signals_blocked_false_by_default` — `ObjectBase::new().signals_blocked()` is `false`.
- `block_signals_sets_flag` — call `block_signals()`, assert `signals_blocked()` is `true`.
- `unblock_signals_clears_flag` — call `block_signals()` then `unblock_signals()`, assert `false`.
- `unblock_when_not_blocked_is_noop` — call `unblock_signals()` on a fresh base, assert still `false`.

Fixtures: none (all tests construct `ObjectBase::new()` inline).

### Task 4 — Codegen unit tests for `emit_emit_wrappers` (`quartzite-macros/src/object/codegen.rs`)

Location: existing `#[cfg(test)] mod tests` block.

Entry point: `emit` helper (parses + runs `codegen`).

Scenarios:
- `emit_wrappers_generated_for_signal` — struct with one `#[signal]`, assert output contains
  `pub fn emit_value_changed` and `signals_blocked`.
- `emit_wrappers_no_signals_no_block` — struct with no `#[signal]` fields, assert no
  `emit_` function in output.
- `emit_wrappers_multi_arg_parameters_flattened` — `Signal<(i32, bool)>`, assert `arg0 : i32`
  and `arg1 : bool` in output.
- `emit_wrappers_single_unit_arg` — `Signal<()>` (zero-element tuple), assert `emit_foo` has no
  parameters beyond `&mut self`.
- `emit_wrappers_inline_attribute_present` — assert `# [inline]` appears before the wrapper.
- `emit_wrappers_live_outside_hidden_mod` — assert the `impl TypeName` wrapper block is NOT inside
  `mod __quartzite_TypeName` (check relative position in the emitted string, or check the hidden
  mod does not contain `emit_`).

### Task 6 — Codegen unit test for `write_property` guard

Location: existing `#[cfg(test)] mod tests` block.

Entry point: `emit` helper.

Scenarios:
- `write_property_notify_guarded_by_signals_blocked` — struct with `#[prop(notify = changed)]` and
  matching `#[signal]`, assert output contains `signals_blocked` in the write arm alongside
  `changed . emit`.
- `write_property_no_notify_no_guard` — struct with bare `#[prop]` (no notify), assert `signals_blocked`
  does NOT appear in the write arm (so we don't add unnecessary checks).

### Task 7 — Integration tests (`quartzite-macros/tests/object.rs`)

Location: `quartzite-macros/tests/object.rs` — new test functions appended to the file.

Setup: a new `Counter` variant or extend the existing one. The existing `Counter` struct already has
`count_changed: Signal<(i32,)>` — it is sufficient.

Scenarios:

| Test name | Scenario | AC |
|-----------|----------|----|
| `emit_wrapper_suppressed_when_blocked` | Call `block_signals()` on the object; call `emit_count_changed(42)` (generated wrapper); assert connected slot was NOT called | AC3 |
| `emit_wrapper_delivers_when_unblocked` | Normal (unblocked) state; call `emit_count_changed(7)`; assert slot received `7` | AC4 |
| `write_property_notify_suppressed_when_blocked` | Call `block_signals()`; call `write_property("count", Value::Int(99))`; assert count updated to 99 but notify slot was NOT called | AC5 |
| `write_property_notify_fires_when_unblocked` | Normal state; `write_property` updates count and fires notify | regression guard for existing AC5 |
| `unblock_restores_emit_wrapper` | Block, unblock, then call `emit_count_changed`; assert slot IS called | AC3+AC1 combined |

Fixtures: inline struct construction (same pattern as existing tests) using `Arc<Mutex<...>>`
for the slot capture. The `AsObject` trait methods `block_signals` / `unblock_signals` are
called as `c.object_base_mut().block_signals()`.

## Open questions

- None — all key decisions are recorded in the spec.
