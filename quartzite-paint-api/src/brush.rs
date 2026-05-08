use crate::Color;

/// Describes how a shape's interior is filled.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{Brush, BrushKind, Color};
///
/// let brush = Brush::solid(Color::RED);
/// assert_eq!(brush.kind(), BrushKind::Solid(Color::RED));
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Brush {
    kind: BrushKind,
}

/// The fill pattern variant for a [`Brush`].
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{BrushKind, Color};
///
/// let kind = BrushKind::Solid(Color::BLUE);
/// // BrushKind is Copy
/// let _k2 = kind;
/// let _k3 = kind;
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum BrushKind {
    /// A solid fill with a single color.
    Solid(Color),
}

impl Brush {
    /// Creates a solid-color brush.
    ///
    /// # Parameters
    ///
    /// - `color`: the fill color.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Brush, BrushKind, Color};
    ///
    /// let brush = Brush::solid(Color::GREEN);
    /// assert_eq!(brush.kind(), BrushKind::Solid(Color::GREEN));
    /// ```
    #[inline]
    pub const fn solid(color: Color) -> Self {
        Self {
            kind: BrushKind::Solid(color),
        }
    }

    /// Returns the fill kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Brush, BrushKind, Color};
    ///
    /// assert_eq!(Brush::solid(Color::WHITE).kind(), BrushKind::Solid(Color::WHITE));
    /// ```
    #[inline]
    pub const fn kind(self) -> BrushKind {
        self.kind
    }
}

impl Default for Brush {
    /// Returns a solid opaque white brush.
    #[inline]
    fn default() -> Self {
        Self::solid(Color::WHITE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_stores_color() {
        let brush = Brush::solid(Color::RED);
        assert_eq!(brush.kind(), BrushKind::Solid(Color::RED));
    }

    #[test]
    fn default_is_solid_white() {
        assert_eq!(Brush::default().kind(), BrushKind::Solid(Color::WHITE));
    }

    #[test]
    fn brush_kind_is_copy() {
        let kind = BrushKind::Solid(Color::BLUE);
        let _k2 = kind;
        let _k3 = kind;
    }

    #[test]
    fn brush_is_copy() {
        let brush = Brush::solid(Color::BLACK);
        let _b2 = brush;
        let _b3 = brush;
    }
}
