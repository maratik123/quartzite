use quartzite_geometry::{Alignment, Point, Rect};

use crate::{Brush, Font, Image, Path, Pen, TextCaretCursor, TextVisualLineCursor};

/// A 2D drawing surface.
///
/// All methods take `&mut self` and have no generic parameters, making the trait
/// object-safe: `Box<dyn Painter>` and `&mut dyn Painter` both compile.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{
///     Brush, Color, Font, Image, Painter, Path, Pen,
///     TextCaretCursor, TextVisualLine, TextVisualLineCursor,
/// };
/// use quartzite_geometry::{Alignment, Point, Rect, Size};
///
/// struct NullCaretCursor;
/// impl TextCaretCursor for NullCaretCursor {
///     fn advance_to(&mut self, _byte_offset: usize) {}
///     fn caret_x(&self) -> i32 { 0 }
///     fn line_top(&self) -> i32 { 0 }
///     fn line_height(&self) -> i32 { 12 }
/// }
///
/// struct NullLineCursor;
/// impl TextVisualLineCursor for NullLineCursor {
///     fn next_line(&mut self) -> Option<TextVisualLine> { None }
/// }
///
/// struct NullPainter {
///     caret_cursor: NullCaretCursor,
///     line_cursor: NullLineCursor,
/// }
///
/// impl NullPainter {
///     fn new() -> Self { Self { caret_cursor: NullCaretCursor, line_cursor: NullLineCursor } }
/// }
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
///         _h_align: Alignment,
///         _v_align: Alignment,
///     ) {}
///     fn draw_image(&mut self, _rect: Rect, _image: &Image) {}
///     fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {}
///     fn text_carets(&mut self, _text: &str, _font: &Font) -> &mut dyn TextCaretCursor {
///         &mut self.caret_cursor
///     }
///     fn text_visual_lines(
///         &mut self,
///         _text: &str,
///         _font: &Font,
///         _wrap_width: i32,
///     ) -> &mut dyn TextVisualLineCursor {
///         &mut self.line_cursor
///     }
/// }
///
/// let mut p: Box<dyn Painter> = Box::new(NullPainter::new());
/// p.save();
/// p.fill_rect(Rect::new(Point::new(0, 0), Size::new(100, 100)), &Brush::solid(Color::WHITE));
/// p.draw_path(&Path::new(), &Pen::default(), &Brush::solid(Color::BLACK));
/// p.restore();
/// ```
pub trait Painter {
    /// Draws the outline of `rect` using `pen` and fills it with `brush`.
    ///
    /// # Parameters
    ///
    /// - `rect`: the rectangle to outline.
    /// - `pen`: stroke style (color and width).
    /// - `brush`: fill style for the interior.
    fn draw_rect(&mut self, rect: Rect, pen: &Pen, brush: &Brush);

    /// Fills `rect` with `brush`, with no stroke.
    ///
    /// # Parameters
    ///
    /// - `rect`: the rectangle to fill.
    /// - `brush`: fill style.
    fn fill_rect(&mut self, rect: Rect, brush: &Brush);

    /// Draws a line from `from` to `to` using `pen`.
    ///
    /// # Parameters
    ///
    /// - `from`: start point of the line.
    /// - `to`: end point of the line.
    /// - `pen`: stroke style (color and width).
    fn draw_line(&mut self, from: Point, to: Point, pen: &Pen);

    /// Clips subsequent drawing to `rect`.
    ///
    /// # Parameters
    ///
    /// - `rect`: the clip region; drawing outside this rect is discarded.
    fn clip_rect(&mut self, rect: Rect);

    /// Translates the coordinate system by `delta`.
    ///
    /// # Parameters
    ///
    /// - `delta`: offset applied to all subsequent draw calls.
    fn translate(&mut self, delta: Point);

    /// Saves the current graphics state onto a stack.
    fn save(&mut self);

    /// Restores the most recently saved graphics state.
    fn restore(&mut self);

    /// Draws `text` at `pos` using `font` and `brush`.
    ///
    /// `pos` is the text baseline origin. The brush colours the glyph fills.
    ///
    /// # Parameters
    ///
    /// - `pos`: baseline origin of the first glyph.
    /// - `text`: the string to render.
    /// - `font`: typeface, size, and style flags.
    /// - `brush`: fill style for the glyphs.
    fn draw_text(&mut self, pos: Point, text: &str, font: &Font, brush: &Brush);

    /// Draws `text` aligned within `rect` using `font` and `brush`.
    ///
    /// # Parameters
    ///
    /// - `rect`: the bounding box for the laid-out text.
    /// - `text`: the string to render.
    /// - `font`: typeface, size, and style flags.
    /// - `brush`: fill style for the glyphs.
    /// - `h_align`: horizontal alignment within `rect`.
    /// - `v_align`: vertical alignment within `rect`. [`Alignment::Left`] means
    ///   top; [`Alignment::Center`] means centred; [`Alignment::Right`] means
    ///   bottom. [`Alignment::Justify`] is invalid on the vertical axis:
    ///   implementations debug-assert in debug builds and fall back to
    ///   [`Alignment::Left`] (top) in release builds.
    ///
    /// **Parameter order:** `h_align` always precedes `v_align`. Both are
    /// [`Alignment`] — call sites must rely on positional order; treat any
    /// `draw_text_in(..., v, h)` ordering as a defect.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quartzite_paint_api::{Brush, Color, Font, Painter};
    /// # use quartzite_geometry::{Alignment, Point, Rect, Size};
    /// # struct NullCaret;
    /// # impl quartzite_paint_api::TextCaretCursor for NullCaret {
    /// #     fn advance_to(&mut self, _: usize) {}
    /// #     fn caret_x(&self) -> i32 { 0 }
    /// #     fn line_top(&self) -> i32 { 0 }
    /// #     fn line_height(&self) -> i32 { 12 }
    /// # }
    /// # struct NullLines;
    /// # impl quartzite_paint_api::TextVisualLineCursor for NullLines {
    /// #     fn next_line(&mut self) -> Option<quartzite_paint_api::TextVisualLine> { None }
    /// # }
    /// # struct NullP { c: NullCaret, l: NullLines }
    /// # impl Painter for NullP {
    /// #     fn draw_rect(&mut self, _: Rect, _: &quartzite_paint_api::Pen, _: &Brush) {}
    /// #     fn fill_rect(&mut self, _: Rect, _: &Brush) {}
    /// #     fn draw_line(&mut self, _: Point, _: Point, _: &quartzite_paint_api::Pen) {}
    /// #     fn clip_rect(&mut self, _: Rect) {}
    /// #     fn translate(&mut self, _: Point) {}
    /// #     fn save(&mut self) {}
    /// #     fn restore(&mut self) {}
    /// #     fn draw_text(&mut self, _: Point, _: &str, _: &Font, _: &Brush) {}
    /// #     fn draw_text_in(&mut self, _: Rect, _: &str, _: &Font, _: &Brush, _: Alignment, _: Alignment) {}
    /// #     fn draw_image(&mut self, _: Rect, _: &quartzite_paint_api::Image) {}
    /// #     fn draw_path(&mut self, _: &quartzite_paint_api::Path, _: &quartzite_paint_api::Pen, _: &Brush) {}
    /// #     fn text_carets(&mut self, _: &str, _: &Font) -> &mut dyn quartzite_paint_api::TextCaretCursor { &mut self.c }
    /// #     fn text_visual_lines(&mut self, _: &str, _: &Font, _: i32) -> &mut dyn quartzite_paint_api::TextVisualLineCursor { &mut self.l }
    /// # }
    /// let mut p = NullP { c: NullCaret, l: NullLines };
    /// let rect = Rect::new(Point::new(0, 0), Size::new(200, 64));
    /// let font = Font::new("sans-serif", 16.0);
    /// let brush = Brush::solid(Color::BLACK);
    /// // Horizontally centred, vertically centred:
    /// p.draw_text_in(rect, "Hello", &font, &brush, Alignment::Center, Alignment::Center);
    /// // Left-aligned, top-anchored (e.g. TextEdit):
    /// p.draw_text_in(rect, "Hello", &font, &brush, Alignment::Left, Alignment::Left);
    /// ```
    fn draw_text_in(
        &mut self,
        rect: Rect,
        text: &str,
        font: &Font,
        brush: &Brush,
        h_align: Alignment,
        v_align: Alignment,
    );

    /// Draws `image` scaled into `rect`.
    ///
    /// # Parameters
    ///
    /// - `rect`: destination rectangle the image is scaled into.
    /// - `image`: the source RGBA8 pixel buffer.
    fn draw_image(&mut self, rect: Rect, image: &Image);

    /// Strokes `path` with `pen` and fills it with `brush`.
    ///
    /// # Parameters
    ///
    /// - `path`: the path to draw.
    /// - `pen`: stroke style (color and width).
    /// - `brush`: fill style for the interior.
    fn draw_path(&mut self, path: &Path, pen: &Pen, brush: &Brush);

    /// Returns a cursor for querying pixel-snapped caret positions within `text`
    /// shaped with `font`.
    ///
    /// The returned `&mut dyn TextCaretCursor` borrows `self` mutably, so no
    /// other painter calls may be made while the cursor is live.  Drop the
    /// cursor (let the binding go out of scope) before calling any other
    /// painter method.
    ///
    /// All coordinates returned through the cursor are **pixel-snapped `i32`**
    /// values.  Each implementor rounds its internal sub-pixel advances before
    /// returning.
    ///
    /// # Parameters
    ///
    /// - `text`: the UTF-8 string to shape.
    /// - `font`: typeface description; the implementor uses this to select the
    ///   font and determine the advance per cluster.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let cursor = painter.text_carets("hello", &Font::new("sans", 12.0));
    /// cursor.advance_to(0);
    /// let x = cursor.caret_x();
    /// ```
    fn text_carets(&mut self, text: &str, font: &Font) -> &mut dyn TextCaretCursor;

    /// Returns a cursor for iterating over the visual lines of `text` shaped
    /// with `font` and wrapped at `wrap_width` pixels.
    ///
    /// The returned `&mut dyn TextVisualLineCursor` borrows `self` mutably, so
    /// no other painter calls may be made while the cursor is live.  Drop the
    /// cursor before calling any other painter method.
    ///
    /// All coordinates inside each [`TextVisualLine`](crate::TextVisualLine) are
    /// **pixel-snapped `i32`** values.
    ///
    /// # Parameters
    ///
    /// - `text`: the UTF-8 string to shape.
    /// - `font`: typeface description.
    /// - `wrap_width`: maximum pixel width of a single visual line; the
    ///   implementor wraps the text at cluster boundaries so that no line
    ///   exceeds this width.  A value of `0` or negative means "no wrap".
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let cursor = painter.text_visual_lines("hello world", &Font::new("sans", 12.0), 64);
    /// while let Some(line) = cursor.next_line() {
    ///     println!("line top={} h={}", line.top, line.height);
    /// }
    /// ```
    fn text_visual_lines(
        &mut self,
        text: &str,
        font: &Font,
        wrap_width: i32,
    ) -> &mut dyn TextVisualLineCursor;
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    use quartzite_geometry::Size;

    use super::*;
    use crate::{Color, TextVisualLine};

    // ── Fake fixed-width shaper helpers ──────────────────────────────────────
    //
    // Contract:
    //   • One cluster per `char` (LF clusters are included as zero-advance).
    //   • Advance per visible cluster: 8 px.
    //   • `line_height` = `font.size_pt()` rounded to the nearest integer.
    //   • Wraps at `wrap_width / 8` visible chars per line (integer division).
    //     When `wrap_width <= 0` the whole text is one line.

    /// Pixel advance per character cluster in the fake shaper.
    const FAKE_ADVANCE: i32 = 8;

    /// Returns the line height for `font` using the fake shaper.
    ///
    /// _Simple._
    fn fake_line_height(font: &Font) -> i32 {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "test-only: font size is always a small representable value"
        )]
        let lh = font.size_pt().round() as i32;
        lh
    }

    /// Returns the maximum number of (non-newline) characters per line.
    ///
    /// _Simple._
    fn fake_chars_per_line(wrap_width: i32) -> usize {
        if wrap_width > 0 {
            // wrap_width > 0 guarantees the result is non-negative.
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

    /// A caret cursor backed by the fake fixed-width shaper.
    ///
    /// Stores the source text and pre-computed per-cluster positions so
    /// `advance_to` can resolve a byte offset to a char-cluster index in O(n).
    struct FakeCaretCursor {
        /// Original text, stored to resolve byte offsets → char positions.
        text: alloc::string::String,
        /// `(caret_x, line_top, line_height)` indexed by cluster position
        /// (position == number of chars before this cluster).
        positions: Vec<(i32, i32, i32)>,
        /// Current index into `positions`.
        idx: usize,
    }

    impl FakeCaretCursor {
        fn new(text: &str, font: &Font) -> Self {
            // We treat the whole text as a single unwrapped line.
            let lh = fake_line_height(font);
            let mut positions = Vec::new();
            let mut x = 0i32;
            for _ in text.chars() {
                positions.push((x, 0, lh));
                x += FAKE_ADVANCE;
            }
            // Trailing position (one past last cluster).
            positions.push((x, 0, lh));
            Self {
                text: text.into(),
                positions,
                idx: 0,
            }
        }

        /// Returns the caret position tuple at the current index.
        ///
        /// _Simple._
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

    /// A visual-line cursor backed by the fake fixed-width shaper.
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
            // Final (possibly partial) line.
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

    // ── RecordingPainter ─────────────────────────────────────────────────────

    struct RecordingPainter {
        calls: [u8; 13],
        /// Backing storage for `text_carets` — rebuilt on each call.
        caret_cursor: Option<FakeCaretCursor>,
        /// Backing storage for `text_visual_lines` — rebuilt on each call.
        line_cursor: Option<FakeLineCursor>,
    }

    impl RecordingPainter {
        fn new() -> Self {
            Self {
                calls: [0; 13],
                caret_cursor: None,
                line_cursor: None,
            }
        }
    }

    impl Painter for RecordingPainter {
        fn draw_rect(&mut self, _rect: Rect, _pen: &Pen, _brush: &Brush) {
            self.calls[0] += 1;
        }
        fn fill_rect(&mut self, _rect: Rect, _brush: &Brush) {
            self.calls[1] += 1;
        }
        fn draw_line(&mut self, _from: Point, _to: Point, _pen: &Pen) {
            self.calls[2] += 1;
        }
        fn clip_rect(&mut self, _rect: Rect) {
            self.calls[3] += 1;
        }
        fn translate(&mut self, _delta: Point) {
            self.calls[4] += 1;
        }
        fn save(&mut self) {
            self.calls[5] += 1;
        }
        fn restore(&mut self) {
            self.calls[6] += 1;
        }
        fn draw_text(&mut self, _pos: Point, _text: &str, _font: &Font, _brush: &Brush) {
            self.calls[7] += 1;
        }
        fn draw_text_in(
            &mut self,
            _rect: Rect,
            _text: &str,
            _font: &Font,
            _brush: &Brush,
            _h_align: Alignment,
            v_align: Alignment,
        ) {
            debug_assert!(
                !matches!(v_align, Alignment::Justify),
                "draw_text_in: Alignment::Justify is invalid on the vertical axis"
            );
            self.calls[8] += 1;
        }
        fn draw_image(&mut self, _rect: Rect, _image: &Image) {
            self.calls[9] += 1;
        }
        fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {
            self.calls[10] += 1;
        }
        fn text_carets(&mut self, text: &str, font: &Font) -> &mut dyn TextCaretCursor {
            self.calls[11] += 1;
            self.caret_cursor = Some(FakeCaretCursor::new(text, font));
            self.caret_cursor.as_mut().unwrap()
        }
        fn text_visual_lines(
            &mut self,
            text: &str,
            font: &Font,
            wrap_width: i32,
        ) -> &mut dyn TextVisualLineCursor {
            self.calls[12] += 1;
            let cursor = FakeLineCursor::new(text, font, wrap_width);
            self.line_cursor = Some(cursor);
            self.line_cursor.as_mut().unwrap()
        }
    }

    #[test]
    fn painter_is_object_safe() {
        let mut p: Box<dyn Painter> = Box::new(RecordingPainter::new());
        p.save();
        p.restore();
    }

    #[test]
    fn all_methods_reachable_through_trait_object() {
        let mut inner = RecordingPainter::new();
        {
            let p: &mut dyn Painter = &mut inner;
            let pen = Pen::new(Color::BLACK, 1.0);
            let brush = Brush::solid(Color::WHITE);
            let rect = Rect::new(Point::new(0, 0), Size::new(10, 10));
            let origin = Point::new(0, 0);
            let font = Font::new("Arial", 12.0);
            let image = Image::try_new(1, 1, alloc::vec![0u8, 0, 0, 0]).unwrap();
            let path = Path::new();
            p.draw_rect(rect, &pen, &brush);
            p.fill_rect(rect, &brush);
            p.draw_line(origin, origin, &pen);
            p.clip_rect(rect);
            p.translate(origin);
            p.save();
            p.restore();
            p.draw_text(origin, "hi", &font, &brush);
            p.draw_text_in(rect, "hi", &font, &brush, Alignment::Left, Alignment::Left);
            p.draw_image(rect, &image);
            p.draw_path(&path, &pen, &brush);
        }
        // First 11 methods each called once; text_carets and text_visual_lines not yet.
        assert_eq!(&inner.calls[..11], &[1u8; 11]);
    }

    #[test]
    fn boxed_painter_dispatches_all_new_methods() {
        let mut p: Box<dyn Painter> = Box::new(RecordingPainter::new());
        let pen = Pen::default();
        let brush = Brush::solid(Color::BLACK);
        let rect = Rect::new(Point::new(0, 0), Size::new(1, 1));
        let font = Font::new("sans", 10.0);
        let image = Image::try_new(0, 0, alloc::vec![]).unwrap();
        let path = Path::new();
        p.draw_text(Point::new(0, 0), "x", &font, &brush);
        p.draw_text_in(
            rect,
            "x",
            &font,
            &brush,
            Alignment::Center,
            Alignment::Center,
        );
        p.draw_image(rect, &image);
        p.draw_path(&path, &pen, &brush);
    }

    #[test]
    fn painter_text_carets_reachable_through_trait_object() {
        let mut inner = RecordingPainter::new();
        let font = Font::new("sans", 8.0);
        // "abc" — 3 chars, 8 px each → caret at byte 0 = x=0, byte 3 = x=24.
        {
            let p: &mut dyn Painter = &mut inner;
            let cursor = p.text_carets("abc", &font);
            cursor.advance_to(0);
            assert_eq!(cursor.caret_x(), 0);
            assert_eq!(cursor.line_top(), 0);
            assert_eq!(cursor.line_height(), 8);
        }
        assert_eq!(inner.calls[11], 1, "text_carets dispatch call counted");
    }

    #[test]
    fn painter_text_visual_lines_reachable_through_trait_object() {
        let mut inner = RecordingPainter::new();
        let font = Font::new("sans", 8.0);
        // "abcde" with wrap_width=24 → 3 chars per line (24/8) → 2 lines:
        //   line 0: bytes 0..3, top=0, h=8
        //   line 1: bytes 3..5, top=8, h=8
        {
            let p: &mut dyn Painter = &mut inner;
            let cursor = p.text_visual_lines("abcde", &font, 24);
            let l0 = cursor.next_line().expect("line 0");
            assert_eq!(l0.byte_start, 0);
            assert_eq!(l0.byte_end, 3);
            assert_eq!(l0.top, 0);
            assert_eq!(l0.height, 8);
            let l1 = cursor.next_line().expect("line 1");
            assert_eq!(l1.byte_start, 3);
            assert_eq!(l1.byte_end, 5);
            assert_eq!(l1.top, 8);
            assert_eq!(l1.height, 8);
            assert!(cursor.next_line().is_none());
        }
        assert_eq!(
            inner.calls[12], 1,
            "text_visual_lines dispatch call counted"
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "draw_text_in: Alignment::Justify is invalid on the vertical axis")]
    fn draw_text_in_justify_on_vertical_axis_panics_in_debug() {
        let mut p = RecordingPainter::new();
        let rect = Rect::new(Point::new(0, 0), Size::new(64, 64));
        let font = Font::new("sans", 12.0);
        let brush = Brush::solid(Color::BLACK);
        p.draw_text_in(
            rect,
            "x",
            &font,
            &brush,
            Alignment::Left,
            Alignment::Justify,
        );
    }
}
