# RAII-guard wrapper for `save` / `translate` / `restore` triplet

**Source:** issue #410 (surfaced by `/triage` from `ai-docs/deferred/widget-backlog.md`)
**Date:** 2026-05-22
**Tracked in:** #410

## Context

`quartzite-style-dispatch::dispatch::visit` (`quartzite-style-dispatch/src/dispatch.rs:160-163`) is the only in-tree site that uses the coordinate-transform triplet:

```rust
painter.save();
painter.translate(origin);
visit(child_id, resolver, painter, palette, style);
painter.restore();
```

The origin design (`2026-05-13-renderer-style-dispatch.design.md` § *Open questions*, last bullet) rejected an RAII-guard wrapper on YAGNI grounds and flagged it as viable later "if more bridge crates need the same shape". Issue #410 promotes that follow-up. Today there is still exactly one in-tree call site, but the product owner has chosen to proceed now (Q1 answer in § *Decisions reached during interview*) and ship the guard as a public type ahead of the second consumer, with the type living in a brand-new utility crate (Q2 answer).

The only other workspace uses of `save` / `translate` / `restore` outside the trait definition itself are:
- `quartzite-renderer/src/vello_painter.rs` — `Painter` impl + its own unit tests (`save_then_restore_round_trip`, `translate_modifies_top_only`, etc.) — these are testing the painter primitives, not the coordinate-transform-bridge pattern.
- `quartzite-widgets/tests/snapshots.rs` — `translate_save_restore`, `clip_rect_save_restore` — snapshot tests of painter behaviour.

Neither is a bridge-crate consumer; both stay untouched by this task.

## Scope

1. **New workspace crate `quartzite-paint-util`** — a small, `no_std`-compatible utility crate alongside `quartzite-paint-api`. Added to the root `[workspace] members` list. Same `[workspace.package]` inheritance (version / edition / rust-version / authors / license / repository) as the other workspace crates. Workspace-wide `[lints] workspace = true` opt-in.
2. **Public RAII-guard type** in `quartzite-paint-util` that wraps a `&mut dyn quartzite_paint_api::Painter` for the lifetime of the guard:
   - The constructor calls `Painter::save` and `Painter::translate(origin)` on the borrowed painter in that order.
   - The guard's `Drop` impl calls `Painter::restore` on the borrowed painter exactly once.
   - The guard exposes the wrapped painter to the body of the guarded scope (concrete shape — `DerefMut<Target = dyn Painter>` vs. an accessor method like `as_painter_mut(&mut self) -> &mut dyn Painter` — left to design phase; both preserve object safety of `Painter`).
3. **Refactor `quartzite-style-dispatch::dispatch::visit`** to use the guard. The four-line inline triplet at `quartzite-style-dispatch/src/dispatch.rs:160-163` collapses to a guarded scope; the recursive `visit(child_id, …)` call receives the painter via the guard.
4. **Add `quartzite-paint-util` as a dependency** of `quartzite-style-dispatch` (workspace-relative path dep, like every other intra-workspace edge). No version bump needed on either crate.
5. **Tests** for the guard live alongside it in `quartzite-paint-util` (unit tests under `#[cfg(test)] mod tests`). At minimum: constructor sequence (`Save` then `Translate(origin)`), `Drop` calls `Restore` exactly once, panic-in-scope still triggers `Restore`. Existing dispatch tests in `quartzite-style-dispatch/src/dispatch.rs` (`save_translate_restore_wraps_each_non_root_child` etc.) continue to assert the byte-exact `PaintEvent` sequence — they MUST keep passing without modification.
6. **Doc-comment + `# Examples`** on the guard type per AGENTS.md § *Code Style → Documentation*. The example demonstrates a single guarded recursion frame using a `RecordingPainter`-style stub.

## Out of scope

- Changing the `Painter` trait's `save` / `translate` / `restore` method signatures or contracts. The triplet stays a primitive on the trait; the guard is a thin RAII wrapper around it.
- Reworking the dispatch traversal algorithm (parent-before-child order, visibility filter, resolver-miss path are all unchanged).
- Adding a `clip_rect` parameter to the guard. `clip_rect` is `quartzite-paint-api/src/painter.rs:77` and is paired with `save`/`restore` in `clip_rect_save_restore` snapshot tests, but the dispatch crate doesn't use it (out of v1 scope per `2026-05-13-renderer-style-dispatch.spec.md` § *Out of scope*). If a future caller wants a `(save + clip_rect + restore)` guard, that's a separate task.
- Introducing additional combinators (rotation, scale, full affine) on the guard. The dispatched coordinate transform is translation-only by spec.
- Adding a closure-passing helper (e.g. `Painter::with_translate(origin, |p| …)`). This task ships the explicit RAII guard — see § *Decisions reached during interview*, Q3.
- Moving any code out of `quartzite-paint-api` into `quartzite-paint-util`. The new crate is purely additive; `Painter` and its primitives stay where they are.
- Migrating the `vello_painter.rs` or `quartzite-widgets/tests/snapshots.rs` `save`/`translate`/`restore` sites. They are not bridge-crate-pattern consumers (see § *Context*).

## Deferred

- A second bridge-crate caller's actual translate-pattern needs (e.g. nested clip + translate, or scale + translate). | We don't have one yet; speculative additional combinators without a real consumer risk YAGNI. | Will re-evaluate when a second bridge crate is filed.
- Closure-passing convenience helper (`Painter::with_translate(…)` or `quartzite_paint_util::with_translate(painter, origin, |p| …)`). | The product owner chose the explicit-guard shape (per the issue title's "RAII-guard" wording, Q3 below); a closure helper is a separate ergonomics task if it ever lands. | New issue if needed.

## Key decisions

| Question | Decision |
|---|---|
| Pre-publish API freedom | Per AGENTS.md § *API Stability*: the existing inline `save`/`translate`/`restore` calls in `dispatch.rs` are replaced outright (clean rename / restructure). |
| Naming convention | Per AGENTS.md § *API Naming*: panic-free constructor (`new` or `begin`) returning the guard; `restore` happens in `Drop`. `try_*` is not needed (no fallible path — `Painter::save` and `Painter::translate` are infallible methods returning `()`). Specific guard-type name (e.g. `TranslateGuard`, `PainterTransformGuard`) is a design-phase pick. |
| Tracing | Per AGENTS.md § *Code Style → Tracing*: the guard's `new` / `Drop` are mechanical wrappers around existing `Painter` calls — no new span. The enclosing `dispatch_paint` already has its `debug_span!`. |
| Test approach (TDD) | Per AGENTS.md § *Workflow*: behavioural test asserts the recorded `PaintEvent` sequence (`Save`, `Translate(origin)`, …, `Restore`) matches the pre-refactor shape exactly. The existing `RecordingPainter` in `quartzite-style-dispatch/src/dispatch.rs:191` already records `Save` / `Translate` / `Restore` — the dispatch-side tests stay byte-identical. New unit tests for the guard itself live in `quartzite-paint-util` and use either a local `RecordingPainter` stub or the one already in `quartzite-paint-api/src/painter.rs:149` (re-used through a `dev-dependencies` cycle is not viable; the guard crate ships its own minimal stub). |
| Guard mutability | The guard borrows `&mut dyn Painter` for its lifetime. Re-borrows are sound under NLL: the guard's `DerefMut<Target = dyn Painter>` (or an accessor method, design-phase pick) lets the recursive `visit` call continue to use the painter through the guard. |
| `#[inline]` posture | Per AGENTS.md § *Code Style → `#[inline]` and the `_Simple._` doc tag*: constructor is one trait call + one trait call + a field assignment — simple; `Drop::drop` is one trait call — simple. Both get `#[inline]` (concrete impl methods, cross-crate inlining requires the attribute). |
| Object safety | The guard wraps `&mut dyn Painter`, not a generic `P: Painter`. `Painter` remains object-safe (no new methods added; the guard lives in a separate crate). |
| `no_std` posture | `quartzite-paint-util` is `#![no_std]` with no `alloc` need (the guard holds one `&mut` reference + one `Point` `Copy` value). No features needed beyond what `quartzite-paint-api` already exposes; no `libm` feature flag at the util-crate level (the guard does no float math). |
| Q1 — trigger condition | **Proceed now (public).** Build the guard against the single existing caller as both a clean-up of the inline triplet and future-proofing for a second consumer. (See § *Decisions reached during interview*.) |
| Q2 — guard location | **New utility crate `quartzite-paint-util`.** Cleanest separation; the type does not widen `quartzite-paint-api`'s surface, and `quartzite-style-dispatch` does not become a hub that future bridge crates must depend on for the guard. Adds one workspace member. |
| Q3 — call shape | **Explicit RAII guard.** Per the issue title literally specifying "RAII-guard" and AGENTS.md § *Code Style → Rust idioms* (RAII is the idiomatic Rust pattern for paired enter/leave operations with panic safety). Closure-passing (`with_translate(origin, |p| …)`) is a different shape — not an RAII guard — and is recorded in *Deferred* as a separate task if anyone wants it. Resolved as a Key Decision rather than asked: the design agent could resolve this by convention but the decision lands here for record. |

## Technical constraints

- Object-safe `Painter` (`quartzite-paint-api/src/painter.rs:45`) must remain object-safe — the guard does not add generic methods to the trait.
- `#![no_std]` compatibility — `quartzite-paint-api` is `no_std`-friendly (see `quartzite-paint-api/Cargo.toml`); `quartzite-paint-util` is `#![no_std]` likewise. The guard type and its `Drop` impl must be `no_std`-compatible.
- `Drop` ordering: panic safety — if the wrapped body panics, `Drop` must still call `restore`. Standard RAII guarantees this; no special handling needed.
- The `visit` recursion currently passes `painter: &mut dyn Painter` down. With the guard, the recursion either receives `&mut dyn Painter` extracted from the guard (e.g. `&mut *guard` via `DerefMut`) or via an `as_painter_mut(&mut self) -> &mut dyn Painter` accessor — design phase picks one.
- Workspace lint policy: `quartzite-paint-util` opts in via `[lints] workspace = true` — `missing_docs = "deny"`, `rustdoc::broken_intra_doc_links = "deny"`, `clippy::undocumented_unsafe_blocks = "deny"` (the guard has no `unsafe`, but the lint applies workspace-wide).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | A new workspace crate `quartzite-paint-util` exists at `/quartzite-paint-util/` with `Cargo.toml` inheriting from `[workspace.package]`, `[lints] workspace = true`, `#![no_std]` `src/lib.rs`, and a `Painter`-dependency on `quartzite-paint-api` via the workspace path. The new crate is added to the root `Cargo.toml` `[workspace] members` list. |
| AC2 | The crate defines a public RAII-guard type whose constructor takes `&mut dyn Painter` plus the translation origin (`quartzite_geometry::Point` — the same type `Painter::translate` accepts) and calls `Painter::save` then `Painter::translate(origin)` in that order. The guard's `Drop` impl calls `Painter::restore` on the wrapped painter exactly once. |
| AC3 | The guard exposes the wrapped painter to the guarded body via `DerefMut<Target = dyn Painter>` **or** a `&mut dyn Painter` accessor method (design-phase pick); the chosen shape is documented on the type. |
| AC4 | `quartzite-style-dispatch::dispatch::visit` (currently `dispatch.rs:160-163`) uses the new guard instead of the inline `save` / `translate` / `restore` triplet. The recursive `visit(child_id, …)` call passes the painter via the guard. `quartzite-style-dispatch/Cargo.toml` adds `quartzite-paint-util` as a workspace-path dependency. |
| AC5 | The recorded `PaintEvent` sequence in every existing dispatch test (e.g. `save_translate_restore_wraps_each_non_root_child`, `depth_first_parent_before_child_order`, `hidden_subtree_skipped_with_no_save_or_translate`, `dispatch_paint_invokes_draw_widget_once_per_visible_widget`) is byte-identical to the pre-refactor sequence. Tests pass without modification of their assertions. |
| AC6 | `quartzite-paint-util` ships unit tests under `#[cfg(test)] mod tests` covering: (a) constructor order — `save` recorded then `translate(origin)` recorded; (b) `Drop` records exactly one `restore`; (c) panic inside the guarded scope still records `restore` (verify via `std::panic::catch_unwind` — guarded by `#[cfg(feature = "std")]` or under a `[dev-dependencies]` setup that pulls `std`, since `catch_unwind` is std-only). |
| AC7 | The guard's constructor and `Drop::drop` carry `#[inline]` per AGENTS.md § *Code Style → `#[inline]` and the `_Simple._` doc tag* (both are simple). |
| AC8 | The guard type has a one-line `///` doc summary, a `# Examples` block demonstrating one guarded recursion frame, and a doc-comment paragraph explaining the lifetime relationship to the wrapped `&mut dyn Painter`. |
| AC9 | `cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, and `cargo build -p quartzite --no-default-features --features libm` all pass. |

## Decisions reached during interview

| # | Question | Answer | Round |
|---|----------|--------|-------|
| Q1 | "Issue #410 is conditional on a second bridge crate appearing; today there is still exactly one in-tree caller of the save/translate/restore triplet. How should this task proceed?" | **Proceed now (public)** — build the RAII guard as a public type now, against the one existing caller; future-proof for a second consumer. | 1 |
| Q2 | "If a public guard is built (proceed-now option in Q1), where does it live?" | **New utility crate (e.g. `quartzite-paint-util`)** — separate small crate, cleanest separation; adds one workspace member. | 1 |

## Open questions

None remaining for the spec — design-phase picks (guard type name, `DerefMut` vs. accessor method, exact module layout inside `quartzite-paint-util`) are recorded in § *Technical constraints* and § *Key decisions* as design-phase decisions, not blocking ambiguities.

```yaml
---
status: ready
round: 2
---
```
