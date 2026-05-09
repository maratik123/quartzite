# Paint & Style

**Source:** issue #47; original draft from AI design dialogue (`tmp/qt_01..14.log`) on 2026-05-01; revised 2026-05-09 after `graphics-stack` (#73) shipped `quartzite-paint-api`.
**Date:** 2026-05-09 (revision absorbs the 2026-05-01 draft)
**Tracked in:** #47

> The 2026-05-01 draft predated `graphics-stack` (#73) and proposed u8 colour channels, a
> single-arg `Pen::new`, and a state-based `Painter`. Once `quartzite-paint-api` shipped
> with `f32` colour, two-arg `Pen::new`, and a pass-through `Painter`, those acceptance
> criteria became incompatible with the live code. This spec absorbs the 2026-05-01 draft
> and revises every incompatible decision; see [§ Resolution log](#resolution-log) for the
> round-1 Q&A that produced the final shape.

## Scope

Complete the `Path` implementation in `quartzite-paint`, extend the `Painter` trait with the
remaining draw methods, introduce paint-side `Font` and `Image` types, move `Alignment` into
`quartzite-geometry`, migrate `Palette` out of `quartzite-widgets`, and stand up two new
crates: `quartzite-style-types` (leaf: `Palette`, `ColorRole`) and `quartzite-style`
(downstream: `Style` trait, global `StyleRegistry`). The split exists to break the Cargo
cycle that arose from "`Style::draw_widget` needs `&dyn AsWidget`" *and* "widgets re-exports
`Palette`": widgets depends on the leaf only, the downstream `quartzite-style` crate depends
on widgets and the leaf.

### `quartzite-geometry` — additions

- `Alignment` enum — moved verbatim from `quartzite-widgets::enums::Alignment`. Variants
  `Left = 0`, `Center = 1`, `Right = 2`, `Justify = 3` with `#[default] = Left` and
  `#[derive(MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]` preserved.
- `quartzite-geometry` gains a dependency on `quartzite-macros` so the `MetaEnum` derive
  continues to expand. The crate stays `no_std` — `quartzite-macros` is a proc-macro crate,
  the generated code uses `quartzite-core` traits which are already `no_std`-compatible.

### `quartzite-paint-api` — additions

- `Color::with_alpha(a: f32) -> Color` — `const fn` returning a copy of `self` with the alpha
  channel replaced by `a` (other channels untouched).
- `Painter` trait gains:
  - `draw_text(&mut self, pos: Point, text: &str, font: &Font, brush: &Brush)`
  - `draw_text_in(&mut self, rect: Rect, text: &str, font: &Font, brush: &Brush, alignment: Alignment)`
  - `draw_image(&mut self, rect: Rect, image: &Image)`
  - `draw_path(&mut self, path: &Path, pen: &Pen, brush: &Brush)`
- `Font`, `Image`, and `Path` types — added in `quartzite-paint-api` so `Painter` can
  reference them without a circular re-export through `quartzite-paint`. The issue body lists
  these types under `quartzite-paint`; placing the type definitions in `quartzite-paint-api`
  (with `quartzite-paint` re-exporting them) is the only way to keep the trait object-safe and
  avoid `paint-api ↔ paint` circularity. Callers that depend on `quartzite-paint` see the same
  vocabulary.

### `quartzite-paint` — additions

- `Path` — full implementation lives here as a wrapper around the segment list:
  `move_to(p)`, `line_to(p)`, `cubic_to(c1, c2, p)`,
  `arc_to(centre, radii, start_angle, sweep_angle)`, `close()`. Builder pattern: each method
  returns `&mut Self` so calls can be chained. Internal segment list is the canonical
  representation; `pub fn segments(&self) -> &[Segment]` exposes it for backends to consume.
  `Segment` is a `#[non_exhaustive]` enum (`MoveTo`, `LineTo`, `CubicTo`, `ArcTo`, `Close`).
  Where exactly the type lives between `paint` and `paint-api` is a design-phase decision; the
  Painter-facing alias is the constraint.
- Re-exports `Font`, `Image`, and `Alignment` from upstream crates so callers depending on
  `quartzite-paint` see the full vocabulary in one place.

### `quartzite-widgets` — changes

- `quartzite-widgets::Alignment` is removed; `pub use quartzite_geometry::Alignment;` replaces
  the local definition. Existing widget call sites compile unchanged.
- `quartzite-widgets::Font` and `quartzite-widgets::Palette` are removed; the widget public
  surface re-exports `Font` from `quartzite-paint` and `Palette` (with `ColorRole`) from
  `quartzite-style-types`. Widgets does **not** depend on `quartzite-style` (the cycle-fix —
  see § Scope).

### `quartzite-style-types` — new leaf crate

- `Palette` — moved here from `quartzite-widgets`. Indexed by `ColorRole` (lookup method
  `Palette::color(role: ColorRole) -> Color`); `Palette::with_role(self, role: ColorRole, color: Color) -> Palette`
  for builder-style customisation. The widget palette becomes a re-export from
  `quartzite-style-types` to preserve widget call sites.
- `ColorRole` enum — `Window`, `WindowText`, `Button`, `ButtonText`, `Base`, `Text`,
  `Highlight`, `HighlightedText`, `Link`, `LinkVisited`, `BrightText`. Light/dark variants are
  exposed via `PaletteGroup` rather than enum doubling.
- Depends only on `quartzite-paint-api` for the `Color` type. Stays `no_std + alloc`.

### `quartzite-style` — new downstream crate

- `Style` trait — single generic method:
  `fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette);`
  Concrete `Style` implementations route on widget type via downcast or a visitor; per-widget
  primitive methods (`draw_button`, `draw_label`, …) are **not** part of the trait.
- Re-exports `Palette` and `ColorRole` from `quartzite-style-types` so callers depending on
  `quartzite-style` see the full vocabulary in one place.
- `StyleRegistry` — global registry; `set_style(Box<dyn Style>)`,
  `try_style() -> Option<&'static dyn Style>`, default style installed via `OnceLock`
  initialiser. Storage is `OnceLock<Mutex<Option<&'static dyn Style>>>`; `set_style` calls
  `Box::leak` to obtain the `'static` reference (replacement leaks the old box — acceptable
  for a process-lifetime registry). Lock-poisoning handled per AGENTS.md library-safety
  idioms.
- Depends on `quartzite-style-types`, `quartzite-widgets`, `quartzite-paint`. Pulls in `std`
  for the global registry. Not `no_std`.

## Out of scope

- A working software / GPU rasteriser. `quartzite-renderer` already has a vello-backed
  `VelloPainter` skeleton; concrete rendering of new methods (`draw_text`, `draw_path`,
  `draw_image`) is deferred to its own plan once the API surface here is stable.
- SVG / PDF export.
- Bidirectional or shaped text layout. Basic LTR positioning only.
- Built-in image decoders (PNG/JPEG/etc.) — `Image` is a raw RGBA pixel buffer; loading from
  a file or compressed bytes is deferred.
- Per-widget `Style::draw_*` primitive methods (`draw_button`, `draw_label`, `draw_text_edit`,
  `draw_scroll_area`). The trait surface is `draw_widget` only; specialised dispatch happens
  inside concrete `Style` implementations.
- Sub-pixel font rendering.

## Deferred

- `BrushKind::LinearGradient` / `RadialGradient` variants | needs backend support to render | no separate issue (extension within #47 scope after design-amendment when a backend lands)
- `Image::load_from_file` / `load_from_bytes` decoders | needs an I/O abstraction | no separate issue
- Pixel-format metadata on `Image` (BGRA / premultiplied alpha / etc.) | only RGBA8 needed for v1 | no separate issue
- `Style` per-platform overrides (e.g. macOS-flavoured vs. Windows-flavoured native style) | needs platform-detection plumbing | no separate issue

## Key decisions

| Question | Decision |
|---|---|
| Colour channel type | `f32` ∈ `[0.0, 1.0]` — already shipped in `quartzite-paint-api`; supersedes the 2026-05-01 draft's u8 |
| `Pen::new` signature | `Pen::new(color: Color, width: f32)` — already shipped; supersedes the 2026-05-01 draft's single-arg form |
| `Color::with_alpha` return | Returns `Color` (not `f32`); `with_alpha(0.0).a() == 0.0` |
| `Painter::save` / `restore` | Trait method only — implementation delegated to the concrete backend (`VelloPainter`, mock painters). No internal stack in the trait. |
| `Brush::LinearGradient` | Deferred — `BrushKind` is `#[non_exhaustive]`, future variant added without breakage |
| `Alignment` location | `quartzite-geometry` — the existing `quartzite-widgets::Alignment` is **moved**, not duplicated. `quartzite-widgets` re-exports it. |
| `quartzite-geometry` macro dependency | `quartzite-geometry` gains a `quartzite-macros` dependency so the `MetaEnum` derive on `Alignment` keeps expanding after the move. Crate stays `no_std`. |
| `Path` representation | Stored internally as `Vec<Segment>` where `Segment` is a `#[non_exhaustive]` enum (`MoveTo`, `LineTo`, `CubicTo`, `ArcTo`, `Close`). Builder methods return `&mut Self`. |
| `Font` location | `quartzite-paint-api` (so `Painter` methods can reference it without depending on `quartzite-paint`); `quartzite-paint` re-exports |
| `Image` v1 shape | `Image { width: u32, height: u32, pixels: Vec<u8> }` — RGBA8, row-major, no per-pixel padding. Validated at construction (`pixels.len() == (width * height * 4) as usize`); accessor `Image::pixels() -> &[u8]`. |
| `Image` location | `quartzite-paint-api` (same reason as `Font`); `quartzite-paint` re-exports. |
| `draw_image` Painter method | Included — `draw_image(&mut self, rect: Rect, image: &Image)`. Painter scales the image into `rect`; explicit source-rect cropping is a future extension. |
| Existing `quartzite-widgets::Font` | Removed — replaced by paint's `Font`. Widget call sites updated to point at the new type. AGENTS.md § *API Stability*: pre-crates.io, no compat shim. |
| `Font` fields | `family: String`, `size_pt: f32`, `weight: FontWeight`, `italic: bool`, `underline: bool`, `strikethrough: bool` |
| `FontWeight` variants | Enum with the canonical CSS weights — `Thin`/`ExtraLight`/`Light`/`Normal`/`Medium`/`SemiBold`/`Bold`/`ExtraBold`/`Black` mapping to numeric weights `100`–`900`; `Default = Normal` |
| Existing `quartzite-widgets::Palette` | Moved to `quartzite-style-types` (the leaf crate). Widgets re-exports from `quartzite-style-types`, NOT from `quartzite-style`. |
| `Palette` API shape | Indexed by `ColorRole` via `palette.color(ColorRole::Window)` returning `Color`; the public-field shape is replaced — `pub fn with_role(self, role: ColorRole, color: Color) -> Palette` for builder-style customisation. |
| `Style` trait surface | **Generic only** — `Style::draw_widget(&self, &dyn AsWidget, &mut dyn Painter, &Palette)`. No per-widget methods. Concrete styles dispatch internally. |
| Cargo cycle resolution | Split into `quartzite-style-types` (leaf: `Palette`, `ColorRole`) + `quartzite-style` (downstream: `Style`, `StyleRegistry`). Widgets depends only on the leaf; downstream `quartzite-style` depends on widgets and the leaf. Breaks the otherwise-impossible `style ↔ widgets` cycle. |
| `StyleRegistry` panicking | Non-panicking by default per AGENTS.md § *API Naming*: `try_style() -> Option<&'static dyn Style>` is the default accessor. A panicking convenience wrapper `style() -> &'static dyn Style` may exist alongside; design phase decides. |
| `StyleRegistry` storage | `OnceLock<Mutex<Option<&'static dyn Style>>>`. `set_style` calls `Box::leak` to obtain the `'static` reference; replacement leaks the prior box (process-lifetime registry, acceptable). Lock-poisoning handled via `lock().unwrap_or_else(\|e\| e.into_inner())` per AGENTS.md library-safety idioms. |

## Technical constraints

- `quartzite-paint-api` is `no_std + alloc`. `Font` storing `String` and `Image` storing
  `Vec<u8>` both require `alloc` unconditionally. The crate currently has
  `extern crate alloc;` only behind `#[cfg(test)]`; this gate must be removed so production
  code can name `String`/`Vec`. AC14 (`--no-default-features` build) still holds.
- `quartzite-style-types` is `no_std + alloc` (depends only on `quartzite-paint-api` for
  `Color`).
- `quartzite-style` depends on `quartzite-style-types`, `quartzite-paint`,
  `quartzite-widgets`, and `std` (the global registry uses `std::sync::Mutex` and
  `std::sync::OnceLock`). It is **not** `no_std`.
- `quartzite-geometry` must remain `no_std`. The new `Alignment` enum has no allocations and
  is `Copy`. The added `quartzite-macros` dependency is a proc-macro crate; the generated
  `MetaEnum` code references `quartzite-core` traits, so `quartzite-geometry` also gains a
  `quartzite-core` dependency with `default-features = false` to preserve `no_std`. AC14
  (`--no-default-features` build of geometry) still holds.
- `Painter` must remain object-safe — no generic methods, no `Self` returns, no associated
  types. New methods follow the same pattern as the live ones (`&mut self`, primitive args).
- `quartzite-widgets` already depends on `quartzite-paint-api`; it gains a dependency on
  `quartzite-paint` (for `Font` re-export) and on `quartzite-style-types` (for `Palette` /
  `ColorRole` re-exports). `quartzite-widgets` must **not** depend on `quartzite-style` —
  that direction is `quartzite-style → quartzite-widgets`, broken by the leaf-crate split.
- All new public items: `#![deny(missing_docs)]` + `# Examples` block + `# Parameters` (when
  ≥ 1 non-receiver arg) per workspace doc convention.
- Recursive-simple fns get `#[inline]` (concrete) or `_Simple._` (generic / trait method) per
  AGENTS.md § *`#[inline]` and the `_Simple._` doc tag*.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Color::RED.with_alpha(0.0).a() == 0.0`, with the other channels unchanged: `Color::RED.with_alpha(0.25).r() == 1.0`. |
| AC2 | `Color::with_alpha` is a `const fn` returning `Color` (matches the rest of the `Color` API). |
| AC3 | `Pen::default().width() == 1.0` and `Pen::default().color() == Color::BLACK` — already satisfied by the live impl; AC re-listed so the migration doesn't regress it. |
| AC4 | A `Path` built by `Path::new().move_to(p0).line_to(p1).close()` returns a 3-element `&[Segment]` from `path.segments()` whose discriminants are `MoveTo`, `LineTo`, `Close` in order. |
| AC5 | `Path::new().cubic_to(c1, c2, p).arc_to(centre, radii, 0.0, std::f32::consts::PI)` compiles and round-trips through `path.segments()` without loss of arguments. |
| AC6 | `Font::new("Arial", 12.0)` returns a font with `family() == "Arial"`, `size_pt() == 12.0`, `weight() == FontWeight::Normal`, `italic() == false`, `underline() == false`, `strikethrough() == false`. |
| AC7 | `Image::try_new(2, 2, vec![0u8; 16])` returns `Ok` with `width() == 2`, `height() == 2`, `pixels().len() == 16`; `Image::try_new(2, 2, vec![0u8; 15])` returns `Err` (length mismatch). |
| AC8 | A mock implementor of `Painter` covering all trait methods (including the new `draw_text`, `draw_text_in`, `draw_path`, `draw_image`) compiles and is callable through both `&mut dyn Painter` and `Box<dyn Painter>` — i.e., the trait remains object-safe. |
| AC9 | `quartzite-style-types::Palette::default().color(ColorRole::Window) != Color::TRANSPARENT` — the default palette installs a non-transparent value for every `ColorRole` variant (loop-driven assertion, not per-role enumeration). |
| AC10 | `StyleRegistry::try_style()` returns `None` before any style is installed and `Some(_)` after `StyleRegistry::set_style(custom_style)`; `try_style()` survives a deliberately poisoned mutex (lock returns `Err(PoisonError)` once, recovered via `into_inner`) — `cargo test` includes the poison-recovery scenario. |
| AC11 | A concrete `Style` implementation defining only `fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette)` is sufficient to satisfy the trait — no other required methods. |
| AC12 | `quartzite-geometry::Alignment` exists with the same four variants and discriminants as the previous `quartzite-widgets::Alignment` (`Left=0`, `Center=1`, `Right=2`, `Justify=3`); the `MetaEnum` derive expands and the existing round-trip test (`Alignment::Center.into_value()` ↔ `Value::Int(1)`) passes against the geometry-side type. |
| AC13 | After this plan lands, `quartzite-widgets` no longer defines its own `Alignment`, `Font`, or `Palette` types; `widgets::Alignment` is a re-export from `quartzite-geometry`, `widgets::Font` from `quartzite-paint`, and `widgets::Palette` (with `widgets::ColorRole`) from `quartzite-style-types`. Existing widget call sites compile unchanged. `cargo tree -p quartzite-widgets` does NOT list `quartzite-style` as a dependency (only `quartzite-style-types`). |
| AC14 | `cargo build -p quartzite-paint-api --no-default-features` succeeds (the crate stays `no_std + alloc`); `cargo build -p quartzite-geometry --no-default-features` succeeds (the crate stays `no_std`). |
| AC15 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` is clean. |
| AC16 | `cargo clippy --workspace -- -D warnings` is clean. |

## Open questions

- Concrete `quartzite-renderer` implementations of the new `Painter` methods are deferred to
  a follow-up plan; this spec only nails down the trait surface and types they will be passed.
- Default-style content (e.g. a "Quartzite Default" struct shipped in `quartzite-style`):
  with the trait being generic-only, this becomes "design a single concrete `Style` struct
  whose `draw_widget` covers Button/Label/TextEdit/ScrollArea". Left to the design phase.
- `Image` source-rect cropping (drawing a sub-region of an `Image` into the destination
  `rect`) — not in v1; revisit when a backend gains real `draw_image` support.

## Resolution log

Interview round 1 held 2026-05-09 via `/interview` (issue #47). Decisions taken; all folded into the spec body above.

| Question | Decision |
|---|---|
| Image type — defer for v1 or include a minimal pixel-buffer Image now? | **Pixel buffer** — add `Image { width, height, Vec<u8> RGBA }` plus `draw_image` on `Painter`. |
| Style trait surface — one generic `draw_widget` or enumerated per-widget primitives? | **Generic only** — `Style::draw_widget(&dyn AsWidget, &mut dyn Painter, &Palette)`. Concrete `Style` implementations downcast or use a visitor. |
| `Alignment` in `quartzite-geometry` — move the existing `quartzite-widgets::Alignment`, or add a separate `TextAlignment`? | **Move + macros** — move `Alignment` AND add a `quartzite-macros` dependency to `quartzite-geometry` so the `MetaEnum` derive survives. |

Pre-resolved (not asked in round 1; baked into the spec from the start):

- `Painter::save`/`restore` stack: trait method only, delegated to concrete backend.
- `Brush::LinearGradient` stops: deferred (`BrushKind` is `#[non_exhaustive]`).

Design-review round 1 (2026-05-09) surfaced a Cargo cycle blocker — the spec as drafted made
`quartzite-style` depend on `quartzite-widgets` (for `&dyn AsWidget`) **and** had widgets
re-export `Palette` from `quartzite-style`. Resolution chosen 2026-05-09:

| Question | Decision |
|---|---|
| How to break the `quartzite-style ↔ quartzite-widgets` Cargo cycle? | **Split into types-leaf + main crate** — new `quartzite-style-types` holds `Palette` + `ColorRole`; `quartzite-style` re-exports them and adds `Style` + `StyleRegistry`. Widgets depends on `quartzite-style-types` only. |

Two further design-review issues were classed as design-level (not spec-level) and folded
into the spec inline rather than as separate Q&A:

- `extern crate alloc;` in `quartzite-paint-api` must be moved out of the `#[cfg(test)]`
  gate so production code can name `String`/`Vec` (Font/Image/Path additions).
- `StyleRegistry` storage must be `OnceLock<Mutex<Option<&'static dyn Style>>>` with
  `Box::leak` in `set_style` to make `try_style() -> Option<&'static dyn Style>`
  implementable.
