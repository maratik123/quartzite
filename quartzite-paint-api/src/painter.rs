use quartzite_geometry::{Alignment, Point, Rect};

use crate::{Brush, Font, Image, Path, Pen};

/// A 2D drawing surface.
///
/// All methods take `&mut self` and have no generic parameters, making the trait
/// object-safe: `Box<dyn Painter>` and `&mut dyn Painter` both compile.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{Brush, Color, Font, Image, Painter, Path, Pen};
/// use quartzite_geometry::{Alignment, Point, Rect, Size};
///
/// struct NullPainter;
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
/// }
///
/// let mut p: Box<dyn Painter> = Box::new(NullPainter);
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
    /// - `alignment`: horizontal alignment within `rect`.
    fn draw_text_in(
        &mut self,
        rect: Rect,
        text: &str,
        font: &Font,
        brush: &Brush,
        alignment: Alignment,
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
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use quartzite_geometry::Size;

    use super::*;
    use crate::Color;

    struct RecordingPainter {
        calls: [u8; 11],
    }

    impl RecordingPainter {
        fn new() -> Self {
            Self { calls: [0; 11] }
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
            _alignment: Alignment,
        ) {
            self.calls[8] += 1;
        }
        fn draw_image(&mut self, _rect: Rect, _image: &Image) {
            self.calls[9] += 1;
        }
        fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {
            self.calls[10] += 1;
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
            p.draw_text_in(rect, "hi", &font, &brush, Alignment::Left);
            p.draw_image(rect, &image);
            p.draw_path(&path, &pen, &brush);
        }
        assert_eq!(inner.calls, [1; 11]);
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
        p.draw_text_in(rect, "x", &font, &brush, Alignment::Center);
        p.draw_image(rect, &image);
        p.draw_path(&path, &pen, &brush);
    }
}
