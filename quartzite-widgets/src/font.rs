//! Font stub used by [`WidgetBase`](crate::WidgetBase).
//!
//! Full font rasterization is deferred to `quartzite-style` (plan #47).

/// Minimal font descriptor stored in each [`WidgetBase`](crate::WidgetBase).
///
/// The fields represent the most commonly needed font attributes. A full rasterization
/// and theming pass is deferred to the `quartzite-style` crate.
///
/// # Examples
///
/// ```
/// use quartzite_widgets::Font;
///
/// let f = Font::default();
/// assert_eq!(f.family.as_str(), "sans-serif");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    /// Font family name (e.g. `"sans-serif"`, `"monospace"`).
    pub family: String,
    /// Point size (e.g. `12`).
    pub point_size: u32,
    /// Whether the font is rendered bold.
    pub bold: bool,
    /// Whether the font is rendered italic.
    pub italic: bool,
}

impl Font {
    /// Creates a new `Font` with the given `family` and `point_size`.
    ///
    /// # Parameters
    ///
    /// - `family`: font family name.
    /// - `point_size`: size in points.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Font;
    ///
    /// let f = Font::new("monospace".into(), 14);
    /// assert_eq!(f.point_size, 14);
    /// ```
    #[inline]
    pub fn new(family: String, point_size: u32) -> Self {
        Self {
            family,
            point_size,
            bold: false,
            italic: false,
        }
    }
}

impl Default for Font {
    /// Returns a 12-point sans-serif font with no bold or italic.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Font;
    ///
    /// let f = Font::default();
    /// assert_eq!(f.point_size, 12);
    /// ```
    #[inline]
    fn default() -> Self {
        Self {
            family: "sans-serif".into(),
            point_size: 12,
            bold: false,
            italic: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_font() {
        let f = Font::default();
        assert_eq!(f.family, "sans-serif");
        assert_eq!(f.point_size, 12);
        assert!(!f.bold);
        assert!(!f.italic);
    }

    #[test]
    fn new_font() {
        let f = Font::new("monospace".into(), 16);
        assert_eq!(f.family, "monospace");
        assert_eq!(f.point_size, 16);
    }

    #[test]
    fn font_clone() {
        let f = Font::new("serif".into(), 10);
        let g = f.clone();
        assert_eq!(f, g);
    }
}
