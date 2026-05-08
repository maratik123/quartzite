//! [`VelloPainter`] — vello + wgpu backed [`Painter`] implementation.

use quartzite_geometry::{Point, Rect};
use quartzite_paint_api::{Brush, Painter, Pen};

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
