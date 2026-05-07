use crate::Color;

/// Stroke parameters: color and line width.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{Color, Pen};
///
/// let pen = Pen::new(Color::BLACK, 1.0);
/// assert_eq!(pen.color(), Color::BLACK);
/// assert_eq!(pen.width(), 1.0);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pen {
    color: Color,
    width: f32,
}

impl Pen {
    /// Creates a new pen with the given color and line width.
    ///
    /// # Parameters
    ///
    /// - `color`: stroke color.
    /// - `width`: stroke width in pixels; must be positive for a visible stroke.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Color, Pen};
    ///
    /// let pen = Pen::new(Color::RED, 2.0);
    /// assert_eq!(pen.width(), 2.0);
    /// ```
    #[inline]
    pub const fn new(color: Color, width: f32) -> Self {
        Self { color, width }
    }

    /// Returns the stroke color.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Color, Pen};
    /// assert_eq!(Pen::new(Color::BLUE, 1.0).color(), Color::BLUE);
    /// ```
    // _Simple._
    #[inline]
    pub const fn color(self) -> Color {
        self.color
    }

    /// Returns the line width.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Color, Pen};
    /// assert_eq!(Pen::new(Color::BLACK, 3.5).width(), 3.5);
    /// ```
    // _Simple._
    #[inline]
    pub const fn width(self) -> f32 {
        self.width
    }
}

impl Default for Pen {
    /// Returns a 1-pixel opaque black pen.
    // _Simple._
    #[inline]
    fn default() -> Self {
        Self::new(Color::BLACK, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_color_and_width() {
        let pen = Pen::new(Color::RED, 2.5);
        assert_eq!(pen.color(), Color::RED);
        assert_eq!(pen.width(), 2.5);
    }

    #[test]
    fn default_is_black_one_pixel() {
        let pen = Pen::default();
        assert_eq!(pen.color(), Color::BLACK);
        assert_eq!(pen.width(), 1.0);
    }

    #[test]
    fn pen_is_copy() {
        let pen = Pen::new(Color::GREEN, 1.0);
        let _p2 = pen;
        let _p3 = pen;
    }
}
