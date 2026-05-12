# Design: Default Style content for Button / Label / TextEdit / ScrollArea

**Issue:** #290
**Spec:** [`2026-05-13-default-style-content.spec.md`](./2026-05-13-default-style-content.spec.md)
**Date:** 2026-05-13

## Approach

Ship one concrete `Style` implementation — `DefaultStyle` — inside `quartzite-style` whose `draw_widget` body routes on the runtime type of `&dyn AsWidget` via `as_any().downcast_ref::<T>()` and dispatches to one private inherent method per supported widget (`Button`, `Label`, `TextEdit`, `ScrollArea`). Unknown widget types are a silent no-op.

### Why `as_any` works straight off `&dyn AsWidget`

The `Extend` macro generates `pub trait AsWidget: ::quartzite_core::AsObject` for `WidgetBase`, and every concrete widget (`Button`, `Label`, `TextEdit`, `ScrollArea`) gets an `impl AsObject` via the derive that supplies `fn as_any(&self) -> &dyn Any { self }`. Therefore on a `widget: &dyn AsWidget` the supertrait method `widget.as_any()` is callable directly — no extra import beyond what `Style` already needs, and no extra trait bound. Confirmed by `cargo expand -p quartzite-widgets --lib widget_base` (`pub trait AsWidget: ::quartzite_core::AsObject`) and `cargo expand -p quartzite-widgets --lib widgets::button` (`impl ::quartzite_core::AsObject for Button { … fn as_any … }`).

### Layout of the new module

A single new file: `quartzite-style/src/default_style.rs`. It contains:

- `pub struct DefaultStyle;` — zero-sized, `#[derive(Default, Clone, Copy, Debug)]`.
- `impl Style for DefaultStyle { fn draw_widget(…) { … } }` — body is the downcast router.
- Private inherent methods on `DefaultStyle`:
  - `fn draw_button(&self, w: &Button, painter: &mut dyn Painter, palette: &Palette)`
  - `fn draw_label(&self, w: &Label, painter: &mut dyn Painter, palette: &Palette)`
  - `fn draw_text_edit(&self, w: &TextEdit, painter: &mut dyn Painter, palette: &Palette)`
  - `fn draw_scroll_area(&self, w: &ScrollArea, painter: &mut dyn Painter, palette: &Palette)`
- Private free helpers in the same module:
  - `fn brush(palette: &Palette, role: ColorRole) -> Brush`
  - `fn disabled(color: Color) -> Color` (multiplies `a` by `0.5` via `Color::with_alpha`)

`lib.rs` adds `mod default_style;` and a single new re-export `pub use default_style::DefaultStyle;`. Doc comment on `DefaultStyle` references `StyleRegistry::set_style` and documents the unknown-widget no-op contract.

### Router body (sketch)

```text
fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
    let any = widget.as_any();
    if let Some(w) = any.downcast_ref::<Button>()     { return self.draw_button(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<Label>()      { return self.draw_label(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<TextEdit>()   { return self.draw_text_edit(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<ScrollArea>() { return self.draw_scroll_area(w, painter, palette); }
    // Unknown widget — deliberate no-op.
}
```

The chain is order-stable, exits on first match, and falls through silently. No early `let-else`/`Option::or` collapse — each arm calls a different method with a different concrete type, so a chain of `if let Some(_) = …` is the most idiomatic shape and minimises noise.

### Per-widget content (matches the spec exactly)

| Widget | Operations recorded on `painter` |
|---|---|
| `Label` | `fill_rect(geom, brush(Window))` then `draw_text_in(geom, &label.text, &widget.widget_base().font, brush(WindowText), label.alignment)`. |
| `Button` (idle, enabled) | `fill_rect(geom, brush(Button))`; `draw_rect(geom, Pen::new(palette.color(ButtonText), 1.0), Brush::solid(Color::TRANSPARENT))`; `draw_text_in(geom, &button.text, &font, brush(ButtonText), Alignment::Center)`. |
| `Button` (checked) | Same as idle but `Button`→`Highlight`, `ButtonText`→`HighlightedText` (both fill and outline-pen). |
| `Button` (disabled) | Same colours as the idle/checked variant for that button, then every brush passed to `fill_rect`/`draw_text_in` has its colour run through `disabled()` (half-alpha). |
| `TextEdit` | `fill_rect(geom, brush(Base))`; `draw_rect(geom, Pen::new(palette.color(Text), 1.0), Brush::solid(TRANSPARENT))`; `draw_text_in(geom, &text_edit.plain_text, &font, brush(Text), Alignment::Left)`. If `read_only`: additional `fill_rect(geom, Brush::solid(disabled(palette.color(Window))))` overlay between background and outline. |
| `ScrollArea` | `fill_rect(geom, brush(Base))`; `draw_rect(geom, Pen::new(palette.color(WindowText), 1.0), Brush::solid(TRANSPARENT))`. No text. No recursion into `content_widget`. |

Font is read once per call as `let font = widget.widget_base().font.clone();` — `Arc<Font>` clone is cheap and lets us pass `&*font` directly to the painter without lifetime gymnastics. (`widget_base().font` is `Arc<Font>`, and `draw_text_in` wants `&Font`.)

### Rejected alternatives

1. **Visitor trait on `Style`** — would force every `Style` impl to support every widget. The source paint-style spec is explicit that per-widget primitives are not on the trait surface. Rejected.
2. **`HashMap<TypeId, fn(&dyn AsWidget, …)>`** — adds a `HashMap` allocation per `DefaultStyle` instance (or a `OnceLock` for a static map), runtime hash lookup, and dyn-fn dispatch on top of the downcast that already happens inside `Any::downcast_ref`. The four-arm chain is simpler, equally cheap (each `downcast_ref` is a single `TypeId` compare), and matches the source spec wording. Rejected.
3. **Auto-installing `DefaultStyle` on first `StyleRegistry::try_style()` miss** — spec § Key decisions resolves this to opt-in. Rejected.
4. **Recording-painter fixture shared with `style.rs` / `registry.rs`** — those modules' `NullPainter`s don't capture call arguments. Lifting them into a shared helper would force a richer return shape on tests that don't need it. Cheaper to write a dedicated `RecordingPainter` inside `default_style.rs`'s `#[cfg(test)]` module that captures the events `DefaultStyle` tests assert on. Rejected (shared fixture).
5. **Returning a fluent builder for `DefaultStyle` (e.g., theme toggles)** — YAGNI. Spec is explicit: zero-sized, no knobs in v1. Rejected.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `mod default_style;` + `pub use default_style::DefaultStyle;` to `lib.rs`. Create empty `default_style.rs` skeleton: doc module comment, `pub struct DefaultStyle;` with `#[derive(Default, Clone, Copy, Debug)]`, public `impl` block stub. | `quartzite-style/src/lib.rs`, `quartzite-style/src/default_style.rs` | — |
| 2 | Private helpers `fn brush(&Palette, ColorRole) -> Brush` and `fn disabled(Color) -> Color`. Inline-marked, `_Simple._` doc tag (both bodies are one expression). | `quartzite-style/src/default_style.rs` | 1 |
| 3 | `impl Style for DefaultStyle` with the downcast-chain router body (no per-widget bodies yet — the four arms call `self.draw_button(…)` etc., each of which is initially `{}`). Verify `cargo build` compiles and the trait-object boxing works. | `quartzite-style/src/default_style.rs` | 2 |
| 4 | Implement `draw_label` (simplest — two painter calls). Add `#[cfg(test)]` `RecordingPainter` fixture + AC3 test. | `quartzite-style/src/default_style.rs` | 3 |
| 5 | Implement `draw_button` (handles `checked`, `is_enabled()`). Tests for AC2, AC7, AC8. | `quartzite-style/src/default_style.rs` | 4 |
| 6 | Implement `draw_text_edit` (handles `read_only`). Test for AC4. | `quartzite-style/src/default_style.rs` | 4 |
| 7 | Implement `draw_scroll_area`. Tests for AC5 (chrome present, no text, no recursion). | `quartzite-style/src/default_style.rs` | 4 |
| 8 | Cross-cutting tests: AC1 (`Send + Sync` on `Box<dyn Style>`), AC6 (unknown-widget no-op against a bare `WidgetBase`), AC10 (round-trip through `StyleRegistry::set_style` / `try_style`). AC9 is enforced by the existing `cargo clippy --workspace -- -D warnings` and rustdoc CI gates — no new test needed, just verify locally before commit. | `quartzite-style/src/default_style.rs` | 5, 6, 7 |

8 tasks, all in one file (plus a single re-export in `lib.rs`). Tasks 4–7 are independent of each other once 3 lands — they can be done in any order or in parallel; the dependency table above lists `4` as a sentinel for the shared `RecordingPainter` fixture (introduced with the first per-widget implementation).

## Risks

- **Disabled-alpha semantics on already-translucent palette colours.** `disabled()` multiplies the existing alpha by `0.5`. With `Palette::default()` every role is opaque (`alpha == 1.0`), so AC8's comparison is well-defined. If a future custom palette sets a translucent colour, `disabled()` still halves — that's the documented contract. *Mitigation:* document the multiplicative behaviour on `disabled` (the helper's doc comment).
- **`Palette::default()` collapses background roles to `Color::WHITE`.** Every background role (`Window`, `Button`, `Base`, `Highlight`, `BrightText`) defaults to `Color::WHITE` in `quartzite-style-types::Palette::default` — only foregrounds (`WindowText`/`ButtonText`/`Text`) and link/highlighted-text roles deviate. Tests that compare *role* colours against each other (e.g. AC7's idle-vs-checked button-fill colour) must construct an explicit palette with `Palette::default().with_role(ColorRole::Highlight, …)` (or any other role under test) — using bare `Palette::default()` would silently degenerate the assertion to `WHITE != WHITE`. *Mitigation:* AC7's test-design entry (above) specifies the explicit palette; future role-equality tests follow the same pattern.
- **Routing chain becomes a maintenance burden** as widgets are added. *Mitigation:* an unknown-type fall-through is a silent no-op, so adding new widgets does not break existing callers; new arms are append-only. The source spec already lists `Container` / `LineEdit` as follow-up extensions.
- **`StyleRegistry` is process-global and the AC10 test mutates it.** Existing `registry.rs` tests use `serial_test::serial` + a `clear_for_test()` helper to avoid cross-test interference. `default_style.rs`'s AC10 test must do the same — annotate with `#[serial]` and call `quartzite_style::registry::clear_for_test()` first. `clear_for_test` is `pub(crate)`, so the test lives in the same crate (`quartzite-style`) — this is satisfied since `default_style.rs` is inside `quartzite-style`. *Mitigation:* test pattern documented; no API change needed.
- **`AsWidget` trait-object dispatch through `as_any` requires concrete widgets to be `'static`.** Every concrete widget in `quartzite-widgets` already satisfies `'static` (no lifetime parameters), so this is a non-issue today. *Mitigation:* none required.
- **Doc gate (`RUSTDOCFLAGS=-D missing-docs`).** Every public item needs a `///` doc. `DefaultStyle`, `DefaultStyle::default`, `Default for DefaultStyle`, and `Style::draw_widget`'s impl are the public surface; per-widget methods stay private. *Mitigation:* enumerated in the file outline above; the `#[derive(Default)]` already supplies `Default`.
- **No API backward-compat concern.** The crate has not been published; new public surface is additive only.
- **No panic / unsafe surface.** Every path returns `()`; no `unwrap()` is needed (the `Arc<Font>::clone()` is infallible; downcast misses fall through cleanly). No `unsafe` block added.

## Test Design

All tests live in `quartzite-style/src/default_style.rs` under `#[cfg(test)] mod tests`. Shared fixture: an in-module `RecordingPainter` that captures every painter method call as a typed event in a `Vec<PaintEvent>`.

### Recording painter fixture

```text
// In default_style.rs, #[cfg(test)] mod tests:

#[derive(Clone, Debug, PartialEq)]
enum PaintEvent {
    DrawRect { rect: Rect, pen: Pen, brush: Brush },
    FillRect { rect: Rect, brush: Brush },
    DrawLine { from: Point, to: Point, pen: Pen },
    ClipRect(Rect),
    Translate(Point),
    Save,
    Restore,
    DrawText  { pos: Point, text: String, font: Font, brush: Brush },
    DrawTextIn { rect: Rect, text: String, font: Font, brush: Brush, alignment: Alignment },
    // No `DrawImage` / `DrawPath` variants — `DefaultStyle` never calls
    // `Painter::draw_image` or `Painter::draw_path` for any supported widget.
    // The `RecordingPainter` impl maps both methods to `unreachable!()` so a
    // future regression that wires either call into `DefaultStyle` fails
    // loudly during the test run (vs. silently being recorded and missed).
}

#[derive(Default)]
struct RecordingPainter { events: Vec<PaintEvent> }

impl Painter for RecordingPainter {
    // Every other method pushes its arguments into `events`.
    // …
    fn draw_image(&mut self, _: Rect, _: &Image) { unreachable!("DefaultStyle never calls draw_image"); }
    fn draw_path(&mut self, _: &Path, _: &Pen, _: &Brush) { unreachable!("DefaultStyle never calls draw_path"); }
}
```

`Font` clones into the event so we keep a snapshot. All AC assertions read from `painter.events` after the call. The `unreachable!()` arms are scoped to `#[cfg(test)]` test-only code (not production), so the AGENTS.md *API Naming* "panicking APIs require explicit user approval" rule does not apply — they exist to surface regressions, not to ship as a library contract.

### Per-AC test layout

- **AC1 — trait-object `Send + Sync`.**
  Location: `default_style.rs` `#[cfg(test)] mod tests`.
  Entry: `assert_send_sync::<Box<dyn Style>>(); assert_send_sync::<DefaultStyle>();`. Construction: `let _b: Box<dyn Style> = Box::new(DefaultStyle);`.
- **AC2 — button → `fill_rect` + centred `draw_text_in("OK")`.**
  Entry: `DefaultStyle::default().draw_widget(&Button::new("OK".into()), …)`. The enabled, idle `Button` path records exactly three events in this order: `FillRect` (background) → `DrawRect` (outline) → `DrawTextIn` (label). The single `FillRect` is unambiguously the background, so the assertion targets `events[0]` (equivalently `first_fill(&events)`): `assert_matches!(events[0], PaintEvent::FillRect { rect, .. } if rect == geom)`. Then assert the **first `DrawTextIn`**: `assert_matches!(first_draw_text_in(&events), PaintEvent::DrawTextIn { text, alignment, .. } if text == "OK" && *alignment == Alignment::Center)`. Helpers: `fn first_fill(events: &[PaintEvent]) -> &PaintEvent` and `fn first_draw_text_in(events: &[PaintEvent]) -> &PaintEvent` (used by AC2/AC4/AC7/AC8/AC10).
- **AC3 — label → `fill_rect` + `draw_text_in` with the label's alignment.**
  Entry: `DefaultStyle::default().draw_widget(&Label::new("hi".into()), …)`. The label path records exactly two events: `FillRect` then `DrawTextIn`. Assert against `first_fill(&events)` and `first_draw_text_in(&events)`: `DrawTextIn { alignment: Alignment::Left, text == "hi", .. }`.
- **AC4 — `TextEdit` with `plain_text == "abc"`.**
  Setup: `let mut e = TextEdit::new(); e.plain_text = "abc".into();`. The `TextEdit` path with `read_only == false` records: `FillRect` (base) → `DrawRect` (outline) → `DrawTextIn`. With `read_only == true` it records: `FillRect` (base) → `FillRect` (read-only overlay) → `DrawRect` (outline) → `DrawTextIn`. AC4 covers `read_only == false`, so the assertion targets **the first `FillRect`**: `assert_matches!(first_fill(&events), PaintEvent::FillRect { brush, .. } if brush_color(brush) == palette.color(ColorRole::Base))` and `assert_matches!(first_draw_text_in(&events), PaintEvent::DrawTextIn { text, .. } if text == "abc")`. (A separate sub-case in this same test covers `read_only == true` and asserts the overlay is `events[1]` — `FillRect` whose brush colour equals `disabled(palette.color(ColorRole::Window))`.)
- **AC5 — `ScrollArea` chrome + no text + no recursion.**
  Entry: `DefaultStyle::default().draw_widget(&ScrollArea::new(), …)`. Assert exactly one `FillRect` and one `DrawRect`; no `DrawText` / `DrawTextIn` events in the vec. Recursion-absence is implicit: `DefaultStyle` never holds a `WidgetResolver`, so no child is ever fetched — assertion is the literal "no extra events fired" check.
- **AC6 — unknown widget = no-op.**
  Entry: `DefaultStyle::default().draw_widget(&WidgetBase::new(), …)`. Assert `painter.events.is_empty()` after the call.
- **AC7 — checked vs. idle button colours differ.**
  **Cannot use `Palette::default()` here** — that constructor seeds *all* background roles with `Color::WHITE`, so `palette.color(ColorRole::Button)` equals `palette.color(ColorRole::Highlight)` and the "checked vs. idle colours differ" assertion would degenerate into `WHITE != WHITE` → `false`. Setup uses an explicit palette with a distinguishable `Highlight`:
  ```text
  let palette = Palette::default()
      .with_role(ColorRole::Highlight, Color::new(0.0, 0.5, 1.0, 1.0));
  // Optionally also override HighlightedText to a non-WHITE for the text-brush variant.
  ```
  Build two buttons with this palette; flip `checked` on one. Compare the brush colour of the first `FillRect` event in each capture via `first_fill(&events)` + `brush_color(&brush)`. Helper: `fn brush_color(b: &Brush) -> Color { match b.kind { BrushKind::Solid(c) => c, _ => unreachable!() } }`. Assert `checked_color != idle_color` *and* — to lock the spec wording — `checked_color == palette.color(ColorRole::Highlight)` and `idle_color == palette.color(ColorRole::Button)`. The stronger pair of equalities catches the regression where someone accidentally swaps `Button`/`Highlight` while still having them differ.
- **AC8 — disabled button halves brush alpha (both fill and text).**
  Setup: same button drawn twice with `Palette::default()` (every role is fully opaque there, so `enabled.a == 1.0` and `disabled.a` must equal `0.5`); second has `set_enabled(false)` applied. Compare alphas of the first `FillRect.brush.kind` and of the `DrawTextIn.brush.kind`. Assert `disabled.a == enabled.a * 0.5` with `f32` equality (the helper math is exact since `0.5` is exact in IEEE 754 and the multiplicand is a finite palette colour).
- **AC9 — clippy + rustdoc.** Enforced by CI; no test artefact. Run `cargo clippy --workspace -- -D warnings` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` locally before commit.
- **AC10 — registry round-trip.**
  Location: `default_style.rs` `#[cfg(test)] mod tests` (so `clear_for_test` is reachable as `crate::registry::clear_for_test`). Annotate with `#[serial]` from `serial_test` (already a `dev-dependency`). Steps: clear registry → `StyleRegistry::set_style(Box::new(DefaultStyle))` → fetch via `StyleRegistry::try_style().unwrap()` → call `draw_widget` → assert the same `FillRect` + `DrawTextIn` events as AC2.

### Fixtures / helpers

- `RecordingPainter::default()` returns an empty-capture painter.
- `fn assert_send_sync<T: Send + Sync>() {}` (compile-time-only).
- `fn first_fill(events: &[PaintEvent]) -> &PaintEvent` — returns the first `FillRect` event (used by AC2/AC4/AC7/AC8/AC10).
- `fn first_draw_text_in(events: &[PaintEvent]) -> &PaintEvent` — returns the first `DrawTextIn` event (used by AC2/AC3/AC4/AC8/AC10).
- `fn brush_color(b: &Brush) -> Color` — local helper that matches `BrushKind::Solid(c) => c`.

### Sample sizes & numbers

The full file is one new `.rs` (`default_style.rs`) with roughly:

- ~30 lines: module docs + struct + Default impl.
- ~10 lines: `brush` / `disabled` helpers.
- ~20 lines: `Style::draw_widget` body (router).
- ~80 lines: four `draw_*` private methods.
- ~250 lines: `#[cfg(test)]` block (recording painter + 10 tests).

≈ 390 lines total — well inside the 200–400 soft target (AGENTS.md *Code Style* → File size). Single `+1` to `lib.rs`. No changes to any other crate.

## Open questions

None of these block implementation; they are flagged for visibility:

- **Scrollbar track / thumb rendering on `ScrollArea`.** Deferred — needs an extra `ColorRole::ScrollBar` slot and a thumb-fraction model. Tracked in the spec's § Deferred. Re-open after #230 (`Slider`) lands.
- **Hover / pressed / focused button states.** Deferred until `WidgetBase` carries the necessary flags (input plumbing pass).
- **`Container` / `LineEdit` arms.** Deliberately omitted from v1; both fall through the unknown-widget no-op until a follow-up plan extends `DefaultStyle`.
- **`Justify` alignment on `Label`.** The painter contract accepts `Alignment::Justify`, but the issue body's only AC for label alignment is "matches the label's stored alignment" — `DefaultStyle` passes the value through unchanged. No code path treats `Justify` specially; the backend decides. Not a blocker.
