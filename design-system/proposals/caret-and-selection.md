# Proposal: caret + selection visual spec for `LineEdit` and `TextEdit`

**Affects:** `quartzite-style/src/default_style.rs` (paint
impls for `LineEdit` and `TextEdit`), `quartzite-widgets/src/widgets/{line_edit,text_edit}.rs`
(caret + selection model fields), `quartzite-style/tests/snapshots/shared/*`
(new golden PNGs).

**Type:** new visual contract on `DefaultStyle`. Public API gains
fields on `LineEdit` / `TextEdit` for caret + selection state; no
existing signatures move.

**Unblocks:** #405 (LineEdit caret + selection rendering),
#317 (TextEdit caret + selection rendering).

## Context

`DefaultStyle` paints `LineEdit` and `TextEdit` as a `Base` fill
with a 1 px `Text` outline and a single string drawn at
`Alignment::Left`. The framework has no caret primitive, no
selection model, and no per-character text positioning beyond
what `painter.draw_text_in(rect, str, font, brush, alignment)`
resolves internally.

`ColorRole::Highlight` + `ColorRole::HighlightedText` already
exist and are seeded so that `Highlight` is visually distinct
from `HighlightedText` (an invariant of `Palette`). Those two
slots cover the selection axis. `ColorRole::FocusRing` (added by
#402) covers focus, which is orthogonal to caret/selection.

Below: a complete answer to every numbered question in the
umbrella issue, in source order.

## 1. Caret geometry

- **Width:** **1 px.** Matches every other stroke in `DefaultStyle`
  (the 1 px widget outline; the path-icon stroke convention in
  iconography). Two-pixel carets compete visually with the 2 px
  `FocusRing` and read as a second focus indicator.
- **Height:** **full line-box height** for the current `Font`. On
  the default `sans-serif` 12.0 pt, that is whatever metric the
  shaper reports for `ascent + descent + line_gap` of the resolved
  family. Cap-height alone reads short next to the framed widget;
  matching the selection rectangle (also full line-box) keeps the
  two affordances dimensionally consistent.
- **Color slot:** **`ColorRole::Text`**, not `WindowText`. The
  caret is a glyph-adjacent affordance, drawn over `Base`. Inputs
  paint their text from the `Text` slot; the caret follows the
  text it indexes.
- **AA / pixel-snap:** **pixel-snapped, no AA.** The caret x
  origin rounds to the nearest integer pixel column. Matches the
  integer-pixel geometry rule in `quartzite-geometry`.

In Rust the caret paint pass appends one fill rect after the text
draw, only when `is_focused && !read_only && !disabled`:

```rust
if w.is_focused() && !w.read_only && w.is_enabled() && caret_visible_now {
    let (x, y, h) = caret_metrics(w, painter, &font);
    let caret = Rect::new(Point::new(x, y), Size::new(1, h));
    painter.fill_rect(caret, &Brush::solid(palette.color(ColorRole::Text)));
}
```

## 2. Caret position when input is empty

**Left-aligned at the padding inset.** Vertically centered within
the rect for `LineEdit` (single-line widget); top-left of the
content rect for `TextEdit` (multi-line widget). Matches the
`Alignment::Left` the text would render at if non-empty.

Centering an empty caret horizontally would make the empty and
non-empty states inconsistent — the caret would visibly jump on
first keystroke. Left-aligned matches Qt, GTK, and the macOS
text engine.

## 3. Caret blink

- **Blink, 530 ms on / 530 ms off** (1.06 s full cycle). Mirrors
  the X11/GNOME / Qt default for `QApplication::cursorFlashTime()`
  on a stock install.
- **Reduced-motion fallback:** **steady-on, no blink.** The host
  is expected to surface a `prefers_reduced_motion: bool` via
  `winit`'s `AccessKitWindowExt` or equivalent; when true, the
  paint pass treats `caret_visible_now` as `true` unconditionally.
- **Duty cycle is symmetric 50/50.** Asymmetric duty cycles (e.g.
  GTK's 1200/400) read as "broken" against the flat aesthetic.

Implementation hint: a single `Instant`-based phase function on
`Application`; widgets opt in by requesting redraw on the next
phase flip. No animation system needed.

```rust
fn caret_visible_now(now: Instant, start: Instant) -> bool {
    let elapsed = now.saturating_duration_since(start);
    (elapsed.as_millis() / 530) % 2 == 0
}
```

## 4. Selection rectangle

- **Hugs the line-box height**, not glyph metrics. Same height
  the caret uses; same vertical extent the next line of text
  would occupy. Glyph-metric hugging produces ragged top/bottom
  edges on mixed ascender/descender selections and is hard to
  pixel-snap.
- **Fill:** `palette.color(ColorRole::Highlight, ColorGroup::Normal)`.
  Painted **under** the text, **over** the `Base` fill.
- **Foreground (selected glyphs):** `ColorRole::HighlightedText`.
  Painted as a second text pass clipped to the selection range,
  on top of the selection fill.
- **AA / pixel-snap:** integer pixel rects.

## 5. Multi-line wrap (TextEdit only)

- **Per-line rectangles, tiled vertically with no inter-line
  gap.** The rectangles share edges so a multi-line selection
  reads as one continuous shape with a tidy right edge.
- **Per-visual-line geometry:** for each visual line `i` the
  rect is `[line_start_x, line_end_x)`:
  - `line_start_x` = `sel_start_x` if the selection **starts**
    on line `i`; otherwise `content_left`.
  - `line_end_x`   = `sel_end_x`   if the selection **ends**
    on line `i`; otherwise `content_right`.
- **First (leading) line right edge** is `content_right` when
  the selection wraps onto a later line. Only when the
  selection both starts and ends on the same line (no wrap)
  does the rect hug the last glyph on the right. Rationale: a
  multi-line selection reads as one block; a stepped right edge
  between L1 and L2 makes the selection look ragged and
  suggests trailing whitespace that wasn't selected on L1
  (which doesn't exist — the line wraps).
- **Last (trailing) line:** from `content_left` to `sel_end_x`.
  The left edge hugs `content_left` because the selection
  enters this line from above; the right edge hugs the last
  selected glyph because the selection ends here.
- **Middle lines** (selection passes through but neither starts
  nor ends): full content width — `(content_left, line_top, content_width, line_height)`.
- **Single-line selection** (no wrap): rect is the inline span
  from `sel_start_x` to `sel_end_x` — hugs glyphs on both
  sides. The line-end-extension only kicks in when wrap is
  involved.
- **Wrapped vs hard-broken lines** are treated identically. The
  text layout already collapses both into a list of visual
  lines; the selection painter walks that list.

## 6. Read-only behaviour

- **Caret:** **hidden.** Read-only means "no insertion point" —
  there is nowhere for the caret to advance to.
- **Selection:** **allowed.** Read means copy-allowed; copy
  requires a selection. Removing selection from read-only would
  amount to disallowing copy, which contradicts the field's name.
- **Focus ring:** still painted on focus, since the widget is
  still focusable for keyboard copy.

## 7. Disabled behaviour

- **Caret:** **no caret.** Disabled widgets do not accept focus.
- **Selection:** **no selection rendered.** Any selection the
  widget held before being disabled is hidden until re-enabled.
  The selection range itself is _not_ cleared (state is preserved
  through enable cycles), only the paint.
- **Disabled overlay** (× 0.5 alpha) applies after the
  selection-aware composition — i.e. disabled text-with-selection
  paints as if the selection weren't there, then α-halves the
  whole composition.

## 8. Unfocused with selection

**macOS-style greyed.** When the widget holds a selection but is
not the focused widget:

- **Selection fill:** `Highlight.with_alpha(0.5)` — reuses the
  existing `disabled(c)` mathematical helper. On the default
  light palette this lands on `rgba(0,128,255,0.5)`, which
  composites over `#FFFFFF` to `≈#80BFFF`.
- **Selected text foreground:** reverts to `ColorRole::Text`
  (the normal text colour), _not_ `HighlightedText`. The
  α-halved blue is too pale to support white-on-blue contrast.

Hiding the selection entirely on blur would lose user state.
Keeping it saturated would falsely suggest the widget still has
focus. The greyed variant is the long-established compromise.

In Rust this is one more arm of the selection-fill picker:

```rust
let selection_fill = if w.is_focused() {
    palette.color(ColorRole::Highlight, ColorGroup::Normal)
} else {
    // unfocused-with-selection — alpha-half the Highlight
    disabled(palette.color(ColorRole::Highlight, ColorGroup::Normal))
};
let selected_text_color = if w.is_focused() {
    palette.color(ColorRole::HighlightedText)
} else {
    palette.color(ColorRole::Text)
};
```

## Diff sketch

Pseudo-Rust — the real patch lands in #405 / #317.

```diff
--- a/quartzite-widgets/src/widgets/line_edit.rs
+++ b/quartzite-widgets/src/widgets/line_edit.rs
@@ pub struct LineEdit {
     pub text: String,
     pub placeholder: String,
     pub read_only: bool,
+    /// Byte index of the caret within `text` (0..=text.len()).
+    pub caret: usize,
+    /// Optional anchor for the selection. None = no selection.
+    /// When Some, the selection range is `min(anchor,caret)..max(anchor,caret)`.
+    pub selection_anchor: Option<usize>,
 }

--- a/quartzite-style/src/default_style.rs
+++ b/quartzite-style/src/default_style.rs
@@ impl Paint<LineEdit> for DefaultStyle {
     // ... existing fill + overlay + outline ...
+    // selection fill (under text)
+    if let Some((start, end)) = w.selection_range() {
+        let sel_rect = selection_rect(w, painter, &font, start, end);
+        let fill = if w.is_focused() {
+            palette.color(ColorRole::Highlight, ColorGroup::Normal)
+        } else {
+            disabled(palette.color(ColorRole::Highlight, ColorGroup::Normal))
+        };
+        painter.fill_rect(sel_rect, &Brush::solid(fill));
+    }
     // ... existing text draw ...
+    // selected-text overdraw
+    if let Some((start, end)) = w.selection_range() {
+        let color = if w.is_focused() {
+            palette.color(ColorRole::HighlightedText)
+        } else {
+            palette.color(ColorRole::Text)
+        };
+        painter.draw_text_clipped(sel_rect, &w.text[start..end], &font,
+            &Brush::solid(color), Alignment::Left);
+    }
+    // caret
+    if w.is_focused() && !w.read_only && w.is_enabled() && caret_visible_now(now, w.focus_start) {
+        let caret = caret_rect(w, painter, &font);
+        painter.fill_rect(caret, &Brush::solid(palette.color(ColorRole::Text)));
+    }
 }
```

## Snapshot tests

New golden PNGs to commit under
`quartzite-style/tests/snapshots/shared/`:

- `line_edit_focused_empty.png` — empty value, caret visible at left padding inset.
- `line_edit_focused_caret.png` — `"abc"` with caret between `b` and `c`.
- `line_edit_focused_selection.png` — `"abc"` with `bc` selected.
- `line_edit_unfocused_selection.png` — `"abc"` with `bc` selected and blur applied.
- `text_edit_focused_caret.png` — caret in a 64×64 multi-line field.
- `text_edit_selection_wrap.png` — selection spanning two wrapped visual lines.
- `text_edit_read_only_selection.png` — read-only with selection, no caret.

Determinism: tests freeze `caret_visible_now → true` via a
test-only `Style::TestClock`.

## Reference card

- `preview/comp-line-edit-caret.html` — caret width, color, blink phases, empty + populated positions.
- `preview/comp-line-edit-selection.html` — focused, unfocused, read-only, disabled selection states.
- `preview/comp-text-edit-selection.html` — multi-line wrap selection geometry.

All three cards carry light + dark side-by-side following the
`comp-text-edit.html` precedent.

## Open questions / follow-ups

- **IME composition underline.** Out of scope for this proposal.
  When IME lands (no upstream issue yet), the composition
  underline will need its own colour role or `Highlight × Normal`
  derivation.
- **Block-cursor mode** (overwrite vs insert). The Rust source has
  no `overwrite_mode` field. If `Insert`/`Overwrite` is added,
  block-mode caret can re-use the existing selection-fill code
  path with a 1-char selection.
- **Cursor blink synchronisation across multiple focusable
  widgets.** Recommended: phase is global on `Application`, not
  per-widget — all visible carets blink in lockstep.
- **Triple-click line selection / double-click word selection.**
  Pure input-handling concerns; not visual. Logged for the
  per-widget implementation PRs.
