//! Sibling test module for `default_style.rs`; entire body is `#[cfg(test)]`
//! (test lines are excluded from the per-file size cap per AGENTS.md *Code Style*).
//!
//! Attached via `#[cfg(test)] #[path = "default_style_tests.rs"] mod tests;`
//! in `default_style.rs`. The module name is still `tests`, so `super::` still
//! resolves to `crate::default_style` — all `super::disabled` / `super::brush`
//! calls remain valid.

use quartzite_core::ObjectId;
use quartzite_geometry::{Point, Rect};
use quartzite_paint_api::{Brush, Color, Font, Image, Painter, Path, Pen};
use quartzite_style_types::{ColorRole, Palette};
use quartzite_widgets::{
    Alignment, AsWidget, Button, Container, Label, LineEdit, ScrollArea, TextEdit, WidgetBase,
    WidgetExt,
};

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
            brush: brush.clone(),
        });
    }
    fn fill_rect(&mut self, rect: Rect, brush: &Brush) {
        self.events.push(PaintEvent::FillRect {
            rect,
            brush: brush.clone(),
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
            brush: brush.clone(),
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
            brush: brush.clone(),
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

fn first_draw_rect(events: &[PaintEvent]) -> &PaintEvent {
    events
        .iter()
        .find(|e| matches!(e, PaintEvent::DrawRect { .. }))
        .expect("expected at least one DrawRect event")
}

/// Returns a representative `Color` for assertions — not a perceptual average.
///
/// - `Solid(c)` → `c`
/// - `LinearGradient { start_color, .. }` / `RadialGradient { start_color, .. }` → `start_color`
/// - `Custom(g)` → first stop of `g` converted to sRGB, or `TRANSPARENT` if no stops
fn brush_color(b: &Brush) -> Color {
    use quartzite_paint_api::BrushKind;
    match b.kind() {
        BrushKind::Solid(c) => *c,
        BrushKind::LinearGradient { start_color, .. }
        | BrushKind::RadialGradient { start_color, .. } => *start_color,
        BrushKind::Custom(gradient) => gradient.stops.first().map_or(Color::TRANSPARENT, |stop| {
            let alpha = stop.color.to_alpha_color::<peniko::color::Srgb>();
            let [r, g, b, a] = alpha.components;
            Color::new(r, g, b, a)
        }),
        _ => Color::TRANSPARENT,
    }
}

// ── brush_color helper ───────────────────────────────────────────────────

#[test]
fn brush_color_linear_gradient_returns_start_color() {
    let brush =
        Brush::linear_gradient(Point::new(0, 0), Point::new(10, 0), Color::RED, Color::BLUE);
    assert_eq!(brush_color(&brush), Color::RED);
}

#[test]
fn brush_color_radial_gradient_returns_start_color() {
    let brush = Brush::radial_gradient(Point::new(5, 5), 3.0, Color::WHITE, Color::BLACK);
    assert_eq!(brush_color(&brush), Color::WHITE);
}

#[test]
fn brush_color_custom_first_stop_returns_its_color() {
    let gradient = peniko::Gradient::new_linear((0.0f64, 0.0f64), (10.0f64, 0.0f64))
        .with_stops([peniko::Color::new([1.0f32, 0.0, 0.0, 1.0])]);
    let brush = Brush::custom_gradient(gradient);
    assert_eq!(brush_color(&brush), Color::new(1.0, 0.0, 0.0, 1.0));
}

#[test]
fn brush_color_custom_empty_stops_returns_transparent() {
    let gradient = peniko::Gradient::new_linear((0.0f64, 0.0f64), (10.0f64, 0.0f64));
    let brush = Brush::custom_gradient(gradient);
    assert_eq!(brush_color(&brush), Color::TRANSPARENT);
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
    let expected_overlay = palette
        .color(ColorRole::WindowText)
        .with_alpha(super::READ_ONLY_OVERLAY_ALPHA);
    assert!(
        matches!(&painter.events[1],
            PaintEvent::FillRect { brush, .. }
                if brush_color(brush) == expected_overlay),
        "second FillRect must be the read-only overlay"
    );
}

#[test]
fn text_edit_read_only_dims_text() {
    let mut edit = TextEdit::new();
    edit.plain_text = "abc".into();
    edit.read_only = true;
    let mut painter = RecordingPainter::default();
    let palette = Palette::default();
    DefaultStyle.draw_widget(&edit, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events for read-only TextEdit with text"
    );
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Text).with_alpha(super::READ_ONLY_TEXT_ALPHA)),
        "events[3] DrawTextIn brush must be Text dimmed to READ_ONLY_TEXT_ALPHA"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn text_edit_writable_keeps_full_alpha_text() {
    let mut edit = TextEdit::new();
    edit.plain_text = "abc".into();
    edit.read_only = false;
    let mut painter = RecordingPainter::default();
    let palette = Palette::default();
    DefaultStyle.draw_widget(&edit, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        3,
        "expected 3 events for writable TextEdit"
    );
    assert!(
        matches!(&painter.events[2], PaintEvent::DrawTextIn { brush, .. }
            if brush_color(brush).a() == 1.0),
        "writable TextEdit text brush must have full alpha"
    );
}

#[test]
fn read_only_overlay_derives_from_custom_window_text() {
    let palette =
        Palette::default().with_role(ColorRole::WindowText, Color::new(0.0, 0.5, 1.0, 1.0));
    let mut edit = TextEdit::new();
    edit.read_only = true;
    let mut painter = RecordingPainter::default();
    DefaultStyle.draw_widget(&edit, &mut painter, &palette);

    let expected_overlay = Color::new(0.0, 0.5, 1.0, super::READ_ONLY_OVERLAY_ALPHA);
    assert!(
        matches!(&painter.events[1], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == expected_overlay),
        "overlay must derive from custom WindowText colour"
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
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
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
        if let PaintEvent::DrawTextIn { brush, .. } = first_draw_text_in(&enabled_painter.events) {
            brush
        } else {
            panic!("expected DrawTextIn")
        },
    );
    let disabled_text = brush_color(
        if let PaintEvent::DrawTextIn { brush, .. } = first_draw_text_in(&disabled_painter.events) {
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

// ── New visual states (AC3 spec, AC4 spec, AC5 spec, AC6 spec) ───────────

fn pinned_palette() -> Palette {
    Palette::default()
        .with_role(ColorRole::Button, Color::WHITE)
        .with_role(ColorRole::Highlight, Color::SKY_BLUE)
        .with_role(ColorRole::ButtonText, Color::BLACK)
        .with_role(ColorRole::HighlightedText, Color::WHITE)
}

#[test]
fn hovered_button_uses_blended_fill() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle.draw_widget(&btn, &mut painter, &palette);

    let expected_blend = palette
        .color(ColorRole::Button)
        .blend(palette.color(ColorRole::Highlight), 0.25);
    let idle_fill = palette.color(ColorRole::Button);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("first_fill did not return FillRect")
        },
    );
    assert_eq!(
        fill_color, expected_blend,
        "hovered fill must be 25% blend toward Highlight"
    );
    assert_ne!(
        fill_color, idle_fill,
        "hovered fill must differ from idle baseline"
    );

    let text_color = brush_color(
        if let PaintEvent::DrawTextIn { brush, .. } = first_draw_text_in(&painter.events) {
            brush
        } else {
            panic!("expected DrawTextIn")
        },
    );
    assert_eq!(
        text_color,
        palette.color(ColorRole::ButtonText),
        "hovered button text role must remain ButtonText"
    );
}

#[test]
fn pressed_button_uses_highlight_roles() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_pressed(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle.draw_widget(&btn, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("first_fill did not return FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight),
        "pressed fill must be Highlight"
    );
    assert_ne!(
        fill_color,
        palette.color(ColorRole::Button),
        "pressed fill must differ from idle baseline"
    );

    let text_color = brush_color(
        if let PaintEvent::DrawTextIn { brush, .. } = first_draw_text_in(&painter.events) {
            brush
        } else {
            panic!("expected DrawTextIn")
        },
    );
    assert_eq!(
        text_color,
        palette.color(ColorRole::HighlightedText),
        "pressed text must be HighlightedText"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn focused_button_uses_2px_highlight_outline() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle.draw_widget(&btn, &mut painter, &palette);

    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::Highlight),
        "focused outline color must be Highlight"
    );
    assert_eq!(pen_width, 2.0, "focused outline must be 2 px wide");

    // Idle baseline has width 1.0 — verify it changes.
    let mut idle_painter = RecordingPainter::default();
    let idle_btn = Button::new("x".into());
    DefaultStyle.draw_widget(&idle_btn, &mut idle_painter, &palette);
    let idle_width = if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&idle_painter.events)
    {
        pen.width()
    } else {
        panic!("expected DrawRect")
    };
    assert_ne!(
        pen_width, idle_width,
        "focused outline width must differ from idle baseline"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn precedence_disabled_pressed_focused() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_pressed(true);
    btn.set_focused(true);
    btn.set_enabled(false);
    let mut painter = RecordingPainter::default();
    DefaultStyle.draw_widget(&btn, &mut painter, &palette);

    // Fill: pressed selects Highlight, then disabled halves its alpha.
    let expected_highlight = palette.color(ColorRole::Highlight);
    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color.r(),
        expected_highlight.r(),
        "disabled+pressed fill r must match Highlight r"
    );
    assert_eq!(
        fill_color.g(),
        expected_highlight.g(),
        "disabled+pressed fill g must match Highlight g"
    );
    assert_eq!(
        fill_color.b(),
        expected_highlight.b(),
        "disabled+pressed fill b must match Highlight b"
    );
    assert_eq!(
        fill_color.a(),
        expected_highlight.a() * 0.5,
        "disabled fill alpha must be half of Highlight's alpha"
    );

    // Focused outline survives disabled: 2 px, full-alpha Highlight.
    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color, expected_highlight,
        "focus outline color must be full-alpha Highlight even when disabled"
    );
    assert_eq!(
        pen_width, 2.0,
        "focus outline must still be 2 px wide when disabled"
    );
}

#[test]
fn precedence_checked_hovered_keeps_checked_fill() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.checked = true;
    btn.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle.draw_widget(&btn, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight),
        "checked wins over hover: fill must be Highlight"
    );
}

#[test]
fn precedence_pressed_checked_both_map_to_highlight() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_pressed(true);
    btn.checked = true;
    let mut painter = RecordingPainter::default();
    DefaultStyle.draw_widget(&btn, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight),
        "pressed+checked both map to Highlight role"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn precedence_focused_hovered_blend_plus_outline() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_focused(true);
    btn.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle.draw_widget(&btn, &mut painter, &palette);

    let expected_blend = palette
        .color(ColorRole::Button)
        .blend(palette.color(ColorRole::Highlight), 0.25);
    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color, expected_blend,
        "focused+hovered fill must be 25% blend"
    );

    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::Highlight),
        "focused outline color must be Highlight"
    );
    assert_eq!(pen_width, 2.0, "focused outline must be 2 px wide");
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn idle_button_three_events_unchanged() {
    let btn = Button::new("OK".into());
    let mut painter = RecordingPainter::default();
    let palette = pinned_palette();
    DefaultStyle.draw_widget(&btn, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        3,
        "idle button still produces exactly 3 events"
    );
    assert_eq!(
        brush_color(
            if let PaintEvent::FillRect { brush, .. } = &painter.events[0] {
                brush
            } else {
                panic!()
            }
        ),
        palette.color(ColorRole::Button),
        "idle fill is Button role"
    );
    let outline_width = if let PaintEvent::DrawRect { pen, .. } = &painter.events[1] {
        pen.width()
    } else {
        panic!()
    };
    assert_eq!(outline_width, 1.0, "idle outline is 1 px");
}

// ── Container + LineEdit (spec 2026-05-15) ────────────────────────────────

fn container_palette() -> Palette {
    Palette::default()
        .with_role(ColorRole::Window, Color::new(0.9, 0.9, 0.9, 1.0))
        .with_role(ColorRole::WindowText, Color::new(0.1, 0.1, 0.1, 1.0))
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn container_records_fill_and_outline() {
    let c = Container::new();
    let mut painter = RecordingPainter::default();
    let palette = container_palette();
    DefaultStyle.draw_widget(&c, &mut painter, &palette);

    assert_eq!(painter.events.len(), 2, "expected 2 events for Container");
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { rect, brush }
            if *rect == c.widget_base().geometry
                && brush_color(brush) == palette.color(ColorRole::Window)),
        "events[0] must be FillRect(Window) covering widget geometry"
    );
    assert!(
        matches!(&painter.events[1], PaintEvent::DrawRect { rect, pen, brush }
            if *rect == c.widget_base().geometry
                && pen.color() == palette.color(ColorRole::WindowText)
                && pen.width() == 1.0
                && brush_color(brush) == Color::TRANSPARENT),
        "events[1] must be DrawRect with WindowText 1px outline"
    );
    assert!(
        !painter.events.iter().any(|e| matches!(
            e,
            PaintEvent::DrawText { .. } | PaintEvent::DrawTextIn { .. }
        )),
        "Container must not emit any text draw events"
    );
}

#[test]
fn container_routing_ignores_children() {
    let mut c = Container::new();
    c.add_child(ObjectId::new());
    c.add_child(ObjectId::new());
    let mut painter = RecordingPainter::default();
    let palette = container_palette();
    DefaultStyle.draw_widget(&c, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        2,
        "add_child must not change the number of recorded events"
    );
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Window)),
        "FillRect must still use Window role regardless of children"
    );
}

fn line_edit_palette() -> Palette {
    Palette::default()
        .with_role(ColorRole::Base, Color::new(0.95, 0.95, 0.95, 1.0))
        .with_role(ColorRole::Text, Color::BLACK)
}

fn line_edit_read_only_palette() -> Palette {
    line_edit_palette().with_role(ColorRole::Window, Color::new(0.9, 0.9, 0.9, 1.0))
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn line_edit_records_fill_outline_and_empty_text() {
    let e = LineEdit::new();
    let mut painter = RecordingPainter::default();
    let palette = line_edit_palette();
    DefaultStyle.draw_widget(&e, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        3,
        "expected 3 events for empty LineEdit"
    );
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Base)),
        "events[0] must be FillRect(Base)"
    );
    assert!(
        matches!(&painter.events[1], PaintEvent::DrawRect { pen, brush, .. }
            if pen.color() == palette.color(ColorRole::Text)
                && pen.width() == 1.0
                && brush_color(brush) == Color::TRANSPARENT),
        "events[1] must be DrawRect with Text 1px outline"
    );
    assert!(
        matches!(&painter.events[2], PaintEvent::DrawTextIn { text, alignment, brush, .. }
            if text.is_empty()
                && *alignment == Alignment::Left
                && brush_color(brush) == palette.color(ColorRole::Text)),
        "events[2] must be DrawTextIn with empty text, Left align, full-alpha Text brush"
    );
}

#[test]
fn line_edit_records_text_when_non_empty() {
    let mut e = LineEdit::new();
    e.text = "abc".into();
    let mut painter = RecordingPainter::default();
    let palette = line_edit_palette();
    DefaultStyle.draw_widget(&e, &mut painter, &palette);

    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { text, alignment, brush, .. }
                if text == "abc"
                    && *alignment == Alignment::Left
                    && brush_color(brush) == palette.color(ColorRole::Text)),
        "DrawTextIn must carry 'abc', Left, full-alpha Text brush"
    );
}

#[test]
fn line_edit_placeholder_drawn_when_text_empty() {
    let mut e = LineEdit::new();
    e.placeholder = "hint".into();
    let mut painter = RecordingPainter::default();
    let palette = line_edit_palette();
    DefaultStyle.draw_widget(&e, &mut painter, &palette);

    let draw_text_count = painter
        .events
        .iter()
        .filter(|ev| matches!(ev, PaintEvent::DrawTextIn { .. }))
        .count();
    assert_eq!(
        draw_text_count, 1,
        "exactly one DrawTextIn event (no duplicate)"
    );
    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { text, alignment, brush, .. }
                if text == "hint"
                    && *alignment == Alignment::Left
                    && brush_color(brush) == super::disabled(palette.color(ColorRole::Text))),
        "placeholder DrawTextIn must carry 'hint', Left, half-alpha Text brush"
    );
}

#[test]
fn line_edit_non_empty_text_ignores_placeholder() {
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.placeholder = "hint".into();
    let mut painter = RecordingPainter::default();
    let palette = line_edit_palette();
    DefaultStyle.draw_widget(&e, &mut painter, &palette);

    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { text, brush, .. }
                if text == "abc"
                    && brush_color(brush) == palette.color(ColorRole::Text)),
        "non-empty text wins over placeholder: DrawTextIn must carry 'abc' with full-alpha Text"
    );
}

#[test]
fn line_edit_read_only_inserts_overlay() {
    let mut e = LineEdit::new();
    e.read_only = true;
    let mut painter = RecordingPainter::default();
    let palette = line_edit_read_only_palette();
    DefaultStyle.draw_widget(&e, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events for read-only LineEdit (bg + overlay + outline + text)"
    );
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Base)),
        "events[0] must be FillRect(Base background)"
    );
    assert!(
        matches!(&painter.events[1], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::WindowText).with_alpha(super::READ_ONLY_OVERLAY_ALPHA)),
        "events[1] must be FillRect(WindowText @ READ_ONLY_OVERLAY_ALPHA) read-only overlay"
    );
    assert!(
        matches!(&painter.events[2], PaintEvent::DrawRect { .. }),
        "events[2] must be DrawRect (outline)"
    );
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { text, .. }
            if text.is_empty()),
        "events[3] must be DrawTextIn with empty text (empty text + no placeholder path)"
    );
}

#[test]
fn line_edit_read_only_with_placeholder_overlays_and_renders_placeholder() {
    let mut e = LineEdit::new();
    e.read_only = true;
    e.placeholder = "hint".into();
    let mut painter = RecordingPainter::default();
    let palette = line_edit_read_only_palette();
    DefaultStyle.draw_widget(&e, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events: bg + overlay + outline + text"
    );
    assert!(
        matches!(&painter.events[1], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::WindowText).with_alpha(super::READ_ONLY_OVERLAY_ALPHA)),
        "events[1] must be the read-only overlay"
    );
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { text, alignment, brush, .. }
            if text == "hint"
                && *alignment == Alignment::Left
                && brush_color(brush) == super::disabled(palette.color(ColorRole::Text))),
        "events[3] must be DrawTextIn('hint', Left, half-alpha Text) — placeholder path"
    );
}

#[test]
fn line_edit_read_only_dims_text() {
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.read_only = true;
    let mut painter = RecordingPainter::default();
    let palette = line_edit_read_only_palette();
    DefaultStyle.draw_widget(&e, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events for read-only LineEdit with text"
    );
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Text).with_alpha(super::READ_ONLY_TEXT_ALPHA)),
        "events[3] DrawTextIn brush must be Text dimmed to READ_ONLY_TEXT_ALPHA"
    );
}

#[test]
fn line_edit_read_only_empty_text_dims_text() {
    let mut e = LineEdit::new();
    e.read_only = true;
    e.text = String::new();
    e.placeholder = String::new();
    let mut painter = RecordingPainter::default();
    let palette = line_edit_read_only_palette();
    DefaultStyle.draw_widget(&e, &mut painter, &palette);

    // Overlay brush check
    assert!(
        matches!(&painter.events[1], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::WindowText).with_alpha(super::READ_ONLY_OVERLAY_ALPHA)),
        "events[1] must be the read-only overlay"
    );
    // Text brush check — empty text, no placeholder → read-only text path
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Text).with_alpha(super::READ_ONLY_TEXT_ALPHA)),
        "events[3] DrawTextIn brush must be dimmed for read-only even with empty text"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn line_edit_writable_keeps_full_alpha_text() {
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.read_only = false;
    let mut painter = RecordingPainter::default();
    let palette = line_edit_palette();
    DefaultStyle.draw_widget(&e, &mut painter, &palette);

    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { brush, .. }
                if brush_color(brush).a() == 1.0),
        "writable LineEdit text brush must have full alpha"
    );
}

// ── AC10: StyleRegistry round-trip ────────────────────────────────────────

#[test]
fn registry_round_trip_dispatches_default_style() {
    let _lock = quartzite_test_helpers::test_lock();
    StyleRegistry::clear_for_test();
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
