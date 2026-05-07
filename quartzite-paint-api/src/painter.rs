use quartzite_geometry::{Point, Rect};

use crate::{Brush, Pen};

/// A 2D drawing surface.
///
/// All methods take `&mut self` and have no generic parameters, making the trait
/// object-safe: `Box<dyn Painter>` and `&mut dyn Painter` both compile.
///
/// `draw_image` and `draw_text_in` are deferred until image and font types are
/// defined; they will be added in a later plan without breaking this trait
/// (new methods with default implementations or behind a feature gate).
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{Brush, Color, Painter, Pen};
/// use quartzite_geometry::{Point, Rect, Size};
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
/// }
///
/// let mut p: Box<dyn Painter> = Box::new(NullPainter);
/// p.save();
/// p.fill_rect(Rect::new(Point::new(0, 0), Size::new(100, 100)), &Brush::solid(Color::WHITE));
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
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use quartzite_geometry::Size;

    use super::*;
    use crate::Color;

    struct RecordingPainter {
        calls: [u8; 7],
    }

    impl RecordingPainter {
        fn new() -> Self {
            Self { calls: [0; 7] }
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
            p.draw_rect(rect, &pen, &brush);
            p.fill_rect(rect, &brush);
            p.draw_line(origin, origin, &pen);
            p.clip_rect(rect);
            p.translate(origin);
            p.save();
            p.restore();
        }
        assert_eq!(inner.calls, [1, 1, 1, 1, 1, 1, 1]);
    }
}
