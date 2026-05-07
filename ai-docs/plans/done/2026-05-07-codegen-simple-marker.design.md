# Design: Codegen — drop `#[inline]` from generated trait-impl methods

**Issue:** #117 (Part 2)
**Date:** 2026-05-07

## Approach

Generated trait-impl methods (`impl AsObject for Foo { fn object_base ... }`,
`impl Object for Foo { fn meta_object ... }`, `impl IntoValue for Foo { fn into_value ... }`,
etc.) are in trait-impl position. Per the marker decision tree in
`ai-docs/code-style.md`, that position requires `// _Simple._`; `#[inline]` is
reserved for concrete free fns and concrete inherent methods.

Option (c) from the issue: emit **no marker** on generated trait-impl methods.
The trait declaration carries `/// _Simple._`; generated impls satisfy the contract
by construction. `// _Simple._` cannot be emitted from `quote!` because it is not
a syntactic token, so this is also the only option that avoids fighting the macro
machinery.

The one exception is `__meta_init_<Foo>()` in `object_impl/codegen.rs`: it is a
**concrete free fn** (not a trait-impl method), so `#[inline]` on it is correct
and must be preserved.

**Rejected alternatives:**

- Option (a) — emit `// _Simple._` via `quote!`: not syntactically possible.
- Option (b) — emit `/// _Simple._` on each generated impl method: pollutes user
  docs with a maintenance-convention annotation that is internal to the project;
  also the trait-decl `/// _Simple._` already covers the contract.

**Additional fix identified during investigation:** `IntoValue::into_value` in
`quartzite-core/src/value.rs` does not yet carry `/// _Simple._` on its trait
declaration. PR #120 added the tag to `AsObject` and `Object` methods but missed
`IntoValue`. This must be added in the same PR so that dropping `#[inline]` from
the codegen does not leave the method unmarked at both levels.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `/// _Simple._` to `IntoValue::into_value` trait declaration | `quartzite-core/src/value.rs` | — |
| 2 | Drop `#[inline]` from self-ref accessor pair in `emit_root_trait_and_impl` | `quartzite-macros/src/extend/codegen.rs` | — |
| 3 | Drop `#[inline]` from all 4 methods in `emit_as_object_impl` | `quartzite-macros/src/extend/codegen.rs` | — |
| 4 | Drop `#[inline]` from parent accessor pair in `emit_delegation_impl` | `quartzite-macros/src/extend/codegen.rs` | — |
| 5 | Drop `#[inline]` from mixin accessor pair in `emit_mixin_impl` | `quartzite-macros/src/extend/codegen.rs` | — |
| 6 | Drop `#[inline]` from all 6 Object trait-impl methods in `emit_object_impl` (`meta_object`, `read_property`, `write_property`, `invoke_method`, `connect_signal`, `emit_signal`); leave `emit_meta_static` unchanged so `__meta_init_<Foo>()` retains `#[inline]` | `quartzite-macros/src/object_impl/codegen.rs` | — |
| 7 | Drop `#[inline]` from `IntoValue::into_value` in `codegen` fn | `quartzite-macros/src/meta_enum/codegen.rs` | 1 |
| 8 | Update existing AC9 codegen tests in `extend/codegen.rs` — invert assertions so they verify absence of `#[inline]` on all generated trait-impl methods | `quartzite-macros/src/extend/codegen.rs` | 2, 3, 4, 5 |
| 9 | Update existing AC9 codegen tests in `object_impl/codegen.rs` — update `object_impl_shims_are_inline` to assert count == 1 (only `__meta_init_Foo`); keep `meta_init_fn_is_inline` as-is | `quartzite-macros/src/object_impl/codegen.rs` | 6 |
| 10 | Update existing AC9 codegen test in `meta_enum/codegen.rs` — update `into_value_is_inline_from_value_is_not` to assert count == 0; add assertion that `# [inline]` is absent from the output | `quartzite-macros/src/meta_enum/codegen.rs` | 7 |

## Risks

- **Existing tests assert `#[inline]` is present**: four existing AC9 tests in
  `extend/codegen.rs` and two in `object_impl/codegen.rs` will fail immediately
  after the production changes. They must be updated in tasks 8–10 before the CI
  batch runs. Execute production change + test update atomically per file.
- **`IntoValue::into_value` missing `/// _Simple._`**: dropping `#[inline]` from
  the generated impl without adding it to the trait declaration would leave the
  method with no marker at either level. Task 1 must land before task 7.
- **`__meta_init_<Foo>()` must not lose `#[inline]`**: it is a concrete free fn,
  not a trait-impl method. `meta_init_fn_is_inline` test in `object_impl/codegen.rs`
  guards this. Do not touch `emit_meta_static`.
- **No clippy / doc regression**: `#[inline]` removal cannot introduce new clippy
  warnings. The `// _Simple._` comment form is not emitted so there is no doc-gate
  impact.
- **no_std path**: the three codegen files do not emit `std`-specific code; the
  change is annotation-only and cannot affect `cargo build -p quartzite --no-default-features`.

## Test Design

### Tasks 2–5 / Task 8 — `extend/codegen.rs`

- Location: `quartzite-macros/src/extend/codegen.rs` `#[cfg(test)] mod tests`
- Existing tests to UPDATE (invert assertions):
  - `self_ref_accessors_are_inline` — change `assert!(out.contains("# [inline]"))` to
    `assert!(!out.contains("# [inline]"), "unexpected #[inline] on self-ref accessors: {out}")`
  - `as_object_impl_methods_are_inline` — change to assert `!out.contains("# [inline]")`
    and rename to `as_object_impl_methods_have_no_inline`
  - `delegation_methods_are_inline` — invert to `!out.contains("# [inline]")`
  - `mixin_accessors_are_inline` — invert to `!out.contains("# [inline]")`
- Scenarios:
  - Root no-base: self-ref accessor pair emits no `# [inline]`
  - Root with ObjectBase: all 4 AsObject impl methods emit no `# [inline]`
  - Child with base: parent delegation pair emits no `# [inline]`
  - Mixin-only: mixin accessor pair emits no `# [inline]`

### Task 6 / Task 9 — `object_impl/codegen.rs`

- Location: `quartzite-macros/src/object_impl/codegen.rs` `#[cfg(test)] mod tests`
- Existing tests to UPDATE:
  - `object_impl_shims_are_inline` (line 579): change assertion from `count >= 6` to
    `count == 1` with message "expected exactly 1 #[inline] (only __meta_init_Foo)";
    rename to `object_trait_methods_have_no_inline_meta_init_has_one`
- Existing test to KEEP unchanged:
  - `meta_init_fn_is_inline` — verifies `# [inline] fn __meta_init_Foo`; no change needed
- New assertion to add inside the updated `object_impl_shims_are_inline`:
  - `assert!(!out.contains("fn meta_object") || ...)` is too indirect; simpler:
    assert that `out.matches("# [inline]").count() == 1` and that the one occurrence
    is adjacent to `__meta_init_Foo` (already covered by `meta_init_fn_is_inline`)

### Task 7 / Task 10 — `meta_enum/codegen.rs`

- Location: `quartzite-macros/src/meta_enum/codegen.rs` `#[cfg(test)] mod tests`
- Existing test to UPDATE:
  - `into_value_is_inline_from_value_is_not` (line 253): change `count == 1` to
    `count == 0`; replace `out.contains("# [inline] fn into_value")` assertion with
    `assert!(!out.contains("# [inline]"), "unexpected #[inline] in output: {out}")`
- Scenarios:
  - Any `MetaEnum` input: zero `# [inline]` tokens in output
  - `from_value` (non-simple body with branches): still no `# [inline]` — already covered
    by count == 0

### Task 1 — `quartzite-core/src/value.rs`

- No new test needed: annotation-only change to the trait declaration. The existing
  doc-gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`)
  will catch any malformed doc comment. The codegen test update in task 10 provides
  the behavioral guard.

## Open questions

- None. The `IntoValue::into_value` gap (missing `/// _Simple._` on the trait
  declaration) was identified during investigation and is incorporated as task 1.
  The spec's claim that it was covered by PR #120 was incorrect; investigation
  confirms the tag is absent from `quartzite-core/src/value.rs` line 282.
