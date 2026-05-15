//! [`DefaultStyle`] — built-in flat default style for quartzite widgets.

use quartzite_paint_api::{Brush, Color, Painter, Pen};
use quartzite_style_types::{ColorRole, Palette};
use quartzite_widgets::{
    Alignment, AsWidget, Button, Container, Label, ScrollArea, TextEdit, WidgetExt,
};

use crate::Style;

/// Built-in concrete [`Style`] implementation using a flat visual design.
///
/// `DefaultStyle` is a zero-sized, `Default`-implementing struct that ships
/// inside `quartzite-style`. Its [`draw_widget`](Style::draw_widget) body
/// routes on the runtime widget type via downcast and dispatches to a
/// dedicated private method for each supported widget:
///
/// - [`Button`] — flat fill, 1 px outline, centered label; checked/disabled variants.
/// - [`Label`] — background fill + left-aligned (or widget-specified) text.
/// - [`TextEdit`] — base fill, 1 px outline, plain-text content; read-only overlay.
/// - [`ScrollArea`] — chrome only (background fill + 1 px outline); no child traversal.
/// - [`Container`] — Window background fill + 1 px `WindowText` outline; no child traversal.
///
/// Unknown widget types fall through silently — no painter methods are called
/// and no panic is issued. This is intentional: new widget types can be added
/// to the widget tree without breaking `DefaultStyle`.
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
        let any = widget.as_any();
        if let Some(w) = any.downcast_ref::<Button>() {
            return self.draw_button(w, painter, palette);
        }
        if let Some(w) = any.downcast_ref::<Label>() {
            return self.draw_label(w, painter, palette);
        }
        if let Some(w) = any.downcast_ref::<TextEdit>() {
            return self.draw_text_edit(w, painter, palette);
        }
        if let Some(w) = any.downcast_ref::<ScrollArea>() {
            return self.draw_scroll_area(w, painter, palette);
        }
        if let Some(w) = any.downcast_ref::<Container>() {
            self.draw_container(w, painter, palette);
        }
        // Unknown widget type — deliberate no-op; does not panic.
    }
}

impl DefaultStyle {
    fn draw_button(&self, w: &Button, painter: &mut dyn Painter, palette: &Palette) {
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

    fn draw_label(&self, w: &Label, painter: &mut dyn Painter, palette: &Palette) {
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

    fn draw_text_edit(&self, w: &TextEdit, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();

        painter.fill_rect(geom, &brush(palette, ColorRole::Base));
        if w.read_only {
            let overlay = disabled(palette.color(ColorRole::Window));
            painter.fill_rect(geom, &Brush::solid(overlay));
        }
        painter.draw_rect(
            geom,
            &Pen::new(palette.color(ColorRole::Text), 1.0),
            &Brush::solid(Color::TRANSPARENT),
        );
        painter.draw_text_in(
            geom,
            &w.plain_text,
            &font,
            &brush(palette, ColorRole::Text),
            Alignment::Left,
        );
    }

    fn draw_scroll_area(&self, w: &ScrollArea, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        painter.fill_rect(geom, &brush(palette, ColorRole::Base));
        painter.draw_rect(
            geom,
            &Pen::new(palette.color(ColorRole::WindowText), 1.0),
            &Brush::solid(Color::TRANSPARENT),
        );
    }

    fn draw_container(&self, w: &Container, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        painter.fill_rect(geom, &brush(palette, ColorRole::Window));
        painter.draw_rect(
            geom,
            &Pen::new(palette.color(ColorRole::WindowText), 1.0),
            &Brush::solid(Color::TRANSPARENT),
        );
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

/// Returns [`disabled`]`(color)` when `enabled` is `false`; otherwise `color` unchanged.
fn maybe_disabled(color: Color, enabled: bool) -> Color {
    if enabled { color } else { disabled(color) }
}

#[cfg(test)]
#[path = "default_style_tests.rs"]
mod tests;
