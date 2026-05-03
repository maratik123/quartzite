# Design: Macro Codegen Improvements

**Issue:** #57
**Date:** 2026-05-03

## Approach

### Feature 1 — `proc_macro_crate` path detection

All 65 occurrences of `::quartzite::core::` in macro-generated code are hardcoded as
`::quartzite::core::*`. This is wrong when the user's crate either renames `quartzite` or depends
directly on `quartzite-core` without the facade.

**Chosen approach: central `crate_root()` helper in `util.rs`**

Add `proc-macro-crate` as a dependency of `quartzite-macros`. Introduce a single function
`crate_root() -> TokenStream` in `util.rs` that returns the leading path fragment (a `TokenStream`)
to use as the prefix for all `::quartzite::core::*` references. The function:

1. Calls `crate_name("quartzite")`. On `FoundCrate::Itself`, prefix is `::quartzite::core`
   (absolute path using `CARGO_PKG_NAME` with `-` → `_`; `crate::` is rejected because
   `#[object_impl]` may expand inside example/binary targets where `crate` ≠ `quartzite`).
   On `FoundCrate::Name(n)`, prefix is `#n::core`.
2. If step 1 fails (crate not found), calls `crate_name("quartzite-core")`. On
   `FoundCrate::Itself`, prefix is `::quartzite_core` (same absolute-path rationale). On
   `FoundCrate::Name(n)`, prefix is `#n`.
3. If both fail, silently falls back to `::quartzite_core` (no compile error — AC3).

All four `codegen.rs` files and `util.rs` are updated to call `crate_root()` and use the returned
fragment wherever `::quartzite::core` is currently hardcoded. The `::core::` prefix (from libcore)
is unaffected — it stays as `::core::`.

The public `crate_root() -> TokenStream` function is a thin wrapper that calls the testable inner
helper `fn crate_root_from(facade: Option<FoundCrate>, core: Option<FoundCrate>, pkg_name: &str) -> TokenStream`.
Unit tests inject `Option<FoundCrate>` values directly into `crate_root_from`, covering all five
cases (facade `Itself`, facade `Name(n)`, core-only `Name(n)`, core `Itself`, neither) without
reading `Cargo.toml`.

The `crate_root()` call happens once per `codegen()` invocation (it reads `Cargo.toml` at
proc-macro expansion time, which is fine).

**Rejected: separate helper crate** — no value at this stage; extra complexity for zero benefit.
**Rejected: always resolving both names** — facade-first is the stated policy; once `quartzite`
is found there is no need to probe `quartzite-core`.

### Feature 2 — `#[object_impl]` on multiple impl blocks + trait impls

The core challenge is that proc-macro invocations are stateless between calls: the second
`#[object_impl]` on a type cannot see data emitted by the first.

**Rejected alternatives:**
- **`linkme`/`inventory` distributed slices** — shifts aggregation to runtime, adds a heavy
  dependency, and is not idiomatic for a purely compile-time metadata feature.
- **Terminal `#[object_meta]` without shared state** — any stateless terminal attribute still
  cannot see what earlier `#[object_impl]` calls emitted; the names of the method-list constants
  those calls would emit are unknown to the aggregator.
- **Unconditional `thread_local!` + single `#[object_impl]` final emit** — `#[object_impl]`
  cannot know in advance whether a later invocation will follow, so there is no safe point to
  emit the `MetaObject` without also emitting it prematurely on every earlier call (duplicate
  statics, duplicate `impl Object` — compile error).

**Chosen approach: two-mode `#[object_impl]` + optional `#[object_meta]`, with `thread_local!`
accumulation for the multi-block case.**

- **Single-block mode (existing, AC8):** plain `#[object_impl]` — no flag. Emits everything as
  today: cleaned impl block, `__METHODS__` static, invoke function, lookup functions, `MetaObject`
  static, `impl Object`. No shared state involved.

- **Multi-block mode (new):** user opts in explicitly:
  - `#[object_impl(partial)]` on every block except the last: emits only the cleaned impl block;
    accumulates this block's `MethodItem`s into a `thread_local!` cell keyed by the stringified
    self-type (e.g. `"Counter"`). No `MetaObject` or `impl Object` emitted.
  - `#[object_impl(final)]` on the terminal block, **or** a separate `#[object_meta]` applied to
    an empty `impl Counter {}` block placed after all partial blocks: reads the accumulated
    methods for the type from the `thread_local!` cell (type name taken from the impl block's
    self-type), merges them (no additional methods from the `#[object_meta]` block itself), emits
    the full output, then **drains the cell** to prevent leakage across
    crate compilations in the same rustc process.

The explicit `partial`/`final` annotation makes the user's intent visible in source and eliminates
any ambiguity about which invocation is terminal. `thread_local!` is safe here because proc-macro
expansion is single-threaded within a crate compilation unit.

**Trait impl handling (AC6):** the restriction in `parse.rs` (lines 29–33) is removed. Both
`partial` and `final` blocks accept trait impls. The self-type is extracted from `item.self_ty`
(the `Foo` in `impl Trait for Foo`) — the existing `extract_self_ty_ident` logic is correct as-is.

`ObjectImplInput` gains a `trait_path: Option<syn::Path>` field populated from
`item.trait_.as_ref().map(|(_, path, _)| path.clone())`. Codegen uses it when re-emitting the
cleaned impl block: if `trait_path.is_some()`, emit `impl #trait_path for #self_ty { … }`;
otherwise `impl #self_ty { … }`. This preserves the trait identity for `#[slot]`-stripped
methods that must still satisfy the trait contract.

**Duplicate detection (AC5):** when a `partial` block adds methods to the accumulator, each
method name is checked against names already in the cell. A duplicate triggers a `compile_error!`
token in the macro output at that method's span.

### Feature 3 — Generic struct support for `#[derive(Extend)`

`extend/parse.rs` currently rejects any struct with `generics.params` non-empty. The fix:

1. Remove the hard rejection in `parse.rs`.
2. Store `derive.generics` in `ExtendInput`.
3. In `codegen.rs`, when emitting `impl ... for Type`, use `impl<#generics> ... for Type<#ty_params>`
   with the standard `syn::Generics::split_for_impl()` method, which splits into
   `(impl_generics, ty_generics, where_clause)`.
4. Emit only the bounds the generated code strictly requires: none, unless the generated code
   calls a trait method on a generic type parameter. For `#[derive(Extend)]` codegen, the
   generated methods are all field-access delegation (`&self.field`, `&mut self.field`) — no
   bounds are needed on `T` itself. The `where_clause` from the struct definition is **not**
   propagated (minimal-bounds policy).

This is straightforward: `split_for_impl()` handles all the boilerplate.

**Rejected: propagating the struct's own `where_clause`** — spec says minimal bounds only; the
struct's own constraints do not belong on the delegation impls.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `proc-macro-crate` dep; implement `crate_root()` in `util.rs` | `quartzite-macros/Cargo.toml`, `quartzite-macros/src/util.rs` | — |
| 2 | Replace all hardcoded `::quartzite::core` paths in `object/codegen.rs` with `crate_root()` | `quartzite-macros/src/object/codegen.rs` | 1 |
| 3 | Replace all hardcoded `::quartzite::core` paths in `object_impl/codegen.rs` with `crate_root()` | `quartzite-macros/src/object_impl/codegen.rs` | 1 |
| 4 | Replace all hardcoded `::quartzite::core` paths in `extend/codegen.rs` with `crate_root()` | `quartzite-macros/src/extend/codegen.rs` | 1 |
| 5 | Replace all hardcoded `::quartzite::core` paths in `meta_enum/codegen.rs` with `crate_root()` | `quartzite-macros/src/meta_enum/codegen.rs` | 1 |
| 6 | Lift trait-impl restriction in `object_impl/parse.rs`; add `MethodKind` (partial/final/sole) and `trait_path: Option<syn::Path>` as fields on `ObjectImplInput` (per-invocation); handle `partial`/`final` attribute flags by giving `object_impl::expand` an `(attr: TokenStream, item: TokenStream)` signature (bypassing `make_expand!()`) and updating `lib.rs` to pass both `attr` and `item`; thread-local accumulator keyed by `(std::env::var("CARGO_PKG_NAME"), type_name)` string pair | `quartzite-macros/src/lib.rs`, `quartzite-macros/src/object_impl/parse.rs`, `quartzite-macros/src/object_impl/accumulator.rs` (new) | — |
| 7 | Update `object_impl/codegen.rs` to branch on `MethodKind`: sole/final emit full output; partial emits only cleaned impl | `quartzite-macros/src/object_impl/codegen.rs` | 6 |
| 8 | Add `#[object_meta]` proc-macro attribute in `lib.rs`; implement `object_meta` module | `quartzite-macros/src/lib.rs`, `quartzite-macros/src/object_meta/` (new: `mod.rs`, `parse.rs`, `codegen.rs`) | 6, 7 |
| 9 | Generic struct support in `extend/parse.rs` (remove rejection, store generics) | `quartzite-macros/src/extend/parse.rs` | — |
| 10 | Generic struct support in `extend/codegen.rs` (use `split_for_impl`, minimal bounds) | `quartzite-macros/src/extend/codegen.rs` | 9 |

10 tasks — slightly over the 7-task guideline, but each of the three features maps to a
clearly independent group (1–5: path detection; 6–8: multi-block impl; 9–10: generics). The
scope covers three distinct spec items and splitting into multiple issues was explicitly not
requested. Tasks within each group are logically independent of the other groups.

## Risks

- **`crate_root()` reads `CARGO_MANIFEST_DIR` at expansion time**: this is the documented contract
  of `proc-macro-crate`; it works in all cargo build modes. Risk: low.
- **`thread_local!` accumulator persistence across test invocations**: unit tests for
  `object_impl` call `parse()` and `codegen()` directly without going through `#[object_impl]`.
  Tests are independent; no cross-test contamination. Integration tests use a clean process per
  `cargo test` invocation. Risk: low.
- **Duplicate `MetaObject` when user forgets `#[object_impl(partial)]` syntax**: if a user writes
  two plain `#[object_impl]` blocks on the same type, both emit a `MetaObject` static with the
  same name — duplicate-definition compile error, no silent misbehaviour. Risk: acceptable.
- **`split_for_impl` + no where-clause**: if a generic field's type parameter does not implement
  a trait required by generated code, the error appears at the impl site rather than the derive
  site. For `#[derive(Extend)]`'s field-access delegation, no bounds are required, so this is
  not a real concern. Risk: low.
- **`extract_self_ty_ident` on trait impls**: `item.self_ty` for `impl Trait for Foo` is `Foo`
  (unchanged from inherent impls). The existing `extract_self_ty_ident` logic is correct as-is.
  Risk: none.
- **Thread-local accumulator leaking between crate compilations in the same process**: `rustc`
  may reuse the proc-macro dylib process across crates with the same short type name. The
  accumulator is keyed by `(pkg_name, type_name)` where `pkg_name` is read at expansion time
  via `std::env::var("CARGO_PKG_NAME").unwrap_or_default()` (runtime `var`, **not** compile-time
  `env!()` — `env!` would bake `"quartzite-macros"` into the dylib and make every user crate
  share the same key prefix). This prevents a `Counter` in crate A from colliding with a
  `Counter` in crate B. The cell is also drained after `#[object_meta]`/`#[object_impl(final)]`
  reads it to prevent stale data from error-path compilations bleeding into subsequent ones.

## Test Design

### Task 1 — `crate_root()`
- **Location:** `quartzite-macros/src/util.rs` `#[cfg(test)]`
- **Entry point:** `crate_root()`
- **Scenarios:**
  - Cannot easily unit-test `proc_macro_crate::crate_name` in a unit context (it reads
    `CARGO_MANIFEST_DIR`). Test the *output format* via the inner helper
    `crate_root_from(facade: Option<FoundCrate>, core: Option<FoundCrate>, pkg_name: &str) -> TokenStream`
    with injected values. Unit tests cover: facade `Itself` → `::quartzite::core` (absolute path,
    pkg_name-derived); facade `Name("my_quartzite")` → `my_quartzite::core`; only core
    `Name("quartzite_core")` → `quartzite_core`; core `Itself` → `::quartzite_core`; neither
    found → `::quartzite_core`.
  - Integration-level: existing codegen unit tests in each `codegen.rs` module implicitly
    verify that the path prefix is correct for the test environment (where `quartzite` is a
    dev-dependency of `quartzite-macros`).

### Tasks 2–5 — codegen path substitution
- **Location:** each `codegen.rs` `#[cfg(test)]` module.
- **Entry point:** existing `emit()` helper in each module.
- **Scenarios:** update existing assertions to match the new dynamic prefix. Add one test per
  module asserting the facade path (`quartzite :: core`) appears in the output (since
  `quartzite` is a dev-dependency in `quartzite-macros/Cargo.toml`).

### Tasks 6–7 — `object_impl` multi-block + trait impl
- **Location:** `quartzite-macros/src/object_impl/parse.rs` and `codegen.rs` `#[cfg(test)]`.
- **Entry point:** `parse()`, `codegen()`.
- **Scenarios (parse):**
  - Trait impl block accepted (lifts current `trait_impl_errors` test — rename/invert it).
  - `#[object_impl(partial)]` on trait impl: methods accumulated, no `MetaObject` in output.
  - `#[object_impl(final)]` on inherent impl: merges accumulated + current methods, emits full output.
  - Duplicate method name across two partial blocks: `compile_error!` token in output.
  - Plain `#[object_impl]` (no flag): sole mode, emits full output as before (AC8 regression test).
- **Scenarios (codegen):**
  - `MethodKind::Sole` output identical to current output (regression).
  - `MethodKind::Partial` output: only cleaned impl block, no `MetaObject`, no `impl Object`.
  - `MethodKind::Final` output: full output including merged methods from accumulator.

### Task 8 — `#[object_meta]`
- **Location:** `quartzite-macros/src/object_meta/parse.rs` and `codegen.rs` `#[cfg(test)]`.
- **Entry point:** `parse()`, `codegen()`.
- **Scenarios:**
  - `#[object_meta] impl Counter {}` after two `#[object_impl(partial)]` blocks: `MetaObject`
    contains methods from both blocks.
  - Empty accumulator (no prior `partial` blocks): `#[object_meta] impl Counter {}` emits an
    empty method list `MetaObject`.
  - After `codegen()` runs, thread-local cell for `Counter` is cleared.

### Tasks 9–10 — generic `#[derive(Extend)]`
- **Location:** `quartzite-macros/src/extend/parse.rs` and `codegen.rs` `#[cfg(test)]`.
- **Entry point:** `parse()`, `codegen()`.
- **Scenarios (parse):**
  - Generic struct `struct Foo<T> { #[base] widget: Widget }` parses successfully.
  - Generics are stored in `ExtendInput`.
- **Scenarios (codegen):**
  - `impl<T> AsWidget for Foo<T>` is emitted (not `impl AsWidget for Foo`).
  - No `where T: …` clause unless required (generated code uses only field access — no bounds needed).
  - Regression: non-generic struct output unchanged.
  - Struct with lifetime parameter `struct Foo<'a>` compiles (via `split_for_impl`).

## Open questions

- None. The multi-block design question from the spec has been resolved: `#[object_impl(partial)]`
  / `#[object_impl(final)]` with `thread_local!` accumulation. `#[object_meta]` is the
  alternative terminal form for users who prefer an explicit declaration point.
