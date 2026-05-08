//! Palette stub used by [`WidgetBase`](crate::WidgetBase).
//!
//! Full palette-driven theming is deferred to `quartzite-style` (plan #47).

use quartzite_paint_api::Color;

/// A minimal colour palette for a widget.
///
/// Stores the most frequently needed colour roles. Full theming and inheritance is
/// deferred to the `quartzite-style` crate.
///
/// # Examples
///
/// ```
/// use quartzite_widgets::Palette;
/// use quartzite_paint_api::Color;
///
/// let p = Palette::default();
/// assert_eq!(p.base, Color::WHITE);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    /// Background colour of the widget surface.
    pub window: Color,
    /// Foreground (text) colour on the widget surface.
    pub window_text: Color,
    /// Background colour for interactive controls (buttons, inputs).
    pub button: Color,
    /// Foreground colour for interactive controls.
    pub button_text: Color,
    /// Background colour for text-editing areas.
    pub base: Color,
    /// Foreground colour for text in editing areas.
    pub text: Color,
    /// Colour used to highlight selected items.
    pub highlight: Color,
    /// Foreground colour on highlighted (selected) items.
    pub highlighted_text: Color,
}

impl Default for Palette {
    /// Returns a light-themed palette.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Palette;
    /// use quartzite_paint_api::Color;
    ///
    /// let p = Palette::default();
    /// assert_eq!(p.base, Color::WHITE);
    /// ```
    #[inline]
    fn default() -> Self {
        Self {
            window: Color::new(0.94, 0.94, 0.94, 1.0),
            window_text: Color::BLACK,
            button: Color::new(0.88, 0.88, 0.88, 1.0),
            button_text: Color::BLACK,
            base: Color::WHITE,
            text: Color::BLACK,
            highlight: Color::new(0.0, 0.47, 0.83, 1.0),
            highlighted_text: Color::WHITE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_palette_base_is_white() {
        let p = Palette::default();
        assert_eq!(p.base, Color::WHITE);
    }

    #[test]
    fn palette_clone() {
        let p = Palette::default();
        let q = p.clone();
        assert_eq!(p, q);
    }
}
