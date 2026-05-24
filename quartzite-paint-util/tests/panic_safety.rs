//! Panic-safety integration tests for [`quartzite_paint_util::TranslateGuard`].
//!
//! These tests verify that `Drop` runs and calls `restore` on the wrapped
//! painter even when the guarded scope terminates via unwinding.
#![cfg(feature = "std")]

use quartzite_geometry::{Alignment, Point, Rect};
use quartzite_paint_api::{
    Brush, Font, Image, Painter, Path, Pen, TextCaretCursor, TextVisualLine, TextVisualLineCursor,
};
use quartzite_paint_util::TranslateGuard;

// Local minimal RecordingPainter stub — standalone integration-test version.
#[derive(Debug, PartialEq)]
enum PaintEvent {
    Save,
    Restore,
    Translate(Point),
    Other,
}

// Fake fixed-width shaper for cursor impls (null impl; no shaping needed here).
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

struct RecordingPainter {
    events: Vec<PaintEvent>,
    null_caret: NullCaretCursor,
    null_lines: NullLineCursor,
}

impl RecordingPainter {
    const fn new() -> Self {
        Self {
            events: Vec::new(),
            null_caret: NullCaretCursor,
            null_lines: NullLineCursor,
        }
    }
}

impl Painter for RecordingPainter {
    fn draw_rect(&mut self, _rect: Rect, _pen: &Pen, _brush: &Brush) {
        self.events.push(PaintEvent::Other);
    }

    fn fill_rect(&mut self, _rect: Rect, _brush: &Brush) {
        self.events.push(PaintEvent::Other);
    }

    fn draw_line(&mut self, _from: Point, _to: Point, _pen: &Pen) {
        self.events.push(PaintEvent::Other);
    }

    fn clip_rect(&mut self, _rect: Rect) {
        self.events.push(PaintEvent::Other);
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

    fn draw_text(&mut self, _pos: Point, _text: &str, _font: &Font, _brush: &Brush) {
        self.events.push(PaintEvent::Other);
    }

    fn draw_text_in(
        &mut self,
        _rect: Rect,
        _text: &str,
        _font: &Font,
        _brush: &Brush,
        _alignment: Alignment,
    ) {
        self.events.push(PaintEvent::Other);
    }

    fn draw_image(&mut self, _rect: Rect, _image: &Image) {
        self.events.push(PaintEvent::Other);
    }

    fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {
        self.events.push(PaintEvent::Other);
    }

    fn text_carets(&mut self, _text: &str, _font: &Font) -> &mut dyn TextCaretCursor {
        &mut self.null_caret
    }

    fn text_visual_lines(
        &mut self,
        _text: &str,
        _font: &Font,
        _wrap_width: i32,
    ) -> &mut dyn TextVisualLineCursor {
        &mut self.null_lines
    }
}

/// Verify that `Drop` runs and calls `restore` even when the guarded scope panics.
#[test]
fn drop_records_restore_when_scope_panics() {
    let mut p = RecordingPainter::new();
    let origin = Point::new(5, 7);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = TranslateGuard::new(&mut p, origin);
        panic!("simulated panic inside guarded scope");
    }));

    assert!(result.is_err(), "catch_unwind should have caught the panic");
    assert_eq!(
        p.events.last(),
        Some(&PaintEvent::Restore),
        "Restore must be recorded even after a panic"
    );
}
