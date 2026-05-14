# Paint Brush gradient variants

**Source:** issue #281 (surfaced from `ai-docs/deferred/widget-backlog.md`; originally deferred by the [paint-style spec](done/2026-05-09-paint-style.spec.md))
**Date:** 2026-05-14
**Tracked in:** #281

## Scope

Add gradient variants to `BrushKind` in `quartzite-paint-api` and wire `VelloPainter` in `quartzite-renderer` to render them. The v1 surface is a three-variant split per the round-1/round-2 product-owner answers:

1. **Ergonomic 2-stop variants** for the 80% case — `LinearGradient { start, end, start_color, end_color }` and `RadialGradient { centre, radius, start_color, end_color }`. Cheap, no heap.
2. **Rich/escape-hatch variant** — `Custom(peniko::Gradient)` for callers needing >2 stops, transforms, focal points, alternative extend modes, or any other peniko-shaped gradient. `quartzite-paint-api` gains a `peniko` dependency to host the type directly (round-2 decision: "Peniko in API"). The crate stays `no_std + alloc`; `peniko 0.6` supports `no_std`.

The plan also lifts the AC10 carve-out comment in `quartzite-widgets/tests/snapshots.rs` ("`BrushKind::LinearGradient / RadialGradient` are not yet implemented…") and replaces the silent `_ => None` fallthrough in `VelloPainter::brush_color` (and the `BrushKind::Solid(c) => c` arm in `quartzite-style::default_style`) with explicit coverage of every live gradient variant.

### `quartzite-paint-api` — additions

- `BrushKind` gains three new variants:
  - `LinearGradient { start: Point, end: Point, start_color: Color, end_color: Color }`
  - `RadialGradient { centre: Point, radius: f32, start_color: Color, end_color: Color }`
  - `Custom(peniko::Gradient)` — escape hatch that owns a full peniko gradient (N stops, transform, extend mode, etc.).
- `Brush` constructors (companions to the existing `Brush::solid`):
  - `Brush::linear_gradient(start: Point, end: Point, start_color: Color, end_color: Color) -> Self`
  - `Brush::radial_gradient(centre: Point, radius: f32, start_color: Color, end_color: Color) -> Self`
  - `Brush::custom_gradient(gradient: peniko::Gradient) -> Self`
- The two 2-stop constructors are `const fn` (mirroring `Brush::solid`) and take primitives by value — no `Result` because the 2-stop form has no construction-time invariant beyond what the type system already enforces. Negative radius / coincident endpoints are caller-visible (rendered as empty / degenerate). No `try_*` constructor.
- `Brush::custom_gradient` is **not** `const` (peniko's `Gradient` has heap-allocated stops; it cannot be constructed in const context). Takes ownership.
- New `peniko` dep on `quartzite-paint-api` (`peniko = { version = "0.6", default-features = false }` to preserve `no_std + alloc`). The crate is **no longer backend-agnostic** — this is an accepted clean break per AGENTS.md § *API Stability* (pre-`crates.io`, no downstream clients).
- `BrushKind` and `Brush`: the addition of `Custom(peniko::Gradient)` forces `BrushKind` and `Brush` to **lose `Copy`** (peniko's `Gradient` owns a `SmallVec`/`Vec` of stops and is `Clone`-only). They remain `Clone + Debug + PartialEq`. Existing call sites that relied on `Copy` (`let b2 = b; let b3 = b;` patterns; `Brush::kind(self) -> BrushKind` by value) are updated as a clean break:
  - `Brush::kind(self) -> BrushKind` becomes `Brush::kind(&self) -> &BrushKind` (borrow rather than copy).
  - Two existing `#[cfg(test)]` Copy-asserting tests in `quartzite-paint-api/src/brush.rs` (`brush_kind_is_copy`, `brush_is_copy`) are removed.
  - Call sites in `quartzite-style::default_style` (`BrushKind::Solid(c) => c` arms — currently destructures by-value via `Brush::kind()`) switch to `match brush.kind()` borrowing pattern (`&BrushKind::Solid(c) => c` or equivalent — `Color` itself is `Copy`).
- `BrushKind` keeps `#[non_exhaustive]`. The four variants after this change: `Solid`, `LinearGradient`, `RadialGradient`, `Custom`. Existing `match` arms that handle `Solid` keep compiling; exhaustive matches without `_` get a compile error (grep before commit in design phase confirms no in-tree call sites do this — the renderer and default_style both use catch-alls today).
- All new payload types stay `Clone + Debug + PartialEq`. `f32`/`Color`/`Point` for the 2-stop variants; `peniko::Gradient`'s own `Clone + Debug + PartialEq` for `Custom`.

### `quartzite-renderer` — `VelloPainter` wiring

- `VelloPainter::brush_color` is replaced by `brush_to_peniko(&self, brush: &Brush) -> Option<peniko::Brush>` (returns owned because constructing a `peniko::Gradient` from the 2-stop variants must allocate stops; the existing `peniko::Color` path becomes `peniko::Brush::Solid(_)`):
  - `BrushKind::Solid(c)` → `Some(peniko::Brush::Solid(color_to_peniko(c)))`.
  - `BrushKind::LinearGradient { start, end, start_color, end_color }` → `Some(peniko::Brush::Gradient(gradient))` where `gradient.kind = peniko::GradientKind::Linear { start: scale_pt(start), end: scale_pt(end) }`, `gradient.stops = [{ offset: 0.0, color: start_color.into() }, { offset: 1.0, color: end_color.into() }]`, `gradient.extend = peniko::Extend::Pad`. Endpoints are scaled by `self.scale` (same convention as `draw_line` / `draw_rect`).
  - `BrushKind::RadialGradient { centre, radius, start_color, end_color }` → `Some(peniko::Brush::Gradient(gradient))` with `gradient.kind = peniko::GradientKind::Radial { start_center: scale_pt(centre), start_radius: 0.0, end_center: scale_pt(centre), end_radius: radius * self.scale }`, same stops/extend as linear. (Single-circle form: collapses peniko's two-circle radial to a centred circle.)
  - `BrushKind::Custom(gradient)` → `Some(peniko::Brush::Gradient(gradient.clone()))`. The `Custom` variant is passed through verbatim — no scaling applied; the caller is responsible for using the gradient in `self.scale`-aware coordinates. Design phase may revisit if a scaling rule is preferred; the spec's default is **pass-through, no scale**.
  - Catch-all `_ => None` may remain because `BrushKind` is `#[non_exhaustive]`, but it no longer fires for any in-tree variant.
- The exact return type of `brush_to_peniko` (owned `peniko::Brush` vs. borrowed `peniko::BrushRef`) is a design-phase pick. The spec-level constraint is: every live `BrushKind` variant resolves to a renderable peniko brush; no silent no-op for gradient variants.
- `VelloPainter::draw_rect` with a gradient brush fills via the gradient AND strokes via the pen's solid colour. Pen-side gradients stay deferred (§ Deferred).

### `quartzite-style` — `default_style` updates

- `quartzite-style::default_style::brush_color(&Brush) -> Color` (helper used by snapshot/identity assertions) gains explicit arms:
  - `BrushKind::Solid(c) => *c` (now borrowing).
  - `BrushKind::LinearGradient { start_color, .. }` and `BrushKind::RadialGradient { start_color, .. }` → return `*start_color` as the "representative colour" for compositing/equality decisions.
  - `BrushKind::Custom(g)` → return the first stop's colour (`g.stops.first().map(|s| s.color.into()).unwrap_or(Color::TRANSPARENT)`); empty-stops case is documented but not constructible via `Brush::custom_gradient` in normal use.
- Doc comment on the helper documents the "representative colour" rule and warns it is not a perceptual average.

### `quartzite-widgets` snapshot tests

- The AC10 carve-out comment block in `quartzite-widgets/tests/snapshots.rs` (lines ~372–375, the comment that references `https://github.com/maratik123/quartzite/issues/281`) is removed.
- A positive snapshot test asserts `Painter::fill_rect` with a 2-stop `linear_gradient` brush draws a non-uniform pattern: in a 10×1 horizontal gradient from `Color::RED` to `Color::BLUE`, the leftmost rendered pixel and the rightmost rendered pixel differ by `> 0.5` in at least one of the R/B channels. A second positive test does the same for `radial_gradient` (centre vs. radius-edge pixel). A third positive test exercises `custom_gradient` with a 3-stop linear (RED → GREEN → BLUE) and asserts the middle pixel is roughly GREEN-dominant.

## Out of scope

- Conic / sweep gradients via the 2-stop ergonomic variants. `Custom(peniko::Gradient)` callers may pass `GradientKind::Sweep`, but no `Brush::sweep_gradient` convenience constructor.
- Two-circle radial gradients via the 2-stop ergonomic variant. `Custom` callers may pass a peniko radial with distinct `start_center` / `end_center`.
- Per-gradient affine transforms via the 2-stop ergonomic variants. `Custom` callers retain peniko's transform field.
- Non-default extend / spread modes via the 2-stop ergonomic variants — `Pad` only. `Custom` callers may pass `Extend::Reflect` / `Extend::Repeat`.
- Snapshot tests beyond a sanity assertion per variant. Per-pixel colour-curve verification is a separate task.
- A `Pen::brush(&Brush)` overload that lets strokes use gradients. Pens stay solid-colour for v1.

## Deferred

- Non-default extend modes (Reflect / Repeat) on the 2-stop ergonomic variants | requires threading an `enum ExtendMode { Pad, Reflect, Repeat }` through `LinearGradient` / `RadialGradient` | follow-up issue if a use case lands (today, callers use `Custom` for non-Pad extend)
- Two-circle / focused radial gradients in the ergonomic variant | needs `focal: Option<Point>` plus inner-circle radius | follow-up issue (today, use `Custom`)
- Convenience `Brush::sweep_gradient` constructor | not requested in #281; `Custom` covers it | follow-up issue
- Gradient-aware `Pen` (strokes drawn with gradient brushes) | requires `Pen::with_brush` and stroke-side peniko brush plumbing | follow-up issue

## Key decisions

| Question | Decision |
|---|---|
| v1 gradient surface — minimal-and-only vs. richer | **Three-variant split**: minimal 2-stop `LinearGradient` / `RadialGradient` for the 80% case AND a peniko-shaped `Custom(peniko::Gradient)` escape hatch. Round-1 product-owner answer. |
| Rich/escape-hatch placement | **(Round-2) Peniko in `quartzite-paint-api`.** `BrushKind::Custom(peniko::Gradient)` lives directly in `paint-api`; the crate gains a `peniko = { version = "0.6", default-features = false }` dependency and **stops** being backend-agnostic. Accepted clean break (AGENTS.md § *API Stability* — pre-`crates.io`). `peniko 0.6` supports `no_std + alloc`, so `paint-api` keeps its `no_std + alloc` posture. |
| 2-stop `LinearGradient` shape | `{ start: Point, end: Point, start_color: Color, end_color: Color }`. Coordinates are widget-local pixels (same space as every other `Painter` method's input). |
| 2-stop `RadialGradient` shape | `{ centre: Point, radius: f32, start_color: Color, end_color: Color }`. Single-circle form (no separate focus point — use `Custom` for that). |
| 2-stop constructor style | `const fn` returning `Self` directly (no `Result`); negative-radius / coincident endpoints are caller-visible (render as empty / degenerate). No panicking ctor. AGENTS.md § *API Naming* — non-panicking by default; `try_*` only when there's a real fallible invariant. |
| `Custom` constructor style | `Brush::custom_gradient(g: peniko::Gradient) -> Self` — takes ownership, not `const` (peniko's `Gradient` is heap-backed). No `try_*`: any `peniko::Gradient` value is renderable (empty stops degrades gracefully — peniko handles it). |
| `Brush` / `BrushKind` `Copy` derive | **Both lose `Copy`.** `Custom(peniko::Gradient)` payload is `Clone`-only (heap-allocated stops), so the enum/struct cannot keep `Copy`. Existing API signatures change: `Brush::kind(self) -> BrushKind` becomes `Brush::kind(&self) -> &BrushKind`. Clean break per AGENTS.md § *API Stability*. The round-1 product-owner answer ("defer to design, API can be freely broken pre-publish") is resolved at the spec level because the round-2 peniko decision forces it — design phase has no choice left here. |
| `BrushKind::Solid` payload | Remains `Solid(Color)` unchanged. New variants are additive in `#[non_exhaustive]`. The internal representation of the existing `Solid` arm is untouched. |
| Coordinate space for gradient endpoints (2-stop variants) | Widget-local pixels (the same space `draw_line` / `draw_rect` already use). The renderer multiplies by `self.scale` per axis exactly as for other painter inputs. |
| Coordinate space for `Custom` | Pass-through; caller is responsible for `self.scale`-aware coordinates. No automatic scaling applied by `VelloPainter` because peniko gradients carry their own transform and the caller may have already baked scale into it. Design phase may revisit if real usage shows the opposite is desired. |
| Extend / spread mode (2-stop) | `Pad` only for v1 (peniko's default). `Custom` callers may pick any peniko `Extend`. |
| Default style fallback for gradient brushes | `quartzite-style::default_style::brush_color` returns **`start_color`** for the 2-stop variants and the **first stop's colour** for `Custom`. Documented in the function's doc comment as the "representative colour" for compositing decisions (not a perceptual average). |
| Where new types live | `quartzite-paint-api/src/brush.rs` — both the 2-stop variant payloads and the `Custom(peniko::Gradient)` variant. No new modules. |
| `quartzite-paint` re-exports | `quartzite-paint::prelude` and the workspace `quartzite::paint` mirror gain `Brush::linear_gradient` / `Brush::radial_gradient` / `Brush::custom_gradient`; `BrushKind` continues to be re-exported. Whether to re-export `peniko::Gradient` itself from `quartzite-paint` is a design-phase pick (probably yes, for one-stop ergonomics). |
| `peniko` feature gating | No cargo feature gate — peniko is unconditional. Adding a `peniko` cargo feature to make it optional would defeat the round-2 decision (the `Custom` variant is part of v1, not opt-in). |
| `peniko` default-features | `default-features = false` (peniko's `std` feature is gated; the crate is `no_std + alloc` by default). Mirror what `quartzite-geometry` does with `thiserror`. |

## Technical constraints

- `quartzite-paint-api` stays `no_std + alloc`. The 2-stop variants need no heap. `peniko 0.6` ships with `no_std + alloc` support; we depend on it with `default-features = false`. AC `cargo build -p quartzite-paint-api --no-default-features` continues to hold.
- `quartzite-paint-api` gains a `peniko` dep — this is an explicit, accepted layering change. The crate is no longer backend-agnostic. Spec-level acknowledgement so design-review doesn't flag it as a regression. (AGENTS.md § *API Stability*: pre-`crates.io`, no downstream clients, free to break.)
- `BrushKind` stays `#[non_exhaustive]`. Adding three variants does not change that.
- `Painter` trait stays object-safe — no signature changes; backends pull richer brush data out of `Brush::kind()`. Per the `Copy`-loss decision, `Brush::kind(&self) -> &BrushKind` returns a reference now; trait methods that take `&Brush` are unaffected.
- All new public items follow workspace doc convention: `#![deny(missing_docs)]`, `# Examples` block, `# Parameters` block when ≥ 1 non-receiver arg. The two `const fn` 2-stop constructors are recursively-simple — they get `#[inline]` per AGENTS.md § *`#[inline]` and the `_Simple._` doc tag*. `Brush::custom_gradient` is also simple (single field-init) and gets `#[inline]`.
- No new error enum needed — none of the constructors are fallible. AGENTS.md § *Error types* (thiserror) is not triggered.
- Per AGENTS.md § *API Stability* (pre-`crates.io`, no downstream clients), the `Brush::kind(self) -> BrushKind` → `Brush::kind(&self) -> &BrushKind` signature change ships as a clean break. No compat shim, no `#[deprecated]` wrapper.
- The existing `quartzite-widgets::tests/snapshots.rs` AC10 carve-out is removed; the AC list in `done/2026-05-09-paint-style.spec.md` AC10 is unaffected (this spec supersedes the carve-out in code, not in the historical spec text).
- `peniko::Gradient` is `Clone + Debug + PartialEq` (verified — peniko 0.6 derives all three on `Gradient`). The `BrushKind` derive list (`Clone, Debug, PartialEq`) keeps compiling after adding `Custom(peniko::Gradient)`.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Brush::linear_gradient(Point::new(0.0, 0.0), Point::new(10.0, 0.0), Color::RED, Color::BLUE)` returns a `Brush` whose `kind()` matches `&BrushKind::LinearGradient { start, end, start_color: Color::RED, end_color: Color::BLUE }`. |
| AC2 | `Brush::radial_gradient(Point::new(5.0, 5.0), 3.0, Color::WHITE, Color::BLACK)` returns a `Brush` whose `kind()` matches `&BrushKind::RadialGradient { centre, radius: 3.0, start_color: Color::WHITE, end_color: Color::BLACK }`. |
| AC3 | Both 2-stop constructors are `const fn` and usable in a `const` context (e.g. assigning to a `const BRUSH: Brush`). |
| AC4 | `Brush::custom_gradient(g)` for some `g: peniko::Gradient` constructs a `Brush` whose `kind()` matches `&BrushKind::Custom(_)` and round-trips the gradient (`if let BrushKind::Custom(ref got) = brush.kind() { assert_eq!(got, &g); }`). |
| AC5 | `BrushKind` retains `#[non_exhaustive]`. `Clone + Debug + PartialEq` continue to hold; `Copy` is **removed** from both `Brush` and `BrushKind`. Two assertions in `brush.rs` tests (`brush_kind_is_copy`, `brush_is_copy`) are removed. |
| AC6 | A mock `Painter` invoking `painter.fill_rect(rect, &Brush::linear_gradient(...))`, `painter.fill_rect(rect, &Brush::radial_gradient(...))`, and `painter.fill_rect(rect, &Brush::custom_gradient(g))` compiles and runs without panic. |
| AC7 | `VelloPainter::fill_rect` with a 2-stop linear-gradient brush renders a non-uniform pattern: in a 10×1 horizontal gradient from `Color::RED` to `Color::BLUE`, the leftmost rendered pixel and the rightmost rendered pixel differ by `> 0.5` in at least one of the R/B channels (snapshot assertion). |
| AC8 | `VelloPainter::fill_rect` with a 2-stop radial-gradient brush renders a non-uniform pattern: the centre pixel and a pixel at radius `r` differ by `> 0.5` in at least one channel. |
| AC9 | `VelloPainter::fill_rect` with a `Custom` 3-stop linear gradient (RED → GREEN → BLUE) renders a non-uniform pattern: the middle pixel of a 11×1 strip is green-dominant (`g > r` and `g > b` by at least `0.2`). |
| AC10 | `VelloPainter::draw_rect` with a gradient brush fills via the gradient AND strokes via the pen's solid colour (gradient applies to fill only; pen stays solid). Verified by snapshot showing distinct fill-vs-stroke colours. |
| AC11 | The `_ => None` catch-all arm in `VelloPainter::brush_color` (or its successor `brush_to_peniko`) no longer triggers for any live `BrushKind` variant (`Solid`, `LinearGradient`, `RadialGradient`, `Custom`). The arm may remain because of `#[non_exhaustive]`, but is dead for in-tree variants. |
| AC12 | The AC10 carve-out comment in `quartzite-widgets/tests/snapshots.rs` (`// AC10 — BrushKind::LinearGradient / RadialGradient are not yet implemented…`) is removed. |
| AC13 | `quartzite-style::default_style::brush_color` returns `start_color` for `LinearGradient` / `RadialGradient`, the first stop's colour for `Custom` (or `Color::TRANSPARENT` if `Custom`'s stops are empty), and `c` for `Solid(c)`. Documented in the helper's doc comment. |
| AC14 | `cargo build -p quartzite-paint-api --no-default-features` succeeds (the crate stays `no_std + alloc`; `peniko` is depended-on with `default-features = false`). |
| AC15 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` is clean. |
| AC16 | `cargo clippy --workspace -- -D warnings` is clean. |
| AC17 | `cargo test --workspace` is green. |

## Open questions

- Pen-side gradient support (strokes drawn with gradient brushes) — deferred, see § Deferred. May resurface once a use case appears.
- Per-vertex / per-corner gradient APIs (e.g. four-colour interpolation across a quad) — not requested in #281; peniko does not expose this directly.
- Whether `quartzite-paint::prelude` re-exports `peniko::Gradient` itself (so callers of `Brush::custom_gradient` don't need a direct `peniko` dep) — design-phase ergonomics call. Default: re-export.
- Whether `Custom` should auto-scale coordinates by `self.scale` in `VelloPainter` (rather than pass-through) — design phase may flip the default if real usage shows pass-through is surprising. The current spec defaults to pass-through.
