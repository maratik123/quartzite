# Examples Crate

**Source:** user description
**Date:** 2026-05-02

## Scope

1. New `quartzite-examples` workspace member added to root `Cargo.toml`
2. Four working example binaries in `examples/` directory:
   - `hello_object.rs` — define a type with `#[derive(Object)]` + `#[prop]`, read/write a property
   - `signals_slots.rs` — define signals, connect a slot, emit the signal, observe the call
   - `object_tree.rs` — create objects in an `ObjectTree`, set parent/child, find by name
   - `timer.rs` — `Application` + `EventLoop`, create a `Timer`, fire a slot on tick, stop after N ticks
3. All examples compile and run without errors
4. `AGENTS.md` updated to document that `quartzite-examples` is excluded from the `#![deny(missing_docs)]` rule and the `#[cfg(test)]` requirement (example targets, not library code)
5. `README.md` updated to list `quartzite-examples` in the status table with description of available examples
6. `quartzite/src/lib.rs` publicly re-exports `quartzite_core` and `quartzite_runtime` as named items — gives users access via `quartzite::quartzite_core::X`, ensures type identity when mixing facade and direct sub-crate deps, and provides a clear migration path to direct deps
7. `ai-docs/context.md` updated with an open question on `proc_macro_crate` ergonomics
8. `.claude/agents/self-review.md` and `.claude/agents/review-findings.md` updated with `quartzite-examples` exemptions so future reviews don't flag missing `#[cfg(test)]` or `#![deny(missing_docs)]` incorrectly

## Out of scope

- Examples for unimplemented crates (geometry, events, widgets, paint, style)
- Unit tests inside example files
- Python interop examples
- Changing `quartzite-macros` codegen (deferred — `proc_macro_crate` is a separate future task)

## Deferred

- Examples for geometry-events, widgets, paint-style | those crates not yet implemented
- `proc_macro_crate` based path detection in `quartzite-macros` | would let users depend on only `quartzite` or only `quartzite-core`; non-trivial, separate task

## Key decisions

| Question | Decision |
|---|---|
| Crate name | `quartzite-examples` |
| Binary structure | `examples/` directory — run via `cargo run --example <name> -p quartzite-examples` |
| Macro user deps | `quartzite` + `quartzite-core` both required — macros emit `::quartzite_core::` paths that require the sub-crate as a direct dep regardless of facade presence |
| Facade re-exports | `pub use quartzite_core;` + `pub use quartzite_runtime;` — public API, not a workaround; ensures type identity and easy migration |
| "One-dep facade" | Not claimed — the facade is a prelude convenience and stable re-export surface; dep reduction requires future `proc_macro_crate` work |
| `missing_docs` gate | `quartzite-examples` exempted — minimal `src/lib.rs` with crate doc comment, no public items |
| `#[cfg(test)]` requirement | Exempted for example files — they are short runnable demos |

## Technical constraints

- Must compile with `cargo build` (workspace)
- Must pass `cargo clippy -- -D warnings` (all crates)
- Must pass `cargo fmt -- --check`
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` must remain clean

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `cargo build` in workspace root succeeds with `quartzite-examples` included |
| AC2 | `cargo run --example hello_object -p quartzite-examples` runs and exits cleanly, printing evidence of property read/write |
| AC3 | `cargo run --example signals_slots -p quartzite-examples` runs and exits cleanly, printing evidence that a slot was called |
| AC4 | `cargo run --example object_tree -p quartzite-examples` runs and exits cleanly, printing evidence of parent/child and name lookup |
| AC5 | `cargo run --example timer -p quartzite-examples` runs, fires at least one tick, prints evidence, then stops and exits cleanly |
| AC6 | `cargo clippy -- -D warnings` passes for all workspace crates |
| AC7 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` passes |
| AC8 | `AGENTS.md` documents the `quartzite-examples` exclusion from `#![deny(missing_docs)]` and `#[cfg(test)]` rules |
| AC9 | `README.md` lists `quartzite-examples` in the status table with description of available examples |
| AC10 | `quartzite::quartzite_core` and `quartzite::quartzite_runtime` are accessible as public items in the `quartzite` facade |
| AC11 | `ai-docs/context.md` records `proc_macro_crate` ergonomics as an open question |
| AC12 | `.claude/agents/self-review.md` and `.claude/agents/review-findings.md` carry the `quartzite-examples` exemption for `#[cfg(test)]` and `#![deny(missing_docs)]` checks |

## Open questions

(none — `proc_macro_crate` is captured as deferred)
