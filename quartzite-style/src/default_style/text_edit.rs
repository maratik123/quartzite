//! [`Paint<TextEdit>`](crate::Paint) impl for [`DefaultStyle`](super::DefaultStyle),
//! plus the `paint_caret` and `paint_selection` helper free functions.

use quartzite_geometry::{Point, Rect, Size};
use quartzite_paint_api::{Brush, Color, Painter, Pen};
use quartzite_style_types::{ColorGroup, ColorRole, Palette};
use quartzite_widgets::{Alignment, AsWidget, TextEdit, WidgetExt};

use crate::{Paint, Style as _};

use super::{
    DefaultStyle, FOCUS_RING_WIDTH, READ_ONLY_TEXT_ALPHA, disabled, maybe_disabled,
    read_only_overlay, state_group,
};

impl Paint<TextEdit> for DefaultStyle {
    fn paint(&self, w: &TextEdit, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();
        let enabled = w.is_enabled();
        let hovered = w.is_hovered();
        let pressed = w.is_pressed();
        let focused = w.is_focused();

        let group = state_group(pressed, hovered);
        let (fill_role, text_role) = if pressed {
            (ColorRole::Highlight, ColorRole::HighlightedText)
        } else {
            (ColorRole::Base, ColorRole::Text)
        };
        // Idle/hover keep outline = Text (tracks the text colour); pressed
        // swaps to HighlightedText for legibility under the inverted fill.
        let outline_role_idle = if pressed {
            ColorRole::HighlightedText
        } else {
            ColorRole::Text
        };
        let fill_color = maybe_disabled(palette.color(fill_role, group), enabled);
        let text_color = maybe_disabled(palette.color(text_role, group), enabled);
        let outline_color_idle = maybe_disabled(palette.color(outline_role_idle, group), enabled);

        painter.fill_rect(geom, &Brush::solid(fill_color));
        if w.read_only {
            painter.fill_rect(geom, &Brush::solid(read_only_overlay(palette)));
        }
        // `focused` widens the outline to 2 px FocusRing (full alpha — never alpha-halved).
        let (outline_color, outline_width) = if focused {
            (
                palette.color(ColorRole::FocusRing, ColorGroup::Normal),
                FOCUS_RING_WIDTH,
            )
        } else {
            (outline_color_idle, 1.0)
        };
        painter.draw_rect(
            geom,
            &Pen::new(outline_color, outline_width),
            &Brush::solid(Color::TRANSPARENT),
        );
        // Read-only dims the state-resolved text colour; does NOT collapse to idle.
        let final_text_color = if w.read_only {
            text_color.with_alpha(READ_ONLY_TEXT_ALPHA)
        } else {
            text_color
        };
        painter.draw_text_in(
            geom,
            &w.plain_text,
            &font,
            &Brush::solid(final_text_color),
            Alignment::Left,
        );
    }
}

/// Returns the state-resolved text colour for `w`, honouring the read-only dim rule.
///
/// Mirrors the colour computation in [`Paint<TextEdit>::paint`] so that `paint_caret`
/// and `paint_selection` use the identical colour as the main text draw pass.
///
/// # Parameters
///
/// - `w`: the [`TextEdit`] widget being painted.
/// - `palette`: the active colour palette.
// Wired into Paint<TextEdit>::paint in subtask 10.
#[allow(dead_code)]
pub(super) fn state_resolved_text_color(w: &TextEdit, palette: &Palette) -> Color {
    let enabled = w.is_enabled();
    let hovered = w.is_hovered();
    let pressed = w.is_pressed();

    let group = state_group(pressed, hovered);
    let text_role = if pressed {
        ColorRole::HighlightedText
    } else {
        ColorRole::Text
    };
    let text_color = maybe_disabled(palette.color(text_role, group), enabled);

    if w.read_only {
        text_color.with_alpha(READ_ONLY_TEXT_ALPHA)
    } else {
        text_color
    }
}

/// Paints the 1 px caret line for `w` if all conditions are met.
///
/// The caret is painted only when the widget is focused, writable, enabled, and
/// `style.caret_visible_now()` returns `true`. Under any other condition this
/// function emits zero painter calls.
///
/// The caret position is clamped to `0..=plain_text.len()` as defence in depth
/// (the `caret` field is `pub` and may be written directly).
///
/// # Parameters
///
/// - `w`: the [`TextEdit`] widget being painted.
/// - `painter`: the active painter.
/// - `palette`: the active colour palette.
/// - `style`: the owning [`DefaultStyle`]; used to query `caret_visible_now`.
// Wired into Paint<TextEdit>::paint in subtask 10.
#[allow(dead_code)]
pub(super) fn paint_caret(
    w: &TextEdit,
    painter: &mut dyn Painter,
    palette: &Palette,
    style: &DefaultStyle,
) {
    if !w.is_focused() || w.read_only || !w.is_enabled() || !style.caret_visible_now() {
        return;
    }

    let font = w.widget_base().font.clone();
    let caret_pos = w.caret.min(w.plain_text.len());

    // Scope the cursor borrow so it is released before the fill_rect call below.
    let (caret_x, line_top, line_height) = {
        let cursor = painter.text_carets(&w.plain_text, &font);
        cursor.advance_to(caret_pos);
        (cursor.caret_x(), cursor.line_top(), cursor.line_height())
    };

    let color = state_resolved_text_color(w, palette);
    painter.fill_rect(
        Rect::new(Point::new(caret_x, line_top), Size::new(1, line_height)),
        &Brush::solid(color),
    );
}

/// Paints the selection highlight rects and the selected-glyph overdraw for `w`.
///
/// Emits nothing when the widget is disabled or `w.selection_range()` returns `None`
/// (no selection or zero-length selection).
///
/// Paint order: selection fill rects (per visual line) → `save` →
/// `clip_rect(union_rect)` → `draw_text_in` with the highlight glyph colour →
/// `restore`.  Focused uses [`ColorRole::Highlight`] + [`ColorRole::HighlightedText`];
/// unfocused-with-selection uses the same fill at half alpha +
/// [`ColorRole::Text`].
///
/// The `selection_range` and `caret`/`selection_anchor` fields are clamped to
/// `0..=plain_text.len()` at paint time as defence in depth (fields are `pub`).
///
/// # Parameters
///
/// - `w`: the [`TextEdit`] widget being painted.
/// - `painter`: the active painter.
/// - `palette`: the active colour palette.
/// - `is_focused`: whether the widget currently has keyboard focus.
// Wired into Paint<TextEdit>::paint in subtask 10.
#[allow(dead_code)]
pub(super) fn paint_selection(
    w: &TextEdit,
    painter: &mut dyn Painter,
    palette: &Palette,
    is_focused: bool,
) {
    if !w.is_enabled() {
        return;
    }
    let Some((sel_start, sel_end)) = w.selection_range() else {
        return;
    };
    // Defence-in-depth clamp (fields are pub; caller may have set them beyond text len).
    let text_len = w.plain_text.len();
    let sel_start = sel_start.min(text_len);
    let sel_end = sel_end.min(text_len);
    if sel_start >= sel_end {
        return;
    }

    let font = w.widget_base().font.clone();
    let geom = w.geometry();
    let wrap_width = geom.size().width();
    let content_left = geom.left();
    let content_right = geom.right();

    // Step 1: drain the visual-line cursor into a Vec BEFORE any further Painter call.
    // The cursor's &mut-self lifetime ties it to the &mut Painter borrow; we must drop
    // the cursor before calling fill_rect or text_carets below.
    let mut lines = Vec::new();
    {
        let cursor = painter.text_visual_lines(&w.plain_text, &font, wrap_width);
        while let Some(line) = cursor.next_line() {
            lines.push(line);
        }
    }

    // Step 2: get pixel-snapped x-positions for the two selection boundaries using the
    // caret cursor (released in its own scope before the fill_rect calls below).
    let (start_x, end_x) = {
        let cursor = painter.text_carets(&w.plain_text, &font);
        cursor.advance_to(sel_start);
        let sx = cursor.caret_x();
        cursor.advance_to(sel_end);
        let ex = cursor.caret_x();
        (sx, ex)
    };

    // Step 3: compute one fill rect per visual line that overlaps [sel_start, sel_end).
    let mut sel_rects: Vec<Rect> = Vec::new();
    for line in &lines {
        let line_start = line.byte_start;
        let line_end = line.byte_end;
        // Skip lines with no overlap.
        if line_start >= sel_end || line_end <= sel_start {
            continue;
        }

        // Left edge: if selection starts inside this line use start_x; else content_left.
        let left_x = if line_start < sel_start {
            start_x
        } else {
            content_left
        };

        // Right edge: if selection ends inside this line use end_x; else content_right.
        let right_x = if line_end > sel_end {
            end_x
        } else {
            content_right
        };

        if right_x > left_x {
            sel_rects.push(Rect::new(
                Point::new(left_x, line.top),
                Size::new(right_x - left_x, line.height),
            ));
        }
    }

    if sel_rects.is_empty() {
        return;
    }

    // Union rect for the clip pass (bounding box over all selection rects).
    let sel_union = {
        let mut r = sel_rects[0];
        for &rect in &sel_rects[1..] {
            r = r.united(rect);
        }
        r
    };

    // Step 4: emit fill rects (selection background, under the text).
    let highlight_color = palette.color(ColorRole::Highlight, ColorGroup::Normal);
    let fill_color = if is_focused {
        highlight_color
    } else {
        disabled(highlight_color)
    };
    let fill_brush = Brush::solid(fill_color);
    for &rect in &sel_rects {
        painter.fill_rect(rect, &fill_brush);
    }

    // Step 5: selected-glyph overdraw — clip to the union rect and draw with the
    // appropriate highlight glyph colour (HighlightedText focused; Text unfocused).
    let glyph_color = if is_focused {
        palette.color(ColorRole::HighlightedText, ColorGroup::Normal)
    } else {
        palette.color(ColorRole::Text, ColorGroup::Normal)
    };
    painter.save();
    painter.clip_rect(sel_union);
    painter.draw_text_in(
        geom,
        &w.plain_text,
        &font,
        &Brush::solid(glyph_color),
        Alignment::Left,
    );
    painter.restore();
}
