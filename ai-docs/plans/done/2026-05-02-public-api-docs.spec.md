# Public API Documentation

**Source:** user description
**Date:** 2026-05-02

## Scope

1. Document all 4 proc-macros in `quartzite-macros` (`Extend`, `Object`, `object_impl`, `MetaEnum`) with attribute syntax, expansion description, and usage examples.
2. Document 18 undocumented public items in `quartzite-core`: `ObjectId`, `ConnectionId` (structs + constructors + raw()), `ObjectBase` struct + `named()` + `is_on_current_thread()`, `Signal::new` / `connect_queued` / `connect_auto`, `ReceiverGuard`.
3. Document 15 undocumented public items in `quartzite-runtime`: `ApplicationError`, `Application::connection_table` / `event_loop`, `ObjectRef<T>` struct + all 3 methods, `WeakRef<T>` struct + all 3 methods, `ObjectFactory::new`, `EventLoop::new`, `ConnectionTable::new`, `ThreadPool::new`, `Timer::stop` / `is_running`.
4. Add at least one `# Examples` code block to all single-line-only public docs across all crates that currently have none (~40 items).
5. Add `#![deny(missing_docs)]` to `lib.rs` of all four crates.
6. Update `.github/workflows/ci.yml` to include `-D missing-docs` in `RUSTDOCFLAGS`.
7. Update `.claude/agents/self-review.md` to verify `#![deny(missing_docs)]` is present in any crate `lib.rs` with new or modified public items, and that every new non-trivial public item has a `# Examples` block.
8. Add a documentation rule to `AGENTS.md` (Code Style section): all new public items must have `///` doc comments; non-trivial public items must include a compiling `# Examples` block (`no_run` only where full execution is impractical).

## Out of scope

- Private/internal items.
- `quartzite-widgets` (not yet implemented).
- Separate EXAMPLES.md or README additions beyond existing content.

## Deferred

- none

## Key decisions

| Question | Decision |
|---|---|
| Lint level | `#![deny(missing_docs)]` — hard error, fails build |
| Example quality | Compiling doctests preferred; `no_run` only where full execution is impractical (proc-macro self-use, items requiring a running event loop) |
| Self-review check | Add checklist item: verify `#![deny(missing_docs)]` present in `lib.rs` of any crate with new/modified public items |

## Technical constraints

- Proc-macro crates cannot use their own macros in doctests; examples for `quartzite-macros` items use `no_run` with illustrative code.
- Runtime items that require an initialized `Application` / event loop use `no_run` doctests.
- `cargo test --doc --workspace` must pass: all non-`no_run` / non-`ignore` doctests compile and run.
- Edition 2024; let-chains allowed; `cargo fmt` before every commit.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite/src/lib.rs` contains `#![deny(missing_docs)]` |
| AC2 | `quartzite-core/src/lib.rs` contains `#![deny(missing_docs)]` |
| AC3 | `quartzite-macros/src/lib.rs` contains `#![deny(missing_docs)]` |
| AC4 | `quartzite-runtime/src/lib.rs` contains `#![deny(missing_docs)]` |
| AC5 | All previously undocumented public items in `quartzite-macros` have `///` doc comments with attribute syntax and at least one `# Examples` block |
| AC6 | All previously undocumented public items in `quartzite-core` have `///` doc comments; items with non-trivial usage have at least one compiling `# Examples` block |
| AC7 | All previously undocumented public items in `quartzite-runtime` have `///` doc comments; items with non-trivial usage have at least one `# Examples` block |
| AC8 | All previously single-line-only public doc items across all crates have at least one `# Examples` block added |
| AC9 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` passes with zero errors or warnings |
| AC10 | `cargo test --doc --workspace` passes green (all non-`no_run`/non-`ignore` doctests compile and run) |
| AC11 | `.github/workflows/ci.yml` `RUSTDOCFLAGS` includes `-D missing-docs` |
| AC12 | `.claude/agents/self-review.md` includes two checklist items: (1) verify `#![deny(missing_docs)]` is present in any modified crate's `lib.rs`; (2) verify every new non-trivial public item has a `# Examples` block (compiling, or `no_run` only where full execution is impractical) |

## Open questions

- none
