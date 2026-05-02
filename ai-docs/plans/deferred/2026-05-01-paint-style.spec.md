# Paint & Style

**Source:** AI design dialogue (tmp/qt_01..14.log)
**Date:** 2026-05-01
**Tracked in:** #47

## Scope

Two crates with no platform backend dependency — all drawing goes through the `Painter` trait abstraction.

### `quartzite-paint`

- `Color` — RGBA, u8 channels; named constructors (`Color::RED`, `Color::TRANSPARENT`, …); `from_rgb(r,g,b)`, `from_rgba(r,g,b,a)`, `with_alpha(a)`
- `Pen` — color, width (f32), line style (Solid, Dash, Dot, None), cap, join
- `Brush` — fill style: NoBrush, SolidColor(Color), LinearGradient, RadialGradient, Pattern
- `Font` — family (String), size (f32 pt), weight (Thin…Black), style (Normal, Italic, Oblique), underline, strikethrough
- `Image` — pixel buffer (width, height, RGBA bytes); `load_from_bytes`, `pixel(x,y)`, `set_pixel(x,y,color)`
- `Path` — vector path: `move_to`, `line_to`, `cubic_to`, `arc_to`, `close`
- `Painter` trait (object-safe):
  - `set_pen(&mut self, pen: Pen)`
  - `set_brush(&mut self, brush: Brush)`
  - `set_font(&mut self, font: Font)`
  - `draw_rect(&mut self, rect: Rect)`
  - `fill_rect(&mut self, rect: Rect, brush: Brush)`
  - `draw_line(&mut self, from: Point, to: Point)`
  - `draw_text(&mut self, pos: Point, text: &str)`
  - `draw_text_in(&mut self, rect: Rect, text: &str, alignment: Alignment)`
  - `draw_image(&mut self, pos: Point, image: &Image)`
  - `draw_path(&mut self, path: &Path)`
  - `save(&mut self)` / `restore(&mut self)` — state stack
  - `translate(&mut self, offset: Point)`
  - `clip_rect(&mut self, rect: Rect)`

### `quartzite-style`

- `Style` trait — `draw_widget(&mut self, widget: &dyn AsWidget, painter: &mut dyn Painter)`, plus primitive drawing methods for each widget type (button, label, input)
- `Palette` — color roles: WindowBackground, WindowText, Button, ButtonText, Highlight, HighlightedText, Base, Text, Link, …; light/dark/mid variants
- `ColorRole` enum
- `StyleRegistry` — global singleton: `set_style(Box<dyn Style>)`, `style() -> &dyn Style`; default style built-in

## Out of scope

- Actual platform backend (OpenGL, Vulkan, software rasterizer) — backend is a separate future crate
- SVG / PDF rendering
- Complex text layout (bidirectional, shaping) — basic left-to-right only for v1

## Deferred

- `LinearGradient` / `RadialGradient` Brush variants | needs backend support to test
- `Image::load_from_file` | needs I/O abstraction
- Sub-pixel font rendering | backend-specific
- Custom widget draw primitives beyond basic shapes | add as needed

## Key decisions

| Question | Decision |
|---|---|
| Painter is a trait | Yes — allows multiple backends (software, GPU, test/mock) |
| Color channels | u8 per channel (0–255); RGBA order |
| Font size unit | Points (f32) |
| Path API | Builder-pattern methods mutating `Path` in place |
| Style singleton | `StyleRegistry` global; panic if accessed before `Application::new()` |

## Technical constraints

- `quartzite-paint` depends on `quartzite-geometry` for Point, Size, Rect
- `quartzite-style` depends on `quartzite-paint` and `quartzite-widgets` (for `AsWidget`)
- `Painter` must be object-safe (`Box<dyn Painter>` and `&mut dyn Painter` must work)
- No platform APIs in either crate — zero system dependencies

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Color::from_rgba(255, 0, 0, 128).alpha() == 128` |
| AC2 | `Color::RED.with_alpha(0).alpha() == 0` |
| AC3 | `Pen::new(Color::BLACK).width() == 1.0` (default width) |
| AC4 | A `Path` with `move_to(0,0)`, `line_to(10,10)`, `close()` has 3 segments |
| AC5 | `Font::new("Arial", 12.0)` reports `family() == "Arial"` and `size() == 12.0` |
| AC6 | `Palette::new()` contains a color for every `ColorRole` variant |
| AC7 | A mock `Painter` implementation can be constructed and passed as `&mut dyn Painter` without compile error |
| AC8 | `StyleRegistry::set_style(custom_style)` causes subsequent `StyleRegistry::style()` to return the custom style |

## Open questions

- Should `Painter::save`/`restore` use a stack internally or delegate entirely to the backend?
- Should `Brush::LinearGradient` store stops as `Vec<(f32, Color)>` even if deferred?
- Should `Alignment` (used in draw_text_in) live in quartzite-geometry or quartzite-paint?
