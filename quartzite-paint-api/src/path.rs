//! [`Path`] and [`Segment`] — vector graphics path used by `Painter::draw_path`.
//!
//! `Path` lives in `quartzite-paint-api` (rather than `quartzite-paint`) so the
//! [`Painter`](crate::Painter) trait can name `&Path` directly without a circular
//! re-export through the higher-level paint crate. `quartzite-paint` re-exports
//! the type so callers depending on it see the same vocabulary.

use alloc::vec::Vec;

use quartzite_geometry::{Point, Size};

/// A single segment of a [`Path`].
///
/// Marked `#[non_exhaustive]` so future variants (e.g. `QuadTo`) can land
/// without a `SemVer` bump.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::Segment;
/// use quartzite_geometry::Point;
///
/// let s = Segment::MoveTo(Point::new(0, 0));
/// assert!(matches!(s, Segment::MoveTo(_)));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Segment {
    /// Move the pen to the given point without drawing.
    MoveTo(Point),
    /// Draw a straight line to the given point.
    LineTo(Point),
    /// Draw a cubic Bezier to `p` using `c1` and `c2` as control points.
    CubicTo(Point, Point, Point),
    /// Draw an elliptical arc.
    ///
    /// Semantics are centre-and-radii, angles in radians, with positive
    /// `sweep_angle` going counter-clockwise. Matches vello/peniko's arc model.
    ArcTo {
        /// Centre of the ellipse.
        centre: Point,
        /// Semi-axes of the ellipse (`width` = horizontal radius, `height` = vertical radius).
        radii: Size,
        /// Starting angle in radians.
        start_angle: f32,
        /// Sweep angle in radians; positive is counter-clockwise.
        sweep_angle: f32,
    },
    /// Close the current sub-path with a straight line back to the start point.
    Close,
}

/// A vector graphics path: an ordered list of [`Segment`]s.
///
/// Built incrementally via the chainable builder methods ([`Path::move_to`],
/// [`Path::line_to`], [`Path::cubic_to`], [`Path::arc_to`], [`Path::close`]) and
/// consumed by backends through [`Path::segments`].
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::Path;
/// use quartzite_geometry::Point;
///
/// let mut p = Path::new();
/// p.move_to(Point::new(0, 0)).line_to(Point::new(10, 0)).close();
/// assert_eq!(p.segments().len(), 3);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    segments: Vec<Segment>,
}

impl Path {
    /// Creates an empty path.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Path;
    ///
    /// let p = Path::new();
    /// assert!(p.segments().is_empty());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Returns the path's segments in insertion order.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Path;
    /// use quartzite_geometry::Point;
    ///
    /// let mut p = Path::new();
    /// p.move_to(Point::new(0, 0));
    /// assert_eq!(p.segments().len(), 1);
    /// ```
    #[inline]
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Moves the pen to `p` without drawing. Starts a new sub-path.
    ///
    /// # Parameters
    ///
    /// - `p`: the new pen position.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Path;
    /// use quartzite_geometry::Point;
    ///
    /// let mut p = Path::new();
    /// p.move_to(Point::new(5, 5));
    /// assert_eq!(p.segments().len(), 1);
    /// ```
    #[inline]
    pub fn move_to(&mut self, p: Point) -> &mut Self {
        self.segments.push(Segment::MoveTo(p));
        self
    }

    /// Draws a straight line to `p`.
    ///
    /// # Parameters
    ///
    /// - `p`: the endpoint of the line.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Path;
    /// use quartzite_geometry::Point;
    ///
    /// let mut path = Path::new();
    /// path.move_to(Point::new(0, 0)).line_to(Point::new(10, 0));
    /// assert_eq!(path.segments().len(), 2);
    /// ```
    #[inline]
    pub fn line_to(&mut self, p: Point) -> &mut Self {
        self.segments.push(Segment::LineTo(p));
        self
    }

    /// Draws a cubic Bezier to `p` using `c1` and `c2` as control points.
    ///
    /// # Parameters
    ///
    /// - `c1`: first control point.
    /// - `c2`: second control point.
    /// - `p`: endpoint.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Path;
    /// use quartzite_geometry::Point;
    ///
    /// let mut path = Path::new();
    /// path.cubic_to(Point::new(1, 0), Point::new(2, 0), Point::new(3, 0));
    /// assert_eq!(path.segments().len(), 1);
    /// ```
    #[inline]
    pub fn cubic_to(&mut self, c1: Point, c2: Point, p: Point) -> &mut Self {
        self.segments.push(Segment::CubicTo(c1, c2, p));
        self
    }

    /// Draws an elliptical arc.
    ///
    /// Semantics are centre-and-radii, angles in radians, positive `sweep_angle`
    /// is counter-clockwise.
    ///
    /// # Parameters
    ///
    /// - `centre`: centre of the ellipse.
    /// - `radii`: semi-axes (`width` = horizontal radius, `height` = vertical radius).
    /// - `start_angle`: starting angle in radians.
    /// - `sweep_angle`: sweep angle in radians; positive is counter-clockwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Path;
    /// use quartzite_geometry::{Point, Size};
    ///
    /// let mut path = Path::new();
    /// path.arc_to(Point::new(0, 0), Size::new(10, 5), 0.0, core::f32::consts::PI);
    /// assert_eq!(path.segments().len(), 1);
    /// ```
    #[inline]
    pub fn arc_to(
        &mut self,
        centre: Point,
        radii: Size,
        start_angle: f32,
        sweep_angle: f32,
    ) -> &mut Self {
        self.segments.push(Segment::ArcTo {
            centre,
            radii,
            start_angle,
            sweep_angle,
        });
        self
    }

    /// Closes the current sub-path with a straight line back to its start point.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Path;
    /// use quartzite_geometry::Point;
    ///
    /// let mut path = Path::new();
    /// path.move_to(Point::new(0, 0)).line_to(Point::new(10, 0)).close();
    /// assert_eq!(path.segments().len(), 3);
    /// ```
    #[inline]
    pub fn close(&mut self) -> &mut Self {
        self.segments.push(Segment::Close);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_returns_empty_slice() {
        let p = Path::new();
        assert!(p.segments().is_empty());
    }

    #[test]
    fn path_default_is_empty() {
        let p = Path::default();
        assert!(p.segments().is_empty());
    }

    #[test]
    fn move_then_line_then_close_round_trips() {
        let p0 = Point::new(0, 0);
        let p1 = Point::new(10, 5);
        let mut path = Path::new();
        path.move_to(p0).line_to(p1).close();
        let segs = path.segments();
        assert_eq!(segs.len(), 3);
        assert!(matches!(segs[0], Segment::MoveTo(p) if p == p0));
        assert!(matches!(segs[1], Segment::LineTo(p) if p == p1));
        assert!(matches!(segs[2], Segment::Close));
    }

    #[test]
    #[allow(clippy::float_cmp, reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction")]
    fn cubic_and_arc_round_trip() {
        let c1 = Point::new(1, 2);
        let c2 = Point::new(3, 4);
        let endpoint = Point::new(5, 6);
        let centre = Point::new(7, 8);
        let radii = Size::new(9, 10);
        let mut path = Path::new();
        path.cubic_to(c1, c2, endpoint)
            .arc_to(centre, radii, 0.0, core::f32::consts::PI);
        let segs = path.segments();
        assert_eq!(segs.len(), 2);
        assert!(matches!(
            segs[0],
            Segment::CubicTo(a, b, c) if a == c1 && b == c2 && c == endpoint
        ));
        assert!(matches!(
            segs[1],
            Segment::ArcTo {
                centre: cc,
                radii: rr,
                start_angle,
                sweep_angle,
            } if cc == centre
                && rr == radii
                && start_angle == 0.0
                && sweep_angle == core::f32::consts::PI
        ));
    }

    #[test]
    fn builder_returns_mut_self() {
        // Compile-time proof: chaining requires `&mut Self` returns.
        let mut p = Path::new();
        p.move_to(Point::new(0, 0))
            .line_to(Point::new(1, 1))
            .cubic_to(Point::new(2, 2), Point::new(3, 3), Point::new(4, 4))
            .arc_to(Point::new(5, 5), Size::new(1, 1), 0.0, 1.0)
            .close();
        assert_eq!(p.segments().len(), 5);
    }

    #[test]
    fn path_clone_round_trip() {
        let mut a = Path::new();
        a.move_to(Point::new(1, 2)).line_to(Point::new(3, 4));
        let b = a.clone();
        assert_eq!(a, b);
    }
}
