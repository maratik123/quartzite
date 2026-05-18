//! [`DefaultStyle`] — built-in flat default style for quartzite widgets.

use quartzite_paint_api::{Brush, Color, Painter, Pen};
use quartzite_style_types::{ColorRole, Palette};
use quartzite_widgets::{
    Alignment, AsWidget, Button, Container, Label, LineEdit, ScrollArea, TextEdit, WidgetExt,
    WidgetView,
};

use crate::{Paint, Style};

/// Alpha applied to [`ColorRole::WindowText`] to form the read-only surface overlay.
///
/// Low enough to remain translucent, high enough to be visually distinct on any palette.
const READ_ONLY_OVERLAY_ALPHA: f32 = 0.10;

/// Alpha applied to [`ColorRole::Text`] when a widget is in read-only mode.
///
/// Preserves legibility while visually conveying the non-editable state.
const READ_ONLY_TEXT_ALPHA: f32 = 0.65;

/// Built-in concrete [`Style`] implementation using a flat visual design.
///
/// `DefaultStyle` is a zero-sized, `Default`-implementing struct that ships
/// inside `quartzite-style`. Its [`draw_widget`](Style::draw_widget) body
/// routes on the runtime widget type via [`WidgetView`] pattern matching and
/// dispatches to the appropriate [`Paint<W>`](crate::Paint) impl:
///
/// - [`Button`] — flat fill, 1 px outline, centered label; checked/disabled variants.
/// - [`Label`] — background fill + left-aligned (or widget-specified) text.
/// - [`TextEdit`] — base fill, 1 px outline, plain-text content; read-only overlay.
/// - [`ScrollArea`] — chrome only (background fill + 1 px outline); no child traversal.
/// - [`Container`] — Window background fill + 1 px `WindowText` outline; no child traversal.
/// - [`LineEdit`] — Base fill, 1 px outline, single-line text; read-only overlay; placeholder.
///
/// Unknown widget types (the [`WidgetView::Other`] arm) fall through silently — no
/// painter methods are called and no panic is issued. This is intentional: new widget
/// types can be added to the widget tree without breaking `DefaultStyle`.
///
/// `DefaultStyle` is **not** auto-installed. Callers must register it
/// explicitly via [`StyleRegistry::set_style`](crate::StyleRegistry::set_style).
///
/// # Examples
///
/// ```
/// use quartzite_style::{DefaultStyle, StyleRegistry};
///
/// StyleRegistry::set_style(Box::new(DefaultStyle));
/// assert!(StyleRegistry::try_style().is_some());
/// ```
///
/// ```
/// use quartzite_style::{DefaultStyle, Style};
///
/// // DefaultStyle implements Style — it can be boxed as a trait object.
/// let _: Box<dyn Style> = Box::new(DefaultStyle);
/// ```
#[derive(Default, Clone, Copy, Debug)]
pub struct DefaultStyle;

impl Style for DefaultStyle {
    fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
        match widget.widget_view() {
            WidgetView::Button(w) => self.paint(w, painter, palette),
            WidgetView::Label(w) => self.paint(w, painter, palette),
            WidgetView::TextEdit(w) => self.paint(w, painter, palette),
            WidgetView::ScrollArea(w) => self.paint(w, painter, palette),
            WidgetView::Container(w) => self.paint(w, painter, palette),
            WidgetView::LineEdit(w) => self.paint(w, painter, palette),
            // Unknown widget type — deliberate no-op; does not panic.
            _ => {}
        }
    }
}

impl Paint<Button> for DefaultStyle {
    fn paint(&self, w: &Button, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();
        let enabled = w.is_enabled();
        let hovered = w.is_hovered();
        let pressed = w.is_pressed();
        let focused = w.is_focused();

        // Precedence for fill/text axis: pressed > checked > hovered > idle.
        // `disabled` is an alpha modifier applied after role selection (not a role-selector).
        let fill_color = if pressed || w.checked {
            maybe_disabled(palette.color(ColorRole::Highlight), enabled)
        } else if hovered {
            let blended = palette
                .color(ColorRole::Button)
                .blend(palette.color(ColorRole::Highlight), 0.25);
            maybe_disabled(blended, enabled)
        } else {
            maybe_disabled(palette.color(ColorRole::Button), enabled)
        };

        let text_role = if pressed || w.checked {
            ColorRole::HighlightedText
        } else {
            ColorRole::ButtonText
        };
        let text_color = maybe_disabled(palette.color(text_role), enabled);

        // `focused` is an additive outline modifier — always 2 px Highlight, never alpha-halved.
        let (outline_color, outline_width) = if focused {
            (palette.color(ColorRole::Highlight), 2.0)
        } else {
            (text_color, 1.0)
        };

        painter.fill_rect(geom, &Brush::solid(fill_color));
        painter.draw_rect(
            geom,
            &Pen::new(outline_color, outline_width),
            &Brush::solid(Color::TRANSPARENT),
        );
        painter.draw_text_in(
            geom,
            &w.text,
            &font,
            &Brush::solid(text_color),
            Alignment::Center,
        );
    }
}

impl Paint<Label> for DefaultStyle {
    fn paint(&self, w: &Label, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();

        painter.fill_rect(geom, &brush(palette, ColorRole::Window));
        painter.draw_text_in(
            geom,
            &w.text,
            &font,
            &brush(palette, ColorRole::WindowText),
            w.alignment,
        );
    }
}

impl Paint<TextEdit> for DefaultStyle {
    fn paint(&self, w: &TextEdit, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();

        painter.fill_rect(geom, &brush(palette, ColorRole::Base));
        if w.read_only {
            painter.fill_rect(geom, &Brush::solid(read_only_overlay(palette)));
        }
        painter.draw_rect(
            geom,
            &Pen::new(palette.color(ColorRole::Text), 1.0),
            &Brush::solid(Color::TRANSPARENT),
        );
        let text_color = if w.read_only {
            palette
                .color(ColorRole::Text)
                .with_alpha(READ_ONLY_TEXT_ALPHA)
        } else {
            palette.color(ColorRole::Text)
        };
        painter.draw_text_in(
            geom,
            &w.plain_text,
            &font,
            &Brush::solid(text_color),
            Alignment::Left,
        );
    }
}

impl Paint<ScrollArea> for DefaultStyle {
    fn paint(&self, w: &ScrollArea, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        painter.fill_rect(geom, &brush(palette, ColorRole::Base));
        painter.draw_rect(
            geom,
            &Pen::new(palette.color(ColorRole::WindowText), 1.0),
            &Brush::solid(Color::TRANSPARENT),
        );
    }
}

impl Paint<Container> for DefaultStyle {
    fn paint(&self, w: &Container, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        painter.fill_rect(geom, &brush(palette, ColorRole::Window));
        painter.draw_rect(
            geom,
            &Pen::new(palette.color(ColorRole::WindowText), 1.0),
            &Brush::solid(Color::TRANSPARENT),
        );
    }
}

impl Paint<LineEdit> for DefaultStyle {
    fn paint(&self, w: &LineEdit, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();

        painter.fill_rect(geom, &brush(palette, ColorRole::Base));
        if w.read_only {
            painter.fill_rect(geom, &Brush::solid(read_only_overlay(palette)));
        }
        painter.draw_rect(
            geom,
            &Pen::new(palette.color(ColorRole::Text), 1.0),
            &Brush::solid(Color::TRANSPARENT),
        );
        let text_role_color = palette.color(ColorRole::Text);
        let (text_arg, text_brush) = if w.text.is_empty() && !w.placeholder.is_empty() {
            (
                w.placeholder.as_str(),
                Brush::solid(disabled(text_role_color)),
            )
        } else if w.read_only {
            (
                w.text.as_str(),
                Brush::solid(text_role_color.with_alpha(READ_ONLY_TEXT_ALPHA)),
            )
        } else {
            (w.text.as_str(), Brush::solid(text_role_color))
        };
        painter.draw_text_in(geom, text_arg, &font, &text_brush, Alignment::Left);
    }
}

/// Returns a solid [`Brush`] using the colour at `role` in `palette`.
#[inline]
fn brush(palette: &Palette, role: ColorRole) -> Brush {
    Brush::solid(palette.color(role))
}

/// Halves the alpha of `color` to signal the "disabled" visual state.
///
/// With the default palette (all roles fully opaque), maps `1.0 → 0.5`.
#[inline]
fn disabled(color: Color) -> Color {
    color.with_alpha(color.a() * 0.5)
}

/// Returns the read-only overlay colour for `palette`.
///
/// Tints the editable surface with [`ColorRole::WindowText`] at a low alpha.
/// This guarantees a visible effect on every palette — even when `Window`
/// and `Base` share a colour (as on `Palette::default`) — because
/// `WindowText` always carries contrast against `Window` and `Base`.
#[inline]
fn read_only_overlay(palette: &Palette) -> Color {
    palette
        .color(ColorRole::WindowText)
        .with_alpha(READ_ONLY_OVERLAY_ALPHA)
}

/// Returns [`disabled`]`(color)` when `enabled` is `false`; otherwise `color` unchanged.
#[allow(
    clippy::doc_link_code,
    reason = "adjacency-to-(args) pattern: renders disabled(color) with disabled intra-doc-linked; flattening to [disabled](path) would drop the surrounding code styling on (color)"
)]
fn maybe_disabled(color: Color, enabled: bool) -> Color {
    if enabled { color } else { disabled(color) }
}

#[cfg(test)]
#[path = "default_style_tests.rs"]
mod tests;
