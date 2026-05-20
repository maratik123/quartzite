# Rename Derive macros to Object

**Source:** issue #489
**Date:** 2026-05-20
**Tracked in:** #489

## Scope

1. Drop the `as DeriveObject` / `as ObjectDerive` rename aliases on every import / re-export of the `quartzite_macros::Object` derive macro. The derive macro is **already named `Object`** at its definition site (`#[proc_macro_derive(Object, ...)]` in `quartzite-macros/src/lib.rs:103`) — `DeriveObject` and `ObjectDerive` are import-site rename aliases, not distinct items.
2. Update every call site to use `Object` directly:
   - `src/lib.rs:374` — prelude re-export (replace `Object as DeriveObject` with `Object`; the prelude also re-exports the `quartzite_core::Object` **trait**, which lives in a different namespace from the derive macro — Rust resolves `#[derive(Object)]` vs `impl Object for T` by namespace, same model as `std::fmt::Debug` trait + `#[derive(Debug)]`).
   - `src/lib.rs:25` (lib-crate doc-example), `src/lib.rs:179` (doc link `[`DeriveObject`](macros::Object)`), `src/lib.rs:371` (comment explaining the rename), `src/lib.rs:13` (lib-crate doc): replace `DeriveObject` with `Object` and drop the obsolete comment about shadowing.
   - `benches/macro_object.rs:5,10` — replace `use ... Object as DeriveObject` + `#[derive(Extend, DeriveObject)]` with bare `Object`.
   - `quartzite-macros/tests/via_facade.rs:6,8` — same shape.
   - `quartzite-macros/tests/object_impl.rs:4,8,34,67` — switch `Object as ObjectDerive` → `Object`, `ObjectDerive` → `Object`.
   - `tests/single_dep.rs:7`, `tests/signal_to_signal.rs:13,27`, `examples/object_tree.rs:5`, `examples/signals_slots.rs:5`, `examples/hello_object.rs:5`, `examples/combined.rs:1,15,80` — replace `DeriveObject` with `Object`.
3. Update doc strings, comments, and historic-spec/design references where they're still load-bearing (lib.rs doc block at line 13, line 179 intra-doc link, line 371 comment). The completed-plan `ai-docs/plans/done/**` references are historical artefacts — leave them untouched unless they are intra-doc-linked.
4. Update `ai-docs/plans-summary.md:22` — the live (non-`done/`) docs that mention `DeriveObject` get refreshed; `ai-docs/learnings.md:5` is append-only per AGENTS.md and is left as-is.
5. No `pub use Object as DeriveObject` alias is left behind — per AGENTS.md § *API Stability* (pre-`cargo publish`, clean breaks).

## Out of scope

- Renaming the proc-macro definition itself — it is **already** `Object`. The issue title's phrasing "rename DeriveObject and ObjectDerive to Object" is satisfied by removing aliases.
- Renaming `derive_extend`, `derive_meta_enum`, `Extend`, `MetaEnum`, `object_impl`, `object_part`, or any other macro — they don't carry a `Derive*` / `*Derive` shape.
- Renaming `syn::DeriveInput` references (`quartzite-macros/src/object/parse.rs:2,41`, `quartzite-macros/src/meta_enum/parse.rs:1`, `quartzite-macros/src/extend/parse.rs:1,127`) — that is a third-party type from `syn`.
- Touching `ai-docs/plans/done/**` historical specs/designs except where they're intra-doc-linked from live code.
- Editing `ai-docs/learnings.md` (append-only per AGENTS.md Boundary rule 1).

## Deferred

(none)

## Key decisions

| Question | Decision |
|---|---|
| Target name for the derive macro at every import site | `Object` — the macro's actual definition name; no aliasing |
| Leave a `pub use Object as DeriveObject` aliasing line behind in the prelude? | No — AGENTS.md § *API Stability* mandates clean breaks pre-publish (no aliasing wrappers) |
| Resolve trait-vs-derive collision in the prelude (`quartzite_core::Object` trait + `quartzite_macros::Object` derive macro both re-exported)? | Rely on Rust's namespace separation: derive macros live in the macro namespace, traits/types in the type namespace. `use quartzite::prelude::*; #[derive(Object)] impl Object for Foo { ... }` is well-formed (same model as `std::fmt::Debug` + `#[derive(Debug)]`). |
| Single-pass rename across both `DeriveObject` and `ObjectDerive` variants? | Yes — both are import-site aliases for the same macro; a `git grep -l 'DeriveObject\|ObjectDerive'` sweep covers all of them. |
| Inconsistent `ObjectDerive` alias in `quartzite-macros/tests/object_impl.rs` while the rest of the tree uses `DeriveObject` | One-off inconsistency; collapses to `Object` in the same pass. No separate decision needed. |

## Technical constraints

- **`cargo build` must pass on every feature combination CI exercises**, including `--no-default-features --features libm`, `--no-default-features --features std`, and the full default set. The `derive` feature gate around the prelude re-export (`#[cfg(feature = "derive")]`) is preserved.
- **`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`** must pass after the rename. Intra-doc links that currently target `macros::Object` already resolve to the right item; the alias removal only affects display text and the `as` clause.
- **`cargo test --workspace`** must pass — including the doctest in `src/lib.rs:25` that uses the macro, and the `via_facade.rs` / `object_impl.rs` integration tests.
- **`cargo clippy --workspace --all-targets -- -D warnings`** must pass.
- The `Extend` derive macro is untouched (already named `Extend`, no alias).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `git grep -nE 'DeriveObject\|ObjectDerive' -- '*.rs' '*.toml'` returns no matches inside `src/`, `tests/`, `examples/`, `benches/`, `quartzite-macros/`, or any other live crate's `src/`/`tests/`/`examples/`/`benches/`. |
| AC2 | `git grep -nE 'DeriveObject\|ObjectDerive' -- 'ai-docs/**/*.md' ':!ai-docs/plans/done/**' ':!ai-docs/learnings.md'` returns no matches (live agent docs are clean; `done/` history and append-only `learnings.md` retain originals). |
| AC3 | `quartzite/src/lib.rs` prelude re-export reads `pub use quartzite_macros::{Extend, Object, object_impl, object_part};` (no `as`-clause for `Object`). The `// `Object` is re-exported as `DeriveObject` …` comment is removed. |
| AC4 | The lib-crate doc-example in `src/lib.rs:25` compiles as a doctest with `#[derive(Extend, Object)]`; the intra-doc link at `src/lib.rs:179` reads `[`Object`](macros::Object)`. |
| AC5 | `cargo build --workspace` passes. |
| AC6 | `cargo test --workspace` passes — including doctest, `tests/single_dep.rs`, `tests/signal_to_signal.rs`, `quartzite-macros/tests/via_facade.rs`, `quartzite-macros/tests/object_impl.rs`, and every example under `examples/`. |
| AC7 | `cargo clippy --workspace --all-targets -- -D warnings` passes. |
| AC8 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes. |
| AC9 | `cargo build -p quartzite --no-default-features --features libm` passes (derive-free / `no_std` smoke path stays green). |
| AC10 | No `pub use ... as DeriveObject` or `pub use ... as ObjectDerive` aliasing line remains anywhere in the workspace. |

## Open questions

(none — the user's instruction is concrete, the AGENTS.md API-stability AXIOM resolves the alias-removal question, and the trait-vs-derive coexistence is a documented Rust pattern.)
