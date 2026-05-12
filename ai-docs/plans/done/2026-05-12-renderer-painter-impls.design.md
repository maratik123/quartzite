# Design: Renderer Painter Method Implementations

**Issue:** #289 (spec: [`2026-05-12-renderer-painter-impls.spec.md`](2026-05-12-renderer-painter-impls.spec.md); closes #277 on merge)
**Date:** 2026-05-12

## Approach

The spec freezes every macro-level decision (backend = vello + peniko + wgpu; text
stack = `skrifa` + `parley`; scene wiring = lifetime borrow; coordinate space =
logical-pixels-with-opt-out; render-harness construction = builder; brush gradients
deferred). The remaining design surface is:

1. How the lifetime-borrowed `VelloPainter<'a>` keeps a transform/clip stack and
   feeds shaped glyphs from `parley` into `vello::Scene::draw_glyphs`.
2. Where the workspace-shared `parley::FontContext` lives, given that *both* the
   headless `RenderHarness` and the windowed `WrappedHandler` need to feed the
   same painter and we want font loading to amortise across frames.
3. The ergonomics of the coordinate-space opt-out.
4. The probe mechanism for transform/clip stack unit tests.

### Chosen solutions

**Painter internals.** Make `VelloPainter<'a>` carry:

```text
scene:        &'a mut vello::Scene
fonts:        &'a mut FontCache       // see (2) below
scale:        f32                     // active DPR multiplier, 1.0 by default
xforms:       Vec<kurbo::Affine>      // initially [Affine::IDENTITY]
clips:        Vec<u32>                // count of layers pushed in each save-frame
```

`save` pushes a clone of the top transform and a fresh `0` clip count. `restore`
pops one frame and calls `scene.pop_layer()` that many times. `translate` left-
multiplies (or right-multiplies, see below) the top transform. `clip_rect` calls
`scene.push_layer(Mix::Clip.into(), 1.0, top_xform, &rect_kurbo)` and increments
the top clip count. **Transform composition order:** `translate(delta)` is
applied *after* any existing transform on the stack — i.e. new top =
`old_top * Affine::translate(delta_scaled)`. This matches the spec's AC5
expectation that `save() ; translate((10,0)) ; fill_rect(r) ; restore() ;
fill_rect(r)` draws the *first* rect shifted by 10 px.

**Coordinate-space opt-out — chosen shape.** A no-suffix builder-style setter
on the painter: `VelloPainter::with_scale(self, scale: f32) -> Self`. The
spec leaves the choice between `with_physical_pixels(...)` and
`set_scale_factor(1.0)` open; `with_scale(scale)` is more general (callers can
also dial intermediate values, e.g. for tests of fractional DPR), composes with
the `new(scene)` constructor as a chain (`VelloPainter::new(scene).with_scale(2.0)`),
and avoids the `_unchecked`/`_unverified` suffix ambiguity. The default — when
`with_scale` is never called — is `1.0` so callers that pre-scaled their
coordinates get exactly what they passed. Both call sites build the painter
inline and chain `with_scale` from a scalar they already hold:

- `RenderHarness::render_widget` chains `.with_scale(self.scale_factor)`.
- `WrappedHandler` (the `RedrawRequested` arm — the spec calls it
  `draw_window`, but the actual entry is `dispatch_window_event_inner`) chains
  `.with_scale(window.scale_factor() as f32)`.

**Font cache placement.** A new private module
`quartzite-renderer/src/font.rs` exposes:

```text
pub(crate) struct FontCache {
    fctx: parley::FontContext,
    lctx: parley::LayoutContext<peniko::Brush>,
    // family-key -> resolved peniko::FontData blob, populated on first lookup
    blobs: HashMap<FontKey, peniko::FontData>,
}
```

- `parley::FontContext` is configured with `parley::FontContext::new()` (its
  default constructor uses the platform's system font source — fontconfig on
  Linux, CoreText on macOS, DirectWrite on Windows). No bundled font.
- `parley::LayoutContext<peniko::Brush>` is held alongside so each frame's
  `RangedBuilder` can reuse its scratch buffers (parley's design — the layout
  context is the per-process arena).
- A `FontKey` (family-string + weight + italic) is sufficient to dedupe blob
  loading: text variations are realised by `parley` via the `FontContext` at
  layout time, but converting a resolved `parley::Font` into a
  `peniko::FontData` (an `Arc<Vec<u8>>`-style blob handle) is the only
  per-family cost worth caching here — `parley` itself caches further glyph
  outlines internally.

`FontCache` lives **on `RenderHarness`** (one instance per harness, reused
across `render_widget` calls) and **on `WrappedHandler`** (one instance per
windowed application, reused across windows and frames). Both surfaces pass
`&mut self.fonts` into `VelloPainter::new(&mut scene)` via a follow-on
`with_fonts(&mut fonts)` chain — kept symmetric with `with_scale`. The cache is
**not** a global singleton; that would couple the two entry points and break
the explicit "harness deliberately doesn't construct an `Application`"
posture (`render_harness.rs` rustdoc). The cost is one extra `parley::FontContext`
per harness instance — acceptable because tests dominate harness usage and a
single test process typically constructs one harness.

**Glyph emission.** Per shaped line from `parley::Layout::break_all_lines`,
iterate runs (`parley::PositionedLayoutItem::GlyphRun`). For each run:

1. Resolve the run's font to a `peniko::FontData` blob via `FontCache` (cache
   key = run's `parley::Font` blob identity; ie `Arc::as_ptr`).
2. Build `Scene::draw_glyphs(&font_data)`:
   - `.font_size(run.font_size())`
   - `.transform(top_xform * Affine::translate((line_x_offset + run.offset(), baseline_y)))`
   - `.brush(peniko_brush_from(brush))`
   - `.draw(Fill::NonZero, run.positioned_glyphs().map(|g| Glyph { id: g.id.to_u32(), x: g.x, y: g.y }))`
3. After the run, if `font.underline()` or `font.strikethrough()`, emit a
   `Scene::stroke` over a horizontal segment at the run's underline/strikethrough
   metrics (sourced from the run's metrics; brush colour matches the glyph fill,
   per spec § Scope).

`draw_text` builds a single-line layout (`break_all_lines(None)`) and emits at
`pos`. `draw_text_in` builds the layout with `break_all_lines(Some(rect.size().width() as f32))`
and applies the run's `Alignment` to each line by offsetting `line_x_offset` by
`(rect.width - line.metrics().advance) * factor` where `factor` is `0.0` /
`0.5` / `1.0` / parley's justify pass for `Left` / `Center` / `Right` /
`Justify`.

**Stack/clip test probe — chosen shape.** Add a `#[cfg(test)]`-only inherent
accessor on `VelloPainter` that returns the stack lengths:
`pub(crate) fn debug_stack_state(&self) -> (usize, u32)` — `(transform_depth,
total_active_clips)`. This is strictly internal: the trait surface stays object-
safe and frozen per AC7. Combined with `vello::Scene`'s public encoding-size
accessor (`Scene::encoding()` returns `&Encoding`, `Encoding::is_empty()` is
public), the unit tests can assert that:

- After `save(); translate(p); fill_rect(r); restore(); fill_rect(r);` the
  stack depth is back to 1 and the scene encoding is non-empty (we don't peek
  at exact transforms; AC5 already does pixel-level verification end-to-end).
- After `save(); clip_rect(small); fill_rect(big); restore();` the
  `total_active_clips` returns to `0` (and AC6 verifies pixel-level clipping).

No external mocking framework is needed; the probe is one accessor.

### Rejected alternatives

| Alternative | Why rejected |
|---|---|
| **Owned `Scene` inside `VelloPainter`** | Spec round 2 already chose `&'a mut Scene` — the painter is per-frame, the scene is owned by the harness/handler. Re-litigating would contradict spec § Key decisions. |
| **Single global `parley::FontContext` via `OnceLock`** | Couples harness + windowed entry points (breaks the spec's "harness deliberately does not construct an Application"). Also blocks future per-test font isolation. The per-instance cost (one mmap-backed font enumeration) is acceptable. |
| **Per-frame `parley::LayoutContext`** | Defeats parley's whole point — `LayoutContext` is its scratch arena. Allocating one per frame would re-create glyph-cluster buffers every redraw. Kept on `FontCache` instead. |
| **Translating `quartzite-geometry`'s integer coords with `f64` immediately** | `kurbo::Affine` is `f64`-based; lossless conversion from `i32` → `f64` is free. We use `f64` inside the affine then float-cast back if needed for glyph offsets (parley returns `f32`). Pure `f32` math everywhere would force `as f32` casts that lose precision on translate; `f64` is what vello expects anyway. |
| **`with_physical_pixels()` constructor variant** | Open question 2 listed this as an option. Rejected in favour of `with_scale(scale)` which is more general (admits 1.5, 2.0, 3.0, …) and named for the *what*, not the *how*. |
| **Implementing widget-side `paint` overrides in this plan** | Spec § Out of scope freezes that out; widget `paint` overrides are a follow-up. |

## Decomposition

The seven tasks below are atomic and have explicit dependencies. Tasks 1, 4, and 6
each touch a single primary file plus its `#[cfg(test)] mod tests`; tasks 2/3
build on 1.

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Replace `RenderHarness::new(w, h)` with `RenderHarnessBuilder`; store `scale_factor` on `RenderHarness`; update `render_widget` to thread `scale_factor` into the future `VelloPainter::with_scale`. Update existing snapshot callers (`quartzite-widgets/tests/snapshots.rs`) and inline-doc `no_run` snippets in `render_harness.rs` to the builder shape. Preserve zero-extent rejection in `.build()`. | `quartzite-renderer/src/render_harness.rs`, `quartzite-widgets/tests/snapshots.rs` | — |
| 2 | Add `quartzite-renderer/src/font.rs` (`FontCache` owning `parley::FontContext` + `parley::LayoutContext<peniko::Brush>` + family-keyed `HashMap<FontKey, peniko::FontData>`). Add `skrifa` + `parley` to `quartzite-renderer/Cargo.toml` (versions registry-verified per AGENTS.md). Wire the cache into `RenderHarness` (new field, init in `.build()`) and into `WrappedHandler` (new field, init in `WrappedHandler::new`). | `quartzite-renderer/src/font.rs`, `quartzite-renderer/Cargo.toml`, `quartzite-renderer/src/render_harness.rs`, `quartzite-renderer/src/wrapped_handler.rs`, `quartzite-renderer/src/lib.rs` (module declaration) | 1 |
| 3 | Rewrite `VelloPainter`. Replace unit struct with `VelloPainter<'a> { scene, fonts, scale, xforms, clips }`. Replace `new()` with `new(scene: &'a mut Scene) -> Self`; add chainable `with_scale(self, scale: f32) -> Self` and `with_fonts(self, fonts: &'a mut FontCache) -> Self`. Remove the `Default` impl. Implement all non-text trait methods (`draw_rect`, `fill_rect`, `draw_line`, `clip_rect`, `translate`, `save`, `restore`, `draw_image`, `draw_path`) atop `Scene`. Map `BrushKind::Solid` → peniko solid brush; non-solid variants are matched and `return` without drawing (no panic, per spec key decision). `draw_path` lowers `Segment::ArcTo` via `kurbo::Arc::to_cubic_beziers`. Update both call sites (`RenderHarness::render_widget`, `WrappedHandler::dispatch_window_event_inner::WindowEvent::RedrawRequested`) to build the painter with the new signature. | `quartzite-renderer/src/vello_painter.rs`, `quartzite-renderer/src/render_harness.rs`, `quartzite-renderer/src/wrapped_handler.rs` | 2 |
| 4 | Implement `VelloPainter::draw_text` and `VelloPainter::draw_text_in`. Shape via `parley::RangedBuilder` from the cached `LayoutContext`; resolve the run's font to a `peniko::FontData` via `FontCache`; emit glyphs via `Scene::draw_glyphs(&font_data).font_size(...).transform(...).brush(...).draw(Fill::NonZero, glyph_iter)`. Handle alignment in `draw_text_in` (`Alignment::Left`/`Center`/`Right`/`Justify`). Emit underline / strikethrough as `Scene::stroke` passes on the baseline metrics when `Font::underline()` / `Font::strikethrough()` is set. | `quartzite-renderer/src/vello_painter.rs`, `quartzite-renderer/src/font.rs` | 3 |
| 5 | Add unit tests for the stack/clip logic via the `#[cfg(test)]` `debug_stack_state` probe; add `RenderHarnessBuilder` unit tests (default scale_factor=1.0, explicit setter round-trip, zero-extent rejection in `.build()`); preserve and rename the existing `new_zero_*` tests. Update / refresh `all_painter_methods_are_invocable` to construct the painter with the new lifetime-borrow signature (still asserts object safety per AC7). | `quartzite-renderer/src/vello_painter.rs` (`#[cfg(test)]`), `quartzite-renderer/src/render_harness.rs` (`#[cfg(test)]`) | 3, 4 |
| 6 | Add new snapshot tests in `quartzite-widgets/tests/snapshots.rs` exercising each painter method against a small in-test `WidgetExt` wrapper that calls the appropriate painter method in its `paint()`. New snapshots: `fill_rect`, `draw_rect`, `draw_line`, `draw_path`, `clip_rect_save_restore`, `translate_save_restore`, `draw_image`, `draw_text`, `draw_text_in_center`, `draw_image_quadrants`, `gradient_brush_no_panic`, `hidpi_2x_extent` (HiDPI uses `RenderHarnessBuilder::scale_factor(2.0)`). Regenerate goldens via `scripts/update-snapshots.sh` on Linux; per-OS goldens for `draw_text*` follow the existing `continue-on-error: true` pattern for non-bootstrapped backends. | `quartzite-widgets/tests/snapshots.rs`, `quartzite-widgets/tests/snapshots/<backend>/<name>.png` (regenerated goldens), `quartzite-widgets/tests/snapshots/shared/<name>.png` (for visually-stable non-text snapshots) | 5 |
| 7 | Final hygiene: doc-gate (`cargo doc --no-deps --workspace --all-features` clean under `-D warnings -D missing-docs`); `cargo clippy --workspace -- -D warnings` clean; `# Examples` block on every new public item (`RenderHarnessBuilder::new`, `RenderHarnessBuilder::scale_factor`, `RenderHarnessBuilder::build`, `VelloPainter::new`, `VelloPainter::with_scale`, `VelloPainter::with_fonts`); update `lib.rs` `pub use` for `RenderHarnessBuilder`; close #277 via `Closes #277` in the PR body. Run `cargo build -p quartzite --no-default-features` to confirm the no_std-feature root still compiles (renderer is std-only, but the gate matters). | `quartzite-renderer/src/lib.rs`, `quartzite-renderer/src/render_harness.rs`, `quartzite-renderer/src/vello_painter.rs` (doc additions) | 6 |

> Decomposition stays at exactly 7 tasks (the spec's "if > 7 → propose splitting"
> threshold). Task 6 carries the bulk of the new goldens but is one logical step
> (add tests + regenerate). Splitting it further (e.g. by AC) would create
> spurious commits that all touch the same file and the same regen script.

## Risks

- **Skrifa version skew between vello (`0.40`) and parley (`0.42`).** The `Cargo.lock`
  already shows vello 0.8 transitively pulling skrifa 0.40; parley 0.9 will pull
  skrifa 0.42 alongside. Both can coexist as parallel crates but type identity is
  not shared — when emitting glyphs into `Scene::draw_glyphs`, the `Glyph { id, x, y }`
  iterator items must use the type vello expects (`vello_encoding::Glyph`, re-
  exported through vello), constructed from `u32` IDs that parley returns.
  *Mitigation:* always convert `parley::Glyph` → `vello_encoding::Glyph` at the
  call site (a 3-field construction); never pass a `skrifa::GlyphId` directly.
  Document this conversion in `font.rs`. Re-verify on each `cargo update` that
  both still resolve to compatible majors.

- **`parley::FontContext::new()` may block on first call (system font enumeration).**
  On Linux fontconfig walks the cache; on Wayland-only desktops without
  fontconfig populated, enumeration can fail. *Mitigation:* `FontCache` is
  built lazily inside `.build()` — failure propagates as `RendererError::Paint`.
  Existing tests already short-circuit on missing GPU; missing fonts get the
  same "skip with notice" treatment via per-backend `continue-on-error` in CI.

- **HiDPI rounding (AC11 says "within ±1 px").** Multiplying integer logical
  coordinates by a non-integer scale (e.g. `1.5`) produces non-integer pixel
  edges. `kurbo`'s rasteriser anti-aliases these correctly, but the pixel-count
  assertion in AC11 must tolerate edge AA. *Mitigation:* AC11 explicitly says
  "approximately 20×20 *physical* pixels (within ±1 px)" and the test counts
  non-clear-colour pixels with a tolerance.

- **Snapshot golden churn for existing tests.** `label_renders`, `button_renders`,
  `line_edit_renders`, `box_layout_renders`, `grid_layout_renders` currently
  produce all-clear-colour goldens. Even with no widget-side paint overrides,
  the lifetime-borrow refactor of `VelloPainter` itself can't alter the pixels
  for a no-op `paint`, so existing goldens *should* stay byte-identical.
  *Mitigation:* run the snapshot suite before regenerating — if the existing
  five snapshots stay green without regen, leave them alone. Only regenerate
  the new snapshots added in Task 6. AC13 explicitly permits either outcome.

- **API breakage: `RenderHarness::new` removed, `VelloPainter::new()` signature
  changed, `Default for VelloPainter` removed.** Per AGENTS.md § *API Stability*
  this is a clean break (no compat shims). *Mitigation:* every call site is
  inside this workspace — listed in the decomposition; the PR diff is the audit
  trail. Project has no downstream clients.

- **Object-safety regression on `Painter` trait.** AC7 explicitly tests
  `Box<dyn Painter>`. The new fields on `VelloPainter` are not associated types
  and not in the trait — they're on the impl. *Mitigation:* keep the trait
  signatures byte-identical; the existing `all_painter_methods_are_invocable`
  test is the regression guard.

- **Panic surface.** Per spec § Key decisions (renderer error handling),
  rendering errors are non-recoverable; methods "panic or log on failure".
  `peniko::FontData::new` (blob construction) and `parley::FontContext` lookups
  return `Result`; we propagate via `expect()` with a descriptive message so a
  panic message identifies the failing family. *Mitigation:* document the
  panic conditions on the method-level rustdoc for `draw_text` / `draw_text_in`
  in a `# Panics` block — both methods can now panic on font-resolution failure
  where the v0 no-ops could not.

## Test Design

### Task 1 — `RenderHarnessBuilder` & scale-factor propagation

- **Location:** `quartzite-renderer/src/render_harness.rs` `#[cfg(test)] mod tests`.
- **Entry points:** `RenderHarnessBuilder::new`, `.scale_factor()`, `.build()`,
  `RenderHarness::scale_factor()` (new getter).
- **Scenarios:**
  - `builder_default_scale_factor_is_1_0` — `Builder::new(64, 64).build()?.scale_factor() == 1.0`.
  - `builder_explicit_scale_factor_round_trips` — `.scale_factor(2.0).build()?.scale_factor() == 2.0`.
  - `builder_zero_width_returns_err_from_build` — preserves the existing zero-extent
    rejection but the error fires from `.build()`, not the builder constructor.
  - `builder_zero_height_returns_err_from_build`, `builder_zero_both_returns_err_from_build`.
- **Fixtures:** none beyond `RenderHarnessBuilder::new(w, h)`. GPU-touching
  builds keep the existing GPU-skip pattern (`SKIP_RENDER_SNAPSHOT=1` /
  no-adapter early return).

### Task 3 — non-text `VelloPainter` bodies

- **Location:** `quartzite-renderer/src/vello_painter.rs` `#[cfg(test)] mod tests`
  (unit-level invariants); end-to-end pixel assertions move to Task 6.
- **Entry points:** every `Painter` trait method on `VelloPainter`.
- **Scenarios:**
  - `painter_starts_with_identity_transform_and_no_clips` — fresh painter, probe
    returns `(1, 0)`.
  - `save_then_restore_round_trip` — probe returns `(1, 0)` again after save+restore.
  - `translate_modifies_top_only` — save+translate+restore leaves probe at `(1, 0)`;
    nested save→translate→save→translate exposes depth 3.
  - `clip_rect_increments_active_clip_count` — probe shows `1` clip after
    `clip_rect`; `restore` returns it to `0`.
  - `gradient_brush_early_returns_without_panic` — exercises AC10's no-panic
    contract by calling `fill_rect` with a (test-helper-constructed)
    `BrushKind::LinearGradient` value. Probe state unchanged, scene encoding
    unchanged. *Note:* requires either exposing a `#[cfg(test)]` constructor in
    `quartzite-paint-api` for `BrushKind::LinearGradient`, or using a
    `Brush::solid` placeholder with a comment that the early-return arm is
    end-to-end verified by AC10's snapshot in Task 6. Decision: use the snapshot
    in Task 6 as the AC10 anchor and skip a unit test for the panic-free path
    to avoid extending the public Brush API.
  - `all_painter_methods_are_invocable` — preserved, updated for the new
    `VelloPainter::new(scene)` signature inside a `let mut scene = Scene::new();`
    scope. Asserts AC7.
- **Fixtures:** an inline `vello::Scene::new()` + a freshly-constructed
  `FontCache` (test helper `FontCache::new_for_test()` that uses
  `parley::FontContext::new()` directly).

### Task 4 — text rendering

- **Location:** `quartzite-renderer/src/font.rs` `#[cfg(test)] mod tests` for cache
  behaviour; `quartzite-renderer/src/vello_painter.rs` `#[cfg(test)]` for shape
  invariants.
- **Entry points:** `FontCache::resolve_blob`, `VelloPainter::draw_text`,
  `VelloPainter::draw_text_in`.
- **Scenarios:**
  - `font_cache_resolves_sans_serif` — `FontCache::resolve_blob(&Font::default())`
    returns `Some(_)`. May be `#[ignore]`-gated when the host has no fontconfig
    cache (same skip pattern as `SKIP_RENDER_SNAPSHOT`).
  - `font_cache_dedups_by_family_weight_italic` — two lookups for the same
    `Font` produce the same blob handle (`Arc::ptr_eq` on the underlying data).
  - `draw_text_empty_string_is_no_op` — empty `text` writes nothing to the scene
    (parley returns no glyph runs); probe stack unchanged.
  - `draw_text_in_with_zero_width_rect_wraps_per_char` — defence against
    `wrap_budget = 0` panicking inside parley. We pass `f32::INFINITY` for the
    degenerate case to skip wrap.
- **Fixtures:** the host's "sans-serif" generic family (resolved via fontconfig
  / CoreText / DirectWrite). Tests requiring a specific font are `#[ignore]`d.

### Task 6 — snapshot tests (AC1–AC6, AC8–AC12)

- **Location:** `quartzite-widgets/tests/snapshots.rs`.
- **Entry points:** new `#[test]` fns, one per AC group:
  - `fill_rect_paints_red` (AC1), `draw_rect_outline_differs_from_fill` (AC2),
    `draw_path_emits_curve` (AC3), `draw_image_quadrants` (AC4),
    `translate_save_restore` (AC5), `clip_rect_save_restore` (AC6),
    `draw_text_basic` (AC8), `draw_text_in_center` (AC9),
    `gradient_brush_no_panic` (AC10), `hidpi_2x_extent` (AC11),
    `dpr_1_0_default_extent` (AC12).
- **Fixtures:** a tiny inline `WidgetExt` wrapper struct (or a closure-passed
  `paint` via the harness's existing `render_widget(|p| ...)` API) per test —
  no widget-source changes. Goldens land under
  `quartzite-widgets/tests/snapshots/shared/<name>.png` (for backend-agnostic
  shape tests like `fill_rect_paints_red`) or
  `quartzite-widgets/tests/snapshots/<backend>/<name>.png` (for `draw_text*`,
  which produce per-OS font matcher divergence).

## Open questions

- *(None remaining.)* Spec § Open questions enumerated three items deferred to
  the design agent (probe mechanism, opt-out API shape, snapshot bootstrapping
  policy); each is resolved above:
  - Probe mechanism → `#[cfg(test)]`-only `debug_stack_state` accessor on
    `VelloPainter` (§ Approach).
  - Opt-out shape → `VelloPainter::with_scale(scale)` chained on the new
    constructor (§ Approach).
  - Snapshot bootstrapping → text snapshots start under
    `tests/snapshots/<backend>/draw_text*.png` (per-backend), with `continue-on-
    error: true` for non-bootstrapped backends, mirroring the existing
    `gpu-snapshot-tests-ci` policy. Non-text new snapshots (`fill_rect`,
    `draw_rect`, `clip_rect_save_restore`, etc.) land under
    `tests/snapshots/shared/` because they're byte-identical across backends
    once the renderer is real (verified by running on Linux + at least one
    other backend during regen).
