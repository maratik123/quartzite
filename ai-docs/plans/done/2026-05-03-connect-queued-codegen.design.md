# Design: connect_<signal>_queued codegen

**Issue:** #66
**Date:** 2026-05-03

## Approach

Add `emit_connect_queued_wrappers` in `quartzite-macros/src/object/codegen.rs` as a direct mirror of
`emit_connect_auto_wrappers`, substituting the `connect_queued` call site and dropping the
`thread_id` argument (which `connect_queued` does not take).

Signature difference between the two runtime methods:

| Method | Args |
|---|---|
| `Signal::connect_auto` | `receiver_thread_id, guard, f` |
| `Signal::connect_queued` | `f, guard` |

The generated `connect_<signal>_queued` method therefore omits `receiver.thread_id` from the
delegation call and places `f` before `guard`, matching `Signal::connect_queued`'s parameter order.

Everything else is identical to the `connect_auto` path:
- `#[cfg(feature = "std")]` + `#[cfg_attr(docsrs, doc(cfg(feature = "std")))]`
- `#[allow(unexpected_cfgs)]` on the outer `impl` block
- `#[inline]` on each generated method
- Early-return `quote! {}` when signals slice is empty
- Lives outside the `#[doc(hidden)]` mod in the same position in `quote! { … }` as `connect_auto_wrappers`

### Rejected alternatives

**Shared helper / macro over both generators** — premature abstraction; there are only two
connection-type wrappers and their shapes are nearly identical single-function bodies. YAGNI.

**Merging queued and auto into one impl block** — would complicate the layout test that finds
`rfind("impl Foo")` to verify the outer position. Keep them separate, same as the spec says to
mirror the existing pattern exactly.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 0 | Update `connect_auto_wrapper_lives_outside_hidden_mod` — replace `rfind`-based single-suffix check with `match_indices`-based multi-impl positioning (collect all `"impl Foo"` positions, assert `connect_ticked_auto` appears between `positions[1]` and `positions[2]`) | `quartzite-macros/src/object/codegen.rs` (`#[cfg(test)]`) | — |
| 1 | Add `emit_connect_queued_wrappers` function | `quartzite-macros/src/object/codegen.rs` | 0 |
| 2 | Wire function into `codegen()` top-level and `quote!` output | `quartzite-macros/src/object/codegen.rs` | 1 |
| 3 | Write unit tests for the new codegen path | `quartzite-macros/src/object/codegen.rs` (`#[cfg(test)]`) | 2 |

All four tasks touch one file; tasks 1 and 2 are logically separate (function body vs. wiring) but
can be implemented in a single commit.

## Risks

- **Parameter order mismatch** (`f, guard` vs `guard, f`): `Signal::connect_queued` takes `f`
  first, then `guard` — the opposite mental model from `connect_auto`. The generated call must be
  `self.#field.connect_queued(f, ::std::sync::Arc::downgrade(receiver.receiver_guard()))`.
  Mitigation: unit test explicitly asserts `connect_queued` is present in output and that
  `receiver_guard` is referenced.
- **`Args: Clone + Send` bound**: `connect_queued` requires these bounds (same as `connect_auto`);
  the existing `where F: Fn(Args) + Send + Sync + 'static` clause is unchanged — the runtime
  enforces `Clone + Send` on `Args` independently, no macro-side change needed.
- **`#[allow(unexpected_cfgs)]` placement**: must be on the `impl` block, not on the individual
  `fn`. Mirrors existing `emit_connect_auto_wrappers`. Mitigation: test asserts the attribute.
- **No new public API surface in `quartzite-core`**: the feature is purely codegen; zero risk of
  breaking core.

## Test Design

**Location:** `quartzite-macros/src/object/codegen.rs` — `#[cfg(test)] mod tests` (existing module)

All tests use the existing `emit(ts: TokenStream) -> String` helper and `assert!(out.contains(…))` /
`assert!(!out.contains(…))` assertions — matching the style of the `connect_auto_wrapper_*` tests
immediately above.

### Test: `connect_queued_wrapper_generated_for_signal`

- **Entry point:** `codegen()` via `emit()`
- **Input:** struct with one `#[signal] pub value_changed: Signal<(i32,)>`
- **Scenarios (happy path):**
  - `out.contains("pub fn connect_value_changed_queued")` — method emitted
  - `out.contains("cfg (feature = \"std\")")` — feature gate present
  - `out.contains("# [inline] pub fn connect_value_changed_queued")` — `#[inline]` present
  - `out.contains("receiver : & :: quartzite :: core :: ObjectBase")` — receiver param
  - `out.contains("connect_queued")` — delegates to `connect_queued`
  - `out.contains("receiver . receiver_guard ()")` — guard extracted from receiver
  - `out.contains("allow (unexpected_cfgs)")` — attribute on impl block

### Test: `connect_queued_wrapper_lives_outside_hidden_mod`

- **Entry point:** `codegen()` via `emit()`
- **Input:** struct with `#[signal] pub ticked: Signal<(i32,)>`
- **Scenarios (position check):**
  - Locate `mod __quartzite_Foo` start position
  - Use `rfind("impl Foo")` to find the last impl block
  - Assert `connect_ticked_queued` does not appear in the slice from mod start to last impl
  - Assert `connect_ticked_queued` appears in the slice from last impl to end

### Test: `connect_queued_wrapper_absent_with_no_signals`

- **Entry point:** `codegen()` via `emit()`
- **Input:** struct with only a `#[prop]` field, no signals
- **Scenario (negative):**
  - `!out.contains("connect_queued")` — no wrapper emitted

## Open questions

- none
