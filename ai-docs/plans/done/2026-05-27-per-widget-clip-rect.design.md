# Design: Per-widget clip-rect

**Issue:** #397
**Date:** 2026-05-27

## Approach

Plumb per-widget child clipping through `quartzite_style_dispatch::dispatch_paint` using a new trait method `AsWidget::children_clip_rect(&self) -> Option<Rect>` (default `None`, override on `ScrollArea` via a field-level macro annotation). The dispatcher, when threading from a parent to each visible child, reads the parent's `children_clip_rect()` once and — if `Some(rect)` — inserts a `painter.clip_rect(rect)` call **between** `painter.save()` and `painter.translate(child_origin)` so the rect lives in the **parent's local coordinate frame**. The full per-visible-child sequence is `save() → clip_rect(parent_children_clip_rect) → translate(child_origin) → visit(child) → restore()`. The existing `save`/`restore` envelope already pops both translate and any pushed clip layers atomically (see the `clips`-stack contract in `quartzite-renderer/src/vello_painter.rs`), so no new `save/restore` accounting is needed in the dispatcher.

Five concrete edits realise this:

1. **Macro codegen** (`quartzite-macros/src/extend/codegen.rs::emit_root_trait_and_impl`) — extend the existing `if self_ident == "WidgetBase"` extras block to also emit a third trait method `children_clip_rect(&self) -> Option<#geometry_root::Rect> { None }` with a default body. The root-side `impl AsWidget for WidgetBase` adds no override (the default body suffices). The leading geometry-crate path is produced by a new `geometry_root()` helper in `quartzite-macros/src/util.rs`, mirroring the facade-aware shape of the existing `widgets_root()` / `crate_root()` helpers (resolution order: `quartzite` facade → `::name::geometry`; `quartzite-geometry` → `::name`; fallback → `::quartzite_geometry`). One-time investment; future macro work that needs to reference a `quartzite-geometry::*` type gets the helper for free. The literal-path alternative (`::quartzite_geometry::Rect`) was rejected per the design-review minor finding — inconsistent with the facade-aware helper pattern used everywhere else in this module.

2. **Macro override hook for `ScrollArea`** (field-level annotation). The spec key-decision §Q1 chooses an opt-in field-level annotation `#[clip_rect(method = "<inherent-method-name>")]` on the `WidgetBase` base field; when present (and only when the base field's parent is `WidgetBase`), the macro adds `fn children_clip_rect(&self) -> Option<Rect> { Some(self.<inherent-method-name>()) }` to the macro-generated `impl AsWidget for ScrollArea` block. Rust's trait-coherence rule (E0119) forbids a separate hand-written `impl AsWidget for ScrollArea` block alongside the macro-emitted one, so the override has to land inside the macro emission. The `method = "<ident>"` form is strictly smaller than a free-form `#[clip_rect]` attribute (no expression parsing — just an identifier) and is gated to fire only for the `WidgetBase`-rooted impl. Rejected: free-standing `fn children_clip_rect_dyn(&dyn AsWidget) -> Option<Rect>` that downcasts every widget type — ergonomically worse and breaks the polymorphic-dispatch property AC11 demands.

3. **`ScrollArea` override** (`quartzite-widgets/src/widgets/scroll_area.rs`) — add a `content_rect(&self) -> Rect` inherent method returning `Rect::new(Point::new(0, 0), self.geometry().size())` (v1 zero-inset full local rect). Annotate the `widget_base` field with `#[clip_rect(method = "content_rect")]` so the override emission from edit (2) fires. No hand-written separate `impl AsWidget for ScrollArea` block — the macro produces the override.

4. **Dispatch wiring** (`quartzite-style-dispatch/src/dispatch.rs::visit`) — read `widget.children_clip_rect()` once before the child loop; for each visible child, splice `clip_rect(rect)` between `save()` and `translate(child_origin)` so the rect lives in the **parent's local coordinate frame**. The existing `TranslateGuard::new(painter, origin)` constructor performs `save() + translate(origin)` atomically, with no hook in between — to honour the corrected order `save → clip_rect → translate`, the design extends `TranslateGuard` with a new constructor `TranslateGuard::with_clip(painter, origin, clip)` that does `save() → clip_rect(clip) → translate(origin)` internally, returning the existing guard shape (so `restore()` still fires on drop). The existing `TranslateGuard::new` constructor is unchanged (used for non-clipping parents). Dispatch logic in `visit`:

   ```text
   let maybe_clip = widget.children_clip_rect();
   for child in visible children {
       let mut guard = match maybe_clip {
           Some(rect) => TranslateGuard::with_clip(painter, origin, rect),
           None       => TranslateGuard::new(painter, origin),
       };
       visit(child_id, resolver, guard.painter(), palette, style);
   }
   ```

   Extend the test-only `PaintEvent` enum (recording-painter shape in `dispatch.rs` tests) with a `ClipRect(Rect)` variant; have `RecordingPainter::clip_rect` push it. Add AC5–AC11 tests covering: content present, content absent, hidden ScrollArea, hidden content child, non-clipping container, save/restore-pairing with `ClipRect` events interleaved, and a test-only `ClippingWidget` fixture proving AC11 generalises beyond `ScrollArea`.

5. **Snapshot test (AC12)** — add `scroll_area_clips_oversized_content_renders` in `quartzite-style/tests/snapshots.rs`, mirroring the existing `scroll_area_chrome_renders` shape but routing through `dispatch_paint` (with a small inline HashMap-backed `WidgetResolver`) rather than direct `DefaultStyle::draw_widget`. The `Label` content's geometry extends well beyond the 64×64 canvas; the committed golden shows clipping at the `ScrollArea`'s edge. Snapshot lives in `quartzite-style/tests/snapshots.rs` because the GPU-snapshot infrastructure already lives there.

### Rejected alternatives

- **Trait method on `Style` instead of `AsWidget`.** Puts clipping in the style layer rather than the widget layer. Rejected — `ScrollArea` clips regardless of theme; clipping is widget identity, not styling.
- **Always clip every child at the parent's geometry.** Needless overdraw cost and breaks widgets that intentionally paint outside their bounds (focus rings, drop shadows once they land).
- **New `Painter::push_clip` method.** Spec §Q8 closes this — `clip_rect` already exists end-to-end.
- **Field-level `#[clip_rect]` taking a free expression.** Heavier parse path for the same single-overrider use-case. The `method = "<ident>"` form is the minimum.
- **Hand-written second `impl AsWidget for ScrollArea` block** as the spec's earlier Q1 wording literally suggested. Rejected — Rust coherence (E0119) forbids two `impl Trait for Type` blocks; this is why the spec moved to the field-annotation form, and no hand-written impl remains in the design.
- **Literal absolute `::quartzite_geometry::Rect` path emitted by the macro.** Rejected per design-review minor finding — inconsistent with the facade-aware `widgets_root()` / `crate_root()` pattern; the small `geometry_root()` helper is the consistent shape.
- **Inline `painter.save(); painter.clip_rect(rect); painter.translate(origin); …; painter.restore();` in `visit` (bypassing `TranslateGuard`).** Rejected — duplicates the guard's RAII save/restore in two branches and makes the `save`/`restore` pairing invariant harder to read. The `TranslateGuard::with_clip` constructor centralises the contract.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `geometry_root()` helper in `quartzite-macros/src/util.rs` (facade-aware, mirroring `widgets_root()`). Extend `Extend`-macro to (a) emit `children_clip_rect(&self) -> Option<#geometry_root::Rect> { None }` on the `WidgetBase`-rooted `AsWidget` trait and (b) parse an opt-in `#[clip_rect(method = "<ident>")]` field-level annotation on the `WidgetBase` base field, gated to emit the override only when the parent is `WidgetBase`. Add codegen unit tests for default-body emission, override emission, override absence, rejection on non-`WidgetBase` parents, plus the `geometry_root_*` tests (facade-itself / facade-name / geometry-itself / geometry-name / fallback) mirroring the existing `widgets_root_*` test set. | `quartzite-macros/src/util.rs`, `quartzite-macros/src/extend/parse.rs`, `quartzite-macros/src/extend/codegen.rs` | — |
| 2 | Add `ScrollArea::content_rect()` (inherent, `Rect::new(Point::new(0, 0), self.geometry().size())`) and annotate the `widget_base` field with `#[clip_rect(method = "content_rect")]` so the override emission from subtask 1 fires. Unit tests for `content_rect()` (zero geometry and non-zero geometry), `ScrollArea::children_clip_rect()` returning the expected `Some(rect)`, and a representative built-in (e.g. `Button`) returning `None` (covers AC2/AC3/AC4). | `quartzite-widgets/src/widgets/scroll_area.rs`, `quartzite-widgets/src/widgets/button.rs` (tests-only addition) | 1 |
| 3 | Add `TranslateGuard::with_clip(painter, origin, clip)` constructor in `quartzite-paint-util/src/lib.rs` (calls `save() → clip_rect(clip) → translate(origin)`; existing `Drop` impl already calls `restore()`). Extend `dispatch.rs::visit` to read `widget.children_clip_rect()` once before the child loop and select between `TranslateGuard::new` (no clip) and `TranslateGuard::with_clip` (clip present) per child — so the per-visible-child sequence becomes `save() → clip_rect(parent_clip_rect) → translate(child_origin) → visit → restore()`. Add `ClipRect(Rect)` to the in-test `PaintEvent` enum and have `RecordingPainter::clip_rect` push it. Update existing event-filter lists to keep ignoring `ClipRect` where appropriate. Add AC5–AC11 dispatch tests, including a test-only `ClippingWidget` fixture for AC11. Add a `with_clip_emits_save_clip_translate` unit test in `quartzite-paint-util` covering the new constructor's event order. | `quartzite-paint-util/src/lib.rs`, `quartzite-style-dispatch/src/dispatch.rs` | 2 |
| 4 | Add AC12 end-to-end snapshot test — `scroll_area_clips_oversized_content_renders` — routing through `dispatch_paint` with a small inline `WidgetResolver` setup; commit a new golden PNG under `quartzite-style/tests/snapshots/shared/scroll_area_clips_oversized_content.png`. | `quartzite-style/tests/snapshots.rs`, `quartzite-style/tests/snapshots/shared/scroll_area_clips_oversized_content.png` (new fixture) | 3 |

## Handoff plan

- **Handoff entering Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Group A:** subtasks 1–3 — macro codegen + `geometry_root()` helper + parse changes (1), `ScrollArea` `content_rect()` + clip-rect annotation + unit tests (2), `TranslateGuard::with_clip` + dispatch wiring + recording-painter `ClipRect` variant + AC5–AC11 dispatch unit tests (3). 3 subtasks (non-terminal group at the cap).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtask 4 — AC12 end-to-end Vello snapshot + committed golden PNG. Terminal group (1 subtask; within the 1..=3 range).

## Risks

- **First geometry-type reference emitted by the `Extend` macro.** Mitigation: add a small `geometry_root()` helper in `quartzite-macros/src/util.rs` mirroring `widgets_root()` (facade → `::name::geometry`, `quartzite-geometry` → `::name`, fallback → `::quartzite_geometry`). Documented in a codegen comment and tested by a parallel `geometry_root_*` set in the existing util test module.
- **Doctest preamble drift.** Mitigation: do **not** enumerate `children_clip_rect` in the preamble (matches precedent — the existing `widget_view` / `children` extras aren't enumerated either). Subtask 1 verifies via doc gate.
- **`RecordingPainter` regression from non-no-op `clip_rect`.** Mitigation: explicitly update `matches!` filter in affected tests so unrelated tests stay green; new AC5–AC11 tests explicitly assert `ClipRect` presence/absence.
- **Coordinate-frame ordering — the clip rect lives in the parent's local frame.** The corrected sequence is `save() → clip_rect(parent_children_clip_rect) → translate(child_origin) → visit → restore()`. `clip_rect` is issued AFTER the parent's translate is already in effect (the parent's own paint ran in its own translated frame) and BEFORE the child's `translate` runs against the painter — so the rect argument lives in the same coordinate space the parent paints in (origin `(0, 0)` at the parent's top-left). Mitigation: centralised in `TranslateGuard::with_clip(painter, origin, clip)`, which calls `save() → clip_rect(clip) → translate(origin)` in fixed order; AC5 asserts the exact event sequence (`Save → ClipRect → Translate(child_origin) → child paints → Restore`); AC10 asserts the `Save/Restore` pairing invariant survives the inserted `ClipRect` event.
- **Visibility short-circuit (spec §Q6) — unchanged.** The existing `WidgetState::Visible` check on each child sits BEFORE `save()` (and therefore before `TranslateGuard::new` / `TranslateGuard::with_clip`). Because the new `clip_rect` call now lives inside `TranslateGuard::with_clip` — i.e. AFTER `save()` and BEFORE `translate(child_origin)` — and the entire guard is only constructed for visible children, no clip layer is pushed for an invisible child. The visibility short-circuit therefore continues to prevent any clip from being pushed for hidden children (verified by AC7/AC8).
- **Resolver-miss (spec §Q7) — already correct.** `resolver.resolve(child_id)` returning `None` `continue`s before the `save/translate/clip_rect/restore` envelope.
- **Snapshot-helper sync group impact.** New snapshot test uses a test-local resolver; neither `quartzite-widgets/tests/support/mod.rs` nor `quartzite-style/tests/support/mod.rs` needs editing.

## Test Design

### Subtask 1 — macro codegen + `geometry_root()` helper + parse
- Location: `quartzite-macros/src/util.rs` `#[cfg(test)] mod tests` (new `geometry_root_*` tests); `quartzite-macros/src/extend/codegen.rs` `#[cfg(test)] mod tests`; parse-side rejection test under `quartzite-macros/src/extend/parse.rs`.
- Scenarios (helper):
  - `geometry_root_facade_itself` — `(Some(FoundCrate::Itself), None)` → `crate :: geometry`.
  - `geometry_root_facade_name` — `(Some(FoundCrate::Name("my_quartzite".into())), None)` → `:: my_quartzite :: geometry`.
  - `geometry_root_geometry_itself` — `(None, Some(FoundCrate::Itself))` → `crate`.
  - `geometry_root_geometry_name` — `(None, Some(FoundCrate::Name("quartzite_geometry".into())))` → `:: quartzite_geometry`.
  - `geometry_root_fallback` — `(None, None)` → `:: quartzite_geometry`.
- Scenarios (codegen + parse):
  - `widget_base_root_emits_children_clip_rect_default_none` — output contains `fn children_clip_rect`, `Option`, the resolved `Rect` path, and `None`.
  - `non_widget_base_root_no_children_clip_rect` — output does **not** contain `children_clip_rect`.
  - `widget_base_self_impl_no_clip_rect_override` — self-ref impl does **not** override `children_clip_rect`.
  - `clip_rect_annotation_emits_override` — annotated `ScrollArea.widget_base` → emitted impl contains `fn children_clip_rect` returning `Some(self.content_rect())`.
  - `clip_rect_annotation_absent_no_override` — annotation omitted → no `children_clip_rect` in the emitted impl.
  - `clip_rect_annotation_on_non_widget_base_parent_rejected` — annotation on a non-`WidgetBase` base field → parse error.

### Subtask 2 — ScrollArea content_rect + children_clip_rect
- Location: `quartzite-widgets/src/widgets/scroll_area.rs` tests; `quartzite-widgets/src/widgets/button.rs` tests.
- Scenarios:
  - `content_rect_zero_geometry` — default `ScrollArea` → `content_rect() == Rect::new(Point::new(0, 0), Size::default())`.
  - `content_rect_nonzero_geometry` — `set_geometry(Rect::new(Point::new(10, 20), Size::new(100, 50)))` → `content_rect() == Rect::new(Point::new(0, 0), Size::new(100, 50))`.
  - `scroll_area_children_clip_rect_returns_some` — `area.children_clip_rect() == Some(area.content_rect())`.
  - `button_children_clip_rect_returns_none` — `Button::new("…".into()).children_clip_rect().is_none()`.

### Subtask 3 — `TranslateGuard::with_clip` + dispatch wiring + AC5–AC11
- Location: `quartzite-paint-util/src/lib.rs` `#[cfg(test)] mod tests` (new constructor); `quartzite-style-dispatch/src/dispatch.rs` `#[cfg(test)] mod tests` (dispatch wiring).
- Scenarios (paint-util):
  - `with_clip_emits_save_clip_translate` — `TranslateGuard::with_clip(&mut painter, Point::new(5, 10), Rect::new(Point::new(0, 0), Size::new(40, 40)))` records `[Save, ClipRect(rect), Translate(Point(5, 10))]` on construction; drop appends `Restore`.
- Scenarios (dispatch):
  - `scroll_area_with_content_emits_clip_rect_between_save_and_translate` (AC5) — event sequence `[FillRect(area), Save, ClipRect(content_rect), Translate(label_origin), FillRect(label), Restore]`; the `ClipRect` event sits AFTER `Save` and BEFORE `Translate(label_origin)`, with rect equal to `scroll_area.content_rect()`.
  - `scroll_area_without_content_emits_no_clip_rect` (AC6)
  - `hidden_scroll_area_emits_no_clip_rect` (AC7) — visibility short-circuit returns before any `Save`/`ClipRect`/`Translate` is recorded.
  - `hidden_content_under_scroll_area_emits_no_clip_rect` (AC8) — child's visibility check sits before `TranslateGuard::with_clip`, so no `ClipRect` is recorded for the hidden child.
  - `container_emits_no_clip_rect` (AC9)
  - `clip_rect_pairs_with_save_restore` (AC10) — every `Save` has a matching `Restore`; every `ClipRect` sits between a `Save` and the following `Translate(child_origin)` and is therefore inside a `Save`/`Restore` pair.
  - `custom_clipping_widget_emits_clip_rect` (AC11) — test-only `ClippingWidget` overrides `children_clip_rect` and records the same `Save → ClipRect → Translate` ordering on dispatch.
- Fixtures: extend `PaintEvent` with `ClipRect(Rect)`; hand-roll test-only `ClippingWidget`.

### Subtask 4 — AC12 snapshot
- Location: `quartzite-style/tests/snapshots.rs`.
- Scenarios:
  - `scroll_area_clips_oversized_content_renders` — `ScrollArea` at full canvas with oversized `Label` content → golden shows clipping at `ScrollArea`'s edge.
- Fixtures: small inline HashMap-backed `WidgetResolver`; golden PNG at `quartzite-style/tests/snapshots/shared/scroll_area_clips_oversized_content.png`.

## Open questions

None remaining — all four open questions from the prior design pass were resolved by the spec amendment and the design-review minor finding:

- **(resolved)** Macro override mechanism for `ScrollArea` — opt-in `#[clip_rect(method = "<ident>")]` field annotation per spec §Q1; no hand-written `impl AsWidget for ScrollArea` block remains.
- **(resolved)** `geometry_root()` helper vs literal `::quartzite_geometry::Rect` — design adds the facade-aware `geometry_root()` helper in `quartzite-macros/src/util.rs` per design-review minor finding.
- **(resolved)** AC12 snapshot home — `quartzite-style/tests/snapshots.rs` per spec § Open questions row 1.
- **(resolved)** Doctest preamble enumeration for `children_clip_rect` — not enumerated; matches the existing `widget_view` / `children` extras precedent.
