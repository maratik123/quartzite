//! Text-measurement cursor traits and associated POD types.
//!
//! Two object-safe traits let a [`Painter`](crate::Painter) implementation expose
//! text-layout queries to the style layer without requiring any allocation on the
//! hot paint path:
//!
//! - [`TextCaretCursor`] — iterates or seeks to the pixel-snapped x-position of
//!   a logical byte offset inside a text string.
//! - [`TextVisualLineCursor`] — iterates over the visual lines produced by a
//!   particular wrap width, exposing per-line geometry as [`TextVisualLine`]s.
//!
//! Both traits are obtained through methods on [`Painter`] that return
//! `&mut dyn TraitName`, borrowing the painter for the lifetime of the cursor.
//! Because the borrow is tied to `&mut self`, the caller **must** finish using
//! the cursor before making any further calls on the painter (e.g. before calling
//! `fill_rect`).
//!
//! All pixel coordinates returned by these traits are **pixel-snapped** `i32`
//! values.  Each [`Painter`] implementation is responsible for rounding its
//! internal sub-pixel advances to integer pixel boundaries before returning.

/// Geometry of a single visual line as produced by a text-wrap pass.
///
/// All fields are pixel-snapped `i32` values; each [`Painter`] implementation
/// rounds its internal sub-pixel metrics to integer pixel boundaries before
/// storing them here.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::TextVisualLine;
///
/// let line = TextVisualLine { byte_start: 0, byte_end: 5, top: 0, height: 16 };
/// assert_eq!(line.byte_end - line.byte_start, 5);
/// assert_eq!(line.top + line.height, 16);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextVisualLine {
    /// Byte index of the first byte in this visual line (inclusive).
    pub byte_start: usize,
    /// Byte index one past the last byte in this visual line (exclusive).
    ///
    /// For lines that are not the last, this is the index of the first byte
    /// on the *next* visual line; for the final line it equals the length of
    /// the text slice.
    pub byte_end: usize,
    /// Pixel-snapped top coordinate of this line in text-block space.
    ///
    /// The coordinate origin (0, 0) is the top-left of the text block passed to
    /// [`Painter::text_visual_lines`](crate::Painter::text_visual_lines).
    pub top: i32,
    /// Pixel-snapped height of this line in pixels.
    ///
    /// The bottom edge of the line is `top + height`.
    pub height: i32,
}

/// Cursor that exposes pixel-snapped caret x-positions for a shaped text run.
///
/// Obtained from [`Painter::text_carets`](crate::Painter::text_carets).  The
/// cursor borrows the [`Painter`](crate::Painter) mutably for its lifetime, so
/// the caller must not make any additional painter calls while the cursor is live.
///
/// All coordinates returned by this trait are **pixel-snapped `i32`** values
/// (rounded from any internal sub-pixel advances by the implementor).
///
/// # Examples
///
/// ```ignore
/// // Sketch only — the concrete type is behind &mut dyn Painter.
/// let cursor = painter.text_carets("hello", &font);
/// cursor.advance_to(3); // byte offset 3
/// let x = cursor.caret_x();   // pixel x at byte 3
/// ```
pub trait TextCaretCursor {
    /// Advances the cursor to the cluster that contains `byte_offset`.
    ///
    /// After calling `advance_to`, [`caret_x`](Self::caret_x),
    /// [`line_top`](Self::line_top), and [`line_height`](Self::line_height)
    /// reflect the geometry at `byte_offset`.
    ///
    /// If `byte_offset` is beyond the end of the text, the cursor saturates
    /// at the final position (trailing edge of the last cluster).
    ///
    /// # Parameters
    ///
    /// - `byte_offset`: UTF-8 byte index into the text string that was passed
    ///   to [`Painter::text_carets`](crate::Painter::text_carets).  Need not
    ///   align to a cluster boundary; the implementor snaps to the nearest
    ///   cluster start.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// cursor.advance_to(5); // seek to byte 5
    /// ```
    fn advance_to(&mut self, byte_offset: usize);

    /// Returns the pixel-snapped x-coordinate of the caret at the current
    /// cursor position (as set by the last call to
    /// [`advance_to`](Self::advance_to)).
    ///
    /// The coordinate is measured from the left edge of the text block (i.e.
    /// the `pos` or `rect.left()` argument passed to the originating painter
    /// method is coordinate 0).
    ///
    /// All coordinates are **pixel-snapped `i32`** — implementors round their
    /// internal sub-pixel advances before returning.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// cursor.advance_to(0);
    /// assert_eq!(cursor.caret_x(), 0); // leading edge of first cluster
    /// ```
    fn caret_x(&self) -> i32;

    /// Returns the pixel-snapped top y-coordinate of the visual line that
    /// contains the current cursor position.
    ///
    /// The coordinate is measured from the top of the text block.
    ///
    /// All coordinates are **pixel-snapped `i32`**.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// cursor.advance_to(0);
    /// assert_eq!(cursor.line_top(), 0); // first line starts at top of block
    /// ```
    fn line_top(&self) -> i32;

    /// Returns the pixel-snapped height of the visual line that contains the
    /// current cursor position.
    ///
    /// All coordinates are **pixel-snapped `i32`**.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// cursor.advance_to(0);
    /// let height = cursor.line_height();
    /// assert!(height > 0);
    /// ```
    fn line_height(&self) -> i32;
}

/// Cursor that iterates over the visual lines of a wrapped text block.
///
/// Obtained from
/// [`Painter::text_visual_lines`](crate::Painter::text_visual_lines).  The
/// cursor borrows the [`Painter`](crate::Painter) mutably for its lifetime, so
/// the caller must not make any additional painter calls while the cursor is live.
///
/// The cursor is consumed one line at a time by calling
/// [`next_line`](Self::next_line) in a loop until it returns `None`.
///
/// All pixel coordinates inside [`TextVisualLine`] are **pixel-snapped `i32`**
/// values (rounded from any internal sub-pixel advances by the implementor).
///
/// # Examples
///
/// ```ignore
/// // Sketch only — the concrete type is behind &mut dyn Painter.
/// let cursor = painter.text_visual_lines("hello world", &font, 64);
/// while let Some(line) = cursor.next_line() {
///     println!("line bytes {:?} top={} h={}", line.byte_start..line.byte_end, line.top, line.height);
/// }
/// ```
pub trait TextVisualLineCursor {
    /// Returns the next visual line, or `None` when all lines have been yielded.
    ///
    /// Lines are returned in top-to-bottom visual order.  Each call advances
    /// the cursor by one line; after the final line the cursor always returns
    /// `None` for subsequent calls.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Count the number of visual lines.
    /// let cursor = painter.text_visual_lines("hello\nworld", &font, 200);
    /// let mut count = 0;
    /// while cursor.next_line().is_some() { count += 1; }
    /// assert_eq!(count, 2);
    /// ```
    fn next_line(&mut self) -> Option<TextVisualLine>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal concrete impl used to prove object-safety at compile time.
    struct FakeCaretCursor {
        x: i32,
        top: i32,
        height: i32,
    }

    impl TextCaretCursor for FakeCaretCursor {
        fn advance_to(&mut self, _byte_offset: usize) {}
        fn caret_x(&self) -> i32 {
            self.x
        }
        fn line_top(&self) -> i32 {
            self.top
        }
        fn line_height(&self) -> i32 {
            self.height
        }
    }

    /// Minimal concrete impl used to prove object-safety at compile time.
    struct FakeLineCursor {
        lines: alloc::vec::Vec<TextVisualLine>,
        idx: usize,
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

    #[test]
    fn text_caret_cursor_is_object_safe() {
        let mut c = FakeCaretCursor {
            x: 0,
            top: 0,
            height: 16,
        };
        let dyn_c: &mut dyn TextCaretCursor = &mut c;
        dyn_c.advance_to(0);
        assert_eq!(dyn_c.caret_x(), 0);
        assert_eq!(dyn_c.line_top(), 0);
        assert_eq!(dyn_c.line_height(), 16);
    }

    #[test]
    fn text_visual_line_cursor_is_object_safe() {
        let mut c = FakeLineCursor {
            lines: alloc::vec![TextVisualLine {
                byte_start: 0,
                byte_end: 5,
                top: 0,
                height: 16,
            }],
            idx: 0,
        };
        let dyn_c: &mut dyn TextVisualLineCursor = &mut c;
        let line = dyn_c.next_line().expect("should yield one line");
        assert_eq!(line.byte_start, 0);
        assert_eq!(line.byte_end, 5);
        assert_eq!(line.top, 0);
        assert_eq!(line.height, 16);
        assert!(dyn_c.next_line().is_none());
    }

    #[test]
    fn text_visual_line_fields_round_trip() {
        let line = TextVisualLine {
            byte_start: 2,
            byte_end: 7,
            top: 32,
            height: 20,
        };
        assert_eq!(line.byte_start, 2);
        assert_eq!(line.byte_end, 7);
        assert_eq!(line.top, 32);
        assert_eq!(line.height, 20);
    }
}
