//! [`DefaultStyle`] — built-in flat default style for quartzite widgets.

use quartzite_paint_api::{Brush, Color, Painter, Pen};
use quartzite_style_types::{ColorRole, Palette};
use quartzite_widgets::{Alignment, AsWidget, Button, Label, ScrollArea, TextEdit, WidgetExt};

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
            self.draw_scroll_area(w, painter, palette);
        }
        // Unknown widget type — deliberate no-op; does not panic.
    }
}

impl DefaultStyle {
    fn draw_button(&self, w: &Button, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();
        let enabled = w.is_enabled();

        let (fill_role, text_role) = if w.checked {
            (ColorRole::Highlight, ColorRole::HighlightedText)
        } else {
            (ColorRole::Button, ColorRole::ButtonText)
        };

        let fill_color = maybe_disabled(palette.color(fill_role), enabled);
        let text_color = maybe_disabled(palette.color(text_role), enabled);

        painter.fill_rect(geom, &Brush::solid(fill_color));
        painter.draw_rect(
            geom,
            &Pen::new(text_color, 1.0),
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
mod tests {
    use quartzite_geometry::{Point, Rect};
    use quartzite_paint_api::{Brush, BrushKind, Color, Font, Image, Painter, Path, Pen};
    use quartzite_style_types::{ColorRole, Palette};
    use quartzite_widgets::{
        Alignment, AsWidget, Button, Label, ScrollArea, TextEdit, WidgetBase, WidgetExt,
    };
    use serial_test::serial;

    use crate::registry::clear_for_test;
    use crate::{DefaultStyle, Style, StyleRegistry};

    // ── Recording painter fixture ────────────────────────────────────────────

    #[derive(Clone, Debug, PartialEq)]
    enum PaintEvent {
        DrawRect {
            rect: Rect,
            pen: Pen,
            brush: Brush,
        },
        FillRect {
            rect: Rect,
            brush: Brush,
        },
        DrawLine {
            from: Point,
            to: Point,
            pen: Pen,
        },
        ClipRect(Rect),
        Translate(Point),
        Save,
        Restore,
        DrawText {
            pos: Point,
            text: String,
            font: Font,
            brush: Brush,
        },
        DrawTextIn {
            rect: Rect,
            text: String,
            font: Font,
            brush: Brush,
            alignment: Alignment,
        },
    }

    #[derive(Default)]
    struct RecordingPainter {
        events: Vec<PaintEvent>,
    }

    impl Painter for RecordingPainter {
        fn draw_rect(&mut self, rect: Rect, pen: &Pen, brush: &Brush) {
            self.events.push(PaintEvent::DrawRect {
                rect,
                pen: *pen,
                brush: *brush,
            });
        }
        fn fill_rect(&mut self, rect: Rect, brush: &Brush) {
            self.events.push(PaintEvent::FillRect {
                rect,
                brush: *brush,
            });
        }
        fn draw_line(&mut self, from: Point, to: Point, pen: &Pen) {
            self.events.push(PaintEvent::DrawLine {
                from,
                to,
                pen: *pen,
            });
        }
        fn clip_rect(&mut self, rect: Rect) {
            self.events.push(PaintEvent::ClipRect(rect));
        }
        fn translate(&mut self, delta: Point) {
            self.events.push(PaintEvent::Translate(delta));
        }
        fn save(&mut self) {
            self.events.push(PaintEvent::Save);
        }
        fn restore(&mut self) {
            self.events.push(PaintEvent::Restore);
        }
        fn draw_text(&mut self, pos: Point, text: &str, font: &Font, brush: &Brush) {
            self.events.push(PaintEvent::DrawText {
                pos,
                text: text.to_owned(),
                font: font.clone(),
                brush: *brush,
            });
        }
        fn draw_text_in(
            &mut self,
            rect: Rect,
            text: &str,
            font: &Font,
            brush: &Brush,
            alignment: Alignment,
        ) {
            self.events.push(PaintEvent::DrawTextIn {
                rect,
                text: text.to_owned(),
                font: font.clone(),
                brush: *brush,
                alignment,
            });
        }
        fn draw_image(&mut self, _rect: Rect, _image: &Image) {
            unreachable!("DefaultStyle never calls draw_image");
        }
        fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {
            unreachable!("DefaultStyle never calls draw_path");
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn assert_send_sync<T: Send + Sync>() {}

    fn first_fill(events: &[PaintEvent]) -> &PaintEvent {
        events
            .iter()
            .find(|e| matches!(e, PaintEvent::FillRect { .. }))
            .expect("expected at least one FillRect event")
    }

    fn first_draw_text_in(events: &[PaintEvent]) -> &PaintEvent {
        events
            .iter()
            .find(|e| matches!(e, PaintEvent::DrawTextIn { .. }))
            .expect("expected at least one DrawTextIn event")
    }

    fn brush_color(b: &Brush) -> Color {
        match b.kind() {
            BrushKind::Solid(c) => c,
            _ => unreachable!("expected solid brush"),
        }
    }

    // ── AC1: Send + Sync ─────────────────────────────────────────────────────

    #[test]
    fn default_style_is_send_sync() {
        assert_send_sync::<DefaultStyle>();
        assert_send_sync::<Box<dyn Style>>();
        let _b: Box<dyn Style> = Box::new(DefaultStyle);
    }

    // ── AC2: Button → fill + outline + centred text ───────────────────────────

    #[test]
    fn button_records_fill_outline_and_centred_text() {
        let btn = Button::new("OK".into());
        let mut painter = RecordingPainter::default();
        let palette = Palette::default();
        DefaultStyle.draw_widget(&btn, &mut painter, &palette);

        // Enabled idle button: FillRect → DrawRect → DrawTextIn.
        assert_eq!(
            painter.events.len(),
            3,
            "expected 3 events for enabled idle button"
        );
        assert!(
            matches!(&painter.events[0], PaintEvent::FillRect { rect, .. }
                if *rect == btn.widget_base().geometry),
            "first event must be FillRect covering widget geometry"
        );
        assert!(
            matches!(first_draw_text_in(&painter.events),
                PaintEvent::DrawTextIn { text, alignment, .. }
                    if text == "OK" && *alignment == Alignment::Center),
            "button DrawTextIn must carry text 'OK' with Center alignment"
        );
    }

    // ── AC3: Label → fill + text with widget alignment ────────────────────────

    #[test]
    fn label_records_fill_and_text_with_label_alignment() {
        let lbl = Label::new("hi".into());
        let mut painter = RecordingPainter::default();
        let palette = Palette::default();
        DefaultStyle.draw_widget(&lbl, &mut painter, &palette);

        // FillRect → DrawTextIn (2 events).
        assert_eq!(painter.events.len(), 2, "expected 2 events for label");
        assert!(matches!(&painter.events[0], PaintEvent::FillRect { .. }));
        assert!(
            matches!(first_draw_text_in(&painter.events),
                PaintEvent::DrawTextIn { text, alignment, .. }
                    if text == "hi" && *alignment == Alignment::Left),
            "label DrawTextIn must carry text 'hi' with Left alignment"
        );
    }

    // ── AC4: TextEdit ─────────────────────────────────────────────────────────

    #[test]
    fn text_edit_records_fill_outline_and_text() {
        let mut edit = TextEdit::new();
        edit.plain_text = "abc".into();
        let mut painter = RecordingPainter::default();
        let palette = Palette::default();
        DefaultStyle.draw_widget(&edit, &mut painter, &palette);

        // read_only == false: FillRect(base) → DrawRect → DrawTextIn (3 events).
        assert_eq!(
            painter.events.len(),
            3,
            "expected 3 events for TextEdit (read_only=false)"
        );
        assert!(
            matches!(first_fill(&painter.events),
                PaintEvent::FillRect { brush, .. }
                    if brush_color(brush) == palette.color(ColorRole::Base)),
            "TextEdit fill must use ColorRole::Base"
        );
        assert!(
            matches!(first_draw_text_in(&painter.events),
                PaintEvent::DrawTextIn { text, .. }
                    if text == "abc"),
            "TextEdit DrawTextIn must carry text 'abc'"
        );
    }

    #[test]
    fn text_edit_read_only_inserts_overlay_fill() {
        let mut edit = TextEdit::new();
        edit.plain_text = "abc".into();
        edit.read_only = true;
        let mut painter = RecordingPainter::default();
        let palette = Palette::default();
        DefaultStyle.draw_widget(&edit, &mut painter, &palette);

        // read_only == true: FillRect(base) → FillRect(overlay) → DrawRect → DrawTextIn (4 events).
        assert_eq!(
            painter.events.len(),
            4,
            "expected 4 events for TextEdit (read_only=true)"
        );
        let expected_overlay = super::disabled(palette.color(ColorRole::Window));
        assert!(
            matches!(&painter.events[1],
                PaintEvent::FillRect { brush, .. }
                    if brush_color(brush) == expected_overlay),
            "second FillRect must be the read-only overlay"
        );
    }

    // ── AC5: ScrollArea chrome + no text ─────────────────────────────────────

    #[test]
    fn scroll_area_records_fill_and_outline_only() {
        let area = ScrollArea::new();
        let mut painter = RecordingPainter::default();
        let palette = Palette::default();
        DefaultStyle.draw_widget(&area, &mut painter, &palette);

        // FillRect + DrawRect (2 events), no text draw calls.
        assert_eq!(
            painter.events.len(),
            2,
            "expected 2 chrome events for ScrollArea"
        );
        assert!(matches!(&painter.events[0], PaintEvent::FillRect { .. }));
        assert!(matches!(&painter.events[1], PaintEvent::DrawRect { .. }));
        assert!(
            !painter.events.iter().any(|e| {
                matches!(
                    e,
                    PaintEvent::DrawText { .. } | PaintEvent::DrawTextIn { .. }
                )
            }),
            "ScrollArea must not emit any text draw calls"
        );
    }

    // ── AC6: Unknown widget type = no-op ──────────────────────────────────────

    #[test]
    fn unknown_widget_type_produces_no_events() {
        let base = WidgetBase::new();
        let mut painter = RecordingPainter::default();
        let palette = Palette::default();
        DefaultStyle.draw_widget(&base, &mut painter, &palette);
        assert!(
            painter.events.is_empty(),
            "unknown widget must produce no painter calls"
        );
    }

    // ── AC7: Checked vs idle button colours differ ────────────────────────────

    #[test]
    fn checked_button_uses_highlight_colour() {
        // Construct an explicit palette pinning Highlight to Color::SKY_BLUE so the
        // assertion is meaningful regardless of any future change to Palette::default's
        // seeded Highlight value (today Palette::default already uses Color::SKY_BLUE).
        let palette = Palette::default().with_role(ColorRole::Highlight, Color::SKY_BLUE);

        let idle_btn = Button::new("x".into());
        let mut checked_btn = Button::new("x".into());
        checked_btn.checked = true;

        let mut idle_painter = RecordingPainter::default();
        DefaultStyle.draw_widget(&idle_btn, &mut idle_painter, &palette);
        let idle_color = brush_color(
            if let PaintEvent::FillRect { brush, .. } = &idle_painter.events[0] {
                brush
            } else {
                panic!("first event was not FillRect")
            },
        );

        let mut checked_painter = RecordingPainter::default();
        DefaultStyle.draw_widget(&checked_btn, &mut checked_painter, &palette);
        let checked_color = brush_color(
            if let PaintEvent::FillRect { brush, .. } = &checked_painter.events[0] {
                brush
            } else {
                panic!("first event was not FillRect")
            },
        );

        assert_ne!(
            checked_color, idle_color,
            "checked and idle button fills must differ"
        );
        assert_eq!(
            idle_color,
            palette.color(ColorRole::Button),
            "idle fill must use ColorRole::Button"
        );
        assert_eq!(
            checked_color,
            palette.color(ColorRole::Highlight),
            "checked fill must use ColorRole::Highlight"
        );
    }

    // ── AC8: Disabled button halves fill and text alpha ───────────────────────

    #[test]
    fn disabled_button_halves_fill_and_text_alpha() {
        let enabled_btn = Button::new("x".into());
        let mut disabled_btn = Button::new("x".into());
        disabled_btn.set_enabled(false);

        let palette = Palette::default();

        let mut enabled_painter = RecordingPainter::default();
        DefaultStyle.draw_widget(&enabled_btn, &mut enabled_painter, &palette);

        let mut disabled_painter = RecordingPainter::default();
        DefaultStyle.draw_widget(&disabled_btn, &mut disabled_painter, &palette);

        let enabled_fill = brush_color(
            if let PaintEvent::FillRect { brush, .. } = &enabled_painter.events[0] {
                brush
            } else {
                panic!("first event was not FillRect")
            },
        );
        let disabled_fill = brush_color(
            if let PaintEvent::FillRect { brush, .. } = &disabled_painter.events[0] {
                brush
            } else {
                panic!("first event was not FillRect")
            },
        );
        assert_eq!(
            disabled_fill.a(),
            enabled_fill.a() * 0.5,
            "disabled fill alpha must be half of enabled"
        );

        let enabled_text = brush_color(
            if let PaintEvent::DrawTextIn { brush, .. } =
                first_draw_text_in(&enabled_painter.events)
            {
                brush
            } else {
                panic!("expected DrawTextIn")
            },
        );
        let disabled_text = brush_color(
            if let PaintEvent::DrawTextIn { brush, .. } =
                first_draw_text_in(&disabled_painter.events)
            {
                brush
            } else {
                panic!("expected DrawTextIn")
            },
        );
        assert_eq!(
            disabled_text.a(),
            enabled_text.a() * 0.5,
            "disabled text alpha must be half of enabled"
        );
    }

    // ── AC10: StyleRegistry round-trip ────────────────────────────────────────

    #[test]
    #[serial]
    fn registry_round_trip_dispatches_default_style() {
        clear_for_test();
        StyleRegistry::set_style(Box::new(DefaultStyle));

        let style = StyleRegistry::try_style().expect("style was just installed");
        let btn = Button::new("OK".into());
        let mut painter = RecordingPainter::default();
        let palette = Palette::default();
        style.draw_widget(&btn, &mut painter, &palette);

        assert_eq!(
            painter.events.len(),
            3,
            "expected 3 events for button via registry"
        );
        assert!(
            matches!(first_draw_text_in(&painter.events),
                PaintEvent::DrawTextIn { text, alignment, .. }
                    if text == "OK" && *alignment == Alignment::Center),
            "registry-dispatched DefaultStyle must produce the same events as AC2"
        );
    }
}
