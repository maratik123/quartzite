# Log facade via tracing/log feature

**Source:** user description
**Date:** 2026-05-05
**Tracked in:** #89

## Scope

- Enable `tracing`'s built-in `log` feature in `quartzite-runtime` (always-std crate): change `tracing = "0.1"` to `tracing = { version = "0.1", features = ["log"] }`
- Enable `tracing/log` in `quartzite-core` gated on its existing `std` feature: `std = ["indexmap/std", "tracing/log"]`
- Add a doc note (in the relevant crate-level or module doc) mentioning that `log`-compatible subscribers (e.g. `env_logger`) receive quartzite's diagnostics automatically when the `std` feature is enabled
- Update relevant examples to demonstrate initialising a `log`-compatible subscriber (e.g. `env_logger`) so users can see quartzite's diagnostics in action

## Out of scope

- Adding a separate `log` crate dependency
- Installing or configuring a log subscriber in library code (application/user responsibility)
- Instrumenting `quartzite-geometry` or `quartzite-events` (no tracing macros there)
- Adding new tracing call sites (instrumentation already done in PR #90)

## Deferred

- `tracing-log` bridge crate | not needed; `tracing/log` feature covers the use case | no

## Key decisions

| Question | Decision |
|---|---|
| `log` vs `tracing/log` feature vs `tracing-log` crate | Enable `tracing/log` feature — zero new instrumentation, reuses existing tracing macros, serves both tracing and log-based subscribers |
| Which crates | `quartzite-runtime` (unconditional) + `quartzite-core` (gated on `std` feature to preserve no_std builds) |
| Log subscriber responsibility | Framework user; quartzite only documents the integration point |

## Technical constraints

- `quartzite-core` is no_std when `default-features = false`; `tracing/log` requires std (the `log` crate needs std). Gate via existing `std` feature.
- No new public API surface introduced — purely a Cargo feature flag change.
- Must preserve `cargo build -p quartzite --no-default-features` (no_std path).
- Examples live in the root `quartzite` crate; `env_logger` must be added as `[dev-dependencies]` there.
- All four existing examples (`hello_object`, `signals_slots`, `object_tree`, `timer`) should initialise the subscriber so any example demonstrates log output out of the box.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-runtime/Cargo.toml` declares `tracing` with `features = ["log"]` |
| AC2 | `quartzite-core/Cargo.toml` `std` feature includes `tracing/log` |
| AC3 | `cargo build -p quartzite --no-default-features` compiles without error (no_std path unbroken) |
| AC4 | `cargo build` (std, default) compiles without error |
| AC5 | Crate-level or module doc in `quartzite-core` or `quartzite-runtime` mentions that `log`-compatible subscribers receive quartzite's diagnostics |
| AC6 | At least one example initialises a `log`-compatible subscriber (e.g. `env_logger::init()`) so users can observe quartzite's log output when running the example |

## Open questions

_None._
