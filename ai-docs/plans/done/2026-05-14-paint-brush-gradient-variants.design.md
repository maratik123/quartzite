# Design: Paint Brush Gradient Variants

**Issue:** #281
**Date:** 2026-05-14

## Approach

Three-variant gradient addition to `BrushKind`: `LinearGradient`, `RadialGradient`, and `Custom(peniko::Gradient)`. The spec pre-decides all structural choices (field names, Copy removal, peniko-in-api, Pad-only for 2-stop, pass-through for Custom). The design resolves implementation-level choices that the spec deferred.

### Resolved design-phase questions

**`Brush::kind` return type after Copy removal**

`Brush::kind(&self) -> &BrushKind` (borrow). This is the natural fit — `Color` and `Point` fields are `Copy` so callers do `let &BrushKind::Solid(c) = brush.kind()` or `match brush.kind() { BrushKind::Solid(c) => *c, ... }`. The `const` qualifier is dropped because `const fn` cannot return a reference with a non-static lifetime (Rust limitation). `#[inline]` is kept.

**`brush_to_peniko` return type**

`fn brush_to_peniko(&self, brush: &Brush) -> Option<peniko::Brush>` returning an owned `peniko::Brush`. The gradient arms must construct `peniko::Gradient` (which owns a `SmallVec`), so a reference return is not possible without storing the gradient somewhere. The owned form is the only viable option. `peniko::Brush` is `Clone` (the enum holds a `Gradient` which is `Clone`) — the `Option<peniko::Brush>` is returned, then used with `scene.fill(..., &brush, ...)` which accepts `impl Into<peniko::BrushRef<'_>>` (via `&peniko::Brush → BrushRef` impl).

**`peniko::DynamicColor` construction for `ColorStop`**

`quartzite_paint_api::Color` (f32 r/g/b/a sRGB) maps to `peniko::Color` (`AlphaColor<Srgb>`) via the existing `color_to_peniko(c)` helper. `DynamicColor::from_alpha_color(peniko::Color)` (from `peniko::color::DynamicColor::from_alpha_color`) produces the `DynamicColor` that `ColorStop.color` requires.

**peniko `Brush::Gradient` variant construction for 2-stop helpers**

Use the builder API (`new_linear` / `with_stops`) rather than the struct-literal + `..default()` form — the builder is shorter, avoids accidentally missing `interpolation_cs` / `hue_direction` fields, and matches peniko idiom:

```
let stop0 = peniko::ColorStop { offset: 0.0, color: color_to_dynamic(start_color) };
let stop1 = peniko::ColorStop { offset: 1.0, color: color_to_dynamic(end_color) };
peniko::Brush::Gradient(
    peniko::Gradient::new_linear(self.scale_pt(start), self.scale_pt(end))
        .with_stops([stop0, stop1])
)
```

Similarly for `RadialGradient` — use `Gradient::new_radial(centre, radius)` (or `new_two_point_radial` for the single-circle collapse) then `.with_stops([stop0, stop1])`.

A private helper `color_to_dynamic(c: Color) -> peniko::color::DynamicColor` is added inside `VelloPainter`'s `impl` block.

**`draw_text` / `draw_text_in` gradient handling**

Both `draw_text` and `draw_text_in` call `brush_to_peniko` (previously `brush_color`). When the result is `Some(peniko::Brush::Gradient(_))`, the methods continue the current silent-skip behaviour: `emit_layout_glyphs` requires a `peniko::Color` (solid), so a gradient brush falls through to `return` (no text drawn). This is the same as the current `_ => None` arm and is intentional — text with a gradient brush is a deferred feature. Implementer must not attempt to pass a `peniko::Brush::Gradient` to `emit_layout_glyphs`; extract the `Solid` arm only, or continue to use the existing `brush_color` helper just for the text methods.

**`quartzite-paint` and `quartzite` re-exports of `peniko::Gradient`**

`quartzite-paint/src/lib.rs` re-exports `peniko::Gradient` (and the supporting types `peniko::GradientKind`, `peniko::ColorStop`, `peniko::Extend`) so callers who need `Brush::custom_gradient` do not need a direct `peniko` dependency. `quartzite-paint` gains a `peniko = { version = "0.6", default-features = false }` dep for the re-export. `quartzite/src/lib.rs` exposes them via `pub mod paint { pub use quartzite_paint::*; }` (already using a wildcard re-export — no extra work).

**`quartzite-style/src/default_style.rs` tests — `RecordingPainter` stores `Brush` by value**

The test `PaintEvent::FillRect { brush: Brush }` stores an owned `Brush`. After `Copy` removal, `brush: *brush` fails. Replace with `brush: brush.clone()`. All `*brush` and `*pen` dereferences-as-copy in the `RecordingPainter` impl must switch to `.clone()` for `Brush`; `Pen` stays `Copy` so `*pen` is fine.

**`quartzite-renderer/src/vello_painter.rs` test — `brush.kind()` assertion**

Line 536: `assert_eq!(brush.kind(), BrushKind::Solid(Color::WHITE))` compared a `BrushKind` (by value, `Copy`). After the change, `brush.kind()` returns `&BrushKind`. Fix: `assert_eq!(brush.kind(), &BrushKind::Solid(Color::WHITE))`.

**`quartzite-style/src/default_style.rs` test `brush_color` helper**

```rust
fn brush_color(b: &Brush) -> Color {
    match b.kind() {
        BrushKind::Solid(c) => c,    // was by value
        _ => unreachable!(...)
    }
}
```

After the change, `b.kind()` returns `&BrushKind`, so the arm is `&BrushKind::Solid(c) => *c` (Color is `Copy`).

**Snapshot tests for gradient brushes (AC7–AC9)**

The three gradient snapshot tests go into `quartzite-widgets/tests/snapshots.rs`. They follow the established pixel-assertion pattern (no golden file — direct pixel comparison). Rendering is done via the `RenderHarness` + `VelloPainter` pipeline. A 10×1 canvas (logical pixels) is sufficient for the linear gradient test; a 11×11 canvas for radial (so the centre pixel and a pixel at radius are distinct rows/columns); a 11×1 canvas for the custom 3-stop test.

However, single-row or very small canvases may hit rounding at scale=1.0 — use a 20×1 canvas for linear, 21×21 for radial (centre at 10,10 radius 9), and 21×1 for the 3-stop custom.

### Rejected alternatives

**Return `Option<peniko::BrushRef<'_>>` from `brush_to_peniko`** — `BrushRef<'_>` borrows from the gradient, but the 2-stop arms construct a temporary gradient that would go out of scope immediately. Cannot borrow a temporary. Owned is the only option.

**Place `Color → DynamicColor` conversion in `quartzite-paint-api`** — Adds a `peniko` dep to `quartzite-paint-api` (which the spec mandates for the `Custom` variant anyway), but the conversion is renderer-specific. The `VelloPainter` private helper keeps conversion concerns in the renderer crate. The `quartzite-paint-api` dep on peniko is limited to `BrushKind::Custom(peniko::Gradient)`.

**Feature-gate the peniko dep in `quartzite-paint-api`** — Spec explicitly rejected: `Custom` is v1, not opt-in.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `peniko` dep to `quartzite-paint-api`; add `LinearGradient`, `RadialGradient`, `Custom(peniko::Gradient)` variants to `BrushKind`; drop `Copy` from `BrushKind` and `Brush`; change `Brush::kind(self)` to `Brush::kind(&self) -> &BrushKind`; remove `Copy`-asserting tests; add three constructors (`linear_gradient`, `radial_gradient`, `custom_gradient`); update doc-examples in `brush.rs` | `quartzite-paint-api/Cargo.toml`, `quartzite-paint-api/src/brush.rs` | — |
| 2 | Update `quartzite-renderer/src/vello_painter.rs`: rename `brush_color` to `brush_to_peniko(&self, brush: &Brush) -> Option<peniko::Brush>`; add private `color_to_dynamic` helper; implement all four arms; update all six call sites that used `brush_color` to use `brush_to_peniko` and pass `&brush_val` to `scene.fill`; fix the test assertion `brush.kind()` | `quartzite-renderer/src/vello_painter.rs` | 1 |
| 3 | Update `quartzite-style/src/default_style.rs`: update the production `brush_color` helper to add `LinearGradient`, `RadialGradient`, `Custom` arms; fix `RecordingPainter` in tests (replace `brush: *brush` with `brush: brush.clone()`; fix `brush_color` test helper to use `&BrushKind::Solid(c) => *c`) | `quartzite-style/src/default_style.rs` | 1 |
| 4 | Add `peniko` dep to `quartzite-paint`; re-export `peniko::Gradient`, `peniko::GradientKind`, `peniko::ColorStop`, `peniko::Extend` from `quartzite-paint/src/lib.rs` (and update doc-comment); verify `quartzite/src/lib.rs` wildcard re-export propagates them | `quartzite-paint/Cargo.toml`, `quartzite-paint/src/lib.rs`, `quartzite/src/lib.rs` | 1 |
| 5 | Remove the AC10 carve-out comment block from `quartzite-widgets/tests/snapshots.rs`; add three gradient snapshot tests (AC7 linear, AC8 radial, AC9 custom 3-stop) | `quartzite-widgets/tests/snapshots.rs` | 2 |
| 6 | Run the full gate suite and fix any compile errors or test failures: `cargo build`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, `cargo build -p quartzite-paint-api --no-default-features`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` | — | 1–5 |

## Risks

- **`peniko::Gradient` construction for the 2-stop arms is allocation-heavy on every paint call** — acceptable for v1; the spec documents that the 2-stop constructors are `const fn` (cheap) and the allocation happens at render time in the renderer, not in the API layer. Mitigation: none required for v1; a future optimization could bake stops into a fixed-size array.
- **`scene.fill` accepts `impl Into<BrushRef<'_>>`; passing `&peniko::Brush`** — `From<&Brush> for BrushRef<'_>` is implemented in peniko 0.6 (`brush.rs` line 144). Confirmed present in the source.
- **`Brush::kind` losing `const`** — `const fn` cannot return a reference to a field with a non-static lifetime in stable Rust. All callers in this codebase take the result by reference anyway, so removing `const` is not a correctness risk. No caller needs `kind()` in a `const` context (the `const BRUSH` pattern in the spec refers to constructors, not `kind()`).
- **`quartzite-style/src/default_style.rs` `RecordingPainter` stores `Brush` by value** — `PaintEvent` derives `Clone + Debug + PartialEq`; these all hold for `Brush` after `Copy` removal. The only change is `*brush` → `brush.clone()` in the `impl Painter` body. No logic change.
- **Snapshot tests for gradients may be fragile on headless CI (no GPU)** — existing snapshot tests use `harness_or_skip` which returns `None` gracefully when no GPU adapter is available. The gradient tests follow the same pattern.
- **`quartzite-paint` adding a `peniko` dep** — `quartzite-paint` is not `no_std`; there's no `no_std` constraint on it. The `peniko` dep uses `default-features = false` for consistency. The workspace already locks peniko to 0.6.0.

## Test Design

### Task 1 — `quartzite-paint-api/src/brush.rs`

Location: `quartzite-paint-api/src/brush.rs` `#[cfg(test)] mod tests`

Entry points and scenarios:
- `linear_gradient_stores_fields` — calls `Brush::linear_gradient(Point::new(0.0, 0.0), Point::new(10.0, 0.0), Color::RED, Color::BLUE)`; asserts `brush.kind()` matches `&BrushKind::LinearGradient { start, end, start_color: Color::RED, end_color: Color::BLUE }` (AC1).
- `radial_gradient_stores_fields` — calls `Brush::radial_gradient(Point::new(5.0, 5.0), 3.0, Color::WHITE, Color::BLACK)`; asserts `kind()` matches `&BrushKind::RadialGradient { centre, radius: 3.0, start_color: Color::WHITE, end_color: Color::BLACK }` (AC2).
- `linear_gradient_is_const_fn` — `const BRUSH: Brush = Brush::linear_gradient(...)` compiles (AC3).
- `radial_gradient_is_const_fn` — same for radial (AC3).
- `custom_gradient_round_trips` — constructs a `peniko::Gradient::new_linear(...)` with `.with_stops(...)`, passes to `Brush::custom_gradient`; asserts `if let BrushKind::Custom(ref got) = brush.kind() { assert_eq!(got, &g); }` (AC4).
- `brush_kind_not_copy` and `brush_not_copy` tests are **removed** (AC5).
- Existing `solid_stores_color` and `default_is_solid_white` — fix assertions from `brush.kind() == BrushKind::Solid(...)` to `brush.kind() == &BrushKind::Solid(...)`.

### Task 2 — `quartzite-renderer/src/vello_painter.rs`

Location: existing `#[cfg(test)] mod tests`

Entry points and scenarios:
- `all_painter_methods_are_invocable` — already exercises `fill_rect`, `draw_rect`, `draw_path` etc. Extend to also call with gradient brushes: `Brush::linear_gradient(...)`, `Brush::radial_gradient(...)`, `Brush::custom_gradient(g)` — must not panic (AC6). No pixel assertions here; those are in the snapshot suite.

### Task 3 — `quartzite-style/src/default_style.rs`

Location: existing `#[cfg(test)] mod tests`

Entry points and scenarios:
- All existing tests continue to pass after the `*brush → brush.clone()` mechanical fix.
- The `brush_color` test helper is updated: no new test cases for gradient brushes (the helper is a `tests` private helper; the production `brush_color` function tested via the public `draw_widget` path).

### Task 5 — `quartzite-widgets/tests/snapshots.rs`

New tests (all use `harness_or_skip`):

- `fill_rect_linear_gradient` (AC7):
  - Canvas: 20×1, `scale_factor(1.0)`.
  - Brush: `Brush::linear_gradient(Point::new(0.0, 0.0), Point::new(20.0, 0.0), Color::RED, Color::BLUE)`.
  - Rect: `Rect::new(Point::new(0, 0), Size::new(20, 1))`.
  - Assertions: `image.get_pixel(0, 0).0[0] > 200` (left pixel is red-dominant) and `image.get_pixel(19, 0).0[2] > 200` (right pixel is blue-dominant), and the two pixels differ by `> 0.5 * 255 ≈ 127` on at least one of the R/B channels.

- `fill_rect_radial_gradient` (AC8):
  - Canvas: 21×21, `scale_factor(1.0)`.
  - Brush: `Brush::radial_gradient(Point::new(10.0, 10.0), 9.0, Color::WHITE, Color::BLACK)`.
  - Rect: `Rect::new(Point::new(0, 0), Size::new(21, 21))`.
  - Assertions: centre pixel `(10, 10)` is near-white (`> 200` on all channels); pixel `(0, 10)` is darker (at least one channel `< 100`); they differ.

- `fill_rect_custom_gradient` (AC9):
  - Canvas: 21×1, `scale_factor(1.0)`.
  - Brush: `Brush::custom_gradient(peniko::Gradient::new_linear((0.0f64, 0.0f64), (21.0f64, 0.0f64)).with_stops([(0.0f32, peniko::color::palette::css::RED), (0.5f32, peniko::color::palette::css::LIME), (1.0f32, peniko::color::palette::css::BLUE)]))`. Stops are `(f32, AlphaColor<Srgb>)` tuples, which implement `Into<ColorStop>` via `From<(f32, AlphaColor<CS>)> for ColorStop`.
  - Rect: full canvas width.
  - Assertions: middle pixel `(10, 0)` is green-dominant (`g > r` and `g > b` by at least `0.2 * 255 ≈ 51`).

- `draw_rect_gradient_fill_solid_stroke` (AC10):
  - Canvas: `CANVAS` (64×64), `scale_factor(1.0)`.
  - Brush: `Brush::linear_gradient(...)` (RED→BLUE across 48px), Pen: `Pen::new(Color::WHITE, 2.0)`.
  - Rect: `Rect::new(Point::new(8, 8), Size::new(48, 48))`.
  - Render `draw_rect`; separately render `fill_rect` alone and compare — they differ. Also verify the outer border pixel colour matches the pen (white): pixel at `(8, 8)` should be near-white.

Fixtures needed: `harness_or_skip` (already present in `support` module); `peniko::Gradient` available via `quartzite-paint` re-export (after Task 4).

## Open questions

None — all spec open questions were resolved at the spec level or above in this document.
