# Design: Log facade via tracing/log feature

**Issue:** #89
**Date:** 2026-05-05

## Approach

Enable the `tracing` crate's built-in `log` feature flag in the two crates that already depend on
`tracing`. This bridges `tracing` instrumentation to any `log`-compatible subscriber (e.g.
`env_logger`) at zero additional dependency cost and with no new instrumentation needed.

**Chosen approach:** Cargo feature flag changes only — no new code, no new public API.

- `quartzite-runtime/Cargo.toml`: add `features = ["log"]` to the existing `tracing = "0.1"` dep.
  `quartzite-runtime` is always-std, so no guard is needed.
- `quartzite-core/Cargo.toml`: extend the `std` feature list with `"tracing/log"`. The `log` crate
  is itself `no_std`-compatible, but gating `tracing/log` on `std` is spec-mandated to keep the
  `no_std + alloc` build path clean and explicit.
- Add a doc note in `quartzite-runtime/src/lib.rs` crate-level doc explaining the integration point
  (one sentence suffices; AC5 is satisfied by either crate, and runtime is always-std, making it
  the natural home).
- Add `env_logger` to `[dev-dependencies]` in the root `quartzite/Cargo.toml` and call
  `env_logger::init()` near the top of each example's `main()`.

**Rejected alternatives:**

- Adding `tracing-log` as a separate dep — redundant; `tracing/log` feature already provides the
  same bridge and ships with `tracing` itself.
- Gating `tracing/log` in `quartzite-runtime` behind a feature — unnecessary; the crate is
  always-std and the change is zero-overhead at runtime.
- Documenting the integration in `quartzite-core` lib doc — possible but less discoverable;
  `quartzite-runtime` is the entry-point crate for std users, making it the better home.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Enable `tracing/log` in `quartzite-runtime` | `quartzite-runtime/Cargo.toml` | — |
| 2 | Enable `tracing/log` in `quartzite-core` gated on `std` feature | `quartzite-core/Cargo.toml` | — |
| 3 | Add log-subscriber doc note to `quartzite-runtime` crate-level doc | `quartzite-runtime/src/lib.rs` | 1 |
| 4 | Add `env_logger` dev-dependency to root crate | `quartzite/Cargo.toml` | — |
| 5 | Add `env_logger::init()` to all four examples | `examples/hello_object.rs`, `examples/signals_slots.rs`, `examples/object_tree.rs`, `examples/timer.rs` | 4 |
| 6 | Verify `cargo build -p quartzite --no-default-features` still compiles | — (CI gate) | 1, 2 |

Tasks 1, 2, and 4 are fully independent and can be done in parallel. Task 3 depends on 1 (same
file cluster). Task 5 depends on 4 (new dep must exist before the examples reference it). Task 6 is
a verification step, not a code change.

## Risks

- **`no_std` path broken by `tracing/log`:** mitigated by gating the feature on `quartzite-core`'s
  existing `std` feature (`log` is itself `no_std`-compatible, but the gate is explicit and
  spec-mandated); the `quartzite-runtime` change is irrelevant to the no_std path.
  Confirmed by running `cargo build -p quartzite --no-default-features` (AC3).
- **`env_logger` version compatibility:** `env_logger` `0.11` is the current stable release; use
  version constraint `"0.11"` in dev-dependencies. It is a dev-dep only, so no impact on
  downstream consumers.
- **Example output noise during tests:** `env_logger::init()` in example code will not affect
  `cargo test` runs (examples are binaries, not test targets). No risk.
- **`env_logger` panics on double-init:** `env_logger::init()` panics if called twice in the same
  process. This is acceptable in example `main()` functions — each example is an independent
  binary.

## Test Design

All changes are Cargo manifest edits and one-liner additions to example files. There is no
non-trivial logic introduced, so no new unit tests are required.

Verification approach:
- `cargo build` — checks AC4 (std build compiles)
- `cargo build -p quartzite --no-default-features` — checks AC3 (no_std path)
- `cargo run --example hello_object` — should produce tracing output on stderr when
  `RUST_LOG=trace` is set, confirming the log bridge is active (manual spot-check for AC6)

## Open questions

_None._
