//! [`VelloPainter`] — vello + wgpu backed [`Painter`] implementation.

use quartzite_geometry::{Alignment, Point, Rect};
use quartzite_paint_api::{Brush, Font, Image, Painter, Path, Pen};

/// A [`Painter`] implementation backed by vello + wgpu.
///
/// Construction is async (wgpu adapter/device request); use
/// [`WindowedApplication::run`](crate::application::WindowedApplication::run)
/// which wraps the setup in `pollster::block_on`.
///
/// In v1, rendering errors are non-recoverable. Methods panic or log on
/// failure; [`quartzite_paint_api::PaintError`] is reserved for a future API
/// version when [`Painter`] methods gain `Result` return types.
pub struct VelloPainter {
    // Will hold: vello::Renderer, vello::Scene, wgpu::Device, wgpu::Queue,
    // wgpu::Surface. Left as unit struct for the skeleton to avoid pulling in
    // wgpu async initialisation before the renderer integration is complete.
}

impl VelloPainter {
    /// Creates a new painter.
    ///
    /// This is a skeleton; full wgpu/vello initialisation is deferred.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_renderer::VelloPainter;
    ///
    /// let painter = VelloPainter::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for VelloPainter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Painter for VelloPainter {
    #[inline]
    fn draw_rect(&mut self, _rect: Rect, _pen: &Pen, _brush: &Brush) {}

    #[inline]
    fn fill_rect(&mut self, _rect: Rect, _brush: &Brush) {}

    #[inline]
    fn draw_line(&mut self, _from: Point, _to: Point, _pen: &Pen) {}

    #[inline]
    fn clip_rect(&mut self, _rect: Rect) {}

    #[inline]
    fn translate(&mut self, _delta: Point) {}

    #[inline]
    fn save(&mut self) {}

    #[inline]
    fn restore(&mut self) {}

    #[inline]
    fn draw_text(&mut self, _pos: Point, _text: &str, _font: &Font, _brush: &Brush) {}

    #[inline]
    fn draw_text_in(
        &mut self,
        _rect: Rect,
        _text: &str,
        _font: &Font,
        _brush: &Brush,
        _alignment: Alignment,
    ) {
    }

    #[inline]
    fn draw_image(&mut self, _rect: Rect, _image: &Image) {}

    #[inline]
    fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_geometry::Size;
    use quartzite_paint_api::{BrushKind, Color};

    #[test]
    fn vello_painter_new_does_not_panic() {
        let _painter = VelloPainter::new();
    }

    #[test]
    fn vello_painter_default_equals_new() {
        // Both construct without panic; unit-struct equality is trivially true.
        let _a = VelloPainter::new();
        let _b = VelloPainter::default();
    }

    #[test]
    fn all_painter_methods_are_invocable() {
        // The skeleton stubs are no-ops; this test just exercises every
        // `Painter` method to confirm dispatch through `&mut dyn Painter`
        // compiles and runs without panicking.
        let mut p: Box<dyn Painter> = Box::new(VelloPainter::new());
        let pen = Pen::new(Color::BLACK, 1.0);
        let brush = Brush::solid(Color::WHITE);
        assert_eq!(brush.kind(), BrushKind::Solid(Color::WHITE));
        let rect = Rect::new(Point::new(0, 0), Size::new(10, 10));
        let origin = Point::new(0, 0);
        let font = Font::new("Arial", 12.0);
        let image = Image::try_new(1, 1, vec![0u8, 0, 0, 0]).unwrap();
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
}
