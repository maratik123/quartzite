# TextEdit caret + selection rendering

**Source:** issue #317
**Date:** 2026-05-23
**Tracked in:** #317

> Surfaced from `ai-docs/deferred/widget-backlog.md`. Source spec: [`2026-05-13-default-style-content.spec.md`](done/2026-05-13-default-style-content.spec.md). Blocker #539 is closed (design landed in PR #552); visual contract is fully specified in [`design-system/proposals/caret-and-selection.md`](../../design-system/proposals/caret-and-selection.md). This spec implements that contract on the Rust side for `TextEdit` **only**; the same selection model + caret-paint design extracted here will be re-used by #405 (`LineEdit`) as a near-identical follow-up — a tracking comment on #405 is posted from this PR.

## Scope

Wire a selection model + caret rendering onto `TextEdit` and paint them via `DefaultStyle::paint(&TextEdit, …)`. Concretely:

- **Selection-model state on `TextEdit`** — two new `#[prop]` fields on the struct, mirroring the design-proposal diff sketch (`design-system/proposals/caret-and-selection.md` § *Diff sketch*):
  - `pub caret: usize` — byte index into `plain_text` (0..=`plain_text.len()`), defaults to `0`.
  - `pub selection_anchor: Option<usize>` — byte index, `None` ⇒ no selection. When `Some`, the selection range is `min(anchor, caret)..max(anchor, caret)`.
  - One inherent helper `pub fn selection_range(&self) -> Option<(usize, usize)>` that returns the normalised `(start, end)` byte indices when a selection exists.
  - Property-setter `set_caret(usize)` / `set_selection_anchor(Option<usize>)` slots that clamp + emit one new `Signal` `selection_changed: Signal<()>`; the slots are `#[slot]` and follow the `set_plain_text` precedent in the same file (no-op when value unchanged; no-op write when `read_only`).
- **Caret painting** in `impl Paint<TextEdit> for DefaultStyle`, per design-proposal §§ 1–3, 6, 7:
  - Caret rect = `Rect::new(Point::new(caret_x, line_top), Size::new(1, line_height))`, filled with the state-resolved `Text` brush (read-only dim and disabled half-alpha still apply per the existing precedent in `default_style.rs`).
  - Caret painted only when `is_focused() && !read_only && is_enabled() && caret_visible_now`. `caret_visible_now` follows the 530 ms on / 530 ms off cadence from design-proposal § 3, sourced from a new `StyleClock` seam exposed as **a method on the `Style` trait** (Round-3 Q1): `fn caret_visible_now(&self) -> bool` (and a sibling `fn prefers_reduced_motion(&self) -> bool` with default-impl `false`). `DefaultStyle` implements `caret_visible_now` by reading the `StyleClock` instance it owns; snapshot tests inject a `DefaultStyle` constructed with a clock stub that returns `true` unconditionally.
  - Caret-X positioning reads text-layout from new `Painter`-trait text-measurement methods (Round-3 Q2/Q3): `Painter::text_carets(&mut self, text, font) -> &mut dyn TextCaretCursor` returns a borrowed cursor object exposing per-cluster `(byte_offset, x, advance)` queries; the caret painter calls `cursor.advance_to(caret_byte_index)` then reads `cursor.caret_x()`. Object-safe because `dyn Painter` only returns a `&mut dyn TextCaretCursor` — no generic / `impl Trait` in the signature.
- **Selection painting** in the same impl, per design-proposal §§ 4, 5, 7, 8:
  - Per-visual-line rectangles tiled with no inter-line gap; leading-line right-edge extends to `content_right` when wrap, trailing line from `content_left` to `sel_end_x`, middle lines full content width, single-line hugs glyphs both sides. Per-visual-line geometry is read from a second new `Painter` method: `Painter::text_visual_lines(&mut self, text, font, wrap_width) -> &mut dyn TextVisualLineCursor` returns a borrowed cursor exposing per-visual-line `(byte_range, top, height, left_x_for_byte, right_x_for_byte)` queries; the selection painter walks the cursor, asks for the x-extents of `sel_start` / `sel_end` on the lines that own them, and synthesises the per-line rect array.
  - Selection fill = `palette.color(Highlight, ColorGroup::Normal)` when focused; `disabled(Highlight)` (α-halved) when unfocused-with-selection.
  - Selected-glyph foreground = second text pass clipped via `Painter::save` + `clip_rect` + `draw_text_in` + `restore`, brush = `HighlightedText` (focused) or `Text` (unfocused-with-selection).
  - Disabled: neither caret nor selection painted; selection range preserved in state.
- **Snapshot tests** under `quartzite-style/tests/snapshots/shared/` (and dark variants):
  - `text_edit_focused_caret.png` — caret in a multi-line field, mid-line.
  - `text_edit_selection_wrap.png` — selection spanning two wrapped visual lines.
  - `text_edit_read_only_selection.png` — read-only with selection, no caret.
  - `text_edit_unfocused_selection.png` — selection visible, blur applied (α-halved).
  - Tests freeze the blink phase to `true` by constructing the `DefaultStyle`-under-test with a `StyleClock` whose phase-fn is pinned to `true` (a `StyleClock::pinned(true)` constructor or equivalent test-only helper).
- **Determinism**: snapshot tests freeze `caret_visible_now → true` via the `StyleClock` test constructor — no wall-clock reads happen inside the test path.

## Out of scope

- **Input handling** — keystrokes, mouse-click positioning, arrow-key cursor movement, drag-to-select, double-click word selection, triple-click line selection. No `winit`-side wiring lands in this spec. The caret + selection update purely through the new public `set_caret` / `set_selection_anchor` slots; integration with a future input-handling pass is a follow-up.
- **`LineEdit` caret + selection rendering** — issue #405. This PR will post a tracking comment on #405 noting that the selection model + `StyleClock` + Painter text-measurement seams introduced here are re-used by #405 as a near-identical follow-up. No `LineEdit`-side code or snapshot tests land here.
- **IME composition** — composition underline, candidate-window placement. Logged in design-proposal *Open questions / follow-ups*.
- **Block-cursor (overwrite mode)** — `TextEdit` has no `overwrite_mode` field; the design proposal explicitly defers this.
- **Triple-click / double-click word-or-line selection** — input-handling concern; caret + selection visual contract only.
- **Text scrolling inside `TextEdit`** — when content exceeds the geometry, scroll-offset rendering is *not* added here. The selection painter clips to `geometry()` (already implied by `draw_text_in`); a future scroll-aware spec covers viewport translation.
- **Broader text-layout API** beyond the minimum needed for caret-X + per-visual-line wrap-break + selection-rect right-edge (the surface pinned by Round-3 Q2/Q3 — the two cursor traits). General-purpose text shaping / glyph-cluster APIs / RTL-aware layout / vertical text are deliberately out of scope; a broader text-layout-API design ships separately when those use cases land.
- **Cross-widget caret-blink synchronisation guarantees beyond "all visible carets share the same phase source"** — concurrency / ordering corner cases (e.g. phase jitter when wall-clock leaps) are a follow-up.

## Deferred

- `LineEdit` caret + selection rendering (#405) | shares this spec's selection-model + `StyleClock` + Painter text-measurement seams; ships separately as a near-identical follow-up | #405 stays open; tracking comment posted from this PR
- IME composition underline | needs a `ColorRole` decision and shaping integration | new issue when IME lands
- Block-cursor / overwrite mode | needs a `mode` field on `TextEdit`; small, but not in this spec's contract | new issue when input handling lands
- Scroll-offset rendering on `TextEdit` (content overflow) | needs a viewport-offset model | new issue when text editing's scroll-policy lands
- Word / line selection on multi-click | input-handling concern | folded into the input-handling pass
- Per-widget caret-blink phase deviation (e.g. different blink rates per widget) | not in the design contract | follow-up if a real use case appears

## Key decisions

| Question | Decision |
|---|---|
| Visual contract source | [`design-system/proposals/caret-and-selection.md`](../../design-system/proposals/caret-and-selection.md). Every visual decision (caret width, blink cadence, selection-rect tiling, unfocused-with-selection greying, read-only / disabled rules) follows the proposal verbatim. This spec adds **no** new visual choices; it implements the proposal. |
| Widget scope (Round-1 Q1) | **TextEdit only.** `LineEdit` caret + selection (#405) ships as a near-identical follow-up that re-uses the selection model + `StyleClock` + Painter text-measurement seams introduced here. This PR posts a tracking comment on #405 documenting the extracted seams; #405 itself stays open. |
| Caret-blink phase source (Round-1 Q2) | **`StyleClock`** — new abstraction introduced by this spec, owns the wall-clock-to-blink-phase function and the `prefers_reduced_motion` bool. The paint path reads `caret_visible_now` from the `Style` trait (read-side seam — see next row); `StyleClock` is the implementation detail `DefaultStyle` uses to satisfy that method. Snapshot tests inject a `DefaultStyle` constructed with a clock stub that returns `true` unconditionally. |
| Caret-blink phase read seam (Round-3 Q1) | **Method on the `Style` trait.** Two new methods are added to `quartzite_style::Style`: `fn caret_visible_now(&self) -> bool` (no default-impl — every `Style` implementer answers explicitly) and `fn prefers_reduced_motion(&self) -> bool` (default-impl `false`). `Paint<TextEdit>::paint` reads these via the borrowed `&self`'s outer `Style` instance — `DefaultStyle::draw_widget` passes `self` down to the per-widget `Paint` impl already (see `style.rs:117–125` example), so the `Paint<W>` impls call `self.caret_visible_now()` directly. No change to `Paint::paint`'s parameter list. |
| Caret-blink phase invalidation (Round-2 Q1) | Existing `quartzite-runtime::timer::Timer` + `TimerDriver` machinery is reused for the redraw-tick side. `DefaultStyle` owns a `StyleClock` (containing the start `Instant`) plus a `Timer` configured at the 530 ms half-period; the `Timer`'s callback invalidates the relevant widgets. A `MockTimerDriver` freezes the cadence under snapshot tests; the cadence freeze is independent of the read-side stub (tests typically only need the read-side stub returning `true`). |
| Text-layout API source (Round-1 Q3) | **`Painter` trait** — text-measurement methods are added to `quartzite_paint_api::Painter` so paint code can query caret-X at byte offset, per-visual-line break positions for wrap, and the right edge of the selection inside each visual line. Every existing `Painter` implementer in the workspace (renderer, `RecordingPainter`-style test stubs in `quartzite-paint-api`, `quartzite-paint-util`, `quartzite-style`, `quartzite-style-dispatch`) acquires the new methods. |
| Text-layout API object-safety (Round-3 Q2) | **Cursor trait, borrowed.** Each measurement method returns `&mut dyn TextCaretCursor` / `&mut dyn TextVisualLineCursor` — i.e. an object-safe borrowed cursor trait object whose lifetime is tied to the `&mut self` of the Painter call. No `impl Trait`, no generics, no `Box`-allocation. The cursor traits themselves expose only object-safe methods (no associated types, no generic parameters). Each `Painter` implementer caches a per-call cursor inside `&mut self` (an owned shaper buffer) and returns a `&mut` borrow into it — the renderer reuses its internal `parley` layout, the recording-painter test stubs use a fixed-width fake shaper. |
| Text-layout API method split (Round-3 Q3) | **Two methods.** `Painter::text_carets(&mut self, text: &str, font: &Font) -> &mut dyn TextCaretCursor` answers per-caret queries (yields `(byte_offset, x, advance_to_next)` per cluster). `Painter::text_visual_lines(&mut self, text: &str, font: &Font, wrap_width: i32) -> &mut dyn TextVisualLineCursor` answers per-visual-line queries (yields `(byte_range, top, height)` per line plus an x-extent lookup `fn x_at(&self, byte_offset_in_line: usize) -> i32`). Caret painting uses only `text_carets`; selection painting uses both — `text_visual_lines` for the per-line rects + `x_at` for the partial-line endpoints. |
| Selection-model storage | Two new `#[prop]` fields on `TextEdit`: `pub caret: usize` and `pub selection_anchor: Option<usize>`. Matches design-proposal § *Diff sketch* and the `LineEdit` proposal so the two widgets share a model. |
| Selection-range helper | `pub fn selection_range(&self) -> Option<(usize, usize)>` — inherent method, returns `Some((min(anchor, caret), max(anchor, caret)))` when `selection_anchor.is_some()`. Single source of truth for both selection-paint passes. |
| New signal | `pub selection_changed: Signal<()>` — emitted by `set_caret` / `set_selection_anchor` slots whenever the resolved selection-or-caret state changes. Carries no payload (consistent with framework default of "notify, then re-read"). |
| Read-only / disabled state | Per design-proposal §§ 6–7: read-only ⇒ caret hidden, selection allowed (copy semantics); disabled ⇒ neither rendered, range preserved through enable cycles. |
| Unfocused-with-selection | Per design-proposal § 8: `disabled(Highlight)` selection fill + `Text` (not `HighlightedText`) glyph foreground. Re-uses the existing `disabled(c)` helper in `default_style.rs`. |
| Out-of-range caret / anchor | Both clamped to `0..=plain_text.len()` on write via the setter slots. Out-of-range values from direct field mutation (which Rust permits since fields are `pub`) are clamped at paint time as a defence in depth — paint never panics on bad indices. |
| Byte vs char-index for caret / anchor | **Byte** index into `plain_text` (a `String`). Matches `String::char_indices()` callers and the proposal. Paint code uses `char_indices` to translate to glyph positions; invalid UTF-8 boundary indices are clamped to the nearest valid one. |
| Empty-text caret position | Per design-proposal § 2: left-aligned at the padding inset, top-left of content rect for `TextEdit` (multi-line). Matches the `Alignment::Left` text-draw. |
| Reduced-motion fallback | Per design-proposal § 3: steady-on (no blink) when the host surfaces `prefers_reduced_motion = true`. Plumbing of the host-side signal is out of scope for this spec; the seam used in the paint code reads a single bool from the same source as the blink phase (Q2 pins it). |
| Selected-glyph overdraw mechanism | `Painter::save() → Painter::clip_rect(sel_rect) → Painter::draw_text_in(geometry, plain_text, font, brush(HighlightedText or Text), Left) → Painter::restore()`. Uses existing `Painter` methods; no new `draw_text_clipped` API. |

## Technical constraints

- `quartzite-widgets` adds two `pub` `#[prop]` fields plus one signal on `TextEdit`. No new crate dependency.
- `quartzite-style/src/default_style.rs` `Paint<TextEdit>` body grows; helper free functions live in the same file (`caret_rect`, `selection_rects_for_lines`, `paint_caret`, `paint_selection`). File-size budget per AGENTS.md *Code Style* — file is currently ~390 lines; adding ~150 lines pushes toward the 500-line soft cap. Refactor (move `Paint<TextEdit>` into `default_style/text_edit.rs`) is acceptable but not required by this spec.
- New public API:
  - `quartzite_widgets::TextEdit::caret: usize`
  - `quartzite_widgets::TextEdit::selection_anchor: Option<usize>`
  - `quartzite_widgets::TextEdit::selection_changed: Signal<()>`
  - `quartzite_widgets::TextEdit::selection_range(&self) -> Option<(usize, usize)>`
  - `quartzite_widgets::TextEdit::set_caret(&mut self, usize)` (slot)
  - `quartzite_widgets::TextEdit::set_selection_anchor(&mut self, Option<usize>)` (slot)
  - `quartzite_style::Style::caret_visible_now(&self) -> bool` (no default-impl)
  - `quartzite_style::Style::prefers_reduced_motion(&self) -> bool` (default-impl returns `false`)
  - `quartzite_style::StyleClock` — owns the start `Instant`, exposes `caret_visible_now(&self) -> bool` and `prefers_reduced_motion(&self) -> bool` plus a constructor for tests that pins the phase. `DefaultStyle` holds a `StyleClock` and forwards the two `Style`-trait methods to it.
  - `quartzite_paint_api::Painter::text_carets(&mut self, text: &str, font: &Font) -> &mut dyn TextCaretCursor`
  - `quartzite_paint_api::Painter::text_visual_lines(&mut self, text: &str, font: &Font, wrap_width: i32) -> &mut dyn TextVisualLineCursor`
  - `quartzite_paint_api::TextCaretCursor` (object-safe trait, sealed-style methods: `advance_to(&mut self, byte_offset: usize)`, `caret_x(&self) -> i32`, `line_top(&self) -> i32`, `line_height(&self) -> i32`)
  - `quartzite_paint_api::TextVisualLineCursor` (object-safe trait: `next(&mut self) -> Option<TextVisualLine>` where `TextVisualLine = { byte_range: Range<usize>, top: i32, height: i32 }`, plus `x_at(&self, byte_offset_in_current_line: usize) -> i32`).
- Doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs"`) covers every new public item (every method on the two cursor traits, the two new `Style` methods, the `StyleClock` struct, the two new `Painter` methods, the two new `TextEdit` fields + signal + helper + slots).
- Lint gate (`cargo clippy --workspace --all-targets -- -D warnings`) clean.
- The two new `Style` trait methods MUST have their object-safety asserted (existing `style_trait_object_is_send_sync` test in `quartzite-style/src/style.rs` already covers the trait-object compile; an additional test calls both methods through `&dyn Style` to keep coverage explicit).
- The two new `Painter` trait methods MUST have their object-safety asserted by extending the existing `painter_is_object_safe` / `all_methods_reachable_through_trait_object` tests in `quartzite-paint-api/src/painter.rs`.
- Redraw-tick scheduling reuses the existing `quartzite-runtime::timer::Timer` + `TimerDriver`; a `MockTimerDriver` freezes the cadence under tests. `DefaultStyle` constructs a 530 ms-half-period `Timer` whose callback invalidates the widgets currently rendering carets.
- Every existing `Painter` implementer in the workspace (workspace-internal: `RecordingPainter` in `quartzite-paint-api`, `quartzite-paint-util`, `quartzite-style`, `quartzite-style-dispatch`; future: the live renderer that the runtime injects) acquires the two new measurement methods. Test stubs back the cursors with a fake fixed-width shaper (one cluster per `char`, 8 px advance, line-height = `font.size`); a real `Painter` implementation must back the layout query with whatever shaping engine it already uses internally (the live renderer uses `parley`).
- Caret + selection paint is invariant under widget enabled / hover / pressed / focused state per design-proposal §§ 1, 6, 7, 8. The existing state-aware text-colour ladder in `Paint<TextEdit>` continues to govern the *non-selected* text brush.
- `TextEdit::plain_text` field already exists; this spec does not touch it.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `TextEdit` exposes public fields `caret: usize` and `selection_anchor: Option<usize>`, plus a `selection_changed: Signal<()>` signal and an inherent `selection_range(&self) -> Option<(usize, usize)>` method. Default value of both fields is `0` / `None` respectively on `TextEdit::new()` and `TextEdit::default()`. |
| AC2 | `set_caret(n)` clamps `n` to `0..=plain_text.len()`, emits `selection_changed` only when the resolved `(caret, selection_anchor)` state actually changes, and is a no-op when `read_only`. Same pattern for `set_selection_anchor`. |
| AC3 | `selection_range()` returns `Some((min(a, c), max(a, c)))` when `selection_anchor == Some(a)` and `caret == c` and `a != c`; returns `None` when `selection_anchor.is_none()`; returns `None` when `a == c` (zero-length selection is treated as caret-only). |
| AC4 | `DefaultStyle::paint(&TextEdit, …)` paints a 1 px-wide caret rect at `caret_x` × `line_top..line_top + line_height` filled with the state-resolved `Text` brush *only when* `is_focused() && !read_only && is_enabled() && caret_visible_now`. A `RecordingPainter`-based unit test verifies the caret `fill_rect` is *absent* under `is_focused == false`, under `read_only == true`, under `is_enabled() == false`, and under `caret_visible_now == false`; and *present* otherwise. |
| AC5 | `DefaultStyle::paint(&TextEdit, …)` with a single-line selection (no wrap) paints exactly one selection `fill_rect` whose horizontal span is `(sel_start_x, sel_end_x)` and whose vertical span is one line's `line_height`. A `RecordingPainter` unit test asserts the rect ordering: selection-fill is emitted *before* the text-draw and *after* the read-only overlay. |
| AC6 | Multi-line selection spanning N wrapped visual lines paints exactly N selection `fill_rect` calls. Leading-line right edge extends to `content_right`; trailing-line left edge starts at `content_left`; middle lines span the full content width. (Snapshot test `text_edit_selection_wrap.png` provides the pixel-level evidence; a unit test on a fixed-width fake-shaper fixture asserts the rect count and per-line spans symbolically.) |
| AC7 | When `is_focused == false` *and* `selection_range().is_some()`, the selection `fill_rect` uses `disabled(palette.color(Highlight, Normal))` (α-halved) *and* the selected-glyph overdraw brush is `palette.color(Text, …)` rather than `HighlightedText`. Unit test verifies the captured brush colour matches the alpha-halved Highlight. |
| AC8 | When `is_enabled() == false`, *no* caret `fill_rect` is emitted *and* *no* selection `fill_rect` is emitted. The selection range itself remains in state (a separate assertion reads `selection_range()` after the paint call and verifies it's unchanged). |
| AC9 | Read-only with selection paints the selection but *not* the caret. Snapshot test `text_edit_read_only_selection.png` provides pixel-level evidence; a `RecordingPainter` unit test asserts the rect-count: selection-fill present, caret-fill absent. |
| AC10 | Snapshot tests `text_edit_focused_caret.png`, `text_edit_selection_wrap.png`, `text_edit_unfocused_selection.png`, `text_edit_read_only_selection.png` exist under `quartzite-style/tests/snapshots/shared/` and pass `nv-flip` perceptual diff at `FLIP_TOLERANCE = 0.05`. Dark-variant equivalents (`dark_text_edit_*`) exist for each. Tests inject a `StyleClock` stub that returns `caret_visible_now = true` so the captured PNG is deterministic. |
| AC11 | `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all succeed. |
| AC12 | `quartzite_style::Style` gains `fn caret_visible_now(&self) -> bool` (no default-impl) and `fn prefers_reduced_motion(&self) -> bool` (default-impl returns `false`). `dyn Style` remains object-safe (`let _: Box<dyn Style> = ...` still compiles); both methods are reachable through the trait object (a unit test calls each via `&dyn Style`). |
| AC13 | `quartzite_paint_api::Painter` gains `fn text_carets(&mut self, text: &str, font: &Font) -> &mut dyn TextCaretCursor` and `fn text_visual_lines(&mut self, text: &str, font: &Font, wrap_width: i32) -> &mut dyn TextVisualLineCursor`. `dyn Painter` remains object-safe (extended `painter_is_object_safe` test passes); every workspace `Painter` impl (`RecordingPainter` in `quartzite-paint-api`, `quartzite-paint-util`, `quartzite-style`, `quartzite-style-dispatch`) implements both methods, backed by a fake fixed-width shaper in test stubs. |
| AC14 | `quartzite_style::StyleClock` exposes a constructor that pins the phase (`StyleClock::pinned(phase: bool)` or equivalent) so snapshot tests can deterministically force `caret_visible_now → true`. `DefaultStyle::caret_visible_now` returns the value the clock reports; replacing the clock with a pinned one flips the paint output deterministically (unit test verifies the caret `fill_rect` toggles between two `DefaultStyle` instances differing only in their clock). |

## Open questions

- **Cross-widget caret-blink phase stability under wall-clock leaps** — the design proposal's `caret_visible_now` is an `Instant`-arithmetic phase function; behaviour under suspend / NTP jump is unspecified. Defer until a real concurrency complaint surfaces.
- **Caret painting under non-`Left` alignment** — `DefaultStyle::paint(&TextEdit, …)` currently passes `Alignment::Left` unconditionally. If a future spec adds RTL or alignment-aware text, the caret-X computation needs to reflect the alignment. Logged here so the input-handling pass picks it up.
- **Block-cursor / overwrite mode** — see design-proposal *Open questions* §2; out of scope here.
- **Wrap-width source for the multi-line wrap calculation** — `TextEdit::geometry()` minus any built-in padding is the natural candidate. If a future spec adds a configurable internal padding, the wrap-width source needs to be refreshed. The Painter text-measurement seam (`text_visual_lines`) takes a `wrap_width: i32` argument; the paint code supplies the geometry-derived width.

## Resolution log

**Round 1**

- **Q1 — widget scope (TextEdit vs LineEdit+TextEdit):** TextEdit only. Selection model + caret-paint seams extracted here are re-used by #405 as a near-identical follow-up; tracking comment posted on #405.
- **Q2 — caret-blink phase source:** `StyleClock` — new abstraction introduced by this spec, mockable for snapshot determinism. Concrete shape pinned in later round.
- **Q3 — text-layout source for caret-X / selection-rect bounds:** `Painter` trait — text-measurement methods are added to `quartzite_paint_api::Painter`. Concrete method shape pinned in later round.

**Round 2**

- **Q1 — StyleClock plumbing into the rendering pipeline:** the existing `quartzite-runtime::timer::Timer` + `TimerDriver` machinery should be reused for the redraw-tick side; a `MockTimerDriver` freezes the cadence under snapshot tests. This pins the **invalidation** path. The **phase-read** path (`caret_visible_now: bool` at paint time) is logically separate (Timer emits ticks; the paint code still needs a phase function to call). Concrete read-side seam pinned in Round 3 (Q1).
- **Q2 — Painter text-measurement method shape:** the user prefers iterator-returning methods. Iterator-returning fns on a `dyn Painter` are not object-safe directly, so the spec needs a concrete object-safety strategy (a `PainterGuard`-style returned trait object that exposes the queries, OR a `Box<dyn Iterator + '_>` return), plus the iterator item shape (per-cluster vs. per-visual-line vs. both). Concrete shape pinned in Round 3 (Q2).

**Round 3**

- **Q1 — Read-side seam for `caret_visible_now`:** **method on the `Style` trait.** Two new `Style` methods land: `fn caret_visible_now(&self) -> bool` (no default-impl) and `fn prefers_reduced_motion(&self) -> bool` (default-impl `false`). `Paint<W>::paint` impls call them via `&self` (the outer `DefaultStyle` already passes itself through to `Paint::paint` via `draw_widget`). `DefaultStyle` forwards both methods to the `StyleClock` instance it owns; tests substitute a phase-pinned `StyleClock`.
- **Q2 — Object-safety strategy for the iterator-returning Painter methods:** **cursor trait.** Each new measurement method returns `&mut dyn <Cursor>` — a borrowed object-safe trait-object cursor, lifetime tied to the `&mut self` Painter borrow. No `impl Trait`, no generics, no `Box`-allocation. The cursor traits themselves carry only object-safe methods.
- **Q3 — Iterator yielded-item shape:** **two methods.** `Painter::text_carets(text, font) -> &mut dyn TextCaretCursor` handles per-caret queries (caret-X + advance per cluster). `Painter::text_visual_lines(text, font, wrap_width) -> &mut dyn TextVisualLineCursor` handles per-visual-line queries (byte-range + vertical span + per-line `x_at`). Caret painting uses only `text_carets`; selection painting uses both.
