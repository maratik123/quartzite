# Progress: Public API Documentation

**Spec:** ai-docs/plans/2026-05-02-public-api-docs.spec.md
**Design:** ai-docs/plans/2026-05-02-public-api-docs.design.md
**Branch:** feat/2026-05-02-public-api-docs
**base_commit:** 67c59a264b39b02c09ba7870664efebab4928454

## Subtasks

| # | Task | Status |
|---|------|--------|
| 1 | Document quartzite-macros proc-macro functions | ✅ |
| 2 | Document quartzite-core ID types | ✅ |
| 3 | Document quartzite-core ObjectBase | ✅ |
| 4 | Document Signal::new() and ReceiverGuard | ✅ |
| 5 | Document quartzite-core meta types | ✅ |
| 6 | Document quartzite-core value types + traits | ✅ |
| 7 | Add #![deny(missing_docs)] + crate doc to quartzite-core/src/lib.rs | ✅ |
| 8 | Document quartzite-runtime application items | ✅ |
| 9 | Document ObjectRef<T> and WeakRef<T> | ✅ |
| 10 | Document factory, event_loop, connection_table, thread_pool, timer items | ✅ |
| 11 | Add #![deny(missing_docs)] + crate + module docs to quartzite-runtime/src/lib.rs | ✅ |
| 12 | Add #![deny(missing_docs)] to quartzite-macros/src/lib.rs | ✅ |
| 13 | Add #![deny(missing_docs)] to quartzite/src/lib.rs | ✅ |
| 14 | Add # Examples to all single-line-only public docs | ✅ |
| 15 | Update CI RUSTDOCFLAGS | ✅ |
| 16 | Update self-review agent | ✅ |
| 17 | Add documentation rule to AGENTS.md | ✅ |

## Files touched

- quartzite-macros/src/lib.rs
- quartzite-core/src/id.rs
- quartzite-core/src/object_base.rs
- quartzite-core/src/signal.rs
- quartzite-core/src/receiver_guard.rs
- quartzite-core/src/meta.rs
- quartzite-core/src/value.rs
- quartzite-core/src/traits.rs
- quartzite-core/src/lib.rs
- quartzite-runtime/src/application.rs
- quartzite-runtime/src/object_ref.rs
- quartzite-runtime/src/factory.rs
- quartzite-runtime/src/event_loop.rs
- quartzite-runtime/src/connection_table.rs
- quartzite-runtime/src/thread_pool.rs
- quartzite-runtime/src/timer.rs
- quartzite-runtime/src/lib.rs
- quartzite/src/lib.rs
- .github/workflows/ci.yml
- .claude/agents/self-review.md
- AGENTS.md

## Next action: ready for self-review

All subtasks 1–17 complete. Final verification passed:
- cargo build: clean
- cargo test: all green (155 unit tests + 60 doctests)
- cargo fmt --check: no drift
- cargo clippy -D warnings: clean
- RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace: zero errors/warnings
- cargo test --doc --workspace: 60 passed, 3 ignored (proc-macro self-use examples)

## Self-Review

(not yet started)
