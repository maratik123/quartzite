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

## Self-Review (Round 4)

**What was checked:**

- Round 3 findings (#1–#7): verified each `✅ Fixed` item is correctly implemented — Timer::stop, ObjectRef::new, WeakRef::new, Signal::connect_typed, Signal::connect_queued, Signal::connect_auto, Application::object_tree() — all confirmed fixed.
- AC1–AC4: `#![deny(missing_docs)]` in all four `lib.rs` files — confirmed.
- AC9: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` — exits 0, zero errors/warnings — confirmed.
- AC11: CI RUSTDOCFLAGS includes `-D missing-docs` — confirmed.
- AC12: `.claude/agents/self-review.md` both checklist items present — confirmed.
- AC8 thorough sweep: ran automated script over all 23 changed files comparing base commit vs current state. Identified single-line docs without `# Examples` blocks, distinguishing new (added by diff) vs pre-existing items.
- Checklist §6: Reviewed every newly-added `///` doc comment in the diff for single-line-only items missing `# Examples`.

**Verdict:** REJECT

| # | File:line | Severity | Finding | Status |
|---|-----------|----------|---------|--------|
| 1 | quartzite-core/src/signal.rs:70 | major | Checklist §6: Newly added single-line doc on `QueuedDispatcher::post` (trait method) has no `# Examples` block. | ⚠️ Objected: error/marker/semi-internal types where examples are boilerplate noise; doc serves users, not lint compliance — user approved |
| 2 | quartzite-runtime/src/application.rs:6 | major | Checklist §6: Newly added single-line doc on `ApplicationError` enum has no `# Examples` block. | ⚠️ Objected: same |
| 3 | quartzite-core/src/signal.rs:78 | major | AC8: Pre-existing single-line-only doc on `DispatcherAlreadySet` struct has no `# Examples` block. | ⚠️ Objected: same |
| 4 | quartzite-runtime/src/connection_table.rs:18 | major | AC8: Pre-existing single-line-only doc on `ConnectionRecord` struct has no `# Examples` block. | ⚠️ Objected: same |
| 5 | quartzite-runtime/src/connection_table.rs:34 | major | AC8: Pre-existing single-line-only doc on `SlotKind` enum has no `# Examples` block. | ⚠️ Objected: same |
| 6 | quartzite-core/src/value.rs:124 | major | AC8: Pre-existing single-line-only doc on `TypeError` struct has no `# Examples` block. | ⚠️ Objected: same |
| 7 | quartzite-core/src/meta.rs:7 | minor | AC8: Pre-existing single-line-only struct-level docs on `PropertyFlags`, `PropertyMeta` (105), `ParamMeta` (136), `SignalMeta` (162), `MethodMeta` (188), `EnumEntry` (224), `EnumMeta` (250) have no `# Examples` block (constructors have examples but struct-level docs are bare one-liners). | ⚠️ Objected: constructors already show usage; struct-level one-liners are sufficient descriptions — user approved |

## Self-Review (Round 2)

**What was checked:**

- Round 1 fixes (#1–#6): verified each `✅ Fixed` item is correctly implemented in the current diff.
- AC1–AC4: `#![deny(missing_docs)]` in all four `lib.rs` files — verified directly.
- AC8 (fresh sweep): ran a script to identify all single-line-only public doc items without `# Examples` across the workspace. Found 10+ pre-existing methods/functions that subtask 14 did not address.
- AC9: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` — exits 0, zero errors/warnings.
- AC11: CI RUSTDOCFLAGS updated — verified.
- AC12: `.claude/agents/self-review.md` two checklist items added — verified.
- AGENTS.md documentation rule — verified.

**Verdict:** REJECT

| # | File:line | Severity | Finding | Status |
|---|-----------|----------|---------|--------|
| 1 | quartzite-core/src/object_base.rs:97 | major | AC8: pre-existing single-line-only doc on `ObjectBase::id()` has no `# Examples` block. Same for `receiver_guard()` (102). | ⬜ Open |
| 2 | quartzite-runtime/src/event_loop.rs:44 | major | AC8: pre-existing single-line-only docs on `EventLoop::post()` (44), `sender()` (50), `run()` (55), `stop()` (75), `is_running()` (82) — none have `# Examples`. | ⬜ Open |
| 3 | quartzite-runtime/src/connection_table.rs:75 | major | AC8: pre-existing single-line-only docs on `ConnectionTable::install_as_dispatcher()` (75), `remove()` (115), `remove_by_receiver()` (137), `receivers_for_signal()` (156) — none have `# Examples`. | ⬜ Open |
| 4 | quartzite-runtime/src/factory.rs:30 | major | AC8: pre-existing single-line-only docs on `ObjectFactory::register()` (30) and `create()` (38) have no `# Examples` block. | ⬜ Open |
| 5 | quartzite-runtime/src/thread_pool.rs:59 | major | AC8: pre-existing single-line-only doc on `ThreadPool::spawn()` has no `# Examples` block. | ⬜ Open |
| 6 | quartzite-runtime/src/timer.rs:91 | major | AC8: pre-existing single-line-only doc on `Timer::start()` has no `# Examples` block. | ⬜ Open |
| 7 | quartzite-core/src/signal.rs:84 | major | AC8: pre-existing single-line-only docs on `set_queued_dispatcher()` (84) and `queued_dispatcher()` (92) have no `# Examples` block. | ⬜ Open |
| 8 | quartzite-runtime/src/object_tree.rs:30 | major | AC8: pre-existing single-line-only doc on `ObjectTree::new()` has no `# Examples` block. | ⬜ Open |
| 9 | quartzite-runtime/src/connection_table.rs:81 | major | AC8: pre-existing two-line doc on `ConnectionTable::register()` has no `# Examples` block (multi-line but no examples — still violates AC8 spirit; included as borderline). | ⬜ Open |
| 10 | quartzite-core/src/value.rs:143 | minor | AC8: single-line trait-level docs on `FromValue` (143) and `IntoValue` (159) have no `# Examples` block at trait level (methods have examples, but trait docs themselves are bare one-liners). | ⬜ Open |

## Self-Review (Round 3)

**What was checked:**

- All 10 Round 2 findings: verified each `⬜ Open` item is correctly fixed in the current diff.
- AC1–AC4: `#![deny(missing_docs)]` in all four `lib.rs` files — verified directly.
- AC8 thorough sweep: read every public item in every file changed by the diff (all 23 files) and checked for single-line or multi-line docs with no `# Examples` block.
- AC9: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps` — exits 0, zero errors/warnings — verified.
- AC11: CI RUSTDOCFLAGS includes `-D missing-docs` — verified (line 47 of ci.yml).
- AC12: `.claude/agents/self-review.md` — both checklist items present — verified.
- AGENTS.md documentation rule — present.

**Verdict:** REJECT

| # | File:line | Severity | Finding | Status |
|---|-----------|----------|---------|--------|
| 1 | quartzite-runtime/src/timer.rs:138 | major | AC8: `Timer::stop` has a single-line doc ("No-op if the timer is not running. Blocks until the background thread exits.") with no `# Examples` block. | ⬜ Open |
| 2 | quartzite-runtime/src/object_ref.rs:24 | major | AC8: `ObjectRef::new` has a single-line doc with no `# Examples` block. | ⬜ Open |
| 3 | quartzite-runtime/src/object_ref.rs:103 | major | AC8: `WeakRef::new` has a single-line doc with no `# Examples` block. | ⬜ Open |
| 4 | quartzite-core/src/signal.rs:255 | major | AC8: `Signal::connect_typed` has a multi-line doc with no `# Examples` block. | ⬜ Open |
| 5 | quartzite-core/src/signal.rs:279 | major | AC8: `Signal::connect_queued` has a multi-line doc with no `# Examples` block. | ⬜ Open |
| 6 | quartzite-core/src/signal.rs:300 | major | AC8: `Signal::connect_auto` has a multi-line doc with no `# Examples` block. | ⬜ Open |
| 7 | quartzite-runtime/src/application.rs:134 | major | AC8: `Application::object_tree()` has a two-line doc ("Returns a reference to the process-wide object tree. Lock the mutex before accessing...") with no `# Examples` block. | ⬜ Open |

## Self-Review (Round 1)

**What was checked:**
- AC1–AC4: `#![deny(missing_docs)]` presence in all four `lib.rs` files — verified directly.
- AC5: `quartzite-macros/src/lib.rs` — all 4 proc-macro functions documented with attribute syntax and `no_run` `# Examples` — verified.
- AC6: `quartzite-core` ID types, `ObjectBase`, `Signal::new`, `ReceiverGuard`, `meta.rs`, `value.rs`, `traits.rs` — all formerly-undocumented items now have docs — verified. `# Examples` presence on non-trivial items spot-checked.
- AC7: `quartzite-runtime` application, object_ref, factory, event_loop, connection_table, thread_pool, timer items — all formerly-undocumented items now have docs — verified.
- AC8: Pre-existing single-line-only doc items across all crates — checked application.rs, signal.rs ConnectionType variants, object_tree.rs, traits.rs, object_ref.rs, timer.rs.
- AC9: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` — exits 0, zero errors/warnings — verified.
- AC10: `cargo test --doc --workspace` — 60 passed, 3 ignored — verified.
- AC11: `.github/workflows/ci.yml` RUSTDOCFLAGS updated — verified.
- AC12: `.claude/agents/self-review.md` — two new checklist items added — verified.
- Checklist §6: new single-line-only docs reviewed for `# Examples` — multiple violations found.

**Verdict:** REJECT

| # | File:line | Severity | Finding | Status |
|---|-----------|----------|---------|--------|
| 1 | quartzite-runtime/src/application.rs:41 | major | AC8: Pre-existing single-line-only doc on `Application::new` has no `# Examples` block. Same applies to `global` (65), `post_event` (70), `exec` (75), `quit` (80). | ✅ Fixed |
| 2 | quartzite-runtime/src/application.rs:92 | major | Checklist §6: Newly added single-line-only doc on `connection_table()` has no `# Examples` block. Same for `event_loop()` (97). | ✅ Fixed |
| 3 | quartzite-runtime/src/object_ref.rs:32 | major | Checklist §6: Newly added single-line-only docs on `ObjectRef::id()`, `ObjectRef::downgrade()` (37), `WeakRef::id()` (88), `WeakRef::is_valid()` (93) have no `# Examples` block. | ✅ Fixed |
| 4 | quartzite-core/src/signal.rs:16 | major | AC8: Pre-existing single-line-only docs on `ConnectionType::Direct` (16), `SingleShot` (18). | ✅ Fixed |
| 5 | quartzite-core/src/traits.rs:25 | major | Checklist §6: Newly added single-line-only docs on `AsObject::object_base()`, `object_base_mut()` (28), `as_any()` (30), `as_any_mut()` (32), and `Object::meta_object()` (40) have no `# Examples` block. | ✅ Fixed |
| 6 | quartzite-runtime/src/timer.rs:133 | major | Checklist §6: Newly added single-line-only doc on `Timer::is_running()` has no `# Examples` block. | ✅ Fixed |
