# Code Quality Cleanup (RustRover findings)

**Source:** user description — RustRover static analysis
**Date:** 2026-05-02

## Scope

1. `quartzite-runtime/src/object_tree.rs` — extract `detach_from_parent` private helper to eliminate the duplicated 5-line let-chain shared between `reparent` and `remove_node`.
2. Introduce `make_expand!()` declarative macro in `quartzite-macros/src/lib.rs`; replace the duplicated `expand` function body in each of the 4 `mod.rs` files (`extend`, `meta_enum`, `object_impl`, `object`).
3. Remove unused `pretty_assertions` dev-dependency from `quartzite-macros`, `quartzite-core`, `quartzite-runtime`.
4. Remove unused `rstest` dev-dependency from `quartzite-runtime` only.

## Out of scope

- Removing `rstest` from `quartzite-core` — it is genuinely used in `quartzite-core/src/value.rs`.
- Any other code-quality changes not listed above.

## Deferred

- None.

## Key decisions

| Question | Decision |
|---|---|
| How to deduplicate 4 identical mod.rs files? | Declarative `make_expand!()` macro in lib.rs; each mod.rs keeps `mod codegen; mod parse;` and calls `crate::make_expand!()` |
| Where to define the macro? | `quartzite-macros/src/lib.rs` using `macro_rules!` + `#[macro_export]`... actually `pub(crate) use make_expand;` since it only needs crate visibility |
| rstest in quartzite-core? | Keep — 4 `#[rstest]` uses in `value.rs` |

## Technical constraints

- Rust edition 2024; let-chains valid.
- `cargo clippy -- -D warnings` must stay clean.
- Macro must be `macro_rules!` (declarative); no proc-macro changes.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `object_tree.rs` contains a private `detach_from_parent` method; `reparent` and `remove_node` each call it instead of repeating the let-chain |
| AC2 | A `make_expand!` macro is defined in `quartzite-macros`; each of the 4 `mod.rs` files invokes it rather than repeating the `expand` function body |
| AC3 | `pretty_assertions` does not appear in `quartzite-macros/Cargo.toml`, `quartzite-core/Cargo.toml`, or `quartzite-runtime/Cargo.toml` |
| AC4 | `rstest` does not appear in `quartzite-runtime/Cargo.toml`; it remains in `quartzite-core/Cargo.toml` |
| AC5 | `cargo build` succeeds and all existing tests pass after all changes |

## Open questions

- None.
