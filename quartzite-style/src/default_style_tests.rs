//! Sibling test module for `default_style.rs`; entire body is `#[cfg(test)]`
//! (test lines are excluded from the per-file size cap per AGENTS.md *Code Style*).
//!
//! Attached via `#[cfg(test)] #[path = "default_style_tests.rs"] mod tests;`
//! in `default_style.rs`. The module name is still `tests`, so `super::` still
//! resolves to `crate::default_style` — all `super::disabled` / `super::brush`
//! calls remain valid.

use quartzite_core::ObjectId;
use quartzite_geometry::{Point, Rect};
use quartzite_paint_api::{
    Brush, Color, Font, Image, Painter, Path, Pen, TextCaretCursor, TextVisualLine,
    TextVisualLineCursor,
};
use quartzite_style_types::{ColorGroup, ColorRole, Palette};
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

// ── Fake fixed-width shaper (inline per-impl, design decision §3) ────────
//
// Contract: one cluster per `char`, 8 px advance, line_height = font.size_pt()
// rounded to the nearest integer, wraps at wrap_width / 8 chars per line.

/// Pixel advance per character cluster in the fake shaper.
const FAKE_ADVANCE: i32 = 8;

fn fake_line_height(font: &Font) -> i32 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "test-only: font size is always a small representable value"
    )]
    let lh = font.size_pt().round() as i32;
    lh
}

fn fake_chars_per_line(wrap_width: i32) -> usize {
    if wrap_width > 0 {
        #[allow(
            clippy::cast_sign_loss,
            reason = "test-only: wrap_width > 0 guard makes the cast safe"
        )]
        let n = (wrap_width / FAKE_ADVANCE).max(1) as usize;
        n
    } else {
        usize::MAX
    }
}

struct FakeCaretCursor {
    text: String,
    positions: Vec<(i32, i32, i32)>,
    idx: usize,
}

impl FakeCaretCursor {
    fn new(text: &str, font: &Font) -> Self {
        let lh = fake_line_height(font);
        let mut positions = Vec::new();
        let mut x = 0i32;
        for _ in text.chars() {
            positions.push((x, 0, lh));
            x += FAKE_ADVANCE;
        }
        positions.push((x, 0, lh));
        Self {
            text: text.to_owned(),
            positions,
            idx: 0,
        }
    }

    fn current(&self) -> (i32, i32, i32) {
        self.positions
            .get(self.idx)
            .copied()
            .unwrap_or_else(|| *self.positions.last().unwrap_or(&(0, 0, 0)))
    }
}

impl TextCaretCursor for FakeCaretCursor {
    fn advance_to(&mut self, byte_offset: usize) {
        let clamped = byte_offset.min(self.text.len());
        self.idx = self.text[..clamped].chars().count();
    }
    fn caret_x(&self) -> i32 {
        self.current().0
    }
    fn line_top(&self) -> i32 {
        self.current().1
    }
    fn line_height(&self) -> i32 {
        self.current().2
    }
}

struct FakeLineCursor {
    lines: Vec<TextVisualLine>,
    idx: usize,
}

impl FakeLineCursor {
    fn new(text: &str, font: &Font, wrap_width: i32) -> Self {
        let lh = fake_line_height(font);
        let chars_per = fake_chars_per_line(wrap_width);
        let mut lines = Vec::new();
        let mut top = 0i32;
        let mut byte_pos = 0usize;
        let mut line_char_count = 0usize;
        let mut line_start_byte = 0usize;
        for ch in text.chars() {
            let ch_bytes = ch.len_utf8();
            if ch == '\n' {
                lines.push(TextVisualLine {
                    byte_start: line_start_byte,
                    byte_end: byte_pos + ch_bytes,
                    top,
                    height: lh,
                });
                top += lh;
                byte_pos += ch_bytes;
                line_start_byte = byte_pos;
                line_char_count = 0;
                continue;
            }
            if line_char_count == chars_per {
                lines.push(TextVisualLine {
                    byte_start: line_start_byte,
                    byte_end: byte_pos,
                    top,
                    height: lh,
                });
                top += lh;
                line_start_byte = byte_pos;
                line_char_count = 0;
            }
            byte_pos += ch_bytes;
            line_char_count += 1;
        }
        lines.push(TextVisualLine {
            byte_start: line_start_byte,
            byte_end: byte_pos,
            top,
            height: lh,
        });
        Self { lines, idx: 0 }
    }
}

impl TextVisualLineCursor for FakeLineCursor {
    fn next_line(&mut self) -> Option<TextVisualLine> {
        let line = self.lines.get(self.idx).copied();
        if line.is_some() {
            self.idx += 1;
        }
        line
    }
}

// ── Recording painter ────────────────────────────────────────────────────

#[derive(Default)]
struct RecordingPainter {
    events: Vec<PaintEvent>,
    caret_cursor: Option<FakeCaretCursor>,
    line_cursor: Option<FakeLineCursor>,
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
    fn text_carets(&mut self, text: &str, font: &Font) -> &mut dyn TextCaretCursor {
        self.caret_cursor = Some(FakeCaretCursor::new(text, font));
        self.caret_cursor.as_mut().unwrap()
    }
    fn text_visual_lines(
        &mut self,
        text: &str,
        font: &Font,
        wrap_width: i32,
    ) -> &mut dyn TextVisualLineCursor {
        self.line_cursor = Some(FakeLineCursor::new(text, font, wrap_width));
        self.line_cursor.as_mut().unwrap()
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
    let _b: Box<dyn Style> = Box::new(DefaultStyle::new());
}

// ── AC2: Button → fill + outline + centred text ───────────────────────────

#[test]
fn button_records_fill_outline_and_centred_text() {
    let btn = Button::new("OK".into());
    let mut painter = RecordingPainter::default();
    let palette = Palette::default();
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

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
    DefaultStyle::new().draw_widget(&lbl, &mut painter, &palette);

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
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

    // read_only == false: FillRect(base) → DrawRect → DrawTextIn (3 events).
    assert_eq!(
        painter.events.len(),
        3,
        "expected 3 events for TextEdit (read_only=false)"
    );
    assert!(
        matches!(first_fill(&painter.events),
            PaintEvent::FillRect { brush, .. }
                if brush_color(brush) == palette.color(ColorRole::Base, ColorGroup::Normal)),
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
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

    // read_only == true: FillRect(base) → FillRect(overlay) → DrawRect → DrawTextIn (4 events).
    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events for TextEdit (read_only=true)"
    );
    let expected_overlay = palette
        .color(ColorRole::WindowText, ColorGroup::Normal)
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
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events for read-only TextEdit with text"
    );
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Text, ColorGroup::Normal).with_alpha(super::READ_ONLY_TEXT_ALPHA)),
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
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

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
    let palette = Palette::default().with_role(
        ColorRole::WindowText,
        ColorGroup::Normal,
        Color::new(0.0, 0.5, 1.0, 1.0),
    );
    let mut edit = TextEdit::new();
    edit.read_only = true;
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

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
    DefaultStyle::new().draw_widget(&area, &mut painter, &palette);

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
    DefaultStyle::new().draw_widget(&base, &mut painter, &palette);
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
    let palette =
        Palette::default().with_role(ColorRole::Highlight, ColorGroup::Normal, Color::SKY_BLUE);

    let idle_btn = Button::new("x".into());
    let mut checked_btn = Button::new("x".into());
    checked_btn.checked = true;

    let mut idle_painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&idle_btn, &mut idle_painter, &palette);
    let idle_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = &idle_painter.events[0] {
            brush
        } else {
            panic!("first event was not FillRect")
        },
    );

    let mut checked_painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&checked_btn, &mut checked_painter, &palette);
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
        palette.color(ColorRole::Button, ColorGroup::Normal),
        "idle fill must use ColorRole::Button"
    );
    assert_eq!(
        checked_color,
        palette.color(ColorRole::Highlight, ColorGroup::Normal),
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
    DefaultStyle::new().draw_widget(&enabled_btn, &mut enabled_painter, &palette);

    let mut disabled_painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&disabled_btn, &mut disabled_painter, &palette);

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
        .with_role(ColorRole::Button, ColorGroup::Normal, Color::WHITE)
        .with_role(ColorRole::Highlight, ColorGroup::Normal, Color::SKY_BLUE)
        .with_role(ColorRole::ButtonText, ColorGroup::Normal, Color::BLACK)
        .with_role(ColorRole::HighlightedText, ColorGroup::Normal, Color::WHITE)
}

#[test]
fn hovered_button_uses_derived_hover_fill() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

    // Expected fill: palette.color(Button, Hover) — derived from the pinned palette.
    let expected_fill = palette.color(ColorRole::Button, ColorGroup::Hover);
    let idle_fill = palette.color(ColorRole::Button, ColorGroup::Normal);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("first_fill did not return FillRect")
        },
    );
    assert_eq!(
        fill_color, expected_fill,
        "hovered fill must equal palette.color(Button, Hover)"
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
        palette.color(ColorRole::ButtonText, ColorGroup::Hover),
        "hovered button text must use ButtonText Hover group"
    );
}

#[test]
fn pressed_button_uses_highlight_pressed() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_pressed(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("first_fill did not return FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed fill must be Highlight × Pressed"
    );
    assert_ne!(
        fill_color,
        palette.color(ColorRole::Button, ColorGroup::Normal),
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
        palette.color(ColorRole::HighlightedText, ColorGroup::Pressed),
        "pressed text must be HighlightedText × Pressed"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn focused_button_uses_2px_focus_ring_outline() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::FocusRing, ColorGroup::Normal),
        "focused outline color must be FocusRing × Normal"
    );
    assert_eq!(pen_width, 2.0, "focused outline must be 2 px wide");

    // Idle baseline has width 1.0 — verify it changes.
    let mut idle_painter = RecordingPainter::default();
    let idle_btn = Button::new("x".into());
    DefaultStyle::new().draw_widget(&idle_btn, &mut idle_painter, &palette);
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
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

    // Fill: pressed selects Highlight × Pressed, then disabled halves its alpha.
    let expected_highlight_pressed = palette.color(ColorRole::Highlight, ColorGroup::Pressed);
    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color.r(),
        expected_highlight_pressed.r(),
        "disabled+pressed fill r must match Highlight×Pressed r"
    );
    assert_eq!(
        fill_color.g(),
        expected_highlight_pressed.g(),
        "disabled+pressed fill g must match Highlight×Pressed g"
    );
    assert_eq!(
        fill_color.b(),
        expected_highlight_pressed.b(),
        "disabled+pressed fill b must match Highlight×Pressed b"
    );
    assert_eq!(
        fill_color.a(),
        expected_highlight_pressed.a() * 0.5,
        "disabled fill alpha must be half of Highlight×Pressed alpha"
    );

    // Focused outline survives disabled: 2 px, full-alpha FocusRing × Normal.
    let expected_focus_ring = palette.color(ColorRole::FocusRing, ColorGroup::Normal);
    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color, expected_focus_ring,
        "focus outline color must be full-alpha FocusRing×Normal even when disabled"
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
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    // checked wins over hover: group = Hover, role = Highlight.
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Hover),
        "checked wins over hover: fill must be Highlight × Hover"
    );
}

#[test]
fn precedence_pressed_checked_both_map_to_highlight() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_pressed(true);
    btn.checked = true;
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed+checked both map to Highlight × Pressed"
    );
}

/// AC10 — disabled AND focused button paints half-alpha fill plus 2 px focus outline.
///
/// `disabled` is an alpha modifier; `focused` is an additive outline modifier.
/// Both coexist: the idle fill is halved and the focus ring is drawn at full alpha.
#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn disabled_and_focused_button_paints_half_alpha_fill_plus_outline() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_enabled(false);
    btn.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    // Idle role (not pressed, not checked) → Button × Normal, then alpha-halved.
    assert_eq!(
        fill_color.a(),
        palette.color(ColorRole::Button, ColorGroup::Normal).a() * 0.5,
        "disabled fill alpha must be half of Button×Normal alpha"
    );

    // Focus ring: 2 px, full-alpha FocusRing × Normal.
    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::FocusRing, ColorGroup::Normal),
        "focus outline color must be FocusRing × Normal even when disabled"
    );
    assert_eq!(
        pen_width, 2.0,
        "focus outline must be 2 px wide even when disabled"
    );
}

/// AC10 — pressed AND checked button picks Highlight × Pressed for fill.
///
/// Exercises the `pressed || checked` role-selection branch with both bits set.
#[test]
fn pressed_and_checked_button_picks_highlight_pressed() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_pressed(true);
    btn.checked = true;
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed+checked button fill must be Highlight × Pressed"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn precedence_focused_hovered_hover_fill_plus_focus_ring_outline() {
    let palette = pinned_palette();
    let mut btn = Button::new("x".into());
    btn.set_focused(true);
    btn.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

    // Fill: hover state → Button × Hover (derived, no blend heuristic).
    let expected_hover_fill = palette.color(ColorRole::Button, ColorGroup::Hover);
    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color, expected_hover_fill,
        "focused+hovered fill must equal Button × Hover"
    );

    // Outline: FocusRing × Normal, 2 px.
    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::FocusRing, ColorGroup::Normal),
        "focused outline color must be FocusRing × Normal"
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
    DefaultStyle::new().draw_widget(&btn, &mut painter, &palette);

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
        palette.color(ColorRole::Button, ColorGroup::Normal),
        "idle fill is Button × Normal"
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
        .with_role(
            ColorRole::Window,
            ColorGroup::Normal,
            Color::new(0.9, 0.9, 0.9, 1.0),
        )
        .with_role(
            ColorRole::WindowText,
            ColorGroup::Normal,
            Color::new(0.1, 0.1, 0.1, 1.0),
        )
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
    DefaultStyle::new().draw_widget(&c, &mut painter, &palette);

    assert_eq!(painter.events.len(), 2, "expected 2 events for Container");
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { rect, brush }
            if *rect == c.widget_base().geometry
                && brush_color(brush) == palette.color(ColorRole::Window, ColorGroup::Normal)),
        "events[0] must be FillRect(Window) covering widget geometry"
    );
    assert!(
        matches!(&painter.events[1], PaintEvent::DrawRect { rect, pen, brush }
            if *rect == c.widget_base().geometry
                && pen.color() == palette.color(ColorRole::WindowText, ColorGroup::Normal)
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
    DefaultStyle::new().draw_widget(&c, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        2,
        "add_child must not change the number of recorded events"
    );
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Window, ColorGroup::Normal)),
        "FillRect must still use Window role regardless of children"
    );
}

fn line_edit_palette() -> Palette {
    Palette::default()
        .with_role(
            ColorRole::Base,
            ColorGroup::Normal,
            Color::new(0.95, 0.95, 0.95, 1.0),
        )
        .with_role(ColorRole::Text, ColorGroup::Normal, Color::BLACK)
}

fn line_edit_read_only_palette() -> Palette {
    line_edit_palette().with_role(
        ColorRole::Window,
        ColorGroup::Normal,
        Color::new(0.9, 0.9, 0.9, 1.0),
    )
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
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        3,
        "expected 3 events for empty LineEdit"
    );
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Base, ColorGroup::Normal)),
        "events[0] must be FillRect(Base)"
    );
    assert!(
        matches!(&painter.events[1], PaintEvent::DrawRect { pen, brush, .. }
            if pen.color() == palette.color(ColorRole::Text, ColorGroup::Normal)
                && pen.width() == 1.0
                && brush_color(brush) == Color::TRANSPARENT),
        "events[1] must be DrawRect with Text 1px outline"
    );
    assert!(
        matches!(&painter.events[2], PaintEvent::DrawTextIn { text, alignment, brush, .. }
            if text.is_empty()
                && *alignment == Alignment::Left
                && brush_color(brush) == palette.color(ColorRole::Text, ColorGroup::Normal)),
        "events[2] must be DrawTextIn with empty text, Left align, full-alpha Text brush"
    );
}

#[test]
fn line_edit_records_text_when_non_empty() {
    let mut e = LineEdit::new();
    e.text = "abc".into();
    let mut painter = RecordingPainter::default();
    let palette = line_edit_palette();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { text, alignment, brush, .. }
                if text == "abc"
                    && *alignment == Alignment::Left
                    && brush_color(brush) == palette.color(ColorRole::Text, ColorGroup::Normal)),
        "DrawTextIn must carry 'abc', Left, full-alpha Text brush"
    );
}

#[test]
fn line_edit_placeholder_drawn_when_text_empty() {
    let mut e = LineEdit::new();
    e.placeholder = "hint".into();
    let mut painter = RecordingPainter::default();
    let palette = line_edit_palette();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

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
                    && brush_color(brush) == super::disabled(palette.color(ColorRole::Text, ColorGroup::Normal))),
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
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { text, brush, .. }
                if text == "abc"
                    && brush_color(brush) == palette.color(ColorRole::Text, ColorGroup::Normal)),
        "non-empty text wins over placeholder: DrawTextIn must carry 'abc' with full-alpha Text"
    );
}

#[test]
fn line_edit_read_only_inserts_overlay() {
    let mut e = LineEdit::new();
    e.read_only = true;
    let mut painter = RecordingPainter::default();
    let palette = line_edit_read_only_palette();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events for read-only LineEdit (bg + overlay + outline + text)"
    );
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Base, ColorGroup::Normal)),
        "events[0] must be FillRect(Base background)"
    );
    assert!(
        matches!(&painter.events[1], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::WindowText, ColorGroup::Normal).with_alpha(super::READ_ONLY_OVERLAY_ALPHA)),
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
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events: bg + overlay + outline + text"
    );
    assert!(
        matches!(&painter.events[1], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::WindowText, ColorGroup::Normal).with_alpha(super::READ_ONLY_OVERLAY_ALPHA)),
        "events[1] must be the read-only overlay"
    );
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { text, alignment, brush, .. }
            if text == "hint"
                && *alignment == Alignment::Left
                && brush_color(brush) == super::disabled(palette.color(ColorRole::Text, ColorGroup::Normal))),
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
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events for read-only LineEdit with text"
    );
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Text, ColorGroup::Normal).with_alpha(super::READ_ONLY_TEXT_ALPHA)),
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
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    // Overlay brush check
    assert!(
        matches!(&painter.events[1], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::WindowText, ColorGroup::Normal).with_alpha(super::READ_ONLY_OVERLAY_ALPHA)),
        "events[1] must be the read-only overlay"
    );
    // Text brush check — empty text, no placeholder → read-only text path
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Text, ColorGroup::Normal).with_alpha(super::READ_ONLY_TEXT_ALPHA)),
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
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { brush, .. }
                if brush_color(brush).a() == 1.0),
        "writable LineEdit text brush must have full alpha"
    );
}

// ── LineEdit state branches (issue #406; folds in #407) ─────────────────

#[test]
fn hovered_line_edit_uses_derived_hover_fill() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Base, ColorGroup::Hover),
        "hovered LineEdit fill must equal palette.color(Base, Hover)"
    );

    let pen_color = if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
        pen.color()
    } else {
        panic!("expected DrawRect")
    };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::Text, ColorGroup::Hover),
        "hovered LineEdit outline must equal palette.color(Text, Hover)"
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
        palette.color(ColorRole::Text, ColorGroup::Hover),
        "hovered LineEdit text must equal palette.color(Text, Hover)"
    );
}

#[test]
fn pressed_line_edit_uses_highlight_pressed() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.set_pressed(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed LineEdit fill must equal palette.color(Highlight, Pressed)"
    );

    let pen_color = if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
        pen.color()
    } else {
        panic!("expected DrawRect")
    };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::HighlightedText, ColorGroup::Pressed),
        "pressed LineEdit outline must equal palette.color(HighlightedText, Pressed)"
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
        palette.color(ColorRole::HighlightedText, ColorGroup::Pressed),
        "pressed LineEdit text must equal palette.color(HighlightedText, Pressed)"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn focused_line_edit_uses_2px_focus_ring_outline() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::FocusRing, ColorGroup::Normal),
        "focused LineEdit outline color must be FocusRing × Normal"
    );
    assert_eq!(pen_width, 2.0, "focused LineEdit outline must be 2 px wide");
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn disabled_and_focused_line_edit_paints_outline_under_disabled() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.set_enabled(false);
    e.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    let expected_focus_ring = palette.color(ColorRole::FocusRing, ColorGroup::Normal);
    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color, expected_focus_ring,
        "focus outline color must be full-alpha FocusRing×Normal even when disabled"
    );
    assert_eq!(
        pen_color.a(),
        expected_focus_ring.a(),
        "FocusRing pen alpha must NOT be halved under disabled"
    );
    assert_eq!(
        pen_width, 2.0,
        "focus outline must still be 2 px wide when disabled"
    );
}

#[test]
fn precedence_pressed_hovered_line_edit_picks_pressed_fill() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.set_pressed(true);
    e.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed wins over hovered: LineEdit fill must be Highlight × Pressed"
    );
}

/// AC2 / #407 fold-in anchor — disabled-idle `LineEdit` halves alpha on
/// Base fill + Text outline + Text glyph brush. The pre-spec impl wraps
/// zero colours in `maybe_disabled`, so this test would fail against it.
#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn line_edit_disabled_idle_dims_base_text_outline() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.set_enabled(false);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        super::maybe_disabled(palette.color(ColorRole::Base, ColorGroup::Normal), false),
        "disabled LineEdit fill must equal maybe_disabled(Base × Normal, false) (≈ × 0.5 alpha)"
    );

    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        super::maybe_disabled(palette.color(ColorRole::Text, ColorGroup::Normal), false),
        "disabled LineEdit outline color must equal maybe_disabled(Text × Normal, false)"
    );
    assert_eq!(
        pen_width, 1.0,
        "disabled-idle LineEdit outline must still be 1 px wide"
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
        super::maybe_disabled(palette.color(ColorRole::Text, ColorGroup::Normal), false),
        "disabled LineEdit text glyph brush must equal maybe_disabled(Text × Normal, false)"
    );
}

/// AC5 — `read_only` overlays the hover-state base fill.
///
/// Captures 4 events: fill(Base × Hover) / fill(`WindowText` overlay at `READ_ONLY_OVERLAY_ALPHA`) /
/// outline(Text × Hover @ 1 px) / text(Text × Hover @ `READ_ONLY_TEXT_ALPHA`).
#[test]
fn line_edit_read_only_hovered_overlay_plus_hover_base_fill() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.text = "abc".into();
    e.read_only = true;
    e.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events for read-only + hovered LineEdit"
    );

    // events[0] — Base × Hover fill.
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Base, ColorGroup::Hover)),
        "events[0] FillRect brush must be Base × Hover"
    );

    // events[1] — read-only overlay derived from WindowText × Normal.
    let expected_overlay = palette
        .color(ColorRole::WindowText, ColorGroup::Normal)
        .with_alpha(super::READ_ONLY_OVERLAY_ALPHA);
    assert!(
        matches!(&painter.events[1], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == expected_overlay),
        "events[1] FillRect brush must be the WindowText overlay at READ_ONLY_OVERLAY_ALPHA"
    );

    // events[2] — outline Text × Hover.
    assert!(
        matches!(&painter.events[2], PaintEvent::DrawRect { pen, .. }
            if pen.color() == palette.color(ColorRole::Text, ColorGroup::Hover)),
        "events[2] DrawRect pen must be Text × Hover"
    );

    // events[3] — text Text × Hover, dimmed to READ_ONLY_TEXT_ALPHA.
    let expected_text = palette
        .color(ColorRole::Text, ColorGroup::Hover)
        .with_alpha(super::READ_ONLY_TEXT_ALPHA);
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { brush, .. }
            if brush_color(brush) == expected_text),
        "events[3] DrawTextIn brush must be Text × Hover dimmed to READ_ONLY_TEXT_ALPHA"
    );
}

/// AC4 — placeholder tracks the state-resolved text colour through hover.
#[test]
fn line_edit_hovered_placeholder_tracks_hover_text() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.placeholder = "hint".into();
    e.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    let expected = super::disabled(palette.color(ColorRole::Text, ColorGroup::Hover));
    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { text, brush, .. }
                if text == "hint" && brush_color(brush) == expected),
        "hovered placeholder must be drawn at disabled(Text × Hover)"
    );
}

/// AC4 — placeholder tracks the state-resolved text colour through pressed.
///
/// `HighlightedText` is intentional here (role-swap on press for legibility
/// under the inverted Highlight fill — see spec § Key decisions row
/// "Outline role mapping"), NOT a copy-paste of `Text`.
#[test]
fn line_edit_pressed_placeholder_tracks_pressed_text() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.placeholder = "hint".into();
    e.set_pressed(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    // `HighlightedText` (not `Text`) is the role-swap on press per
    // spec § Key decisions row "Outline role mapping".
    let expected = super::disabled(palette.color(ColorRole::HighlightedText, ColorGroup::Pressed));
    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { text, brush, .. }
                if text == "hint" && brush_color(brush) == expected),
        "pressed placeholder must be drawn at disabled(HighlightedText × Pressed)"
    );
}

/// AC4 / #407 fold-in flows through placeholder — disabled-placeholder
/// composes `disabled()` × `maybe_disabled(_, false)` ≈ `× 0.25` alpha.
#[test]
fn line_edit_disabled_placeholder_composes_double_dim() {
    let palette = Palette::default();
    let mut e = LineEdit::new();
    e.placeholder = "hint".into();
    e.set_enabled(false);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&e, &mut painter, &palette);

    let expected = super::disabled(super::maybe_disabled(
        palette.color(ColorRole::Text, ColorGroup::Normal),
        false,
    ));
    assert!(
        matches!(first_draw_text_in(&painter.events),
            PaintEvent::DrawTextIn { text, brush, .. }
                if text == "hint" && brush_color(brush) == expected),
        "disabled placeholder must be drawn at disabled(maybe_disabled(Text × Normal, false)) ≈ × 0.25 alpha"
    );
}

// ── Label / TextEdit / ScrollArea state branches (spec 2026-05-21 / issue #403) ─

// Label state tests ─ AC1 + AC4 / AC5

#[test]
fn hovered_label_uses_derived_hover_fill() {
    let palette = Palette::default();
    let mut lbl = Label::new("hi".into());
    lbl.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&lbl, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Window, ColorGroup::Hover),
        "hovered Label fill must equal palette.color(Window, Hover)"
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
        palette.color(ColorRole::WindowText, ColorGroup::Hover),
        "hovered Label text must equal palette.color(WindowText, Hover)"
    );
}

#[test]
fn pressed_label_uses_highlight_pressed() {
    let palette = Palette::default();
    let mut lbl = Label::new("hi".into());
    lbl.set_pressed(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&lbl, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed Label fill must equal palette.color(Highlight, Pressed)"
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
        palette.color(ColorRole::HighlightedText, ColorGroup::Pressed),
        "pressed Label text must equal palette.color(HighlightedText, Pressed)"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn focused_label_uses_2px_focus_ring_outline() {
    let palette = Palette::default();
    let mut lbl = Label::new("hi".into());
    lbl.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&lbl, &mut painter, &palette);

    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::FocusRing, ColorGroup::Normal),
        "focused Label outline color must be FocusRing × Normal"
    );
    assert_eq!(pen_width, 2.0, "focused Label outline must be 2 px wide");
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn disabled_and_focused_label_paints_outline_under_disabled() {
    let palette = Palette::default();
    let mut lbl = Label::new("hi".into());
    lbl.set_enabled(false);
    lbl.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&lbl, &mut painter, &palette);

    let expected_focus_ring = palette.color(ColorRole::FocusRing, ColorGroup::Normal);
    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color, expected_focus_ring,
        "focus outline color must be full-alpha FocusRing×Normal even when disabled"
    );
    assert_eq!(
        pen_color.a(),
        expected_focus_ring.a(),
        "FocusRing pen alpha must NOT be halved under disabled"
    );
    assert_eq!(
        pen_width, 2.0,
        "focus outline must still be 2 px wide when disabled"
    );
}

#[test]
fn precedence_pressed_hovered_label_picks_pressed_fill() {
    let palette = Palette::default();
    let mut lbl = Label::new("hi".into());
    lbl.set_pressed(true);
    lbl.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&lbl, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed wins over hovered: Label fill must be Highlight × Pressed"
    );
}

// TextEdit state tests ─ AC2 + AC4 / AC5

#[test]
fn hovered_text_edit_uses_derived_hover_fill() {
    let palette = Palette::default();
    let mut edit = TextEdit::new();
    edit.plain_text = "abc".into();
    edit.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Base, ColorGroup::Hover),
        "hovered TextEdit fill must equal palette.color(Base, Hover)"
    );

    let pen_color = if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
        pen.color()
    } else {
        panic!("expected DrawRect")
    };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::Text, ColorGroup::Hover),
        "hovered TextEdit outline must equal palette.color(Text, Hover)"
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
        palette.color(ColorRole::Text, ColorGroup::Hover),
        "hovered TextEdit text must equal palette.color(Text, Hover)"
    );
}

#[test]
fn pressed_text_edit_uses_highlight_pressed() {
    let palette = Palette::default();
    let mut edit = TextEdit::new();
    edit.plain_text = "abc".into();
    edit.set_pressed(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed TextEdit fill must equal palette.color(Highlight, Pressed)"
    );

    let pen_color = if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
        pen.color()
    } else {
        panic!("expected DrawRect")
    };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::HighlightedText, ColorGroup::Pressed),
        "pressed TextEdit outline must equal palette.color(HighlightedText, Pressed)"
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
        palette.color(ColorRole::HighlightedText, ColorGroup::Pressed),
        "pressed TextEdit text must equal palette.color(HighlightedText, Pressed)"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn focused_text_edit_uses_2px_focus_ring_outline() {
    let palette = Palette::default();
    let mut edit = TextEdit::new();
    edit.plain_text = "abc".into();
    edit.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::FocusRing, ColorGroup::Normal),
        "focused TextEdit outline color must be FocusRing × Normal"
    );
    assert_eq!(pen_width, 2.0, "focused TextEdit outline must be 2 px wide");
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn disabled_and_focused_text_edit_paints_outline_under_disabled() {
    let palette = Palette::default();
    let mut edit = TextEdit::new();
    edit.plain_text = "abc".into();
    edit.set_enabled(false);
    edit.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

    let expected_focus_ring = palette.color(ColorRole::FocusRing, ColorGroup::Normal);
    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color, expected_focus_ring,
        "focus outline color must be full-alpha FocusRing×Normal even when disabled"
    );
    assert_eq!(
        pen_color.a(),
        expected_focus_ring.a(),
        "FocusRing pen alpha must NOT be halved under disabled"
    );
    assert_eq!(
        pen_width, 2.0,
        "focus outline must still be 2 px wide when disabled"
    );
}

#[test]
fn precedence_pressed_hovered_text_edit_picks_pressed_fill() {
    let palette = Palette::default();
    let mut edit = TextEdit::new();
    edit.plain_text = "abc".into();
    edit.set_pressed(true);
    edit.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed wins over hovered: TextEdit fill must be Highlight × Pressed"
    );
}

/// AC2 — `read_only` overlays the hover-state base fill.
///
/// Captures 4 events: fill(Base × Hover) / fill(`WindowText` overlay at `READ_ONLY_OVERLAY_ALPHA`) /
/// outline(Text × Hover @ 1 px) / text(Text × Hover @ `READ_ONLY_TEXT_ALPHA`).
#[test]
fn text_edit_read_only_hovered_overlay_plus_hover_base_fill() {
    let palette = Palette::default();
    let mut edit = TextEdit::new();
    edit.plain_text = "abc".into();
    edit.read_only = true;
    edit.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&edit, &mut painter, &palette);

    assert_eq!(
        painter.events.len(),
        4,
        "expected 4 events for read-only + hovered TextEdit"
    );

    // events[0] — Base × Hover fill.
    assert!(
        matches!(&painter.events[0], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == palette.color(ColorRole::Base, ColorGroup::Hover)),
        "events[0] FillRect brush must be Base × Hover"
    );

    // events[1] — read-only overlay derived from WindowText × Normal.
    let expected_overlay = palette
        .color(ColorRole::WindowText, ColorGroup::Normal)
        .with_alpha(super::READ_ONLY_OVERLAY_ALPHA);
    assert!(
        matches!(&painter.events[1], PaintEvent::FillRect { brush, .. }
            if brush_color(brush) == expected_overlay),
        "events[1] FillRect brush must be the WindowText overlay at READ_ONLY_OVERLAY_ALPHA"
    );

    // events[2] — outline Text × Hover.
    assert!(
        matches!(&painter.events[2], PaintEvent::DrawRect { pen, .. }
            if pen.color() == palette.color(ColorRole::Text, ColorGroup::Hover)),
        "events[2] DrawRect pen must be Text × Hover"
    );

    // events[3] — text Text × Hover, dimmed to READ_ONLY_TEXT_ALPHA.
    let expected_text = palette
        .color(ColorRole::Text, ColorGroup::Hover)
        .with_alpha(super::READ_ONLY_TEXT_ALPHA);
    assert!(
        matches!(&painter.events[3], PaintEvent::DrawTextIn { brush, .. }
            if brush_color(brush) == expected_text),
        "events[3] DrawTextIn brush must be Text × Hover dimmed to READ_ONLY_TEXT_ALPHA"
    );
}

// ScrollArea state tests ─ AC3 + AC4 / AC5

#[test]
fn hovered_scroll_area_uses_derived_hover_fill() {
    let palette = Palette::default();
    let mut area = ScrollArea::new();
    area.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&area, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Base, ColorGroup::Hover),
        "hovered ScrollArea fill must equal palette.color(Base, Hover)"
    );

    let pen_color = if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
        pen.color()
    } else {
        panic!("expected DrawRect")
    };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::WindowText, ColorGroup::Hover),
        "hovered ScrollArea outline must equal palette.color(WindowText, Hover)"
    );
}

#[test]
fn pressed_scroll_area_uses_highlight_pressed() {
    let palette = Palette::default();
    let mut area = ScrollArea::new();
    area.set_pressed(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&area, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed ScrollArea fill must equal palette.color(Highlight, Pressed)"
    );

    let pen_color = if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
        pen.color()
    } else {
        panic!("expected DrawRect")
    };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::HighlightedText, ColorGroup::Pressed),
        "pressed ScrollArea outline must equal palette.color(HighlightedText, Pressed)"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn focused_scroll_area_uses_2px_focus_ring_outline() {
    let palette = Palette::default();
    let mut area = ScrollArea::new();
    area.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&area, &mut painter, &palette);

    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color,
        palette.color(ColorRole::FocusRing, ColorGroup::Normal),
        "focused ScrollArea outline color must be FocusRing × Normal"
    );
    assert_eq!(
        pen_width, 2.0,
        "focused ScrollArea outline must be 2 px wide"
    );
}

#[test]
#[allow(
    clippy::float_cmp,
    reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
)]
fn disabled_and_focused_scroll_area_paints_outline_under_disabled() {
    let palette = Palette::default();
    let mut area = ScrollArea::new();
    area.set_enabled(false);
    area.set_focused(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&area, &mut painter, &palette);

    let expected_focus_ring = palette.color(ColorRole::FocusRing, ColorGroup::Normal);
    let (pen_color, pen_width) =
        if let PaintEvent::DrawRect { pen, .. } = first_draw_rect(&painter.events) {
            (pen.color(), pen.width())
        } else {
            panic!("expected DrawRect")
        };
    assert_eq!(
        pen_color, expected_focus_ring,
        "focus outline color must be full-alpha FocusRing×Normal even when disabled"
    );
    assert_eq!(
        pen_color.a(),
        expected_focus_ring.a(),
        "FocusRing pen alpha must NOT be halved under disabled"
    );
    assert_eq!(
        pen_width, 2.0,
        "focus outline must still be 2 px wide when disabled"
    );
}

#[test]
fn precedence_pressed_hovered_scroll_area_picks_pressed_fill() {
    let palette = Palette::default();
    let mut area = ScrollArea::new();
    area.set_pressed(true);
    area.set_hovered(true);
    let mut painter = RecordingPainter::default();
    DefaultStyle::new().draw_widget(&area, &mut painter, &palette);

    let fill_color = brush_color(
        if let PaintEvent::FillRect { brush, .. } = first_fill(&painter.events) {
            brush
        } else {
            panic!("expected FillRect")
        },
    );
    assert_eq!(
        fill_color,
        palette.color(ColorRole::Highlight, ColorGroup::Pressed),
        "pressed wins over hovered: ScrollArea fill must be Highlight × Pressed"
    );
}

// ── AC10: StyleRegistry round-trip ────────────────────────────────────────

#[test]
fn registry_round_trip_dispatches_default_style() {
    let _lock = quartzite_test_helpers::test_lock();
    StyleRegistry::clear_for_test();
    StyleRegistry::set_style(Box::new(DefaultStyle::new()));

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
