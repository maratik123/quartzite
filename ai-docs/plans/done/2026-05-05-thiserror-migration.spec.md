# Migrate manual Display/Error impls to `#[derive(thiserror::Error)]`

**Source:** user description
**Date:** 2026-05-05
**Tracked in:** #86

## Scope

- Add `thiserror = "2"` to `quartzite-core` Cargo.toml
- Migrate `quartzite-runtime::ApplicationError` — replace manual `Display` + `Error` impls with `#[derive(thiserror::Error)]`
- Migrate `quartzite-runtime::FactoryAlreadySet` — replace manual `Display` + `Error` impls with `#[derive(thiserror::Error)]`
- Migrate `quartzite-core::DispatcherAlreadySet` — replace manual `Display` + `Error` impls (both `#[cfg(feature = "std")]`-gated) with `#[derive(thiserror::Error)]`
- Migrate `quartzite-core::TypeError` — replace manual `Display` impl and add missing `core::error::Error` impl via `#[derive(thiserror::Error)]`

## Out of scope

- `TreeAccessError` — already uses `#[derive(thiserror::Error)]`; no change needed
- Changing any error message strings
- Changing any public API (variant names, field names, struct names)

## Deferred

- None

## Key decisions

| Question | Decision |
|---|---|
| Add `thiserror` to `quartzite-core` (no_std crate)? | Yes — `thiserror` 2.x is no_std-compatible; MSRV 1.95 supports `core::error::Error` |
| Add `Error` impl to `TypeError`? | Yes — via `#[derive(thiserror::Error)]` alongside the migrated `Display` |
| Change error message strings? | No — preserve all existing messages verbatim |

## Technical constraints

- `quartzite-core` is no_std + alloc; `thiserror` 2.x generates `core::error::Error` impls when `std` is not available — compatible.
- `DispatcherAlreadySet`'s `Display` and `Error` impls are `#[cfg(feature = "std")]`-gated. With `thiserror`, the `#[derive(thiserror::Error)]` attribute and `#[error("...")]` handle the gating implicitly via thiserror's own feature detection.
- All changes must pass `cargo build -p quartzite --no-default-features` (no_std path).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ApplicationError` derives `thiserror::Error`; manual `Display` and `Error` impls removed |
| AC2 | `FactoryAlreadySet` derives `thiserror::Error`; manual `Display` and `Error` impls removed |
| AC3 | `DispatcherAlreadySet` derives `thiserror::Error`; manual `Display` and `Error` impls removed; `cfg(feature = "std")` guard preserved correctly |
| AC4 | `TypeError` derives `thiserror::Error`; manual `Display` impl removed; `core::error::Error` is now implemented |
| AC5 | All existing error message strings are preserved verbatim |
| AC6 | `cargo build -p quartzite --no-default-features` compiles cleanly (no_std path unbroken) |
| AC7 | `cargo test` passes green with no regressions |
| AC8 | `cargo clippy -- -D warnings` reports no warnings |

## Open questions

- None
