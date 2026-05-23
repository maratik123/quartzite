//! [`Paint<TextEdit>`](crate::Paint) impl for [`DefaultStyle`](super::DefaultStyle).
//!
//! Helper free functions `paint_caret` and `paint_selection` are added in a later
//! subtask once `TextEdit` gains its `caret` and `selection_anchor` fields.

use quartzite_paint_api::{Brush, Color, Painter, Pen};
use quartzite_style_types::{ColorGroup, ColorRole, Palette};
use quartzite_widgets::{Alignment, AsWidget, TextEdit, WidgetExt};

use crate::Paint;

use super::{
    DefaultStyle, FOCUS_RING_WIDTH, READ_ONLY_TEXT_ALPHA, maybe_disabled, read_only_overlay,
    state_group,
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
