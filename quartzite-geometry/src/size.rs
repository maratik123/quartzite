use core::ops::{Add, AddAssign, Mul, MulAssign};

use crate::round_f32;

macro_rules! impl_size_ops {
    ($Size:ident, $scalar:ty) => {
        impl Add for $Size {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self::new(self.width + rhs.width, self.height + rhs.height)
            }
        }

        impl AddAssign for $Size {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl Mul<$scalar> for $Size {
            type Output = Self;
            #[inline]
            fn mul(self, scale: $scalar) -> Self {
                Self::new(self.width * scale, self.height * scale)
            }
        }

        impl MulAssign<$scalar> for $Size {
            #[inline]
            fn mul_assign(&mut self, scale: $scalar) {
                *self = *self * scale;
            }
        }
    };
}

/// A 2D size with non-negative integer (`i32`) width and height.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::Size;
///
/// let s = Size::new(100, 50);
/// assert_eq!(s.width(), 100);
/// assert_eq!(s.height(), 50);
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Size {
    width: i32,
    height: i32,
}

impl Size {
    /// Creates a new size.
    ///
    /// By contract, `width` and `height` should be non-negative.
    ///
    /// # Parameters
    ///
    /// - `width`: horizontal extent, in pixels (should be `>= 0`).
    /// - `height`: vertical extent, in pixels (should be `>= 0`).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Size;
    ///
    /// let s = Size::new(800, 600);
    /// assert_eq!(s.width(), 800);
    /// ```
    #[inline]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Returns the width.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Size;
    ///
    /// assert_eq!(Size::new(40, 0).width(), 40);
    /// ```
    #[inline]
    pub const fn width(self) -> i32 {
        self.width
    }

    /// Returns the height.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Size;
    ///
    /// assert_eq!(Size::new(0, 30).height(), 30);
    /// ```
    #[inline]
    pub const fn height(self) -> i32 {
        self.height
    }

    /// Returns `true` if either dimension is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::Size;
    ///
    /// assert!(Size::new(0, 10).is_empty());
    /// assert!(!Size::new(1, 1).is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

impl_size_ops!(Size, i32);

/// A 2D size with single-precision floating-point (`f32`) width and height.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::SizeF;
///
/// let s = SizeF::new(1.5, 2.5);
/// assert_eq!(s.width(), 1.5);
/// assert_eq!(s.height(), 2.5);
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SizeF {
    width: f32,
    height: f32,
}

impl SizeF {
    /// Creates a new floating-point size.
    ///
    /// # Parameters
    ///
    /// - `width`: horizontal extent, in logical units (should be `>= 0.0`).
    /// - `height`: vertical extent, in logical units (should be `>= 0.0`).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::SizeF;
    ///
    /// let s = SizeF::new(100.0, 50.0);
    /// assert_eq!(s.width(), 100.0);
    /// ```
    #[inline]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Returns the width.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::SizeF;
    ///
    /// assert_eq!(SizeF::new(1.0, 0.0).width(), 1.0);
    /// ```
    #[inline]
    pub const fn width(self) -> f32 {
        self.width
    }

    /// Returns the height.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::SizeF;
    ///
    /// assert_eq!(SizeF::new(0.0, 2.0).height(), 2.0);
    /// ```
    #[inline]
    pub const fn height(self) -> f32 {
        self.height
    }

    /// Returns `true` if either dimension is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_geometry::SizeF;
    ///
    /// assert!(SizeF::new(0.0, 1.0).is_empty());
    /// assert!(!SizeF::new(1.0, 1.0).is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }
}

impl_size_ops!(SizeF, f32);

/// Converts a [`SizeF`] to a [`Size`] by rounding each dimension to the nearest integer.
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{Size, SizeF};
///
/// assert_eq!(Size::from(SizeF::new(1.7, 2.3)), Size::new(2, 2));
/// ```
impl From<SizeF> for Size {
    #[inline]
    fn from(s: SizeF) -> Self {
        Self::new(round_f32(s.width), round_f32(s.height))
    }
}

/// Converts a [`Size`] to a [`SizeF`] exactly (lossless cast).
///
/// # Examples
///
/// ```
/// use quartzite_geometry::{Size, SizeF};
///
/// assert_eq!(SizeF::from(Size::new(100, 50)), SizeF::new(100.0, 50.0));
/// ```
impl From<Size> for SizeF {
    #[inline]
    fn from(s: Size) -> Self {
        Self::new(s.width as f32, s.height as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_new_and_accessors() {
        let s = Size::new(100, 50);
        assert_eq!(s.width(), 100);
        assert_eq!(s.height(), 50);
    }

    #[test]
    fn size_is_empty() {
        assert!(Size::new(0, 10).is_empty());
        assert!(Size::new(10, 0).is_empty());
        assert!(!Size::new(1, 1).is_empty());
    }

    #[test]
    fn size_add() {
        assert_eq!(Size::new(10, 20) + Size::new(5, 5), Size::new(15, 25));
    }

    #[test]
    fn size_scale() {
        assert_eq!(Size::new(10, 20) * 3, Size::new(30, 60));
    }

    #[test]
    fn size_add_assign() {
        let mut s = Size::new(10, 10);
        s += Size::new(5, 5);
        assert_eq!(s, Size::new(15, 15));
    }

    #[test]
    fn sizef_new_and_accessors() {
        let s = SizeF::new(1.5, 2.5);
        assert_eq!(s.width(), 1.5);
        assert_eq!(s.height(), 2.5);
    }

    #[test]
    fn sizef_to_size_rounds() {
        assert_eq!(Size::from(SizeF::new(1.7, 2.3)), Size::new(2, 2));
        assert_eq!(Size::from(SizeF::new(0.5, 0.5)), Size::new(1, 1));
    }

    #[test]
    fn size_to_sizef_exact() {
        assert_eq!(SizeF::from(Size::new(100, 50)), SizeF::new(100.0, 50.0));
    }
}
