# Move `make_expand!` macro to `util.rs`

**Source:** issue #457
**Date:** 2026-05-18
**Tracked in:** #457

## Scope

1. Move the `macro_rules! make_expand` definition (currently `quartzite-macros/src/lib.rs` lines 14–23) into `quartzite-macros/src/util.rs`.
2. Update the re-export so the three existing call sites (`crate::make_expand!()` in `extend/mod.rs`, `meta_enum/mod.rs`, `object/mod.rs`) keep resolving without edits to those call sites.
3. Remove the now-orphaned `macro_rules! make_expand { ... }` block and its `pub(crate) use make_expand;` line from `lib.rs`.

## Out of scope

- Renaming the `util.rs` file to `utils.rs`. The issue title says "utils.rs", but the existing utility module in the crate is `util.rs` (singular). The move targets the existing file; renaming is a separate concern.
- Changing the macro's expansion, its visibility (`pub(crate)` stays), or its `expand` fn signature.
- Renaming `make_expand` itself.
- Touching call sites — `crate::make_expand!()` must continue to resolve unchanged.
- Restructuring any of the three caller modules (`extend/`, `meta_enum/`, `object/`).

## Deferred

(none)

## Key decisions

| Question | Decision |
|---|---|
| Target file: `util.rs` (existing) vs `utils.rs` (per issue title) | Use the existing `util.rs`. Issue title appears to be a typo for the live filename; renaming the file is out of scope. |
| Re-export mechanism so `crate::make_expand!()` keeps resolving | Design-phase choice. Two viable shapes: (a) define inside `util` module + `pub(crate) use util::make_expand;` at crate root, (b) define inside `util` with a module-level `pub(crate) use make_expand;` and rely on the existing `mod util;` line. Either preserves the call-site path. |
| Visibility | Remain `pub(crate)`. No external consumers. |

## Technical constraints

- `cargo build` and `cargo test` must pass without modifying any of the three caller modules.
- `cargo clippy --workspace --all-targets -- -D warnings` must remain clean.
- `cargo fmt -- --check` must pass.
- The `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` gate must still pass — note `util.rs` items are `pub(crate)` and exempt from the `missing_docs` lint, so no new doc strings are required by the move itself.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `macro_rules! make_expand` no longer appears in `quartzite-macros/src/lib.rs` (verified by `grep -n 'macro_rules! make_expand' quartzite-macros/src/lib.rs` returning no matches). |
| AC2 | `macro_rules! make_expand` is defined in `quartzite-macros/src/util.rs`. |
| AC3 | The three call sites — `quartzite-macros/src/extend/mod.rs`, `quartzite-macros/src/meta_enum/mod.rs`, `quartzite-macros/src/object/mod.rs` — still invoke the macro as `crate::make_expand!();` with no edits to their content. |
| AC4 | `cargo build`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo fmt -- --check` all pass. |
| AC5 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes. |

## Open questions

(none)
