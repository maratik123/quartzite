# Codegen: re-emit `#[inline]` on concrete-struct trait-impl method emissions

**Source:** issue #131
**Date:** 2026-05-07
**Tracked in:** #131

## Scope

1. `quartzite-macros/src/extend/codegen.rs` — add genericity branch in `emit_root_trait_and_impl`, `emit_as_object_impl`, `emit_delegation_impl`, `emit_mixin_impl`: emit `#[inline]` when the user struct has empty `generics.params`, nothing otherwise
2. `quartzite-macros/src/object_impl/codegen.rs` — `emit_object_impl`: same branch for `meta_object`, `connect_signal`, `emit_signal`; `__meta_init_<Foo>()` stays unconditional `#[inline]`
3. `quartzite-macros/src/meta_enum/codegen.rs` — `codegen` fn: same branch for `IntoValue::into_value`
4. Genericity check: `generics.params.is_empty()` — conservative (lifetime-only structs → no `#[inline]`, acceptable; flag for reviewer if it surfaces)
5. Tests: split existing PR #129 assertions into two scenarios per fn — concrete-struct input → `#[inline]` present; generic-struct input → `#[inline]` absent

## Out of scope

- Hand-written `impl Trait for Type` outside `quartzite-macros` (rule is in AGENTS.md; reviewer audit covers it)
- `// _Simple._` codegen (Rust strips comments from token streams before parsing)
- `/// _Simple._` on trait declarations (already emitted by PRs #120/#127)
- `where_clause` predicate inspection (bounds don't affect per-impl symbol count)
- Non-simple methods: `read_property`, `write_property`, `invoke_method`, `from_value`, `__lookup_*`, `__connect_signal_dynamic_*`

## Deferred

None.

## Key decisions

| Question | Decision |
|---|---|
| `generics.params.is_empty()` vs filtering to type/const params only | `params.is_empty()` — conservative; lifetime-only structs are absent in this codebase today |
| Check `where_clause` predicates? | No — only type/const params affect monomorphization count |
| Test style | Token-stream-contains, matching PR #129 test style |
| Update existing PR #129 tests? | Yes — split each assertion into concrete-struct and generic-struct scenarios |

## Technical constraints

- `quote!` macro cannot emit `// _Simple._` comments — only `#[inline]` attr applies here
- The `generics` field (`&syn::Generics`) is already threaded through all affected codegen functions
- Genericity branch pattern:
  ```rust
  let inline = if generics.params.is_empty() {
      quote! { #[inline] }
  } else {
      quote! {}
  };
  ```

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `extend/codegen.rs` emits `#[inline]` on each generated trait-impl method when the input user struct has empty `generics.params`; emits no `#[inline]` when `generics.params` is non-empty. Applies to `emit_root_trait_and_impl`, `emit_as_object_impl`, `emit_delegation_impl`, `emit_mixin_impl`. |
| AC2 | `object_impl/codegen.rs` `emit_object_impl` follows the same branch for `meta_object`, `connect_signal`, `emit_signal`. `read_property` / `write_property` / `invoke_method` continue to emit no marker. `__meta_init_<Foo>()` continues to emit `#[inline]` unconditionally. |
| AC3 | `meta_enum/codegen.rs` `codegen` follows the same branch for `IntoValue::into_value`. `from_value` continues to emit no marker. |
| AC4 | Existing PR #129 tests (asserting `#[inline]` absent) are split into two scenarios per fn: one with a concrete struct input asserting `#[inline]` present, one with a generic struct input asserting `#[inline]` absent. |
| AC5 | `cargo build && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check && cargo test` clean. |
| AC6 | `cargo build -p quartzite --no-default-features` clean. |
| AC7 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` clean. |
| AC8 | Spot-check rendered HTML for a `#[derive(Extend, Object)]` example: trait-method docs (e.g. `AsObject::object_base`) still appear with the inherited summary AND the `Simple.` italic line from the trait declaration on both concrete and generic user structs. |

## Open questions

None.
