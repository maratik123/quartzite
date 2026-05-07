# Design: Codegen re-emit `#[inline]` on concrete-struct trait-impl methods

**Issue:** #131
**Date:** 2026-05-07

## Approach

Emit `#[inline]` on generated trait-impl methods when and only when the user
struct / enum has no generic parameters (`generics.params.is_empty()`).
The pattern pre-decided in the spec is:

```rust
let inline = if generics.params.is_empty() {
    quote! { #[inline] }
} else {
    quote! {}
};
```

Each affected `quote!` block inserts `#inline` immediately before the `fn`
keyword of each simple trait-impl method.

**Why this approach is correct.** Cross-crate inlining without LTO only works
when the method body exists in the caller's codegen unit.  For a concrete-impl
`impl Trait for Foo` the compiler emits one symbol; `#[inline]` makes the body
available across crates.  For `impl<T> Trait for Foo<T>` the method is already
monomorphised at the call site, so `#[inline]` on the trait-impl method buys
nothing — the `// _Simple._` comment approach is blocked because `quote!`
strips comments before the token stream is parsed.

**Rejected alternatives.**

- Filter to type/const params only (`params.iter().any(|p| matches!(p, Type|Const))`) — rejected per spec; lifetime-only structs are absent in this codebase and the conservative `is_empty()` check is simpler.
- Unconditional `#[inline]` on all generated impls — rejected; over-emits for generic impls where the attribute has no cross-crate effect and causes clippy to warn in future editions.
- Conditional `#[inline]` based on `where_clause` inspection — rejected; where-clause predicates do not affect per-impl symbol count.

**Plumbing gaps found during investigation.**

The spec states that `generics` is already threaded through all affected codegen
functions; this is true for `extend/codegen.rs` but not for the other two files:

- `object_impl/codegen.rs`: `ObjectImplInput` carries no `generics` field; the
  `ItemImpl` parsed in `object_impl/parse.rs` does have `.generics` and it must
  be stored in the IR and forwarded to `emit_object_impl`.
- `meta_enum/codegen.rs`: `MetaEnumInput` carries no `generics` field; the
  `DeriveInput` parsed in `meta_enum/parse.rs` does have `.generics` and it
  must be stored in the IR and forwarded to `codegen`.

Both parse files need one-line additions to capture and store `syn::Generics`
before the design can proceed to codegen edits.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `generics: syn::Generics` to `ObjectImplInput`; store `item.generics` in `parse()`; forward to `emit_object_impl` in `codegen()`; update `emit_object_impl` signature | `quartzite-macros/src/object_impl/parse.rs`, `quartzite-macros/src/object_impl/codegen.rs` | — |
| 2 | Add `generics: syn::Generics` to `MetaEnumInput`; store `derive.generics` in `parse()`; make `generics` available inside the `codegen()` fn | `quartzite-macros/src/meta_enum/parse.rs`, `quartzite-macros/src/meta_enum/codegen.rs` | — |
| 3 | `extend/codegen.rs` — add the `inline` branch in `emit_root_trait_and_impl`, `emit_as_object_impl`, `emit_delegation_impl`, `emit_mixin_impl`; insert `#inline` before each simple-method `fn` in the emitted `impl` blocks | `quartzite-macros/src/extend/codegen.rs` | — |
| 4 | `object_impl/codegen.rs` — add the `inline` branch in `emit_object_impl` for `meta_object`, `connect_signal`, `emit_signal`; leave `read_property`, `write_property`, `invoke_method` unchanged; `__meta_init_<Foo>` stays unconditional `#[inline]` | `quartzite-macros/src/object_impl/codegen.rs` | 1 |
| 5 | `meta_enum/codegen.rs` — add the `inline` branch for `IntoValue::into_value`; leave `FromValue::from_value` unchanged | `quartzite-macros/src/meta_enum/codegen.rs` | 2 |
| 6 | Split the existing `no-inline` assertions in all three test modules: one concrete-struct scenario (asserts `# [inline]` present) and one generic-struct scenario (asserts `# [inline]` absent) per affected function; adjust the `object_impl` test that counts exactly 1 `# [inline]` | `quartzite-macros/src/extend/codegen.rs`, `quartzite-macros/src/object_impl/codegen.rs`, `quartzite-macros/src/meta_enum/codegen.rs` | 3, 4, 5 |
| 7 | CI gate: `cargo build && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check && cargo test && cargo build -p quartzite --no-default-features && RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` | — | 6 |

## Risks

- **`emit_root_trait_and_impl` generates both a trait definition and a self-ref impl block.** The trait body carries method declarations (no `#[inline]` there — the rule applies to impls only), while the impl block carries method bodies. The `inline` token must be inserted only in the `impl #self_trait for #self_ident { … }` block, not in the `pub trait #self_trait { … }` block. Mitigation: use a single `inline` variable in `emit_root_trait_and_impl` and insert it only in the impl block; root structs always have empty generics (enforced by `extend/parse.rs` line 69), so `inline` will always be `#[inline]` there — the branch degenerates to a constant but is kept for consistency.
- **`object_impl` generics source is the impl block, not the struct.** The impl block's `item.generics` reflects what the programmer wrote on the `impl<…>` line, which mirrors the struct's generics. If a programmer writes `impl Foo { … }` for a generic struct (unusual but valid if all type params have defaults), the impl-block generics would be empty while the struct is generic. This edge case is irrelevant in this codebase (no such pattern exists); conservative `is_empty()` is still correct because a concrete `impl Foo` really is monomorphic.
- **`meta_enum` generics: `MetaEnum` is only applied to C-like enums.** Generic enums are not prohibited by `syn` but are rejected by `meta_enum/parse.rs` only if they have non-unit variants; a generic unit-only enum could in principle slip through. However, the only semantic caller in the project is `#[derive(MetaEnum)]` on enums representing Qt-style integer enums, which are never generic. The `is_empty()` guard is still correct and harmless for this hypothetical case.
- **Existing test `object_trait_methods_have_no_inline_meta_init_has_one` counts exactly 1 `# [inline]` occurrence.** After task 4, the concrete-struct input in that test will count 4 occurrences (`meta_object`, `connect_signal`, `emit_signal`, plus `__meta_init_Foo`). The test must be rewritten to use two scenarios (concrete vs generic) and updated count expectations. If the test is not updated in task 6 the CI gate in task 7 will catch the failure before merge.
- **`emit_root_trait_and_impl` has no `generics` parameter today** — it takes `&ExtendInput` directly and uses `ir.generics` transitively through callers, but does not call `bare_generics`. The inline branch uses `ir.generics.params.is_empty()` which is correct since root structs always have empty params (enforced by the parse layer).

## Test Design

### Task 3 — `extend/codegen.rs` tests

Location: `quartzite-macros/src/extend/codegen.rs` `#[cfg(test)]` module.

The four existing `*_have_no_inline` tests are each split into two:

**`self_ref_accessors_*`** (covers `emit_root_trait_and_impl`):
- Concrete input (`struct Widget { x: i32 }`, no type params on the root) — assert `# [inline]` present on the impl block's accessor methods.
- Generic input — not applicable for root structs (root structs with generics are rejected at parse time); this scenario is N/A for `emit_root_trait_and_impl`.
  - Supplement with a non-root (delegation) concrete vs generic test instead.

**`as_object_impl_methods_*`** (covers `emit_as_object_impl`):
- Concrete input (`struct Widget { #[base] object_base: ObjectBase }`) — assert `# [inline]` present.
- Generic non-root with base (`struct Foo<T> { #[base] widget: Widget }`) — assert `# [inline]` absent.

**`delegation_methods_*`** (covers `emit_delegation_impl`):
- Concrete input (`struct Button { #[base] widget: Widget }`) — assert `# [inline]` present.
- Generic input (`struct Foo<T> { #[base] widget: Widget }`) — assert `# [inline]` absent.

**`mixin_accessors_*`** (covers `emit_mixin_impl`):
- Concrete input (`struct Panel { #[mixin] layout_base: LayoutBase }`) — assert `# [inline]` present.
- Generic input (`struct Panel<T> { #[mixin] layout_base: LayoutBase, _ph: ::core::marker::PhantomData<T> }`) — assert `# [inline]` absent.

### Task 4 — `object_impl/codegen.rs` tests

Location: `quartzite-macros/src/object_impl/codegen.rs` `#[cfg(test)]` module.

The helper `emit(ts)` passes `quote!{}` as attr and the token stream as the impl body to `parse::parse`, then calls `codegen`. To pass generics, the impl block itself carries the generic params: `impl<T> Foo<T> { … }`.

**`object_trait_methods_have_no_inline_meta_init_has_one`** — rewrite into two tests:
- `concrete_object_impl_methods_inline` with `impl Foo {}` — assert `# [inline]` count == 4 (meta_object, connect_signal, emit_signal, meta_init_Foo).
- `generic_object_impl_methods_no_inline` with `impl<T> Foo<T> {}` — assert `# [inline]` count == 1 (only meta_init_Foo which is unconditional).

**`meta_init_fn_is_inline`** — keep as-is (still true for both concrete and generic; unconditional).

New scenario for `read_property`, `write_property`, `invoke_method` — assert those specific fn lines do NOT carry `# [inline]` in either concrete or generic inputs (they were already excluded per spec AC2 and the existing test coverage covers the aggregate count).

### Task 5 — `meta_enum/codegen.rs` tests

Location: `quartzite-macros/src/meta_enum/codegen.rs` `#[cfg(test)]` module.

**`into_value_and_from_value_have_no_inline`** — rewrite into two tests:
- `concrete_enum_into_value_has_inline` with `enum Color { Red, Green }` (no generics) — assert `# [inline]` present exactly once and that it precedes `fn into_value`.
- `generic_enum_into_value_has_no_inline` — if generic enums cannot be constructed through the parser (the parser only rejects non-unit variants, not generic params), use `enum Foo<T> { … }` with a unit variant — assert `# [inline]` absent. If the parser rejects generic enums at the parse stage, annotate the test as documenting that case; otherwise keep the scenario.
  - Note: `syn::DeriveInput` will parse `enum Foo<T> { A }` without error; `meta_enum/parse.rs` does not check for generic params today. This is a valid test input.

Fixtures: `emit(ts)` already exists in the test module; no new helpers needed.

## Open questions

None — spec has no open questions and all pre-decided answers are reflected above.
