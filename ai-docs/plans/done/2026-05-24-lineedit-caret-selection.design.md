# Design: LineEdit caret + selection rendering

**Issue:** #405
**Date:** 2026-05-24
**Spec:** [`ai-docs/plans/2026-05-24-lineedit-caret-selection.spec.md`](2026-05-24-lineedit-caret-selection.spec.md)

## Approach

Mechanical follow-up to the TextEdit caret/selection PR (#317 / merged
`feat/2026-05-23-textedit-caret-selection`). All seams already exist:

- `quartzite_style::StyleClock` + `DefaultStyle::with_clock` (read-side
  determinism for snapshot tests).
- `Style::caret_visible_now` / `Style::prefers_reduced_motion` (trait
  methods consumed unchanged from `DefaultStyle`).
- `Painter::text_carets(&str, &Font) -> &mut dyn TextCaretCursor` (pixel-x
  + `line_top` + `line_height` on every in-tree painter, both fake-shaped
  and `parley`-backed).
- `Painter::text_visual_lines(&str, &Font, i32) -> &mut dyn TextVisualLineCursor`
  (not consumed by LineEdit — single-line widget).
- `DefaultStyle::start_blink_timer` (invalidation seam unchanged, no new
  wiring on the LineEdit side).

Per spec *Key Decisions* row **Reuse vs. re-introduce seams**: this PR
adds **zero** new public items outside the `LineEdit`-specific selection
model. The work is symmetric to the TextEdit caret + selection paint
extension, with two LineEdit-specific deltas:

1. **Vertical centring** — caret-y / selection-rect-y is
   `geom.top() + (geom.size().height() - line_height) / 2` instead of the
   cursor-reported `line_top` used in TextEdit. The cursor's `line_top()`
   is **not consulted** for LineEdit; only `line_height()` plus widget
   geometry drive the y-axis. This matches design-proposal § 2
   *Vertically centered within the rect for `LineEdit` (single-line widget)*.
2. **Single rectangle, no wrap path** — `Painter::text_visual_lines` is
   never called from `Paint<LineEdit>`. The selection rect is computed
   directly from two `text_carets.advance_to(…).caret_x()` reads, one for
   `sel_start` and one for `sel_end`.

File-size response mirrors the TextEdit precedent: lift the existing
`impl Paint<LineEdit> for DefaultStyle` body (`default_style/mod.rs:332-392`)
verbatim into a new sibling `default_style/line_edit.rs` **before** any
caret/selection code is added. Post-extraction, `mod.rs` drops from
~447 lines to ~387 lines (~ -60 lines); the new `line_edit.rs` starts
at ~60 lines and grows to ~210 with helpers, well inside the 200–400-line
target. Same file-size response the TextEdit work used.

### Selection-model storage on `LineEdit`

Two `pub` fields (`caret: usize`, `selection_anchor: Option<usize>`), one
`Signal<()>` (`selection_changed`), one inherent helper
(`selection_range`), and two setters (`set_caret`, `set_selection_anchor`)
mirror `TextEdit` verbatim (`quartzite-widgets/src/widgets/text_edit.rs:22-247`).

**Slot annotation parity.** The TextEdit precedent attaches `#[slot]` to
`set_caret(usize)` but **not** to `set_selection_anchor(Option<usize>)` —
the latter is a plain inherent method because `Option<usize>` does not
implement `quartzite_core::value::FromValue` and cannot participate in
the slot-dispatch system. The LineEdit setters follow the same pattern.
The spec wording at line 17 *("`set_caret(usize)` / `set_selection_anchor(Option<usize>)` `#[slot]` slots")* would force a slot annotation on
`set_selection_anchor` that does not compile; the design treats this as
a copy-edit oversight in the spec and follows the working precedent
unchanged (verifiable by `ast-index outline quartzite-widgets/src/widgets/text_edit.rs`
showing `set_selection_anchor` outside the `#[object_impl]` block).

### Paint pass ordering (LineEdit — diverges from TextEdit)

The existing `Paint<LineEdit>::paint` in `default_style/mod.rs:332-392`
already paints the outline **before** the main `draw_text_in` (see
`mod.rs:371-375` for the `draw_rect` call followed by `mod.rs:390` for
the `draw_text_in` call). This **LineEdit-specific ordering is
preserved verbatim** — selection-fill + overdraw + caret are appended
**after** the existing pass, leaving every existing event in place.
This is a deliberate divergence from the TextEdit ordering
(`text → selection → overdraw → outline → caret` in
`default_style/text_edit.rs`); the divergence avoids any snapshot
regression from re-layering the existing `line_edit_*` goldens (every
existing `line_edit_*` golden was produced under the
outline-before-text order). The existing 3-arm text-brush selection
ladder (placeholder ⇒ half-alpha; read-only ⇒ `READ_ONLY_TEXT_ALPHA`;
otherwise full-alpha state-resolved) is also preserved verbatim.

Final event ordering when every visual element is active (focused +
read-only-with-selection — the worst case combining every ladder):

1. `fill_rect(geom, Brush::Solid(fill_color))` — base fill (pressed →
   `Highlight × Normal`).
2. `fill_rect(geom, Brush::Solid(read_only_overlay(palette)))` — only
   when `w.read_only`.
3. `draw_rect(geom, &Pen::new(outline_color, outline_width), &Brush::solid(TRANSPARENT))` —
   outline; 2 px `FocusRing` when focused, 1 px otherwise. **Existing
   order, before text — LineEdit-specific divergence from TextEdit.**
4. `draw_text_in(geom, text_arg, &font, &text_brush, Left)` — main text
   draw (state + read-only-resolved brush; placeholder when text
   empty). Existing call, unchanged.
5. `fill_rect(sel_rect, Brush::Solid(selection_fill_color))` — single
   rect; only when `w.is_enabled() && w.selection_range().is_some()`.
6. `save() → clip_rect(sel_rect) → draw_text_in(geom, text, font, overdraw_brush, Left) → restore()` —
   selected-glyph overdraw; same gate as (5). Overdraw brush =
   `HighlightedText` when focused, `Text` when unfocused-with-selection.
7. `fill_rect(caret_rect, &Brush::solid(state_resolved_text_color))` —
   1 px caret; only when `w.is_focused() && !w.read_only && w.is_enabled() && self.caret_visible_now()`.

This ordering **diverges** from the TextEdit precedent
(`quartzite-style/src/default_style/text_edit.rs:42-90`, which paints
`text → selection → overdraw → outline → caret`) in step (3): LineEdit
keeps the outline before text. The divergence is intentional — it
preserves byte-for-byte parity with every existing `line_edit_*`
snapshot golden (no re-layering, no regeneration). See R6 below.

**Spec Amendment on AC12 event ordering.** The spec's AC12 rect
ordering text (line 123) lists `pressed-fill + read-only-overlay →
main-text-draw → selection-fill → outline → selected-glyph overdraw →
caret`, which puts the outline between selection-fill and overdraw and
does not match either the existing implementation order or this
design's preserved order. AC12's text MUST be amended to match the
ordering documented here (Spec Amendment applied alongside this
design). Brush identity assertions in AC12's unit test are unaffected;
only the order-of-events claim shifts.

### Selection-rect computation

```rust
let geom = w.geometry();
let line_height = {
    let cursor = painter.text_carets(&w.text, &font);
    cursor.line_height()
};
let caret_y = geom.top() + (geom.size().height() - line_height) / 2;

let (sel_start_x, sel_end_x) = {
    let cursor = painter.text_carets(&w.text, &font);
    cursor.advance_to(sel_start);
    let sx = cursor.caret_x();
    cursor.advance_to(sel_end);
    let ex = cursor.caret_x();
    (sx, ex)
};

let sel_rect = Rect::new(
    Point::new(sel_start_x, caret_y),
    Size::new(sel_end_x - sel_start_x, line_height),
);
```

Two separate `text_carets` cursor scopes are intentional: the cursor
borrows `&mut self` from the painter, and the `line_height()` read in the
first scope must be released before re-borrowing the painter for the
caret-x reads. The clamps in `paint_selection_line_edit` defend against
out-of-bounds field writes (the `caret` / `selection_anchor` fields are
`pub`).

### Caret-y centring formula

`caret_y = geom.top() + (geom.size().height() - line_height) / 2`.
Integer division floor-rounds when `(geom.size().height() - line_height)`
is odd — acceptable per AGENTS.md design-proposal § 1 *pixel-snapped,
no AA*. If `line_height > geom.size().height()` (degenerate fixture),
the formula yields a negative offset and the caret partially clips
outside the widget — clamping is **not** added: the painter already
clips at `geom` via the existing renderer contract, and a font that
doesn't fit the widget is a fixture bug, not a runtime concern.

### Selection vs. pressed-state co-occurrence (AC12)

The spec's *Selection vs. pressed-state colour ladder* Key Decision
specifies that when `pressed && selection_range().is_some()`, the
selection fill is painted *on top of* the pressed-inverted base fill,
using the focused-with-selection brushes (`Highlight × Normal` fill +
`HighlightedText` overdraw). Non-selected glyphs continue to render with
the pressed `HighlightedText` brush from the existing ladder. The visual
result is a uniform `Highlight` band where the selection-fill colour
matches the pressed-fill colour.

Implementation: no special-case branch is needed — the existing
`paint_selection_line_edit` helper (mirroring `paint_selection` from
text_edit.rs) computes the focused-with-selection brushes whenever
`w.is_focused() && w.selection_range().is_some()`, and the pressed state
flows through the existing `state_resolved_text_color` ladder for the
non-selected glyphs. AC12's unit test asserts the rect ordering + brush
colours; no snapshot golden is added (transient mouse-down state).

### Placeholder + caret co-occurrence (AC15)

When `w.text.is_empty() && !w.placeholder.is_empty()`, the existing
3-arm text-brush selection ladder draws the placeholder text first
(brush = `disabled(text_color)` = α-halved). The caret is then painted
on top at `geom.left()` (empty-text X rule — `Painter::text_carets("", &font)`
returns a cursor whose `advance_to(0).caret_x()` produces `geom.left()`
per the #317 cursor-trait contract). Selection cannot occur on empty
text (no byte range), so placeholder + selection is a non-case.

### Rejected alternatives

- **Special-case the empty-text caret with a `geom.left()` shortcut**
  in `paint_caret_line_edit` instead of relying on the cursor contract.
  Rejected: the cursor contract is pinned by every in-tree `Painter`
  impl (verified in #317 via the `painter_text_carets_reachable_through_trait_object`
  test and the `parley`-backed `VelloPainter` impl). Special-casing
  would duplicate the contract and risk silent drift if the contract
  evolves.
- **Compute `line_height` from the font metric directly** (e.g.
  `font.size as i32`) instead of borrowing the cursor for one read.
  Rejected: the cursor's `line_height()` is the authoritative metric
  (matches what `draw_text_in` resolves internally); reading the font
  field directly would diverge from the painter's measurement model
  and produce off-by-pixel selection rects on real text.
- **Extract `paint_caret_line_edit` / `paint_selection_line_edit` into
  shared free functions in `default_style/mod.rs`** so they coexist
  with the TextEdit equivalents. Rejected: the helper signatures
  differ (LineEdit uses `&LineEdit`, TextEdit uses `&TextEdit`; the y
  computation differs; the wrap-vs-single-rect branch differs) and a
  shared helper would carry both shapes through generics or a trait,
  paying complexity for no shared code. Mirror the TextEdit precedent:
  the helpers are private to `default_style/line_edit.rs`.
- **Tests in a new `quartzite-style/src/default_style/line_edit_tests.rs`
  sibling file** as the spec wording at line 101 suggests. Rejected: the
  TextEdit precedent the spec references *did not actually create
  `text_edit_tests.rs`* — every TextEdit symbolic-AC test lives in the
  consolidated `default_style_tests.rs`
  (`quartzite-style/src/default_style_tests.rs:2354-2742`). Following
  the working precedent (extend `default_style_tests.rs`) keeps the
  test-file surface flat, makes shared fixtures (`is_caret_fill`,
  `brush_color`, `RecordingPainter`, `FakeCaretCursor`,
  `FakeLineCursor`) reachable without `pub(crate)` adjustments, and
  matches what every search tool returns when grepping for the test
  family. Recorded as an Open Question for amendment confirmation.

## Decomposition

| #  | Task | Files | Depends on |
|----|------|-------|------------|
| 1  | Add `LineEdit::caret: usize` and `LineEdit::selection_anchor: Option<usize>` `pub` fields, plus `LineEdit::selection_changed: Signal<()>` `#[signal]`, and `LineEdit::selection_range(&self) -> Option<(usize, usize)>` inherent helper. Update `LineEdit::new()` + `Default` impl to initialise `caret = 0`, `selection_anchor = None`, `selection_changed = Signal::default()`. Verbatim shape mirror of `TextEdit` (`quartzite-widgets/src/widgets/text_edit.rs:22-106`). | `quartzite-widgets/src/widgets/line_edit.rs` | — |
| 2  | Add `LineEdit::set_caret(&mut self, usize)` as `#[slot]` inside the existing `#[object_impl]` block, and `LineEdit::set_selection_anchor(&mut self, Option<usize>)` as a plain inherent method outside the `#[object_impl]` block (mirroring TextEdit — `Option<usize>` is not `FromValue`). Both clamp `n` to `0..=text.len()`, are no-ops when `read_only`, are no-ops when the resolved (caret, selection_anchor) state is unchanged, and emit `selection_changed` exactly once when the state does change. | `quartzite-widgets/src/widgets/line_edit.rs` | 1 |
| 3  | Extend `#[cfg(test)] mod tests` in `quartzite-widgets/src/widgets/line_edit.rs` with the eight `TextEdit`-precedent tests adapted to `LineEdit::text` / `LineEdit::caret`: `default_caret_is_zero_and_anchor_is_none`, `set_caret_clamps_to_text_len`, `set_caret_no_emit_when_unchanged`, `set_caret_no_emit_when_read_only`, `set_selection_anchor_some_then_none_emits_twice`, `selection_range_returns_normalised_pair`, `selection_range_normalises_reversed`, `selection_range_none_when_zero_length`, `selection_range_none_when_anchor_none`, `set_selection_anchor_clamps_to_text_len`, `set_selection_anchor_no_emit_when_read_only`. Direct mirror of `quartzite-widgets/src/widgets/text_edit.rs:337-470`. | `quartzite-widgets/src/widgets/line_edit.rs` | 1, 2 |
| 4  | Lift the existing `impl Paint<LineEdit> for DefaultStyle` body (`quartzite-style/src/default_style/mod.rs:332-392`) verbatim into a new sibling `quartzite-style/src/default_style/line_edit.rs`. Add `mod line_edit;` to `default_style/mod.rs` alongside the existing `mod text_edit;`. Update imports in `mod.rs` to drop `LineEdit` if no longer needed in the parent module; pull `LineEdit`, `Alignment`, and the relevant `quartzite-paint-api` imports into the new file (same import surface as `text_edit.rs`). The `use super::{...}` block in the new file re-exports the same helpers (`FOCUS_RING_WIDTH`, `READ_ONLY_TEXT_ALPHA`, `disabled`, `maybe_disabled`, `read_only_overlay`, `state_group`) the existing impl already consumes via crate-private items. No behavioural change; `cargo test --workspace` MUST still pass before subtask 5 lands. | `quartzite-style/src/default_style/mod.rs`, `quartzite-style/src/default_style/line_edit.rs` (new) | — |
| 5  | Add `state_resolved_text_color(w: &LineEdit, palette: &Palette) -> Color` helper in `line_edit.rs`, mirroring `text_edit.rs:102-120`. Returns the read-only-dimmed state-resolved text colour. Used by `paint_caret_line_edit`. | `quartzite-style/src/default_style/line_edit.rs` | 4 |
| 6  | Add `paint_caret_line_edit(w, painter, palette, style)` helper in `line_edit.rs`. Gate: `w.is_focused() && !w.read_only && w.is_enabled() && style.caret_visible_now()`. Read `(caret_x, line_height)` from one scoped `painter.text_carets(&w.text, &font)` cursor borrow (clamping `w.caret` to `0..=w.text.len()` as defence in depth). Compute `caret_y = geom.top() + (geom.size().height() - line_height) / 2`. Emit one `fill_rect(Rect::new(Point::new(caret_x, caret_y), Size::new(1, line_height)), &Brush::solid(state_resolved_text_color(w, palette)))`. | `quartzite-style/src/default_style/line_edit.rs` | 1, 5 |
| 7  | Add `paint_selection_line_edit(w, painter, palette, is_focused)` helper in `line_edit.rs`. Gate: `w.is_enabled() && w.selection_range().is_some()`. Defence-in-depth clamp `sel_start`/`sel_end` to `0..=w.text.len()`; early-return when `sel_start >= sel_end`. Read `line_height` from one `text_carets` scope, then `(sel_start_x, sel_end_x)` from a second `text_carets` scope (mirroring `text_edit.rs:222-230`). Compute `caret_y` via the same centring formula. Emit one `fill_rect` for the selection background and one `save → clip_rect → draw_text_in → restore` block for the selected-glyph overdraw. Fill colour: `Highlight × Normal` (focused) or `disabled(Highlight)` (unfocused). Overdraw brush: `HighlightedText` (focused) or `Text` (unfocused). | `quartzite-style/src/default_style/line_edit.rs` | 1, 4 |
| 8  | Wire `paint_selection_line_edit` + `paint_caret_line_edit` into `Paint<LineEdit>::paint`. New events are appended **after** the existing pass (base-fill → read-only-overlay → outline → main-text-draw — outline-before-text is the LineEdit-specific existing order, **preserved unchanged**), in the order: selection-fill + clipped overdraw (gate: `enabled && selection_range().is_some()`), then caret (gate: `is_focused && !read_only && is_enabled && caret_visible_now`). The existing main `draw_text_in` is preserved verbatim — it runs *before* selection-fill so the fill covers the already-drawn normal-coloured glyphs; the clipped overdraw renders highlighted glyphs on top. **Diverges** from `text_edit.rs:42-90` in event order (TextEdit paints text → selection → overdraw → outline → caret; LineEdit keeps outline before text); the divergence preserves byte-for-byte parity with every existing `line_edit_*` snapshot golden. | `quartzite-style/src/default_style/line_edit.rs` | 5, 6, 7 |
| 9  | Extend `quartzite-style/src/default_style_tests.rs` (NOT a new sibling; following the actual TextEdit-PR precedent) with the LineEdit-flavoured AC tests. Reuse the existing `FakeCaretCursor`, `FakeLineCursor`, `RecordingPainter`, `is_caret_fill`, `brush_color`, `text_edit_geom`, and `wrap_geom` fixtures. New tests, each gating on a different state combination from AC4 / AC5 / AC6 / AC7 / AC8 / AC9 / AC12 / AC15: `line_edit_caret_rect_emitted_when_focused_enabled_writable_phase_on`, `line_edit_caret_rect_absent_when_not_focused`, `…_when_read_only`, `…_when_disabled`, `…_when_phase_off`, `line_edit_single_selection_emits_one_fill_rect`, `line_edit_selection_emits_exactly_one_fill_rect_count_assertion` (AC6 — paranoia counter that asserts `text_visual_lines` is not consulted), `line_edit_unfocused_with_selection_uses_alpha_half_highlight`, `line_edit_disabled_emits_no_caret_no_selection_preserves_state`, `line_edit_read_only_with_selection_emits_selection_no_caret`, `line_edit_pressed_with_selection_uses_focused_brushes`, `line_edit_caret_y_is_vertically_centred`, `line_edit_placeholder_plus_caret_paint_order`. | `quartzite-style/src/default_style_tests.rs` | 8 |
| 10 | Add 5 light + 5 dark GPU snapshot tests in `quartzite-style/tests/snapshots.rs`: `line_edit_focused_empty_renders`, `line_edit_focused_caret_renders`, `line_edit_focused_selection_renders`, `line_edit_unfocused_selection_renders`, `line_edit_read_only_selection_renders` (and `dark_*` siblings). Each constructs `DefaultStyle::with_clock(StyleClock::pinned(true))` so `caret_visible_now == true` is deterministic. Goldens committed under `quartzite-style/tests/snapshots/shared/{light,dark}_…png`. Tests follow the existing `line_edit_*` snapshot template (lines 469-590) plus the `StyleClock::pinned(true)` injection used by the TextEdit caret/selection snapshots (lines 600-748). | `quartzite-style/tests/snapshots.rs`, `quartzite-style/tests/snapshots/shared/{line_edit_focused_empty,line_edit_focused_caret,line_edit_focused_selection,line_edit_unfocused_selection,line_edit_read_only_selection,dark_line_edit_focused_empty,dark_line_edit_focused_caret,dark_line_edit_focused_selection,dark_line_edit_unfocused_selection,dark_line_edit_read_only_selection}.png` (10 new goldens) | 8 |
| 11 | Post tracking-issue comment on #317 referencing this PR's number once it's open: `gh issue comment 317 --body "LineEdit caret + selection follow-up landed in PR #<N>. All seams shipped by this PR are consumed unchanged: StyleClock, Style::caret_visible_now / prefers_reduced_motion, Painter::text_carets, DefaultStyle::start_blink_timer. text_visual_lines is intentionally not called from the LineEdit paint path (single-line widget)."`. No comment on #405 (this spec **is** #405). | `gh` invocation only (no file edits) | 10 |

## Handoff plan

`M = 11` subtasks. Group sizes within `1..=3` (non-terminal groups exactly 3; terminal group within `1..=3`):

- **Group A** (subtasks 1–3): `LineEdit` selection-model state + slot setters + widget-side unit tests. The entry into Group A is itself the first `/context-reset` handoff per `.claude/skills/task/SKILL.md` Step 8 (every-group contract).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B** (subtasks 4–6): file extraction of `Paint<LineEdit>` into `default_style/line_edit.rs` + `state_resolved_text_color` helper + `paint_caret_line_edit` helper.
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group C with fresh context.
- **Group C** (subtasks 7–9): `paint_selection_line_edit` helper + wire helpers into the `Paint<LineEdit>` body + symbolic-AC tests in `default_style_tests.rs`.
- **Handoff after Group C:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group D with fresh context.
- **Group D** (subtasks 10–11): GPU snapshot goldens + tracking-issue comment on #317 — terminal group (2 subtasks; within the `1..=3` range).

Four groups of 3, 3, 3, 2 = 11 subtasks. Every non-terminal group is exactly 3; the terminal group is 2 (within `1..=3`).

## Risks

- **R1: `paint_selection_line_edit` y-formula off-by-one on odd `(geom.size().height() - line_height)`.** Integer division floors and could yield a 1-px asymmetric padding (1 px more space above than below the selection). **Mitigation:** the design-proposal § 1 mandates pixel-snap with no AA; the formula matches every other vertical-centring computation in `DefaultStyle` (Button text-centring, Label vertical alignment). The 1-px asymmetry is invisible at the framework's 12.0-pt default font (line_height ≈ 14-16 px) inside a 64×64 canvas. AC10 snapshot goldens lock the visual contract — any drift on the y-axis triggers a `nv-flip` failure at `FLIP_TOLERANCE = 0.05`. Acceptable.
- **R2: `Painter::text_carets("", &font)` produces a cursor whose `advance_to(0).caret_x()` does NOT match `geom.left()` on the live `parley`-backed `VelloPainter`.** The spec (line 22) and the cursor-trait rustdoc in #317 require this; the in-tree fake-shaped `Painter` impls in `default_style_tests.rs` (`FakeCaretCursor::new`) return `geom.left()` at byte 0 by construction. **Mitigation:** the contract was verified in PR #552 for the TextEdit case; the `VelloPainter` impl backing the cursor uses `parley::Layout::new()` on an empty string, which exposes a zero-cluster layout whose `start_position` collapses to the layout's content-rect origin. If a regression surfaces, the AC10 `line_edit_focused_empty.png` golden catches it. The widget-side defensive clamp (`w.caret.min(w.text.len())`) ensures we always call `advance_to(0)` for empty text.
- **R3: Spec-vs-precedent mismatch on test-file placement.** Spec line 101 references a `text_edit_tests.rs` precedent file that does not exist — the TextEdit symbolic-AC tests live in the consolidated `quartzite-style/src/default_style_tests.rs`. **Mitigation:** follow the working precedent (extend `default_style_tests.rs`). This is recorded as an Open Question for product-owner / Spec-Amendment confirmation; if the reviewer prefers a new sibling, the test code is mechanically lift-and-shift (the helper fixtures are already `pub(super)` or live in the same `#[cfg(test)] mod tests` scope).
- **R4: Spec-vs-precedent mismatch on `set_selection_anchor` slot annotation.** Spec line 17 attaches `#[slot]` to both setters. The TextEdit precedent (`text_edit.rs:237`) does NOT attach `#[slot]` to `set_selection_anchor` because `Option<usize>` is not `FromValue`. **Mitigation:** follow the working precedent (plain `pub fn`, no `#[slot]`). Recorded as an Open Question. If `Option<usize>` gains `FromValue` in a future PR, both setters can be re-decorated symmetrically.
- **R5: AC6 paranoia assertion ("`text_visual_lines` not called from the LineEdit paint path") requires instrumentation in `RecordingPainter`.** The current `RecordingPainter` does not record a `TextVisualLines` event because the cursor trait methods on `Painter` return a `&mut dyn TextVisualLineCursor` — the trait method itself is not visible as a `PaintEvent`. **Mitigation:** instrument the test-side `RecordingPainter::text_visual_lines` to push a sentinel `PaintEvent::TextVisualLinesCalled` (or extend an existing counter). The TextEdit symbolic tests already inspect rect counts and `Save`/`Restore` events; a direct counter on the cursor-trait call is cheap. Alternative: instead of a sentinel event, the test inspects the **shape** of the cursor output — for a single-line widget, the path through `text_visual_lines` would emit ≥ 1 extra `FillRect` for the line's bounding rectangle even when no overlap exists. Asserting `selection_fill_rect_count == 1` is sufficient and matches AC6's spec wording verbatim ("never emits more than one selection `fill_rect`").
- **R6: Existing `line_edit_*` tests in `default_style_tests.rs` may rely on event counts (e.g. `line_edit_records_fill_outline_and_empty_text` asserts `painter.events.len() == 3`) that would shift if a stray empty-selection branch fires.** The new fields default to `caret = 0` + `selection_anchor = None` ⇒ `selection_range() == None` ⇒ `paint_selection_line_edit` is a no-op ⇒ no event emitted. AC4's `!w.is_focused()` default also keeps `paint_caret_line_edit` a no-op. **Mitigation:** zero-event impact on existing tests when defaults are unchanged. The existing event-count assertions (`assert_eq!(painter.events.len(), 3, …)` and `… == 4` for read-only) MUST stay passing. Subtask 9's first test pass should re-run the existing `line_edit_*` suite to confirm no regression before adding new tests. **Subtask 4 is a verbatim lift with zero behavioural change — the existing outline-before-text draw ordering is preserved; no existing snapshot regeneration is needed.** Every existing `line_edit_*` golden in `quartzite-style/tests/snapshots/shared/` remains byte-identical because (a) subtask 4 only relocates the impl across files, (b) subtasks 5–8 only **append** events after the existing pass, never reorder it.
- **R7: Snapshot golden drift across `parley` minor versions.** The live `VelloPainter` reads cluster x-positions from the active `parley::Layout`. **Mitigation:** the existing snapshot suite already accepts a `FLIP_TOLERANCE = 0.05` perceptual-diff slack which absorbed prior parley updates. If a future bump exceeds tolerance, regenerate via the existing `QUARTZITE_REGENERATE_SNAPSHOTS=1 cargo test …` pipeline. Same mitigation pattern as TextEdit subtask 13's R7 risk.
- **R8: `line_edit_focused_selection.png` golden uses `text = "abc"` with `caret = 1`, `selection_anchor = Some(3)` ⇒ selection is "bc" (2 chars, ~ 16 px under the live parley shaper at 12 pt sans-serif).** At a 64-px-wide canvas, this is a partial-width selection. **Mitigation:** verify the golden's visual contract against design-proposal § 4 (rect hugs glyphs on both sides; line-box-height vertical extent). Generate via `QUARTZITE_REGENERATE_SNAPSHOTS=1`, then perceptual-eyeball before commit. No automated mitigation beyond `FLIP_TOLERANCE`.
- **R9: AGENTS.md *Propagation Rule* fires for the snapshot-helper sync-group (`quartzite-widgets/tests/support/mod.rs` ↔ `quartzite-style/tests/support/mod.rs`)** if the snapshot harness is extended. **Mitigation:** this design does NOT touch either support module; the new snapshot tests in subtask 10 use the existing `snapshot_assert` / `harness_or_skip` helpers verbatim (same as the TextEdit caret-selection PR). Confirm in the self-review checklist.
- **R10: Palette-fragility of the `uniform Highlight band` visual in pressed-with-selection (AC12 / *Selection vs. pressed-state colour ladder* Key Decision).** The "uniform `Highlight` band where the selection-fill colour matches the pressed-fill colour" claim depends on `Palette::default`'s `Highlight × Pressed == Highlight × Normal` collinearity (both groups happen to resolve to the same RGBA in the default palette). A custom palette that distinguishes `Highlight × Pressed` from `Highlight × Normal` would produce a visible band boundary at the selection edges within the pressed widget. **Mitigation:** AC12's unit test validates **brush identity per-event** (i.e. that the selection-fill brush == `Highlight × Normal` and the base-fill brush == `Highlight × Pressed`), not visual equivalence — the test passes regardless of palette. The visual claim is **palette-conditional** and documented as such in this design (and recorded as an Open Question for product-owner review). If a future palette breaks collinearity, the Key Decision text in the spec MUST be revisited; no test breaks.

## Test Design

### Subtask 1 + 2 + 3 — `LineEdit` state model

- **Location:** `quartzite-widgets/src/widgets/line_edit.rs` `#[cfg(test)] mod tests` (extending the existing block at line 134).
- **Entry points:** `LineEdit::new`, `LineEdit::default`, `set_caret`, `set_selection_anchor`, `selection_range`.
- **Scenarios:**
  - `default_caret_is_zero_and_anchor_is_none` — `LineEdit::new().caret == 0` and `selection_anchor.is_none()`.
  - `set_caret_clamps_to_text_len` — set `caret = 999` on `text = "abc"` → `caret == 3`.
  - `set_caret_no_emit_when_unchanged` — emit `selection_changed` once on the first set, zero on the repeat.
  - `set_caret_no_emit_when_read_only` — `read_only = true` + `set_caret(5)` → field unchanged, signal not emitted.
  - `set_selection_anchor_some_then_none_emits_twice` — `Some(3)` → emits once; `None` → emits once.
  - `selection_range_returns_normalised_pair` — anchor=5, caret=2 → `Some((2,5))`.
  - `selection_range_normalises_reversed` — anchor=2, caret=5 → `Some((2,5))` (commutative).
  - `selection_range_none_when_zero_length` — anchor=Some(3), caret=3 → `None`.
  - `selection_range_none_when_anchor_none` — anchor=None → `None`.
  - `set_selection_anchor_clamps_to_text_len` — `text = "abc"` + `set_selection_anchor(Some(100))` → `selection_anchor == Some(3)`.
  - `set_selection_anchor_no_emit_when_read_only` — `read_only = true` + `set_selection_anchor(Some(3))` → field unchanged, signal not emitted.
- **Fixtures:** `Arc<Mutex<u32>>` emission counter following the existing `set_text_emits_text_changed` precedent in the same file (lines 159-170).

### Subtask 9 — `Paint<LineEdit>` symbolic ACs

- **Location:** `quartzite-style/src/default_style_tests.rs` — extend, not new sibling file (see Risk R3).
- **Entry point:** `<DefaultStyle as Paint<LineEdit>>::paint` via `DefaultStyle::draw_widget(&edit, &mut painter, &palette)`.
- **Shared fixtures (already exist; reuse):** `RecordingPainter`, `FakeCaretCursor` (one cluster per char, 8 px advance, `line_height = font.size`), `is_caret_fill`, `brush_color`, `text_edit_geom()` (rename / clone as `line_edit_geom()` for clarity — same 100×20 rect).
- **Scenarios (each constructs a `LineEdit`, a `RecordingPainter`, and a `DefaultStyle::with_clock(StyleClock::pinned(true)`/`false`):**
  - `line_edit_caret_rect_emitted_when_focused_enabled_writable_phase_on` (AC4 positive) — pinned(true), focused, enabled, writable; assert exactly one 1-px `FillRect`; assert caret comes *after* the outline (`DrawRect`) in event order.
  - `line_edit_caret_rect_absent_when_not_focused` / `_when_read_only` / `_when_disabled` / `_when_phase_off` — four sibling negatives, each asserting `events.iter().filter(is_caret_fill).count() == 0` (AC4 negatives).
  - `line_edit_single_selection_emits_one_fill_rect` (AC5) — `text = "abcde"`, `caret = 2`, `selection_anchor = Some(0)`, focused. Pinned(false) to disable caret. Assert exactly 2 `FillRect`s (base + selection); selection width = 16 px (2 chars × 8 px under the fake shaper); selection emitted *after* the main `DrawTextIn`. Verify `Save` event present.
  - `line_edit_selection_emits_exactly_one_fill_rect_count_assertion` (AC6) — `text = "abcdefghijklmnop"` (16 chars), `caret = 16`, `selection_anchor = Some(0)`, focused. Assert exactly 2 `FillRect`s total: base (width=64) + selection (width=??? — single-line widget, so width = 16 chars × 8 = 128, but the painter clips at `geom.width()`; what matters is **rect count == 1 for the selection**). Distinguishes LineEdit (one selection rect) from TextEdit (two rects for wrap).
  - `line_edit_unfocused_with_selection_uses_alpha_half_highlight` (AC7) — capture brush colour, assert `≈ Highlight × 0.5 alpha`; verify the post-`ClipRect` `DrawTextIn` overdraw brush is `Text` (not `HighlightedText`).
  - `line_edit_disabled_emits_no_caret_no_selection_preserves_state` (AC8) — pinned(true), focused, `set_enabled(false)`. Assert no caret; assert no `Save` event (no selection overdraw); re-read `selection_range()` post-paint and assert state preserved.
  - `line_edit_read_only_with_selection_emits_selection_no_caret` (AC9) — pinned(true), focused, `read_only = true`. Assert `Save` event present (selection overdraw); assert no caret; assert read-only overlay `FillRect` present.
  - `line_edit_pressed_with_selection_uses_focused_brushes` (AC12) — pinned(false), focused, `set_pressed(true)`, selection in place. Capture the selection-fill `FillRect` brush — assert `== palette.color(Highlight, Normal)` (NOT `disabled(Highlight)`); capture the post-`ClipRect` `DrawTextIn` brush — assert `== palette.color(HighlightedText, Normal)`.
  - `line_edit_caret_y_is_vertically_centred` — pinned(true), focused, `text = "abc"`, `geom = (0, 0, 100, 20)`, fake `line_height = font.size = 12`. Assert the caret `FillRect.origin().y() == 0 + (20 - 12) / 2 == 4`.
  - `line_edit_placeholder_plus_caret_paint_order` (AC15) — pinned(true), focused, enabled, writable, `text = ""`, `placeholder = "hint"`. Assert exactly one `DrawTextIn` (placeholder, half-alpha `Text` brush) AND exactly one 1-px `FillRect` (caret); assert the caret comes *after* the placeholder `DrawTextIn` in event order; assert the caret's x-origin equals `geom.left()`.
- **Fixtures:** the in-test `RecordingPainter` already backs the `text_carets` method with `FakeCaretCursor` (one cluster per char, 8 px advance); no extension needed for the LineEdit tests (the LineEdit paint path never calls `text_visual_lines`, so the existing `FakeLineCursor` backing is unused but harmless).

### Subtask 10 — GPU snapshot tests

- **Location:** `quartzite-style/tests/snapshots.rs`.
- **Entry point:** `harness.render_widget(|painter| DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(&w, painter, &palette))`.
- **Scenarios:** five light + five dark, one per AC10 golden. Each test follows the existing `text_edit_focused_caret_renders` / `line_edit_focused_renders` templates (already in `snapshots.rs`); only the widget configuration differs.
- **Fixtures:**
  - `line_edit_focused_empty` — `text = ""`, `set_focused(true)`. Light + dark.
  - `line_edit_focused_caret` — `text = "abc"`, `caret = 2`, `set_focused(true)`. Light + dark.
  - `line_edit_focused_selection` — `text = "abc"`, `caret = 1`, `selection_anchor = Some(3)`, `set_focused(true)`. Light + dark.
  - `line_edit_unfocused_selection` — `text = "abc"`, `caret = 1`, `selection_anchor = Some(3)` (NOT focused — α-halved Highlight). Light + dark.
  - `line_edit_read_only_selection` — `text = "abc"`, `caret = 1`, `selection_anchor = Some(3)`, `read_only = true`, `set_focused(true)`. Light + dark.
- All ten goldens use `DefaultStyle::with_clock(StyleClock::pinned(true))` so `caret_visible_now → true` is deterministic.

## Open questions

- **Test-file placement: extend `default_style_tests.rs` (working precedent) vs. new sibling `default_style/line_edit_tests.rs` (spec wording).** The spec at line 101 references a `text_edit_tests.rs` precedent that does not exist in the merged TextEdit caret/selection PR. The design defaults to extending `default_style_tests.rs` (following the actual working precedent); reviewer may direct a Spec Amendment if the new-sibling layout is preferred.
- **`set_selection_anchor` slot annotation: `#[slot]` (spec wording) vs. plain inherent method (TextEdit precedent).** Spec line 17 says `#[slot]` for both setters; TextEdit ships `set_selection_anchor` as a plain inherent method because `Option<usize>` is not `FromValue`. The design defaults to the precedent (plain inherent method); reviewer may direct a Spec Amendment if a `FromValue` impl for `Option<usize>` lands in a sibling PR.
- **Tracking-issue comment timing on #317** — defer until this PR is opened; the comment text references "PR #<number>" verbatim. If the merge SHA is desired instead, edit the comment post-merge (same convention as TextEdit's tracking comment on #405).
