/// An RGBA color with `f32` components in the range `[0.0, 1.0]`.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::Color;
///
/// let red = Color::RED;
/// assert_eq!(red.r(), 1.0);
/// assert_eq!(red.g(), 0.0);
/// assert_eq!(red.b(), 0.0);
/// assert_eq!(red.a(), 1.0);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    /// Creates a new color from RGBA components.
    ///
    /// # Parameters
    ///
    /// - `r`: red channel in `[0.0, 1.0]`.
    /// - `g`: green channel in `[0.0, 1.0]`.
    /// - `b`: blue channel in `[0.0, 1.0]`.
    /// - `a`: alpha channel in `[0.0, 1.0]`; `0.0` is fully transparent.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    ///
    /// let c = Color::new(0.5, 0.25, 0.75, 1.0);
    /// assert_eq!(c.r(), 0.5);
    /// assert_eq!(c.a(), 1.0);
    /// ```
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Opaque black (`#000000ff`).
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// Opaque white (`#ffffffff`).
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    /// Opaque red (`#ff0000ff`).
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);

    /// Opaque green (`#00ff00ff`).
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);

    /// Opaque blue (`#0000ffff`).
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);

    /// Fully transparent black (`#00000000`).
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Returns the red component.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// assert_eq!(Color::RED.r(), 1.0);
    /// ```
    // _Simple._
    #[inline]
    pub const fn r(self) -> f32 {
        self.r
    }

    /// Returns the green component.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// assert_eq!(Color::GREEN.g(), 1.0);
    /// ```
    // _Simple._
    #[inline]
    pub const fn g(self) -> f32 {
        self.g
    }

    /// Returns the blue component.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// assert_eq!(Color::BLUE.b(), 1.0);
    /// ```
    // _Simple._
    #[inline]
    pub const fn b(self) -> f32 {
        self.b
    }

    /// Returns the alpha component.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// assert_eq!(Color::TRANSPARENT.a(), 0.0);
    /// ```
    // _Simple._
    #[inline]
    pub const fn a(self) -> f32 {
        self.a
    }
}

impl Default for Color {
    /// Returns opaque black.
    // _Simple._
    #[inline]
    fn default() -> Self {
        Self::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_round_trips_components() {
        let c = Color::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.r(), 0.1);
        assert_eq!(c.g(), 0.2);
        assert_eq!(c.b(), 0.3);
        assert_eq!(c.a(), 0.4);
    }

    #[test]
    fn black_has_correct_components() {
        assert_eq!(Color::BLACK.r(), 0.0);
        assert_eq!(Color::BLACK.g(), 0.0);
        assert_eq!(Color::BLACK.b(), 0.0);
        assert_eq!(Color::BLACK.a(), 1.0);
    }

    #[test]
    fn white_has_correct_components() {
        assert_eq!(Color::WHITE.r(), 1.0);
        assert_eq!(Color::WHITE.g(), 1.0);
        assert_eq!(Color::WHITE.b(), 1.0);
        assert_eq!(Color::WHITE.a(), 1.0);
    }

    #[test]
    fn color_is_copy() {
        let c = Color::RED;
        let _c2 = c;
        let _c3 = c; // use after copy — must compile
    }

    #[test]
    fn default_is_black() {
        assert_eq!(Color::default(), Color::BLACK);
    }
}
