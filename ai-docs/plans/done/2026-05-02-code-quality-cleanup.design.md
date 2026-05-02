# Design: Code Quality Cleanup (RustRover findings)

**Issue:** user description — RustRover static analysis
**Date:** 2026-05-02

## Approach

Four independent, low-risk refactors. Each is self-contained and can be done in
any order; the decomposition sequences them logically (runtime first, then
macros, then dependency trimmings).

### Task 1 — `detach_from_parent` helper in `object_tree.rs`

Both `reparent` and `remove_node` contain an identical let-chain:

```rust
if let Some(old_parent) = self.parent_map.remove(&id)
    && let Some(siblings) = self.children_map.get_mut(&old_parent)
{
    siblings.retain(|&c| c != id);
}
```

Extract this into a private `fn detach_from_parent(&mut self, id: ObjectId)`
method and call it from both sites. No behaviour change; the existing tests
cover both paths.

**Alternatives considered:** none — the duplication is literal and the fix is
mechanical.

### Task 2 — `make_expand!` macro in `quartzite-macros`

All four `mod.rs` files (`extend`, `meta_enum`, `object`, `object_impl`) are
byte-for-byte identical:

```rust
mod codegen;
mod parse;

pub(crate) fn expand(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match parse::parse(input) {
        Ok(ir) => codegen::codegen(ir),
        Err(e) => e.to_compile_error(),
    }
}
```

Define a `macro_rules! make_expand` in `lib.rs` that emits this boilerplate.
Each `mod.rs` keeps only `mod codegen; mod parse;` plus the macro invocation.

Visibility: the macro only needs crate visibility. Use
`pub(crate) use make_expand;` (re-export from root after `macro_rules!`
definition) so the four inner modules can reference it via `crate::make_expand!`.

**Alternatives considered:**
- A free function taking function pointers — rejected, because it would change
  the public-facing `expand` function from a `pub(crate) fn` to a wrapper,
  adding indirection with no benefit.
- Leaving duplication — rejected, contradicts spec.

### Tasks 3 & 4 — Remove unused dev-dependencies

Confirmed via `rg`: neither `pretty_assertions` nor `rstest` appear in any
source or integration-test file of `quartzite-macros` or `quartzite-runtime`.
`pretty_assertions` also has zero uses in `quartzite-core` source. `rstest` is
used in `quartzite-core/src/value.rs` and must stay.

Removal is a one-line deletion per `Cargo.toml`.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Extract `detach_from_parent` private method | `quartzite-runtime/src/object_tree.rs` | — |
| 2 | Define `make_expand!` macro and update four `mod.rs` files | `quartzite-macros/src/lib.rs`, `quartzite-macros/src/extend/mod.rs`, `quartzite-macros/src/meta_enum/mod.rs`, `quartzite-macros/src/object/mod.rs`, `quartzite-macros/src/object_impl/mod.rs` | — |
| 3 | Remove `pretty_assertions` from three `Cargo.toml` files | `quartzite-macros/Cargo.toml`, `quartzite-core/Cargo.toml`, `quartzite-runtime/Cargo.toml` | — |
| 4 | Remove `rstest` from `quartzite-runtime/Cargo.toml` | `quartzite-runtime/Cargo.toml` | — |
| 5 | Run `cargo build` + `cargo test` + `cargo clippy -- -D warnings` to verify | — | 1, 2, 3, 4 |

All four change tasks are independent and can be executed in any order.

## Risks

- `make_expand!` macro hygiene: `macro_rules!` macros operate in the caller's
  module namespace, so `parse::parse` and `codegen::codegen` must be
  resolvable from the call site. Each `mod.rs` already declares `mod codegen`
  and `mod parse`, so these identifiers will be in scope. Mitigation: compile
  after each mod.rs change, not only at the end.
- Removing `pretty_assertions` from `quartzite-core`: the crate has no
  source-level uses, but if a future test file adds `use pretty_assertions`
  the dependency will need to be re-added. Mitigation: spec is clear on scope;
  proceed.
- No API surface change (all edits are private/internal): zero backward
  compatibility risk.

## Test Design

### Task 1 — `detach_from_parent`

- Location: `quartzite-runtime/src/object_tree.rs` `#[cfg(test)]` module
  (already exists)
- Entry points: `reparent`, `remove_node` (via `destroy`)
- Scenarios covered by existing tests:
  - `reparent_updates_both_parent_and_children` — exercises `reparent` fully
  - `destroy_removes_all_descendants` — exercises `remove_node` for parent
    cleanup
- No new tests needed; refactoring must not break any existing test.

### Task 2 — `make_expand!` macro

- Integration tests in `quartzite-macros/tests/` (`extend.rs`, `meta_enum.rs`,
  `object.rs`, `object_impl.rs`) exercise the `expand` entry points end-to-end
  through `proc_macro` invocation.
- No new tests needed; all four derive/attribute macros must continue to compile
  and produce correct output for the existing test inputs.

### Tasks 3 & 4 — Dependency removal

- No code logic changed; `cargo build` and `cargo test` passing is sufficient
  evidence that no test relied on the removed crate.

## Open questions

- None.
