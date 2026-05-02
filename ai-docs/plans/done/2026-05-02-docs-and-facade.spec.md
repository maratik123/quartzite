# Comprehensive documentation and quartzite facade crate

**Source:** user description
**Date:** 2026-05-02

## Scope

1. Add `authors`, `license`, `repository` to `[workspace.package]`; existing crates inherit with `workspace = true`
2. Add per-crate `description` to all four crates (3 existing + 1 new facade)
3. Add `document-features` dependency to `quartzite-core` and `quartzite` facade; annotate feature entries with `##` comments; add `#![doc = document_features::document_features!()]` in their `lib.rs`
4. Add `[package.metadata.docs.rs]` section to every crate's `Cargo.toml` (`features = ["std"]` for core and facade; `rustdoc-args`/`rustc-args` set to `["--cfg", "docsrs"]` for all)
5. Create `quartzite` facade crate as a new workspace member with a curated `prelude` module; `std` feature forwarding to `quartzite-core/std`

## Out of scope

- `std` feature on `quartzite-runtime` (would gate nothing today)
- Full wildcard re-exports beyond `prelude`
- Future features (`extension`, `8k_pages`, etc.)
- `cargo doc` publishing / CI integration

## Deferred

- Additional facade features | will be added based on needs as the project grows

## Key decisions

| Question | Decision |
|---|---|
| `quartzite-runtime` std feature | Skip — flag would gate nothing today |
| Facade public surface | Curated `prelude` module only |
| docs.rs features for macros/runtime | Empty list — no features to enable |
| SPDX license identifier | `LGPL-3.0-or-later` (matches LICENSE file) |

## Technical constraints

- `document-features` reads `Cargo.toml` at compile time via a proc-macro; features must be annotated with `##` (double `#`) comments directly above each feature entry
- `#![cfg_attr(docsrs, doc(cfg(feature = "std")))]` should annotate items that are only available with `std` to show the badge on docs.rs
- `quartzite-macros` is a proc-macro crate (`proc-macro = true`); it cannot have `document-features` (proc-macro crates cannot depend on other crates in the normal way for doc generation)
- Facade crate cannot be named `quartzite-facade`; must be `quartzite`

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `[workspace.package]` contains `authors`, `license = "LGPL-3.0-or-later"`, and `repository = "https://github.com/maratik123/quartzite"` |
| AC2 | Each of the 4 crates has a non-empty `description` matching the agreed text |
| AC3 | `quartzite-core/Cargo.toml` has `document-features` in `[dependencies]`; the `std` feature has a `##` doc comment; `quartzite-core/src/lib.rs` starts with `#![doc = document_features::document_features!()]` |
| AC4 | `quartzite/Cargo.toml` has `document-features` in `[dependencies]`; the `std` feature has a `##` doc comment; `quartzite/src/lib.rs` contains `#![doc = document_features::document_features!()]` |
| AC5 | All 4 crate `Cargo.toml` files have `[package.metadata.docs.rs]`; `quartzite-core` and `quartzite` have `features = ["std"]`; all have `rustdoc-args = ["--cfg", "docsrs"]` and `rustc-args = ["--cfg", "docsrs"]` |
| AC6 | Root `Cargo.toml` `workspace.members` includes `"quartzite"` |
| AC7 | `quartzite/src/lib.rs` has a `pub mod prelude` re-exporting key types from `quartzite-core`, `quartzite-macros`, and `quartzite-runtime` |
| AC8 | Facade `std` feature in `quartzite/Cargo.toml` activates `quartzite-core/std` |
| AC9 | `cargo build` compiles clean |
| AC10 | `cargo clippy -- -D warnings` passes clean |

## Open questions

- None
