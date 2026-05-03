use crate::{Point, PointF, Size, SizeF};

/// An axis-aligned rectangle with integer coordinates.
///
/// Defined by an `origin` ([`Point`]) and a `size` ([`Size`]).
/// A rect with a zero-area size is considered empty.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{Point, Rect, Size};
///
/// let r = Rect::new(Point::new(10, 10), Size::new(100, 50));
/// assert!(r.contains(Point::new(50, 30)));
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    origin: Point,
    size: Size,
}

impl Rect {
    /// Creates a new rect from an origin point and a size.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// let r = Rect::new(Point::new(0, 0), Size::new(10, 10));
    /// assert_eq!(r.origin(), Point::new(0, 0));
    /// ```
    #[inline]
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// Returns the origin (top-left corner).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// assert_eq!(Rect::new(Point::new(5, 6), Size::new(1, 1)).origin(), Point::new(5, 6));
    /// ```
    #[inline]
    pub const fn origin(self) -> Point {
        self.origin
    }

    /// Returns the size.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// assert_eq!(Rect::new(Point::new(0, 0), Size::new(10, 20)).size(), Size::new(10, 20));
    /// ```
    #[inline]
    pub const fn size(self) -> Size {
        self.size
    }

    /// Returns the x coordinate of the left edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// assert_eq!(Rect::new(Point::new(3, 0), Size::new(10, 10)).left(), 3);
    /// ```
    #[inline]
    pub const fn left(self) -> i32 {
        self.origin.x()
    }

    /// Returns the y coordinate of the top edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// assert_eq!(Rect::new(Point::new(0, 5), Size::new(10, 10)).top(), 5);
    /// ```
    #[inline]
    pub const fn top(self) -> i32 {
        self.origin.y()
    }

    /// Returns the x coordinate of the right edge (exclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// assert_eq!(Rect::new(Point::new(3, 0), Size::new(10, 10)).right(), 13);
    /// ```
    #[inline]
    pub const fn right(self) -> i32 {
        self.origin.x() + self.size.width()
    }

    /// Returns the y coordinate of the bottom edge (exclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// assert_eq!(Rect::new(Point::new(0, 5), Size::new(10, 10)).bottom(), 15);
    /// ```
    #[inline]
    pub const fn bottom(self) -> i32 {
        self.origin.y() + self.size.height()
    }

    /// Returns `true` if the size has a zero-area dimension.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// assert!(Rect::new(Point::new(0, 0), Size::new(0, 10)).is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.size.is_empty()
    }

    /// Returns `true` if `point` lies inside this rect (inclusive left/top, exclusive right/bottom).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// let r = Rect::new(Point::new(10, 10), Size::new(100, 50));
    /// assert!(r.contains(Point::new(50, 30)));
    /// assert!(!r.contains(Point::new(9, 10)));
    /// ```
    #[inline]
    pub const fn contains(self, point: Point) -> bool {
        point.x() >= self.left()
            && point.x() < self.right()
            && point.y() >= self.top()
            && point.y() < self.bottom()
    }

    /// Returns `true` if this rect overlaps with `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// let a = Rect::new(Point::new(0, 0), Size::new(10, 10));
    /// let b = Rect::new(Point::new(20, 20), Size::new(10, 10));
    /// assert!(!a.intersects(b));
    /// ```
    #[inline]
    pub const fn intersects(self, other: Rect) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// Returns the smallest rect that contains both `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// let a = Rect::new(Point::new(0, 0), Size::new(5, 5));
    /// let b = Rect::new(Point::new(3, 3), Size::new(5, 5));
    /// let u = a.united(b);
    /// assert_eq!(u.origin(), Point::new(0, 0));
    /// assert_eq!(u.size(), Size::new(8, 8));
    /// ```
    pub fn united(self, other: Rect) -> Rect {
        let left = self.left().min(other.left());
        let top = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(Point::new(left, top), Size::new(right - left, bottom - top))
    }

    /// Returns this rect translated by `offset`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// let r = Rect::new(Point::new(5, 5), Size::new(10, 10));
    /// assert_eq!(r.translated(Point::new(3, -2)).origin(), Point::new(8, 3));
    /// ```
    #[inline]
    pub fn translated(self, offset: Point) -> Rect {
        Rect::new(self.origin + offset, self.size)
    }

    /// Returns a rect expanded by `dx` on each horizontal side and `dy` on each vertical side.
    ///
    /// Negative values shrink the rect.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// let r = Rect::new(Point::new(10, 10), Size::new(20, 20));
    /// let adjusted = r.adjusted(-2, -2, 2, 2);
    /// assert_eq!(adjusted.origin(), Point::new(8, 8));
    /// assert_eq!(adjusted.size(), Size::new(24, 24));
    /// ```
    pub fn adjusted(self, dx1: i32, dy1: i32, dx2: i32, dy2: i32) -> Rect {
        let left = self.left() + dx1;
        let top = self.top() + dy1;
        let right = self.right() + dx2;
        let bottom = self.bottom() + dy2;
        Rect::new(Point::new(left, top), Size::new(right - left, bottom - top))
    }
}

/// An axis-aligned rectangle with floating-point coordinates.
///
/// Defined by an `origin` ([`PointF`]) and a `size` ([`SizeF`]).
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{PointF, RectF, SizeF};
///
/// let r = RectF::new(PointF::new(0.0, 0.0), SizeF::new(1.0, 1.0));
/// assert!(r.contains(PointF::new(0.5, 0.5)));
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct RectF {
    origin: PointF,
    size: SizeF,
}

impl RectF {
    /// Creates a new floating-point rect.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// let r = RectF::new(PointF::new(1.0, 2.0), SizeF::new(3.0, 4.0));
    /// assert_eq!(r.origin(), PointF::new(1.0, 2.0));
    /// ```
    #[inline]
    pub const fn new(origin: PointF, size: SizeF) -> Self {
        Self { origin, size }
    }

    /// Returns the origin (top-left corner).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// assert_eq!(RectF::new(PointF::new(1.0, 2.0), SizeF::new(1.0, 1.0)).origin(), PointF::new(1.0, 2.0));
    /// ```
    #[inline]
    pub const fn origin(self) -> PointF {
        self.origin
    }

    /// Returns the size.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// assert_eq!(RectF::new(PointF::new(0.0, 0.0), SizeF::new(5.0, 3.0)).size(), SizeF::new(5.0, 3.0));
    /// ```
    #[inline]
    pub const fn size(self) -> SizeF {
        self.size
    }

    /// Returns the x coordinate of the left edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// assert_eq!(RectF::new(PointF::new(2.0, 0.0), SizeF::new(5.0, 5.0)).left(), 2.0);
    /// ```
    #[inline]
    pub const fn left(self) -> f32 {
        self.origin.x()
    }

    /// Returns the y coordinate of the top edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// assert_eq!(RectF::new(PointF::new(0.0, 3.0), SizeF::new(5.0, 5.0)).top(), 3.0);
    /// ```
    #[inline]
    pub const fn top(self) -> f32 {
        self.origin.y()
    }

    /// Returns the x coordinate of the right edge (exclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// assert_eq!(RectF::new(PointF::new(2.0, 0.0), SizeF::new(5.0, 5.0)).right(), 7.0);
    /// ```
    #[inline]
    pub const fn right(self) -> f32 {
        self.origin.x() + self.size.width()
    }

    /// Returns the y coordinate of the bottom edge (exclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// assert_eq!(RectF::new(PointF::new(0.0, 3.0), SizeF::new(5.0, 5.0)).bottom(), 8.0);
    /// ```
    #[inline]
    pub const fn bottom(self) -> f32 {
        self.origin.y() + self.size.height()
    }

    /// Returns `true` if the size has a zero-area dimension.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// assert!(RectF::new(PointF::new(0.0, 0.0), SizeF::new(0.0, 5.0)).is_empty());
    /// assert!(!RectF::new(PointF::new(0.0, 0.0), SizeF::new(1.0, 1.0)).is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.size.is_empty()
    }

    /// Returns `true` if `point` lies inside this rect (inclusive left/top, exclusive right/bottom).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// let r = RectF::new(PointF::new(0.0, 0.0), SizeF::new(1.0, 1.0));
    /// assert!(r.contains(PointF::new(0.5, 0.5)));
    /// assert!(!r.contains(PointF::new(1.0, 0.5)));
    /// ```
    #[inline]
    pub const fn contains(self, point: PointF) -> bool {
        point.x() >= self.left()
            && point.x() < self.right()
            && point.y() >= self.top()
            && point.y() < self.bottom()
    }

    /// Returns `true` if this rect overlaps with `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// let a = RectF::new(PointF::new(0.0, 0.0), SizeF::new(1.0, 1.0));
    /// let b = RectF::new(PointF::new(2.0, 2.0), SizeF::new(1.0, 1.0));
    /// assert!(!a.intersects(b));
    /// ```
    #[inline]
    pub const fn intersects(self, other: RectF) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// Returns the smallest rect containing both `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// let a = RectF::new(PointF::new(0.0, 0.0), SizeF::new(5.0, 5.0));
    /// let b = RectF::new(PointF::new(3.0, 3.0), SizeF::new(5.0, 5.0));
    /// let u = a.united(b);
    /// assert_eq!(u.origin(), PointF::new(0.0, 0.0));
    /// assert_eq!(u.size(), SizeF::new(8.0, 8.0));
    /// ```
    pub fn united(self, other: RectF) -> RectF {
        let left = self.left().min(other.left());
        let top = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        RectF::new(
            PointF::new(left, top),
            SizeF::new(right - left, bottom - top),
        )
    }

    /// Returns this rect translated by `offset`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// let r = RectF::new(PointF::new(1.0, 1.0), SizeF::new(2.0, 2.0));
    /// assert_eq!(r.translated(PointF::new(0.5, -0.5)).origin(), PointF::new(1.5, 0.5));
    /// ```
    #[inline]
    pub fn translated(self, offset: PointF) -> RectF {
        RectF::new(self.origin + offset, self.size)
    }

    /// Returns a rect expanded by `dx` on each horizontal side and `dy` on each vertical side.
    ///
    /// Negative values shrink the rect.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::{PointF, RectF, SizeF};
    ///
    /// let r = RectF::new(PointF::new(1.0, 1.0), SizeF::new(4.0, 4.0));
    /// let adjusted = r.adjusted(-0.5, -0.5, 0.5, 0.5);
    /// assert_eq!(adjusted.origin(), PointF::new(0.5, 0.5));
    /// assert_eq!(adjusted.size(), SizeF::new(5.0, 5.0));
    /// ```
    pub fn adjusted(self, dx1: f32, dy1: f32, dx2: f32, dy2: f32) -> RectF {
        let left = self.left() + dx1;
        let top = self.top() + dy1;
        let right = self.right() + dx2;
        let bottom = self.bottom() + dy2;
        RectF::new(
            PointF::new(left, top),
            SizeF::new(right - left, bottom - top),
        )
    }
}

/// Converts a [`RectF`] to a [`Rect`] by rounding origin and size to nearest integers.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{Point, PointF, Rect, RectF, Size, SizeF};
///
/// let rf = RectF::new(PointF::new(1.7, 2.3), SizeF::new(10.5, 5.5));
/// let r = Rect::from(rf);
/// assert_eq!(r.origin(), Point::new(2, 2));
/// assert_eq!(r.size(), Size::new(11, 6));
/// ```
impl From<RectF> for Rect {
    #[inline]
    fn from(r: RectF) -> Self {
        Rect::new(Point::from(r.origin), Size::from(r.size))
    }
}

/// Converts a [`Rect`] to a [`RectF`] exactly (lossless cast).
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{Point, PointF, Rect, RectF, Size, SizeF};
///
/// let r = Rect::new(Point::new(1, 2), Size::new(10, 5));
/// let rf = RectF::from(r);
/// assert_eq!(rf.origin(), PointF::new(1.0, 2.0));
/// assert_eq!(rf.size(), SizeF::new(10.0, 5.0));
/// ```
impl From<Rect> for RectF {
    #[inline]
    fn from(r: Rect) -> Self {
        RectF::new(PointF::from(r.origin), SizeF::from(r.size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_true() {
        let r = Rect::new(Point::new(10, 10), Size::new(100, 50));
        assert!(r.contains(Point::new(50, 30)));
    }

    #[test]
    fn rect_contains_false_left_boundary() {
        let r = Rect::new(Point::new(10, 10), Size::new(100, 50));
        assert!(!r.contains(Point::new(9, 10)));
    }

    #[test]
    fn rect_contains_on_origin() {
        let r = Rect::new(Point::new(10, 10), Size::new(100, 50));
        assert!(r.contains(Point::new(10, 10)));
    }

    #[test]
    fn rect_contains_right_edge_exclusive() {
        let r = Rect::new(Point::new(0, 0), Size::new(10, 10));
        assert!(!r.contains(Point::new(10, 5)));
    }

    #[test]
    fn rect_intersects_non_overlapping() {
        let a = Rect::new(Point::new(0, 0), Size::new(10, 10));
        let b = Rect::new(Point::new(20, 20), Size::new(10, 10));
        assert!(!a.intersects(b));
    }

    #[test]
    fn rect_intersects_overlapping() {
        let a = Rect::new(Point::new(0, 0), Size::new(10, 10));
        let b = Rect::new(Point::new(5, 5), Size::new(10, 10));
        assert!(a.intersects(b));
    }

    #[test]
    fn rect_intersects_adjacent_not_overlapping() {
        let a = Rect::new(Point::new(0, 0), Size::new(10, 10));
        let b = Rect::new(Point::new(10, 0), Size::new(10, 10));
        assert!(!a.intersects(b));
    }

    #[test]
    fn rect_united() {
        let a = Rect::new(Point::new(0, 0), Size::new(5, 5));
        let b = Rect::new(Point::new(3, 3), Size::new(5, 5));
        let u = a.united(b);
        assert_eq!(u.origin(), Point::new(0, 0));
        assert_eq!(u.size(), Size::new(8, 8));
    }

    #[test]
    fn rect_translated() {
        let r = Rect::new(Point::new(5, 5), Size::new(10, 10));
        assert_eq!(r.translated(Point::new(3, -2)).origin(), Point::new(8, 3));
    }

    #[test]
    fn rect_adjusted_expand() {
        let r = Rect::new(Point::new(10, 10), Size::new(20, 20));
        let adj = r.adjusted(-2, -2, 2, 2);
        assert_eq!(adj.origin(), Point::new(8, 8));
        assert_eq!(adj.size(), Size::new(24, 24));
    }

    #[test]
    fn rectf_contains_true() {
        let r = RectF::new(PointF::new(0.0, 0.0), SizeF::new(1.0, 1.0));
        assert!(r.contains(PointF::new(0.5, 0.5)));
    }

    #[test]
    fn rectf_contains_right_edge_exclusive() {
        let r = RectF::new(PointF::new(0.0, 0.0), SizeF::new(1.0, 1.0));
        assert!(!r.contains(PointF::new(1.0, 0.5)));
    }

    #[test]
    fn rectf_intersects_non_overlapping() {
        let a = RectF::new(PointF::new(0.0, 0.0), SizeF::new(1.0, 1.0));
        let b = RectF::new(PointF::new(2.0, 2.0), SizeF::new(1.0, 1.0));
        assert!(!a.intersects(b));
    }

    #[test]
    fn rectf_is_empty() {
        assert!(RectF::new(PointF::new(0.0, 0.0), SizeF::new(0.0, 5.0)).is_empty());
        assert!(!RectF::new(PointF::new(0.0, 0.0), SizeF::new(1.0, 1.0)).is_empty());
    }

    #[test]
    fn rectf_adjusted_expand() {
        let r = RectF::new(PointF::new(1.0, 1.0), SizeF::new(4.0, 4.0));
        let adj = r.adjusted(-0.5, -0.5, 0.5, 0.5);
        assert_eq!(adj.origin(), PointF::new(0.5, 0.5));
        assert_eq!(adj.size(), SizeF::new(5.0, 5.0));
    }

    #[test]
    fn rectf_from_rect() {
        let r = Rect::new(Point::new(1, 2), Size::new(10, 5));
        let rf = RectF::from(r);
        assert_eq!(rf.origin(), PointF::new(1.0, 2.0));
        assert_eq!(rf.size(), SizeF::new(10.0, 5.0));
    }

    #[test]
    fn rect_from_rectf_rounds() {
        let rf = RectF::new(PointF::new(1.7, 2.3), SizeF::new(10.5, 5.5));
        let r = Rect::from(rf);
        assert_eq!(r.origin(), Point::new(2, 2));
        assert_eq!(r.size(), Size::new(11, 6));
    }
}
