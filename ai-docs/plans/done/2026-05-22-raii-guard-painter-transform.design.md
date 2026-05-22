# Design: RAII-guard wrapper for `save` / `translate` / `restore` triplet

**Issue:** #410
**Date:** 2026-05-22
**Spec:** `ai-docs/plans/2026-05-22-raii-guard-painter-transform.spec.md`

## Approach

Introduce a new workspace member crate **`quartzite-paint-util`** that hosts a public RAII guard type, **`TranslateGuard<'a>`**, wrapping `&'a mut dyn Painter`. The guard's constructor performs `Painter::save` then `Painter::translate(origin)`; its `Drop` impl performs `Painter::restore`. Access to the wrapped painter inside the guarded scope is exposed via an **explicit accessor method `painter(&mut self) -> &mut dyn Painter`** (NOT `DerefMut`).

`quartzite-style-dispatch::dispatch::visit` replaces the four-line inline triplet at `quartzite-style-dispatch/src/dispatch.rs:160-163` with a guarded scope that calls the recursive `visit(child_id, …)` with `guard.painter()`.

### Why an explicit accessor over `DerefMut<Target = dyn Painter>`

Three reasons:
1. **Object-safety preservation is explicit.** A `DerefMut` impl with `Target = dyn Painter` is unusual (unsized target) and forces every call site to think about deref-coercion semantics through trait objects. An accessor returning `&mut dyn Painter` directly preserves the existing call shape (`visit(…, painter: &mut dyn Painter, …)`) with no surprise around method-resolution autoderef.
2. **No `Deref`/`DerefMut` precedent in the workspace.** Introducing a `DerefMut` here would set a new pattern for one use site — YAGNI.
3. **Reads identically to the existing inline shape.** `visit(child_id, resolver, guard.painter(), palette, style)` mirrors the `visit(child_id, resolver, painter, palette, style)` that preceded it; reviewers diffing the dispatch site see only the wrapping/unwrapping, not a coercion mystery.

The spec (AC3) explicitly authorises either shape as a design-phase pick; the chosen shape is documented on the type per AC3.

### Why a new `quartzite-paint-util` crate

Fixed by Q2 in the spec interview. The guard does not widen `quartzite-paint-api`'s surface (which would force every `Painter` implementor to ship the guard transitively), and `quartzite-style-dispatch` is not made a hub other bridge crates depend on for the guard. The new crate is small (`#![no_std]`, one file `src/lib.rs` with the guard + tests), depends only on `quartzite-paint-api` + `quartzite-geometry` (for `Point` in the constructor signature).

### Module layout inside `quartzite-paint-util`

Single-file: `src/lib.rs` defines `TranslateGuard`, `impl TranslateGuard`, `impl Drop`, and `#[cfg(test)] mod tests`. No sub-modules — file is well under 200 lines, target soft cap. The `#[cfg(test)] mod tests` block carries a local minimal `RecordingPainter` stub (the spec § *Key decisions → Test approach (TDD)* row already notes that re-using `quartzite-paint-api`'s `RecordingPainter` via a `dev-dependencies` cycle is not viable; the stub ships locally).

### Type name: `TranslateGuard`

- Conveys exactly what the guard does (save → translate(origin) → restore on drop).
- Future combinator variants (e.g. clip+translate) — if Issue #410's deferred follow-ups ever land — get sibling names (`ClipRectGuard`, `ClipTranslateGuard`), avoiding a one-true-name lock-in.
- `PainterTransformGuard` was considered and rejected: "transform" is broader than what the guard actually does (translate only — rotation/scale/affine are explicitly out-of-scope per spec § *Out of scope*).

### `#[inline]` posture per AGENTS.md

Per AGENTS.md § *Code Style → `#[inline]` and the `_Simple._` doc tag*:
- `TranslateGuard::new` is one trait call (`save`) + one trait call (`translate`) + a field assignment. **Simple.** Concrete method (inherent impl on a concrete struct) → `#[inline]`.
- `impl Drop for TranslateGuard::drop` is one trait call (`restore`). **Simple.** Concrete-impl trait method → `#[inline]` (cross-crate inlining requires the attribute; `// _Simple._` is not a substitute per AGENTS.md).
- `TranslateGuard::painter` is one field reborrow. **Simple.** Concrete method → `#[inline]`.

### Panic-safety test — `std::panic::catch_unwind` gating

`catch_unwind` is std-only. The new crate is `#![no_std]`. Solution: declare a `std` cargo feature on `quartzite-paint-util` as the **default feature** (`[features] default = ["std"]`, `std = []` — empty implementation). The panic-safety integration test in `tests/panic_safety.rs` is gated with `#![cfg(feature = "std")]` at the file top and runs by default under `cargo test --workspace`.

**Why default feature, not root-crate aggregation:** Appending `quartzite-paint-util/std` to the root `quartzite` crate's `[features] std` aggregation would require adding `quartzite-paint-util` to that crate's `[dependencies]` too — Cargo rejects a feature flag referencing an absent dependency. Since the root `quartzite` crate is not the actual consumer (only `quartzite-style-dispatch` is), adding a root-crate dependency just to route a feature flag is incorrect. Making `std` the default feature instead means `cargo test -p quartzite-paint-util` exercises the panic-safety test without any root-crate changes. `cargo build -p quartzite --no-default-features --features libm` is unaffected because nothing in that build path depends on `quartzite-paint-util`.

### Rejected alternatives

1. **`DerefMut<Target = dyn Painter>` exposure.** Rejected — see *Why an explicit accessor* above. No precedent, and `dyn Painter` as a `Deref::Target` is unusual sugar for one call site.
2. **Closure-passing helper (`with_translate(painter, origin, |p| …)`).** Out of scope per spec § *Out of scope*; recorded in spec § *Deferred*.
3. **Hosting the guard in `quartzite-paint-api`.** Rejected by spec Q2 interview answer. Would widen the API crate's surface and make every `Painter` implementor transitively ship the guard.
4. **Hosting the guard in `quartzite-style-dispatch`.** Rejected by spec Q2. Would make any future bridge-crate consumer depend on `quartzite-style-dispatch` just for the guard.
5. **Generic `TranslateGuard<P: Painter>`.** Rejected — the dispatch call site holds `&mut dyn Painter`; a generic over `P` would require monomorphisation that buys nothing here. Object-safe `&mut dyn Painter` matches the existing `visit` signature exactly.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create new workspace member `quartzite-paint-util`: `Cargo.toml` (workspace inheritance, `[lints] workspace = true`, `[features] default = ["std"]` + `std = []`, deps on `quartzite-paint-api` and `quartzite-geometry` both `default-features = false`), `src/lib.rs` with crate-level `//!` doc, `#![no_std]`, and `extern crate alloc;` (needed for `Vec` in the `#[cfg(test)]` stub). Add `"quartzite-paint-util"` to root `Cargo.toml` `[workspace] members` directly after `"quartzite-paint-api"`. No root-crate `[features]` change — `std` is a default feature of the new crate itself. Verify `cargo build -p quartzite-paint-util` and `cargo build -p quartzite --no-default-features --features libm` both succeed. | `quartzite-paint-util/Cargo.toml` (new), `quartzite-paint-util/src/lib.rs` (new), `Cargo.toml` | — |
| 2 | Author **failing** unit tests in `quartzite-paint-util/src/lib.rs` under `#[cfg(test)] mod tests`: local minimal `RecordingPainter` stub (using `alloc::vec::Vec` via `extern crate alloc` declared at crate level in task 1) + tests `constructor_records_save_then_translate`, `drop_records_exactly_one_restore`, `full_lifecycle_records_save_translate_restore_in_order`, `painter_accessor_returns_same_painter`, `translate_origin_zero`. Tests fail until task 3. | `quartzite-paint-util/src/lib.rs` | 1 |
| 3 | Implement `TranslateGuard<'a>` in `quartzite-paint-util/src/lib.rs`: struct, `new`, `painter` accessor, `Drop::drop`, all with `#[inline]`. Add full doc-comments including: (a) one-line `///` summary; (b) AC3-required sentence noting the `painter()` accessor was chosen over `DerefMut`; (c) AC8-required paragraph explaining the lifetime relationship — the wrapped `&'a mut dyn Painter` is borrowed for the guard's lifetime and re-exposed via `painter(&mut self) -> &mut dyn Painter`; (d) `# Examples` block with an inline `NullPainter`-style stub (per `doc-convention.md` § *Self-sufficiency*, no repo-internal type references; model on `quartzite-paint-api/src/painter.rs` lines 12-44). Verify all task-2 tests pass and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps -p quartzite-paint-util --all-features` passes. | `quartzite-paint-util/src/lib.rs` | 2 |
| 4 | Author panic-safety integration test in `quartzite-paint-util/tests/panic_safety.rs` with `#![cfg(feature = "std")]` at the file top. Test `drop_records_restore_when_scope_panics` uses `std::panic::catch_unwind(AssertUnwindSafe(...))` to verify `Drop` runs during unwind. Because `std` is the default feature, `cargo test -p quartzite-paint-util` (and `cargo test --workspace`) both exercise this test without extra flags. | `quartzite-paint-util/tests/panic_safety.rs` (new) | 3 |
| 5 | Refactor `quartzite-style-dispatch::dispatch::visit` to use `TranslateGuard`. Retain line 159 (`let origin = child.widget_base().geometry.origin();`). Replace lines 160-163 (the `save`/`translate`/`visit`/`restore` triplet) with `{ let mut guard = TranslateGuard::new(painter, origin); visit(child_id, resolver, guard.painter(), palette, style); }`. Add `use quartzite_paint_util::TranslateGuard;` at top of `dispatch.rs`. Add `quartzite-paint-util = { path = "../quartzite-paint-util", default-features = false }` to `quartzite-style-dispatch/Cargo.toml` `[dependencies]`. All 12 existing dispatch tests pass without test-code modification. | `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-style-dispatch/Cargo.toml` | 3 |
| 6 | Final verification gate: run full CI command suite per AGENTS.md § *Build & Test* (`cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`). All must pass. The panic-safety test in `tests/panic_safety.rs` is covered by `cargo test --workspace` because `std` is a default feature of `quartzite-paint-util`. No source edits — gate only. | (no source edits) | 1, 2, 3, 4, 5 |

## Handoff plan

Total subtasks: M = 6, two groups of 3.

- **Group A:** subtasks 1–3 — workspace-member creation, TDD failing tests, guard implementation.
- **Group B:** subtasks 4–6 — panic-safety integration test, dispatch refactor, final CI verification gate.

Each group is dispatched via `/context-reset` per `.claude/skills/context-reset/SKILL.md`. The orchestrator re-validates state between groups (branch match, `base_commit` unchanged, clean working tree) before spawning the next group.

## Risks

- **Risk: borrow-checker rejection of `guard.painter()` re-borrow inside recursive `visit`.** NLL handles the implicit reborrow. **Mitigation:** if rejected, narrow accessor signature to `pub fn painter<'b>(&'b mut self) -> &'b mut (dyn Painter + 'a)` explicitly.
- **Risk: object-safety regression.** The guard wraps `&mut dyn Painter` and adds no methods to the `Painter` trait. **Mitigation:** existing `painter_is_object_safe` test in `quartzite-paint-api/src/painter.rs:203` continues to enforce object-safety.
- **Risk: `cargo doc` failure on the `# Examples` block.** The example must construct a `NullPainter` stub inline (no repo-internal type references per `doc-convention.md` § *Self-sufficiency*). **Mitigation:** model on `quartzite-paint-api/src/painter.rs` `# Examples` stub shape (lines 12-44).
- **Risk: `[workspace] members` placement.** **Mitigation:** slot `quartzite-paint-util` directly after `quartzite-paint-api` in the members list (alphabetical / dependency-order grouping). No root-crate `[features]` change needed — `std` is a default feature of `quartzite-paint-util` itself.
- **Risk: panic-safety test miri behaviour.** `catch_unwind` is std-only but Miri-interpretable. **Mitigation:** if miri rejects the test under tree-borrows, add `#[cfg_attr(miri, ignore = "...")]` per `ai-docs/miri-policy.md`.
- **Risk: dispatch-test byte-identity (AC5) regression.** The event sequence `[Save, Translate(origin), ..., Restore]` must be byte-identical. **Mitigation:** existing `save_translate_restore_wraps_each_non_root_child` test asserts exactly this; failure after task 5 means refactor is wrong, revert in place.

## Test Design

### Task 2 + Task 3 — guard unit tests

- **Location:** `quartzite-paint-util/src/lib.rs` `#[cfg(test)] mod tests`
- **Entry point:** `TranslateGuard::new`, `TranslateGuard::painter`, `<TranslateGuard as Drop>::drop`
- **Scenarios:**
  - `constructor_records_save_then_translate` — before drop, assert `p.events == [Save, Translate(Point::new(3, 4))]`
  - `drop_records_exactly_one_restore` — let-binding goes out of scope, assert exactly one trailing `Restore`
  - `full_lifecycle_records_save_translate_restore_in_order` — combined sequence assertion `[Save, Translate(origin), Restore]`
  - `painter_accessor_returns_same_painter` — call `guard.painter().fill_rect(...)` inside guarded scope; assert `FillRect` event recorded on wrapped painter
  - `translate_origin_zero` — guard with `Point::new(0, 0)` records `Translate(Point::new(0, 0))` (no zero-skipping)
- **Fixtures:** local `RecordingPainter { events: Vec<PaintEvent> }` + local `enum PaintEvent { Save, Restore, Translate(Point), FillRect, Other }`

### Task 4 — panic-safety integration test

- **Location:** `quartzite-paint-util/tests/panic_safety.rs` with `#![cfg(feature = "std")]`
- **Scenarios:**
  - `drop_records_restore_when_scope_panics` — `catch_unwind(AssertUnwindSafe(|| { let _guard = TranslateGuard::new(&mut p, origin); panic!("..."); }))`, assert `result.is_err()` and events end with `Restore`

### Task 5 — dispatch tests stay byte-identical

- **Location:** `quartzite-style-dispatch/src/dispatch.rs` `#[cfg(test)] mod tests` (all 12 existing tests)
- **Requirement:** assertions unchanged; AC5

## Open questions

None. All design-phase picks are resolved:
- Guard type name: `TranslateGuard`
- Access shape: explicit `painter(&mut self) -> &mut dyn Painter` accessor
- Module layout: single-file `quartzite-paint-util/src/lib.rs` + `tests/panic_safety.rs`
