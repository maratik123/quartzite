//! [`Paint<LineEdit>`](crate::Paint) impl for [`DefaultStyle`](super::DefaultStyle),
//! plus the `paint_caret_line_edit` and `state_resolved_text_color` helper free functions.

use quartzite_paint_api::{Brush, Color, Painter, Pen};
use quartzite_style_types::{ColorGroup, ColorRole, Palette};
use quartzite_widgets::{Alignment, AsWidget, LineEdit, WidgetExt};

use crate::Paint;

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
        painter.draw_text_in(geom, text_arg, &font, &text_brush, Alignment::Left);
    }
}
