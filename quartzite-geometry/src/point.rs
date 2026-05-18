use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use crate::round_f32;

macro_rules! impl_point_ops {
    ($Point:ident, $scalar:ty) => {
        impl Add for $Point {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self::new(self.x + rhs.x, self.y + rhs.y)
            }
        }

        impl AddAssign for $Point {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl Sub for $Point {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self::new(self.x - rhs.x, self.y - rhs.y)
            }
        }

        impl SubAssign for $Point {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl Neg for $Point {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                Self::new(-self.x, -self.y)
            }
        }
    };
}

/// A point in 2D space with integer (`i32`) coordinates.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::Point;
///
/// let p = Point::new(3, 4);
/// assert_eq!(p.x(), 3);
/// assert_eq!(p.y(), 4);
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    /// Creates a new point with the given coordinates.
    ///
    /// # Parameters
    ///
    /// - `x`: horizontal coordinate, in pixels.
    /// - `y`: vertical coordinate, in pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Point;
    ///
    /// let p = Point::new(1, 2);
    /// assert_eq!(p.x(), 1);
    /// ```
    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the x coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Point;
    ///
    /// assert_eq!(Point::new(5, 0).x(), 5);
    /// ```
    #[inline]
    pub const fn x(self) -> i32 {
        self.x
    }

    /// Returns the y coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Point;
    ///
    /// assert_eq!(Point::new(0, 7).y(), 7);
    /// ```
    #[inline]
    pub const fn y(self) -> i32 {
        self.y
    }
}

impl_point_ops!(Point, i32);

/// A point in 2D space with single-precision floating-point (`f32`) coordinates.
///
/// Suitable for sub-pixel coordinates and GPU rendering.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::PointF;
///
/// let p = PointF::new(1.5, 2.5);
/// assert_eq!(p.x(), 1.5);
/// assert_eq!(p.y(), 2.5);
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PointF {
    x: f32,
    y: f32,
}

impl PointF {
    /// Creates a new point with the given floating-point coordinates.
    ///
    /// # Parameters
    ///
    /// - `x`: horizontal coordinate, in logical units.
    /// - `y`: vertical coordinate, in logical units.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::PointF;
    ///
    /// let p = PointF::new(0.5, -1.0);
    /// assert_eq!(p.x(), 0.5);
    /// ```
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the x coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::PointF;
    ///
    /// assert_eq!(PointF::new(3.0, 0.0).x(), 3.0);
    /// ```
    #[inline]
    pub const fn x(self) -> f32 {
        self.x
    }

    /// Returns the y coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::PointF;
    ///
    /// assert_eq!(PointF::new(0.0, 4.0).y(), 4.0);
    /// ```
    #[inline]
    pub const fn y(self) -> f32 {
        self.y
    }
}

impl_point_ops!(PointF, f32);

/// Converts a [`PointF`] to a [`Point`] by rounding each coordinate to the nearest integer.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{Point, PointF};
///
/// assert_eq!(Point::from(PointF::new(1.7, 2.3)), Point::new(2, 2));
/// assert_eq!(Point::from(PointF::new(-1.7, -2.3)), Point::new(-2, -2));
/// ```
impl From<PointF> for Point {
    #[inline]
    fn from(p: PointF) -> Self {
        Self::new(round_f32(p.x), round_f32(p.y))
    }
}

/// Converts a [`Point`] to a [`PointF`] exactly (lossless cast).
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{Point, PointF};
///
/// assert_eq!(PointF::from(Point::new(3, -5)), PointF::new(3.0, -5.0));
/// ```
impl From<Point> for PointF {
    #[inline]
    #[allow(
        clippy::cast_precision_loss,
        reason = "sub-pixel coordinate widening is intentional"
    )]
    fn from(p: Point) -> Self {
        Self::new(p.x as f32, p.y as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_new_and_accessors() {
        let p = Point::new(3, 4);
        assert_eq!(p.x(), 3);
        assert_eq!(p.y(), 4);
    }

    #[test]
    fn point_add() {
        assert_eq!(Point::new(1, 2) + Point::new(3, 4), Point::new(4, 6));
    }

    #[test]
    fn point_sub() {
        assert_eq!(Point::new(5, 7) - Point::new(2, 3), Point::new(3, 4));
    }

    #[test]
    fn point_neg() {
        assert_eq!(-Point::new(1, -2), Point::new(-1, 2));
    }

    #[test]
    fn point_add_assign() {
        let mut p = Point::new(1, 1);
        p += Point::new(2, 3);
        assert_eq!(p, Point::new(3, 4));
    }

    #[test]
    fn point_sub_assign() {
        let mut p = Point::new(5, 5);
        p -= Point::new(2, 3);
        assert_eq!(p, Point::new(3, 2));
    }

    #[test]
    fn pointf_new_and_accessors() {
        let p = PointF::new(1.5, 2.5);
        assert_eq!(p.x(), 1.5);
        assert_eq!(p.y(), 2.5);
    }

    #[test]
    fn pointf_add() {
        assert_eq!(
            PointF::new(1.0, 2.0) + PointF::new(0.5, 0.5),
            PointF::new(1.5, 2.5)
        );
    }

    #[test]
    fn pointf_neg() {
        assert_eq!(-PointF::new(1.0, -2.0), PointF::new(-1.0, 2.0));
    }

    #[test]
    fn pointf_to_point_rounds_to_nearest() {
        assert_eq!(Point::from(PointF::new(1.7, 2.3)), Point::new(2, 2));
        assert_eq!(Point::from(PointF::new(0.5, 0.5)), Point::new(1, 1));
        assert_eq!(Point::from(PointF::new(-1.7, -2.3)), Point::new(-2, -2));
    }

    #[test]
    fn point_to_pointf_exact() {
        assert_eq!(PointF::from(Point::new(3, -5)), PointF::new(3.0, -5.0));
    }
}
