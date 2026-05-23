#![no_std]
//! RAII painting utilities for quartzite.
//!
//! This crate is `no_std`-compatible and provides thin wrappers around
//! [`quartzite_paint_api::Painter`] primitives. It depends only on
//! `quartzite-paint-api` and `quartzite-geometry`.

extern crate alloc;

use quartzite_geometry::Point;
use quartzite_paint_api::Painter;

/// RAII guard that saves the painter state, translates the origin, and
/// restores it on drop.
///
/// Constructing a `TranslateGuard` calls [`Painter::save`] then
/// [`Painter::translate`] with the given `origin` in that order.  When the
/// guard is dropped, [`Painter::restore`] is called exactly once — even if the
/// guarded body panics.
///
/// ## Accessor shape
///
/// The wrapped painter is exposed via an explicit [`painter`](TranslateGuard::painter)
/// accessor returning `&mut dyn Painter`, rather than implementing
/// `DerefMut<Target = dyn Painter>`.  An explicit accessor preserves the
/// existing call shape (`visit(…, guard.painter(), …)`) with no autoderef
/// surprise, sets no new `DerefMut` precedent in the workspace, and keeps
/// object-safety of [`Painter`] explicit and unambiguous.
///
/// ## Lifetime relationship
///
/// The guard borrows `&'a mut dyn Painter` for its entire lifetime `'a`.
/// [`painter`](TranslateGuard::painter) re-exposes that borrow as
/// `&mut dyn Painter` whose lifetime is tied to `&mut self`, so the caller
/// cannot alias the painter while the guard is live.  When the guard is
/// dropped at the end of `'a`, [`Painter::restore`] unwinds the saved state.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{Alignment, Point, Rect, Size};
/// use quartzite_paint_api::{
///     Brush, Color, Font, Image, Painter, Path, Pen,
///     TextCaretCursor, TextVisualLine, TextVisualLineCursor,
/// };
/// use quartzite_paint_util::TranslateGuard;
///
/// struct NullCaret;
/// impl TextCaretCursor for NullCaret {
///     fn advance_to(&mut self, _: usize) {}
///     fn caret_x(&self) -> i32 { 0 }
///     fn line_top(&self) -> i32 { 0 }
///     fn line_height(&self) -> i32 { 0 }
/// }
/// struct NullLines;
/// impl TextVisualLineCursor for NullLines {
///     fn next_line(&mut self) -> Option<TextVisualLine> { None }
/// }
///
/// struct NullPainter { caret: NullCaret, lines: NullLines }
/// impl NullPainter { fn new() -> Self { Self { caret: NullCaret, lines: NullLines } } }
///
/// impl Painter for NullPainter {
///     fn draw_rect(&mut self, _rect: Rect, _pen: &Pen, _brush: &Brush) {}
///     fn fill_rect(&mut self, _rect: Rect, _brush: &Brush) {}
///     fn draw_line(&mut self, _from: Point, _to: Point, _pen: &Pen) {}
///     fn clip_rect(&mut self, _rect: Rect) {}
///     fn translate(&mut self, _delta: Point) {}
///     fn save(&mut self) {}
///     fn restore(&mut self) {}
///     fn draw_text(&mut self, _pos: Point, _text: &str, _font: &Font, _brush: &Brush) {}
///     fn draw_text_in(
///         &mut self,
///         _rect: Rect,
///         _text: &str,
///         _font: &Font,
///         _brush: &Brush,
///         _alignment: Alignment,
///     ) {}
///     fn draw_image(&mut self, _rect: Rect, _image: &Image) {}
///     fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {}
///     fn text_carets(&mut self, _t: &str, _f: &Font) -> &mut dyn TextCaretCursor { &mut self.caret }
///     fn text_visual_lines(&mut self, _t: &str, _f: &Font, _w: i32) -> &mut dyn TextVisualLineCursor { &mut self.lines }
/// }
///
/// let mut painter = NullPainter::new();
/// let origin = Point::new(10, 20);
/// {
///     // save() + translate(origin) called here
///     let mut guard = TranslateGuard::new(&mut painter, origin);
///     // Obtain the underlying painter to draw in the translated frame
///     let p: &mut dyn Painter = guard.painter();
///     // Subsequent draw calls (fill_rect, draw_text, recursive visit, …)
///     // use the translated coordinate system.
///     let _ = p;
///     // restore() called here when guard drops at end of scope
/// }
/// ```
pub struct TranslateGuard<'a> {
    painter: &'a mut dyn Painter,
}

impl<'a> TranslateGuard<'a> {
    /// Creates a new guard: calls `save()` then `translate(origin)` on `painter`.
    ///
    /// # Parameters
    ///
    /// - `painter`: the painter to borrow for the lifetime of this guard.
    /// - `origin`: the translation delta passed to [`Painter::translate`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Alignment, Point, Rect, Size};
    /// use quartzite_paint_api::{Brush, Color, Font, Image, Painter, Path, Pen,
    ///     TextCaretCursor, TextVisualLine, TextVisualLineCursor};
    /// use quartzite_paint_util::TranslateGuard;
    ///
    /// struct NullCaret;
    /// impl TextCaretCursor for NullCaret {
    ///     fn advance_to(&mut self, _: usize) {}
    ///     fn caret_x(&self) -> i32 { 0 }
    ///     fn line_top(&self) -> i32 { 0 }
    ///     fn line_height(&self) -> i32 { 0 }
    /// }
    /// struct NullLines;
    /// impl TextVisualLineCursor for NullLines {
    ///     fn next_line(&mut self) -> Option<TextVisualLine> { None }
    /// }
    ///
    /// struct NullPainter { caret: NullCaret, lines: NullLines }
    /// impl NullPainter { fn new() -> Self { Self { caret: NullCaret, lines: NullLines } } }
    /// impl Painter for NullPainter {
    ///     fn draw_rect(&mut self, _r: Rect, _p: &Pen, _b: &Brush) {}
    ///     fn fill_rect(&mut self, _r: Rect, _b: &Brush) {}
    ///     fn draw_line(&mut self, _a: Point, _b: Point, _p: &Pen) {}
    ///     fn clip_rect(&mut self, _r: Rect) {}
    ///     fn translate(&mut self, _d: Point) {}
    ///     fn save(&mut self) {}
    ///     fn restore(&mut self) {}
    ///     fn draw_text(&mut self, _pos: Point, _t: &str, _f: &Font, _b: &Brush) {}
    ///     fn draw_text_in(&mut self, _r: Rect, _t: &str, _f: &Font, _b: &Brush, _a: Alignment) {}
    ///     fn draw_image(&mut self, _r: Rect, _i: &Image) {}
    ///     fn draw_path(&mut self, _p: &Path, _pe: &Pen, _b: &Brush) {}
    ///     fn text_carets(&mut self, _t: &str, _f: &Font) -> &mut dyn TextCaretCursor { &mut self.caret }
    ///     fn text_visual_lines(&mut self, _t: &str, _f: &Font, _w: i32) -> &mut dyn TextVisualLineCursor { &mut self.lines }
    /// }
    ///
    /// let mut painter = NullPainter::new();
    /// let _guard = TranslateGuard::new(&mut painter, Point::new(5, 10));
    /// // save() and translate(Point::new(5, 10)) have been called on painter
    /// // restore() will be called when _guard drops
    /// ```
    #[inline]
    pub fn new(painter: &'a mut dyn Painter, origin: Point) -> Self {
        painter.save();
        painter.translate(origin);
        Self { painter }
    }

    /// Returns a mutable reference to the wrapped painter.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Alignment, Point, Rect, Size};
    /// use quartzite_paint_api::{Brush, Color, Font, Image, Painter, Path, Pen,
    ///     TextCaretCursor, TextVisualLine, TextVisualLineCursor};
    /// use quartzite_paint_util::TranslateGuard;
    ///
    /// struct NullCaret;
    /// impl TextCaretCursor for NullCaret {
    ///     fn advance_to(&mut self, _: usize) {}
    ///     fn caret_x(&self) -> i32 { 0 }
    ///     fn line_top(&self) -> i32 { 0 }
    ///     fn line_height(&self) -> i32 { 0 }
    /// }
    /// struct NullLines;
    /// impl TextVisualLineCursor for NullLines {
    ///     fn next_line(&mut self) -> Option<TextVisualLine> { None }
    /// }
    ///
    /// struct NullPainter { caret: NullCaret, lines: NullLines }
    /// impl NullPainter { fn new() -> Self { Self { caret: NullCaret, lines: NullLines } } }
    /// impl Painter for NullPainter {
    ///     fn draw_rect(&mut self, _r: Rect, _p: &Pen, _b: &Brush) {}
    ///     fn fill_rect(&mut self, _r: Rect, _b: &Brush) {}
    ///     fn draw_line(&mut self, _a: Point, _b: Point, _p: &Pen) {}
    ///     fn clip_rect(&mut self, _r: Rect) {}
    ///     fn translate(&mut self, _d: Point) {}
    ///     fn save(&mut self) {}
    ///     fn restore(&mut self) {}
    ///     fn draw_text(&mut self, _pos: Point, _t: &str, _f: &Font, _b: &Brush) {}
    ///     fn draw_text_in(&mut self, _r: Rect, _t: &str, _f: &Font, _b: &Brush, _a: Alignment) {}
    ///     fn draw_image(&mut self, _r: Rect, _i: &Image) {}
    ///     fn draw_path(&mut self, _p: &Path, _pe: &Pen, _b: &Brush) {}
    ///     fn text_carets(&mut self, _t: &str, _f: &Font) -> &mut dyn TextCaretCursor { &mut self.caret }
    ///     fn text_visual_lines(&mut self, _t: &str, _f: &Font, _w: i32) -> &mut dyn TextVisualLineCursor { &mut self.lines }
    /// }
    ///
    /// let mut painter = NullPainter::new();
    /// let mut guard = TranslateGuard::new(&mut painter, Point::new(0, 0));
    /// let p: &mut dyn Painter = guard.painter();
    /// p.fill_rect(Rect::new(Point::new(0, 0), Size::new(10, 10)), &Brush::solid(Color::WHITE));
    /// ```
    #[inline]
    pub fn painter(&mut self) -> &mut dyn Painter {
        self.painter
    }
}

impl Drop for TranslateGuard<'_> {
    /// Calls `restore()` on the wrapped painter.
    #[inline]
    fn drop(&mut self) {
        self.painter.restore();
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use quartzite_geometry::{Point, Rect, Size};
    use quartzite_paint_api::{
        Brush, Color, Font, Painter, TextCaretCursor, TextVisualLine, TextVisualLineCursor,
    };

    use crate::TranslateGuard;

    // Local minimal RecordingPainter stub — re-using the one in
    // quartzite-paint-api via a dev-dependency cycle is not viable.
    #[derive(Debug, PartialEq)]
    enum PaintEvent {
        Save,
        Restore,
        Translate(Point),
        FillRect,
        Other,
    }

    // Fake fixed-width shaper for cursor impls (inline per-impl per design decision §3).
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
        fn new() -> Self {
            Self {
                events: Vec::new(),
                null_caret: NullCaretCursor,
                null_lines: NullLineCursor,
            }
        }
    }

    impl Painter for RecordingPainter {
        fn draw_rect(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _pen: &quartzite_paint_api::Pen,
            _brush: &quartzite_paint_api::Brush,
        ) {
            self.events.push(PaintEvent::Other);
        }

        fn fill_rect(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _brush: &quartzite_paint_api::Brush,
        ) {
            self.events.push(PaintEvent::FillRect);
        }

        fn draw_line(&mut self, _from: Point, _to: Point, _pen: &quartzite_paint_api::Pen) {
            self.events.push(PaintEvent::Other);
        }

        fn clip_rect(&mut self, _rect: quartzite_geometry::Rect) {
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

        fn draw_text(
            &mut self,
            _pos: Point,
            _text: &str,
            _font: &quartzite_paint_api::Font,
            _brush: &quartzite_paint_api::Brush,
        ) {
            self.events.push(PaintEvent::Other);
        }

        fn draw_text_in(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _text: &str,
            _font: &quartzite_paint_api::Font,
            _brush: &quartzite_paint_api::Brush,
            _alignment: quartzite_geometry::Alignment,
        ) {
            self.events.push(PaintEvent::Other);
        }

        fn draw_image(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _image: &quartzite_paint_api::Image,
        ) {
            self.events.push(PaintEvent::Other);
        }

        fn draw_path(
            &mut self,
            _path: &quartzite_paint_api::Path,
            _pen: &quartzite_paint_api::Pen,
            _brush: &quartzite_paint_api::Brush,
        ) {
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

    #[test]
    fn constructor_records_save_then_translate() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(3, 4);
        {
            let _guard = TranslateGuard::new(&mut p, origin);
            // guard borrows p — check after drop
        }
        // The first two events (recorded by the constructor) are Save then Translate(origin)
        assert_eq!(
            &p.events[..2],
            [PaintEvent::Save, PaintEvent::Translate(origin)]
        );
    }

    #[test]
    fn drop_records_exactly_one_restore() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(1, 2);
        {
            let _guard = TranslateGuard::new(&mut p, origin);
        }
        // Exactly one Restore at the end
        assert_eq!(
            p.events,
            [
                PaintEvent::Save,
                PaintEvent::Translate(origin),
                PaintEvent::Restore,
            ]
        );
        // Only one Restore
        let restore_count = p
            .events
            .iter()
            .filter(|e| **e == PaintEvent::Restore)
            .count();
        assert_eq!(restore_count, 1);
    }

    #[test]
    fn full_lifecycle_records_save_translate_restore_in_order() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(10, 20);
        {
            let _guard = TranslateGuard::new(&mut p, origin);
        }
        assert_eq!(
            p.events,
            [
                PaintEvent::Save,
                PaintEvent::Translate(origin),
                PaintEvent::Restore,
            ]
        );
    }

    #[test]
    fn painter_accessor_returns_same_painter() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(5, 6);
        {
            let mut guard = TranslateGuard::new(&mut p, origin);
            // Access the painter through the guard and call fill_rect
            guard.painter().fill_rect(
                Rect::new(Point::new(0, 0), Size::new(10, 10)),
                &Brush::solid(Color::WHITE),
            );
        }
        assert_eq!(
            p.events,
            [
                PaintEvent::Save,
                PaintEvent::Translate(origin),
                PaintEvent::FillRect,
                PaintEvent::Restore,
            ]
        );
    }

    #[test]
    fn translate_origin_zero() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(0, 0);
        {
            let _guard = TranslateGuard::new(&mut p, origin);
        }
        // Zero origin must still record Translate (no zero-skipping)
        assert_eq!(
            p.events,
            [
                PaintEvent::Save,
                PaintEvent::Translate(Point::new(0, 0)),
                PaintEvent::Restore,
            ]
        );
    }
}
