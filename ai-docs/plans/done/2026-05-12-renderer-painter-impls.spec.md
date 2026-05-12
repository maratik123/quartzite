# Renderer Painter Method Implementations

**Source:** issue #289 (surfaced from `ai-docs/deferred/widget-backlog.md`; source spec [paint-style](done/2026-05-09-paint-style.spec.md))
**Date:** 2026-05-12
**Tracked in:** #289

> The `paint-style` plan (#47) shipped the full `Painter` trait surface (`draw_rect`, `fill_rect`,
> `draw_line`, `clip_rect`, `translate`, `save`, `restore`, `draw_text`, `draw_text_in`,
> `draw_image`, `draw_path`) plus paint-side `Font` / `Image` / `Path` types. Every method on
> `quartzite_renderer::VelloPainter` is currently an `#[inline] fn …() {}` no-op
> (`quartzite-renderer/src/vello_painter.rs`). This plan lands concrete vello + peniko + wgpu
> implementations of those methods — **including text rendering via `skrifa` + `parley`** —
> so that `RenderHarness::render_widget` and `WindowedApplication` emit real pixels for
> everything the trait can express today. Landing text in this plan closes the previously-
> deferred text-stack tracking issue (#277).

## Scope

Concrete `quartzite-renderer::VelloPainter` implementations of the `Painter` trait, wired
through the existing `vello::Scene` / `vello::Renderer` pipeline already constructed by
`RenderHarness` and `WrappedHandler`. The single backend implementation is shared between the
headless (`RenderHarness`) and windowed (`WindowedApplication`) entry points; both already
instantiate `VelloPainter` and call `paint(&mut dyn Painter)`. Integration touches outside
`vello_painter.rs` are limited to:

1. The two `VelloPainter` construction sites (`RenderHarness::render_widget`,
   `WrappedHandler::draw_window`) — they pass the per-frame `&mut Scene` and the active scale
   factor.
2. A shared font-context module (`quartzite-renderer/src/font.rs` or similar) owning a
   `parley::FontContext` configured with the platform's system font source (fontconfig on
   Linux, CoreText on macOS, DirectWrite on Windows). No bundled font ships with
   `quartzite-renderer`; tests and consumers rely on the host's system fonts. (See Key
   decisions: *Text font-fallback*.)
3. `RenderHarness::new(width, height)` is replaced by a `RenderHarnessBuilder` (fields:
   `width`, `height`, `scale_factor`) finished by `.build() -> Result<RenderHarness, …>`. The
   active scale factor is stored on `RenderHarness` and forwarded to each per-frame
   `VelloPainter`. (See Key decisions: *RenderHarness construction*.)

### `quartzite-renderer::VelloPainter` — fields

- Lifetime-parameterised borrow of the active `vello::Scene`:
  `VelloPainter<'a> { scene: &'a mut Scene, … }`. The painter is constructed inside the
  per-frame scope where the `&mut Scene` is live; both existing call sites
  (`RenderHarness::render_widget`, `WrappedHandler::draw_window`) already reconstruct
  `VelloPainter` per frame, so the lifetime stays local. Compile-time prevents using the
  painter outside a frame. `VelloPainter::new()` grows a `&'a mut Scene` argument
  (signature: `pub fn new(scene: &'a mut Scene) -> VelloPainter<'a>`); the existing zero-arg
  `new()` is gone — there is no other valid use shape. The current
  `Default for VelloPainter` impl is removed (no zero-arg construction). Per AGENTS.md
  § *API Stability*, this clean break has no compat shim.
- Maintains an internal transform stack (`Vec<vello::kurbo::Affine>` or equivalent) consulted
  by every draw method and mutated by `translate` / `save` / `restore`.
- Maintains a parallel clip stack (`Vec<…>`) so `save` / `restore` un-pushes any clip layer
  pushed via `clip_rect` since the matching `save`.

### `quartzite-renderer::VelloPainter` — method bodies

Each of the following replaces a `#[inline] fn …() {}` body with a real implementation routed
through `vello::Scene`:

- `draw_rect(rect, pen, brush)` → `Scene::stroke` over the rect outline (pen width / colour
  from `pen`) followed/preceded by `Scene::fill` with the brush (when brush kind is
  `BrushKind::Solid`). Non-solid brush kinds map to peniko equivalents; `LinearGradient` /
  `RadialGradient` stay deferred (tracked: #281, listed under § Deferred).
- `fill_rect(rect, brush)` → `Scene::fill` only.
- `draw_line(from, to, pen)` → `Scene::stroke` over a `kurbo::Line`.
- `clip_rect(rect)` → `Scene::push_layer` with a rectangular clip (peniko `Mix::Clip`, identity
  blend); records the layer in the active clip-stack frame so `save`/`restore` can balance it.
- `translate(delta)` → multiplies the current transform stack-top by `Affine::translate`. All
  draw methods consult the active transform.
- `save()` → push a new frame onto the transform stack (cloning the current top) and onto the
  clip stack (empty list).
- `restore()` → pop one frame from both stacks; for each clip recorded in the popped frame,
  emit `Scene::pop_layer`.
- `draw_image(rect, image)` → wrap `image.pixels()` in a `peniko::Image` (RGBA8) and call
  `Scene::draw_image` with an `Affine` that scales the unit image into `rect` (sub-pixel
  centring follows the same convention as `fill_rect`).
- `draw_path(path, pen, brush)` → consume `Path::segments()` and translate each `Segment`
  (`MoveTo` / `LineTo` / `CubicTo` / `ArcTo` / `Close`) into a `kurbo::BezPath`. `ArcTo` is
  flattened to cubics via `kurbo::Arc::to_cubic_beziers`. Then `Scene::stroke` (using `pen`)
  and `Scene::fill` (using `brush`) over the bez-path.
- `draw_text(pos, text, font, brush)` and `draw_text_in(rect, text, font, brush, alignment)`
  → shaped via `parley` (using `Font::family` + `Font::size_pt` + `Font::weight` + `Font::italic`
  to build a `parley::FontStack`/`parley::FontStyle`) and rendered via vello's
  `Scene::draw_glyphs` API. Glyph data sourced via `skrifa::FontRef`, fed by `parley`'s
  layout output. `Font::underline` / `Font::strikethrough` are drawn as separate `Scene::stroke`
  passes over the shaped baseline metrics; the brush colour is reused for these decorations.
  `draw_text_in` uses `parley::Layout::break_all_lines` with the rect width as the wrap budget
  and applies `Alignment` to the resulting line set (`Left` / `Center` / `Right` / `Justify`).
  Font discovery: a workspace-level `parley::FontContext` configured with the platform's
  default system font source (fontconfig on Linux, CoreText on macOS, DirectWrite on Windows)
  resolves any `family` string. Unknown families resolve to `parley`'s best system match per
  the underlying platform's matching rules; the matched glyphs are what the snapshot test
  goldens capture. No font is bundled with `quartzite-renderer`. Per-OS golden divergence is
  absorbed by the existing per-backend snapshot matrix (`quartzite-widgets/tests/snapshots/
  <test-name>/<backend>/`) — each runner stores its own goldens, and the CI matrix
  short-circuits on missing-fonts the same way `gpu-snapshot-tests-ci` already short-circuits
  on missing-GPU runners (`continue-on-error: true` for non-bootstrapped per-backend entries).

### Test surface

- Extend `quartzite-renderer/src/vello_painter.rs` `#[cfg(test)] mod tests` with unit tests
  for the transform / clip stacks (call-counting through a probe struct or by observing the
  resulting `vello::Scene` extent — exact mechanism is a design-phase decision).
- Add `#[cfg(test)] mod tests` coverage for `RenderHarnessBuilder` itself
  (`render_harness.rs`): defaults (`scale_factor = 1.0`), explicit `.scale_factor(2.0)`,
  zero-extent rejection paths preserved from the existing `new_zero_*` tests, builder reuse
  semantics if any.
- `RenderHarness`-driven snapshot tests for each newly-real method, following the existing
  `quartzite-widgets/tests/snapshots` pattern (golden PNGs under
  `quartzite-widgets/tests/snapshots/<test-name>/<backend>/`). Goldens are regenerated by
  this PR via `scripts/update-snapshots.sh`; CI gates on the resulting matrix per
  `gpu-snapshot-tests-ci`.
- HiDPI coverage (AC11/AC12) uses `RenderHarnessBuilder::new(w, h).scale_factor(2.0).build()`
  to assert physical-pixel extent under a 2.0 DPR, plus the documented opt-out at 1.0.
- Existing GPU snapshot tests that currently assert the clear colour pick up real-pixel
  content; goldens for those tests are regenerated as part of this plan (called out in
  `gpu-snapshot-tests-ci`'s deferred row about *"goldens are regenerated by follow-up PRs as
  render code lands"*).

## Out of scope

- SVG / PDF export.
- Right-to-left / bidirectional text and complex shaping beyond what `parley`'s default
  `harfbuzz`-equivalent pipeline produces (BiDi-marked input is not specifically tested in
  this plan — basic LTR shaping is the AC target, matching the `paint-style` spec's
  "Basic LTR positioning only" stance).
- A font-cache eviction policy. `parley::FontContext` keeps fonts loaded for the process
  lifetime; an eviction strategy is deferred until widget workloads expose pressure.
- Built-in image decoders — `Image` is still consumed as the raw RGBA8 buffer set up by
  `paint-style`. File / byte-stream decoding stays tracked under #282.
- Sub-pixel font rendering — vello's default greyscale antialiasing is used; sub-pixel AA
  is a separate concern when it becomes available in vello.
- `BrushKind::LinearGradient` / `RadialGradient` rendering — variant already exists on
  `BrushKind` as `#[non_exhaustive]`; backend support tracked under #281.
- `Image` source-rect cropping — tracked under #291.
- New `Painter` trait methods. The trait surface is frozen by `paint-style`; this plan
  implements bodies only.
- New widget `WidgetExt::paint` overrides (e.g. drawing the actual `Label` text glyph run,
  `Button` chrome). Per `paint-style` and `gpu-snapshot-tests-ci`, widget-side `paint`
  implementations are a follow-up; this plan exercises `VelloPainter` directly via tests
  and lets existing widget no-op `paint` continue to produce the clear-colour scene plus
  whatever the widget's own (still no-op) overrides emit. *Exception:* once `VelloPainter`
  is real, the existing test widgets that draw simple shapes via `paint` implicitly start
  emitting real pixels through the new code; that is the snapshot regen path described in
  § Scope and is in-scope here, not a new widget surface.

## Deferred

- `BrushKind::LinearGradient` / `RadialGradient` rendering | needs gradient-stop API + peniko `Gradient` wiring | already tracked: #281
- `Image` source-rect cropping | trait surface lacks a source rect; would require a `Painter` method addition | already tracked: #291
- Per-test perceptual-diff tolerance tuning | calibration once real pixels exist (mentioned in `gpu-snapshot-tests-ci` open questions) | already tracked: #286
- RTL / BiDi text and complex script shaping | tracked separately when a non-LTR widget surface lands | no separate issue (extension within this scope after BiDi-capable widget arrives)
- Font-cache eviction strategy for `parley::FontContext` | needs workload data | no separate issue

## Key decisions

| Question | Decision |
|---|---|
| Backend | vello + peniko + wgpu (already wired by `graphics-stack`; the trait-impl bodies use `vello::Scene` directly). Text stack: `skrifa` (glyph data) + `parley` (shaping / layout). |
| Trait surface | Frozen by `paint-style` (#47). This plan implements method bodies only — no new methods, no signature changes. |
| Text scope | **Implement now.** `draw_text` / `draw_text_in` ship in this plan via `skrifa` + `parley` + `vello::Scene::draw_glyphs`. Lands the work tracked by #277; that issue closes when this plan merges. |
| Painter ↔ scene wiring | **Borrow scene.** `VelloPainter<'a> { scene: &'a mut Scene, … }`. Compile-time prevents misuse; both call sites already reconstruct the painter per frame. `new()` grows a `&'a mut Scene` argument: `pub fn new(scene: &'a mut Scene) -> VelloPainter<'a>`. The current zero-arg `new()` and `Default` impl are removed (clean break per AGENTS.md § *API Stability*). |
| Coordinate space | **Logical pixels with opt-out.** `Painter` methods accept `quartzite-geometry` integer coordinates interpreted as *logical* pixels by default. `VelloPainter` multiplies by the active scale factor (DPR) before issuing `vello::Scene` calls. An opt-out — likely a constructor variant (e.g. `VelloPainter::with_physical_pixels(...)` or a `set_scale_factor(1.0)` knob) — lets callers that have already pre-scaled their coordinates suppress the multiplication. Scale factor source: `RenderHarnessBuilder` (default `scale_factor = 1.0` — test pixels are physical by default for predictable goldens; HiDPI tests opt in via `.scale_factor(2.0)`); `WrappedHandler::draw_window` propagates `winit::window::Window::scale_factor()` per frame. Exact API shape for the `VelloPainter` opt-out is a design-phase decision. |
| Transform/clip state | Maintained inside `VelloPainter` (transform `Vec<Affine>`, clip stack `Vec<…>`). `save` / `restore` are pure stack push/pop; per the `paint-style` spec they are "delegated to the concrete backend" and this is the backend. |
| `Path::Segment::ArcTo` lowering | Flattened to cubics via `kurbo::Arc::to_cubic_beziers` (kurbo's standard approximation); no separate fast-path. |
| `draw_image` scaling | Image is sampled bilinearly (peniko `ImageQuality::Medium` default) and scaled into the destination `rect` by an `Affine`. Source-rect cropping deferred. |
| Brush dispatch | `BrushKind::Solid` → peniko solid brush. `LinearGradient` / `RadialGradient` variants return early without drawing (deferred, tracked under #281); calls do **not** panic. |
| Pen dispatch | `Pen { color, width }` → `peniko::Stroke { width, … }` + solid brush from `Pen::color()`. |
| Object safety | Preserved. `Painter` already object-safe; no method gains a generic parameter. |
| Renderer error handling | Per the existing `VelloPainter` rustdoc: "rendering errors are non-recoverable. Methods panic or log on failure; `PaintError` is reserved for a future API version when Painter methods gain Result return types." This plan preserves that policy for the new bodies. |
| Text font-fallback | **Use the platform's system font source — no bundled font.** A workspace-level `parley::FontContext` configured with the default system source (fontconfig on Linux, CoreText on macOS, DirectWrite on Windows) resolves any `family` string. Unknown families fall through to the platform matcher's best system match; the snapshot test goldens capture whatever the host produces. Per-OS visual divergence is absorbed by the existing per-backend snapshot matrix (`quartzite-widgets/tests/snapshots/<test-name>/<backend>/`). Test runners that lack the requested family follow the existing `continue-on-error: true` / `#[ignore = "..."]` policy from `gpu-snapshot-tests-ci`. Rejected: bundling a font (licensing footprint, binary-size overhead — outweighed by user preference for fontconfig-style platform integration). |
| `RenderHarness` construction | **Builder.** `RenderHarness::new(width, height)` is removed (clean break per AGENTS.md § *API Stability*). New shape: `RenderHarnessBuilder::new(width: u32, height: u32) -> Self` with field setters (`scale_factor(f32)` — default `1.0`), finished by `.build() -> Result<RenderHarness, RendererError>`. Validation (zero-extent rejection, GPU adapter / device init) happens in `.build()`; the builder itself is infallible. The active `scale_factor` is stored on `RenderHarness` and forwarded to every per-frame `VelloPainter`. `WrappedHandler::draw_window` continues to propagate `winit::window::Window::scale_factor()` directly per frame (no builder there). |
| Snapshot bootstrapping | New goldens regenerated via `scripts/update-snapshots.sh` and committed under `quartzite-widgets/tests/snapshots/**`. Per-backend matrix entries (`win`, `mac`) remain `continue-on-error: true` until their own goldens are bootstrapped (current `gpu-snapshot-tests-ci` policy). |

## Technical constraints

- `quartzite-renderer` continues to depend on `vello`, `peniko`, `wgpu`, `pollster`, `winit`,
  `quartzite-paint-api`, `quartzite-events`, `quartzite-geometry`, `quartzite-runtime`,
  `image`, and **gains** `skrifa` + `parley` for text shaping and glyph data. `parley` is
  configured with the default platform system-font source (no separate fontconfig
  dependency on the Rust side — `parley` brings it transitively on Linux). Versions pinned
  per AGENTS.md § *Dependency Versions* (registry-queried before commit; pinned to the
  major / 0.x semantic).
- `Painter` object-safety preserved (no `Self`-return, no generics, no associated types).
- `VelloPainter::new()` grows a `&'a mut Scene` argument
  (`pub fn new(scene: &'a mut Scene) -> VelloPainter<'a>`); the zero-arg form and the
  `Default` impl are removed. Both call sites (`RenderHarness::render_widget`,
  `WrappedHandler::draw_window`) already build the painter inside the per-frame scope where
  `Scene` is live, so the call-site update is a one-line each.
- All new test code lives under `#[cfg(test)] mod tests` in `vello_painter.rs` (unit tests),
  in `render_harness.rs` (builder unit tests), or in `quartzite-widgets/tests/` snapshot tests
  (per `gpu-snapshot-tests-ci` location policy).
- `RenderHarness::new(width, height)` is removed. New construction shape:
  `RenderHarnessBuilder::new(width, height).scale_factor(2.0).build()?`. Default
  `scale_factor` is `1.0` (preserves existing test pixel-extent semantics for goldens that
  do not opt into HiDPI). The builder is infallible; `.build()` returns the existing
  `Result<RenderHarness, RendererError>`. Both call sites that currently invoke
  `RenderHarness::new` are updated; `xvfb_smoke.rs` and any other downstream test code in
  this workspace is updated in the same PR (clean break per AGENTS.md § *API Stability*).
- GPU-needing snapshot tests follow the existing `#[ignore = "requires GPU"]` /
  `wgpu-bootstrap` gating (see `gpu-snapshot-tests-ci`).
- Coordinate translation: `quartzite-geometry` uses integer `Point` / `Rect` / `Size`; vello
  uses `kurbo` floats. The conversion happens inside `VelloPainter`. Default semantics are
  *logical pixels* (Key decisions: coord-space row) — the painter multiplies by the active
  scale factor before issuing `vello::Scene` calls. Callers that have already pre-scaled
  their coordinates use the documented opt-out.
- `#[deny(missing_docs)]` + `# Examples` block on any new public item per workspace doc
  convention. The new fields on `VelloPainter` should remain non-`pub`; doc impact is limited
  to method-body Examples that already exist on the trait declarations.
- Recursive-simple fns get `#[inline]` per AGENTS.md § *`#[inline]` and the `_Simple._` doc
  tag*.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `RenderHarness::render_widget` on a widget whose `paint` calls `painter.fill_rect(Rect::new(Point::new(8, 8), Size::new(48, 48)), &Brush::solid(Color::RED))` produces an `image::RgbaImage` whose pixel at `(32, 32)` has `R == 255, G == 0, B == 0, A == 255` (within the perceptual-diff tolerance configured by `gpu-snapshot-tests-ci`). |
| AC2 | `RenderHarness::render_widget` on a widget whose `paint` calls `painter.draw_rect(rect, &pen, &brush)` with a non-zero pen width produces a pixel-different image from one that called only `painter.fill_rect(rect, &brush)` with the same brush — i.e. the stroke outline is rendered (not silently dropped). Asserted via byte-comparison of the two `RgbaImage`s. |
| AC3 | `RenderHarness::render_widget` on a widget whose `paint` builds `Path::new().move_to(...).line_to(...).cubic_to(...).arc_to(...).close()` and calls `painter.draw_path(&path, &pen, &brush)` produces non-clear-colour pixels along the resolved curve. Asserted by counting non-`BASE_COLOR` pixels in the resulting `RgbaImage`. |
| AC4 | `RenderHarness::render_widget` on a widget whose `paint` calls `painter.draw_image(rect, &image)` for an `Image::try_new(2, 2, vec![255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255])` produces an `RgbaImage` whose four quadrants of `rect` carry the four source colours (within tolerance). |
| AC5 | Transform state is correct: a widget whose `paint` does `save(); translate(Point::new(10, 0)); fill_rect(r, &brush); restore(); fill_rect(r, &brush);` produces an image with two rectangles, the first offset by 10px on x, the second at the unshifted origin. |
| AC6 | Clip state is correct: a widget whose `paint` does `save(); clip_rect(small_rect); fill_rect(large_rect, &brush); restore();` produces an image where filled pixels exist only inside `small_rect`. |
| AC7 | Object safety preserved — the existing `all_painter_methods_are_invocable` test in `vello_painter.rs` continues to pass under `Box<dyn Painter>` after the implementation lands. |
| AC8 | `RenderHarness::render_widget` on a widget whose `paint` calls `painter.draw_text(Point::new(8, 24), "Hello", &Font::new("sans-serif", 16.0), &Brush::solid(Color::WHITE))` (using the `"sans-serif"` generic family, resolved via the platform's system-font source) produces non-clear-colour pixels along the rendered baseline. Asserted against a committed golden under `quartzite-widgets/tests/snapshots/draw_text/<backend>/`; per-backend goldens absorb the visual divergence between OS font matchers. |
| AC9 | `RenderHarness::render_widget` on a widget whose `paint` calls `painter.draw_text_in(rect, "wrap me", &font, &brush, Alignment::Center)` produces an image where the rendered glyphs are horizontally centred within `rect` (assertion: leftmost-non-clear-colour pixel of the first line and rightmost-non-clear-colour pixel are roughly equidistant from `rect`'s edges, within ±2 pixels). Right- and Justify- variants are smoke-tested via call-path coverage; centring is the only metrics-asserted alignment in this plan. |
| AC10 | `BrushKind::LinearGradient` / `RadialGradient` calls do not panic — they early-return without drawing. Asserted by a `RenderHarness::render_widget` snapshot whose `paint` invokes `fill_rect(rect, &Brush::linear_gradient(...))` and compares the resulting `RgbaImage` to a clear-colour image. |
| AC11 | Coordinate-space default: a `RenderHarness` built via `RenderHarnessBuilder::new(40, 40).scale_factor(2.0).build()?` rendering a widget whose `paint` calls `painter.fill_rect(Rect::new(Point::new(0, 0), Size::new(10, 10)), &brush)` produces an image where the filled region spans approximately 20×20 *physical* pixels (within ±1 px due to integer rounding). |
| AC12 | Coordinate-space opt-out: a `RenderHarness` built via `RenderHarnessBuilder::new(40, 40).scale_factor(1.0).build()?` (the builder default — explicit here for clarity) makes the same `fill_rect(..., Size::new(10, 10), ...)` cover 10×10 physical pixels regardless of any window-DPR caller code. |
| AC12a | `RenderHarnessBuilder` unit tests cover: default `scale_factor` is `1.0`; explicit `.scale_factor(2.0)` round-trips; zero-extent rejection (preserved from the existing `new_zero_width_returns_err` / `new_zero_height_returns_err` / `new_zero_both_returns_err` tests) fires in `.build()` rather than the builder constructor. |
| AC13 | Existing snapshot tests under `quartzite-widgets/tests/snapshots/` either pass against existing goldens or have their goldens regenerated in this PR (file diff visible in the PR). Per-backend matrix entries that were `continue-on-error: true` continue to be so. |
| AC14 | `cargo build` on the linux runner (and the existing matrix) succeeds. |
| AC15 | `cargo clippy --workspace -- -D warnings` is clean. |
| AC16 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` is clean. |
| AC17 | Tracking issue #277 (*Text layout and font loading*) is closed by this PR (via `Closes #277` in the PR body), reflecting that text rendering ships in this plan. |

## Open questions

_(None — all design-affecting ambiguities resolved across interview rounds 1–3. See
*Resolution log* for the decision trail.)_

Items the design agent (`/design`) will still own — not blockers for spec acceptance:

- Exact stack-state probe mechanism for the transform / clip unit tests (call-counting
  probe vs. observing `vello::Scene` extent).
- Exact API shape of the coordinate-space opt-out on `VelloPainter` (constructor variant
  `with_physical_pixels(...)` vs. `set_scale_factor(1.0)` setter) — both satisfy AC11/AC12;
  the design agent picks the more ergonomic shape.
- Per-OS golden divergence absorption strategy in `gpu-snapshot-tests-ci` (whether new
  `draw_text*` snapshots get `continue-on-error: true` until each runner bootstraps its
  golden, vs. seeding linux-only at merge and bootstrapping mac/win in follow-up PRs).

## Resolution log

Interview rounds 1–3 held 2026-05-12 via `/interview` (issue #289). Decisions taken; folded
into the spec body above.

| Round | Question | Decision |
|---|---|---|
| 1 | `draw_text` / `draw_text_in` scope — implement now (pulls in `skrifa` + `parley`, closes #277) or stay no-op pending #277? | **Implement now.** Text ships in this plan; #277 closes when this PR merges. |
| 1 | `VelloPainter` ↔ `vello::Scene` wiring shape? | Pros/cons drafted; user requested explicit comparison and deferred selection to Round 2. |
| 1 | Coordinate-space convention at the Painter boundary? | **Logical + opt-out.** Default: `VelloPainter` interprets `Point`/`Rect`/`Size` as logical pixels and multiplies by the active scale factor. An opt-out (constructor variant or scale-factor knob) lets pre-scaled callers suppress the multiplication. |
| 2 | `VelloPainter` ↔ `vello::Scene` wiring shape (pros/cons provided)? | **Borrow scene.** `VelloPainter<'a> { scene: &'a mut Scene, … }`. Compile-time prevents misuse; reconstruct per frame (already happens). `new()` grows a `&mut Scene` arg. |
| 3 | Which bundled fallback font ships with `quartzite-renderer` for text tests? | **No bundled font — use the platform system source.** `parley::FontContext` is configured with its default system source (fontconfig / CoreText / DirectWrite); tests rely on host fonts. Per-OS visual divergence is absorbed by the existing per-backend snapshot matrix. |
| 3 | How should `RenderHarness` expose scale factor (DPR) for AC11/AC12? | **Builder.** `RenderHarness::new(width, height)` is removed; new shape is `RenderHarnessBuilder::new(width, height).scale_factor(f32).build() -> Result<RenderHarness, RendererError>`. Default `scale_factor` is `1.0`. Clean break per AGENTS.md § *API Stability*. |
