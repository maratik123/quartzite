//! [`Paint<LineEdit>`](crate::Paint) impl for [`DefaultStyle`](super::DefaultStyle),
//! plus the `paint_selection_line_edit`, `paint_caret_line_edit`, and
//! `state_resolved_text_color` helper free functions.

use quartzite_geometry::{Point, Rect, Size};
use quartzite_paint_api::{Brush, Color, Painter, Pen};
use quartzite_style_types::{ColorGroup, ColorRole, Palette};
use quartzite_widgets::{Alignment, AsWidget, LineEdit, WidgetExt};

use crate::{Paint, Style as _};

use super::{
    DefaultStyle, FOCUS_RING_WIDTH, READ_ONLY_TEXT_ALPHA, disabled, maybe_disabled,
    read_only_overlay, state_group,
};

impl Paint<LineEdit> for DefaultStyle {
    fn paint(&self, w: &LineEdit, painter: &mut dyn Painter, palette: &Palette) {
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
        // 3-arm text-brush selection ladder — preserved from pre-spec impl with the
        // state-resolved + disabled-wrapped `text_color` substituted for the old
        // `Normal`-group lookup. Placeholder wins when text is empty; read-only
        // dim composes orthogonally on top of the state-resolved text colour.
        let (text_arg, text_brush) = if w.text.is_empty() && !w.placeholder.is_empty() {
            (w.placeholder.as_str(), Brush::solid(disabled(text_color)))
        } else if w.read_only {
            (
                w.text.as_str(),
                Brush::solid(text_color.with_alpha(READ_ONLY_TEXT_ALPHA)),
            )
        } else {
            (w.text.as_str(), Brush::solid(text_color))
        };
        let line_height = {
            let cursor = painter.text_carets(text_arg, &font);
            cursor.line_height()
        };
        let text_top = geom.top() + (geom.size().height() - line_height) / 2;
        let text_rect = Rect::new(
            Point::new(geom.left(), text_top),
            Size::new(geom.size().width(), line_height),
        );
        painter.draw_text_in(text_rect, text_arg, &font, &text_brush, Alignment::Left);

        // Selection fill + overdraw — after main text, before caret.
        paint_selection_line_edit(w, painter, palette, focused);

        // Caret — last, so it is always on top of the selection and outline.
        paint_caret_line_edit(w, painter, palette, self);
    }
}

/// Returns the state-resolved text colour for `w`, honouring the read-only dim rule.
///
/// Mirrors the colour computation in [`Paint<LineEdit>::paint`] so that
/// `paint_caret_line_edit` uses the identical colour as the main text draw pass.
///
/// # Parameters
///
/// - `w`: the [`LineEdit`] widget being painted.
/// - `palette`: the active colour palette.
fn state_resolved_text_color(w: &LineEdit, palette: &Palette) -> Color {
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

/// Paints the selection highlight and glyph overdraw for `w` when a selection is active.
///
/// Emits zero painter calls when the widget is disabled or has no active selection.
///
/// The selection background fills a single horizontally-aligned rectangle spanning the
/// selected byte range.  A clipped `draw_text_in` overdraw is emitted in a `save`/`restore`
/// guard to render the selected glyphs in a contrasting colour.
///
/// # Parameters
///
/// - `w`: the [`LineEdit`] widget being painted.
/// - `painter`: the active painter.
/// - `palette`: the active colour palette.
/// - `is_focused`: whether the widget currently has keyboard focus.
fn paint_selection_line_edit(
    w: &LineEdit,
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
    let text_len = w.text.len();
    let sel_start = sel_start.min(text_len);
    let sel_end = sel_end.min(text_len);
    if sel_start >= sel_end {
        return;
    }

    let font = w.widget_base().font.clone();
    let geom = w.geometry();

    // Scope 1: get line_height from a text_carets borrow (released before scope 2).
    let line_height = {
        let cursor = painter.text_carets(&w.text, &font);
        cursor.line_height()
    };

    // Vertically centre the selection band in the field geometry (single-line field).
    let caret_y = geom.top() + (geom.size().height() - line_height) / 2;
    let text_rect = Rect::new(
        Point::new(geom.left(), caret_y),
        Size::new(geom.size().width(), line_height),
    );

    // Scope 2: get pixel-snapped x-positions for the two selection boundaries.
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

    // Selection fill: Highlight for focused; half-alpha Highlight for unfocused-with-selection.
    let highlight_color = palette.color(ColorRole::Highlight, ColorGroup::Normal);
    let fill_color = if is_focused {
        highlight_color
    } else {
        disabled(highlight_color)
    };
    painter.fill_rect(sel_rect, &Brush::solid(fill_color));

    // Selected-glyph overdraw: HighlightedText when focused; Text when unfocused.
    let glyph_color = if is_focused {
        palette.color(ColorRole::HighlightedText, ColorGroup::Normal)
    } else {
        palette.color(ColorRole::Text, ColorGroup::Normal)
    };
    painter.save();
    painter.clip_rect(sel_rect);
    painter.draw_text_in(
        text_rect,
        &w.text,
        &font,
        &Brush::solid(glyph_color),
        Alignment::Left,
    );
    painter.restore();
}

/// Paints the 1 px caret line for `w` if all conditions are met.
///
/// The caret is painted only when the widget is focused, writable, enabled, and
/// `style.caret_visible_now()` returns `true`. Under any other condition this
/// function emits zero painter calls.
///
/// The caret is vertically centred within the widget's geometry — unlike
/// [`quartzite_widgets::TextEdit`] which uses the cursor's `line_top`,
/// `LineEdit` aligns the caret to `geom.top() + (geom.size().height() - line_height) / 2`
/// because single-line text is always centred in the field.
///
/// The caret position is clamped to `0..=text.len()` as defence in depth
/// (the `caret` field is `pub` and may be written directly).
///
/// # Parameters
///
/// - `w`: the [`LineEdit`] widget being painted.
/// - `painter`: the active painter.
/// - `palette`: the active colour palette.
/// - `style`: the owning [`DefaultStyle`]; used to query `caret_visible_now`.
fn paint_caret_line_edit(
    w: &LineEdit,
    painter: &mut dyn Painter,
    palette: &Palette,
    style: &DefaultStyle,
) {
    if !w.is_focused() || w.read_only || !w.is_enabled() || !style.caret_visible_now() {
        return;
    }

    let geom = w.geometry();
    let font = w.widget_base().font.clone();
    let caret_pos = w.caret.min(w.text.len());

    // Scope the cursor borrow so it is released before the fill_rect call below.
    let (caret_x, line_height) = {
        let cursor = painter.text_carets(&w.text, &font);
        cursor.advance_to(caret_pos);
        (cursor.caret_x(), cursor.line_height())
    };

    // Vertically centre the caret in the field geometry (single-line field).
    let caret_y = geom.top() + (geom.size().height() - line_height) / 2;
    let color = state_resolved_text_color(w, palette);
    painter.fill_rect(
        Rect::new(Point::new(caret_x, caret_y), Size::new(1, line_height)),
        &Brush::solid(color),
    );
}
