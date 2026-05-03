use crate::Rect;

/// Margins applied to a [`Rect`], shrinking or expanding each edge independently.
///
/// Positive values shrink the rect; negative values expand it.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{Margins, Point, Rect, Size};
///
/// let r = Rect::new(Point::new(10, 10), Size::new(100, 100));
/// let inset = Margins::new(5, 5, 5, 5).apply(r);
/// assert_eq!(inset.origin(), Point::new(15, 15));
/// assert_eq!(inset.size(), Size::new(90, 90));
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Margins {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Margins {
    /// Creates margins with the given values for each edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Margins;
    ///
    /// let m = Margins::new(1, 2, 3, 4);
    /// assert_eq!(m.left(), 1);
    /// assert_eq!(m.bottom(), 4);
    /// ```
    #[inline]
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Returns the left margin.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Margins;
    ///
    /// assert_eq!(Margins::new(5, 0, 0, 0).left(), 5);
    /// ```
    #[inline]
    pub const fn left(self) -> i32 {
        self.left
    }

    /// Returns the top margin.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Margins;
    ///
    /// assert_eq!(Margins::new(0, 5, 0, 0).top(), 5);
    /// ```
    #[inline]
    pub const fn top(self) -> i32 {
        self.top
    }

    /// Returns the right margin.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Margins;
    ///
    /// assert_eq!(Margins::new(0, 0, 5, 0).right(), 5);
    /// ```
    #[inline]
    pub const fn right(self) -> i32 {
        self.right
    }

    /// Returns the bottom margin.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Margins;
    ///
    /// assert_eq!(Margins::new(0, 0, 0, 5).bottom(), 5);
    /// ```
    #[inline]
    pub const fn bottom(self) -> i32 {
        self.bottom
    }

    /// Applies these margins to `rect`, shrinking each edge inward.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Margins, Point, Rect, Size};
    ///
    /// let r = Rect::new(Point::new(0, 0), Size::new(100, 100));
    /// let shrunk = Margins::new(5, 5, 5, 5).apply(r);
    /// assert_eq!(shrunk.origin(), Point::new(5, 5));
    /// assert_eq!(shrunk.size(), Size::new(90, 90));
    /// ```
    #[inline]
    pub fn apply(self, rect: Rect) -> Rect {
        rect.adjusted(self.left, self.top, -self.right, -self.bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point, Size};

    #[test]
    fn margins_apply_shrinks_all_edges() {
        let r = Rect::new(Point::new(10, 10), Size::new(100, 100));
        let inset = Margins::new(5, 5, 5, 5).apply(r);
        assert_eq!(inset.origin(), Point::new(15, 15));
        assert_eq!(inset.size(), Size::new(90, 90));
    }

    #[test]
    fn margins_apply_asymmetric() {
        let r = Rect::new(Point::new(0, 0), Size::new(100, 100));
        let inset = Margins::new(10, 5, 20, 15).apply(r);
        assert_eq!(inset.origin(), Point::new(10, 5));
        assert_eq!(inset.size(), Size::new(70, 80));
    }

    #[test]
    fn margins_apply_negative_expands() {
        let r = Rect::new(Point::new(10, 10), Size::new(50, 50));
        let expanded = Margins::new(-5, -5, -5, -5).apply(r);
        assert_eq!(expanded.origin(), Point::new(5, 5));
        assert_eq!(expanded.size(), Size::new(60, 60));
    }

    #[test]
    fn margins_accessors() {
        let m = Margins::new(1, 2, 3, 4);
        assert_eq!(m.left(), 1);
        assert_eq!(m.top(), 2);
        assert_eq!(m.right(), 3);
        assert_eq!(m.bottom(), 4);
    }
}
