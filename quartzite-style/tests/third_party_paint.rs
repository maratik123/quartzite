//! Third-party widget integration test — AC8.
//!
//! Defines a `ThirdPartyWidget` *outside* `quartzite-widgets` (via
//! `#[derive(Extend)]` inside this test file, no `#[widget_view]` attribute →
//! `widget_view()` returns `WidgetView::Other(self)` automatically).
//!
//! Verifies the open-set contract in both directions:
//! - A custom `Style` with `impl Paint<ThirdPartyWidget> for ThirdPartyStyle`
//!   dispatches into the typed `paint` body.
//! - A third-party widget under `DefaultStyle` → silent no-op (AC6).
//! - A built-in `Button` under a `ThirdPartyStyle` that doesn't handle
//!   `Button` → silent no-op (AC2 documented fallback).

use quartzite_macros::Extend;
use quartzite_paint_api::{Painter, TextCaretCursor, TextVisualLine, TextVisualLineCursor};
use quartzite_style::{DefaultStyle, Paint, Palette, Style};
use quartzite_widgets::{AsWidget, WidgetBase, WidgetExt, WidgetView};

// ── ThirdPartyWidget — defined outside quartzite-widgets ─────────────────────

/// A minimal widget defined outside the `quartzite-widgets` crate.
///
/// Uses `#[derive(Extend)]` without `#[widget_view]`, so `widget_view()`
/// returns `WidgetView::Other(self)` automatically.
#[derive(Extend)]
struct ThirdPartyWidget {
    #[base]
    widget_base: WidgetBase,
}

impl ThirdPartyWidget {
    fn new() -> Self {
        Self {
            widget_base: WidgetBase::new(),
        }
    }
}

// ── ThirdPartyStyle — custom style that handles ThirdPartyWidget ──────────────

/// Records each `paint` call so tests can assert dispatch happened.
struct RecordingPainter {
    paint_calls: usize,
    null_caret: NullCaretCursor,
    null_lines: NullLineCursor,
}

impl RecordingPainter {
    const fn new() -> Self {
        Self {
            paint_calls: 0,
            null_caret: NullCaretCursor,
            null_lines: NullLineCursor,
        }
    }
}

struct NullCaretCursor;
impl TextCaretCursor for NullCaretCursor {
    fn advance_to(&mut self, _byte_offset: usize) {}
    fn caret_x(&self) -> i32 {
        0
    }
    fn line_top(&self) -> i32 {
        0
    }
    fn line_height(&self) -> i32 {
        0
    }
}

struct NullLineCursor;
impl TextVisualLineCursor for NullLineCursor {
    fn next_line(&mut self) -> Option<TextVisualLine> {
        None
    }
}

impl Painter for RecordingPainter {
    fn draw_rect(
        &mut self,
        _rect: quartzite_geometry::Rect,
        _pen: &quartzite_paint_api::Pen,
        _brush: &quartzite_paint_api::Brush,
    ) {
    }
    fn fill_rect(&mut self, _rect: quartzite_geometry::Rect, _brush: &quartzite_paint_api::Brush) {
        self.paint_calls += 1;
    }
    fn draw_line(
        &mut self,
        _from: quartzite_geometry::Point,
        _to: quartzite_geometry::Point,
        _pen: &quartzite_paint_api::Pen,
    ) {
    }
    fn clip_rect(&mut self, _rect: quartzite_geometry::Rect) {}
    fn translate(&mut self, _delta: quartzite_geometry::Point) {}
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn draw_text(
        &mut self,
        _pos: quartzite_geometry::Point,
        _text: &str,
        _font: &quartzite_paint_api::Font,
        _brush: &quartzite_paint_api::Brush,
    ) {
    }
    fn draw_text_in(
        &mut self,
        _rect: quartzite_geometry::Rect,
        _text: &str,
        _font: &quartzite_paint_api::Font,
        _brush: &quartzite_paint_api::Brush,
        _alignment: quartzite_geometry::Alignment,
    ) {
    }
    fn draw_image(&mut self, _rect: quartzite_geometry::Rect, _image: &quartzite_paint_api::Image) {
    }
    fn draw_path(
        &mut self,
        _path: &quartzite_paint_api::Path,
        _pen: &quartzite_paint_api::Pen,
        _brush: &quartzite_paint_api::Brush,
    ) {
    }
    fn text_carets(
        &mut self,
        _text: &str,
        _font: &quartzite_paint_api::Font,
    ) -> &mut dyn TextCaretCursor {
        &mut self.null_caret
    }
    fn text_visual_lines(
        &mut self,
        _text: &str,
        _font: &quartzite_paint_api::Font,
        _wrap_width: i32,
    ) -> &mut dyn TextVisualLineCursor {
        &mut self.null_lines
    }
}

/// A custom style that handles `ThirdPartyWidget` but not built-in widgets.
struct ThirdPartyStyle;

impl Paint<ThirdPartyWidget> for ThirdPartyStyle {
    fn paint(&self, _widget: &ThirdPartyWidget, painter: &mut dyn Painter, _palette: &Palette) {
        // Emit a fill_rect so RecordingPainter can detect the call.
        use quartzite_geometry::{Point, Rect, Size};
        use quartzite_paint_api::{Brush, Color};
        painter.fill_rect(
            Rect::new(Point::new(0, 0), Size::new(1, 1)),
            &Brush::solid(Color::BLACK),
        );
    }
}

impl Style for ThirdPartyStyle {
    fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
        if let WidgetView::Other(other) = widget.widget_view()
            && let Some(w) = other.as_any().downcast_ref::<ThirdPartyWidget>()
        {
            self.paint(w, painter, palette);
        }
        // Built-in variants are not handled — documented no-op per AC2.
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// `ThirdPartyWidget` under `ThirdPartyStyle` dispatches into the typed `Paint<W>` body.
#[test]
fn third_party_widget_under_third_party_style_dispatches() {
    let mut w = ThirdPartyWidget::new();
    w.show();
    let style = ThirdPartyStyle;
    let mut painter = RecordingPainter::new();

    style.draw_widget(&w as &dyn AsWidget, &mut painter, &Palette::default());

    assert_eq!(
        painter.paint_calls, 1,
        "expected exactly 1 fill_rect from Paint<ThirdPartyWidget>::paint"
    );
}

/// `widget_view()` on `ThirdPartyWidget` returns `WidgetView::Other` (no `#[widget_view]` attribute).
#[test]
fn third_party_widget_view_returns_other() {
    let w = ThirdPartyWidget::new();
    assert!(
        matches!(w.widget_view(), WidgetView::Other(_)),
        "ThirdPartyWidget without #[widget_view] should return WidgetView::Other"
    );
}

/// `ThirdPartyWidget` under `DefaultStyle` → silent no-op (AC6 open-set fallback).
#[test]
fn third_party_widget_under_default_style_is_noop() {
    let mut w = ThirdPartyWidget::new();
    w.show();
    let style = DefaultStyle;
    let mut painter = RecordingPainter::new();

    style.draw_widget(&w as &dyn AsWidget, &mut painter, &Palette::default());

    assert_eq!(
        painter.paint_calls, 0,
        "DefaultStyle should produce zero painter calls for an unknown widget type"
    );
}

/// Built-in Button under `ThirdPartyStyle` → silent no-op (AC2 documented fallback).
#[test]
fn builtin_button_under_third_party_style_is_noop() {
    let mut w = quartzite_widgets::Button::new("X".into());
    w.show();
    let style = ThirdPartyStyle;
    let mut painter = RecordingPainter::new();

    style.draw_widget(&w as &dyn AsWidget, &mut painter, &Palette::default());

    assert_eq!(
        painter.paint_calls, 0,
        "ThirdPartyStyle should not paint a Button (no Paint<Button> impl)"
    );
}

/// `Box<dyn Style>` with `ThirdPartyStyle` satisfies the object-safe contract (AC11).
#[test]
fn third_party_style_is_object_safe() {
    let _boxed: Box<dyn Style> = Box::new(ThirdPartyStyle);
}
