//! [`Palette`] — colour lookup table indexed by [`ColorRole`].

use quartzite_paint_api::Color;

use crate::ColorRole;

/// Number of distinct [`ColorRole`] variants and therefore the array length
/// used by [`Palette`].
const ROLE_COUNT: usize = ColorRole::ALL.len();

/// A colour lookup table keyed by [`ColorRole`].
///
/// `Palette` stores one [`Color`] per role in a fixed-size array indexed by
/// `ColorRole as usize`. Lookups via [`color`](Self::color) are constant-time
/// array reads; updates via [`with_role`](Self::with_role) are builder-style
/// (consume `self`, return the modified palette).
///
/// [`Palette::default`] installs a sensible non-transparent value for every
/// role — concrete styles override the slots they care about via
/// [`with_role`](Self::with_role).
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::Color;
/// use quartzite_style_types::{ColorRole, Palette};
///
/// let palette = Palette::default().with_role(ColorRole::Window, Color::RED);
/// assert_eq!(palette.color(ColorRole::Window), Color::RED);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    colors: [Color; ROLE_COUNT],
}

impl Palette {
    /// Returns the colour assigned to `role`.
    ///
    /// # Parameters
    ///
    /// - `role`: which slot to look up.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// use quartzite_style_types::{ColorRole, Palette};
    ///
    /// let palette = Palette::default();
    /// assert_ne!(palette.color(ColorRole::Window), Color::TRANSPARENT);
    /// ```
    #[inline]
    pub const fn color(&self, role: ColorRole) -> Color {
        self.colors[role as usize]
    }

    /// Returns a new palette with `role`'s slot replaced by `color`.
    ///
    /// Builder-style: consumes `self` and returns the modified palette so
    /// callers can chain customisations.
    ///
    /// # Parameters
    ///
    /// - `role`: which slot to replace.
    /// - `color`: the new value for that slot.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// use quartzite_style_types::{ColorRole, Palette};
    ///
    /// let palette = Palette::default()
    ///     .with_role(ColorRole::Window, Color::RED)
    ///     .with_role(ColorRole::Text, Color::BLACK);
    /// assert_eq!(palette.color(ColorRole::Window), Color::RED);
    /// assert_eq!(palette.color(ColorRole::Text), Color::BLACK);
    /// ```
    #[inline]
    pub fn with_role(mut self, role: ColorRole, color: Color) -> Palette {
        self.colors[role as usize] = color;
        self
    }
}

impl Default for Palette {
    /// Returns a palette where every [`ColorRole`] resolves to a
    /// non-transparent colour.
    ///
    /// The defaults are intentionally minimal — the goal is to satisfy the
    /// `default != Color::TRANSPARENT` invariant for every role rather than
    /// to produce a polished theme. Backgrounds resolve to [`Color::WHITE`]
    /// and foregrounds (text, link colours, bright text) resolve to
    /// [`Color::BLACK`] except [`ColorRole::BrightText`] and
    /// [`ColorRole::HighlightedText`], which use [`Color::WHITE`] so they
    /// remain legible against highlighted backgrounds. [`ColorRole::Highlight`]
    /// is seeded to [`Color::SKY_BLUE`] so checked / selected widgets render
    /// visibly under the default palette. Concrete `Style` implementations
    /// override these via [`Palette::with_role`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// use quartzite_style_types::{ColorRole, Palette};
    ///
    /// let palette = Palette::default();
    /// for role in ColorRole::ALL {
    ///     assert_ne!(palette.color(*role), Color::TRANSPARENT);
    /// }
    /// ```
    fn default() -> Self {
        let mut colors = [Color::WHITE; ROLE_COUNT];
        colors[ColorRole::WindowText as usize] = Color::BLACK;
        colors[ColorRole::ButtonText as usize] = Color::BLACK;
        colors[ColorRole::Text as usize] = Color::BLACK;
        colors[ColorRole::HighlightedText as usize] = Color::WHITE;
        colors[ColorRole::Highlight as usize] = Color::SKY_BLUE;
        colors[ColorRole::Link as usize] = Color::BLUE;
        colors[ColorRole::LinkVisited as usize] = Color::BLUE;
        colors[ColorRole::BrightText as usize] = Color::WHITE;
        Self { colors }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_non_transparent_color_for_every_role() {
        let palette = Palette::default();
        for role in ColorRole::ALL {
            assert_ne!(
                palette.color(*role),
                Color::TRANSPARENT,
                "role {role:?} is transparent"
            );
        }
    }

    #[test]
    fn default_highlight_differs_from_highlighted_text() {
        let palette = Palette::default();
        assert_ne!(
            palette.color(ColorRole::Highlight),
            palette.color(ColorRole::HighlightedText),
            "Highlight must be visually distinct from HighlightedText so that \
             white HighlightedText remains legible against the Highlight background"
        );
    }

    #[test]
    fn with_role_replaces_slot_only() {
        let base = Palette::default();
        let modified = base.clone().with_role(ColorRole::Window, Color::RED);
        assert_eq!(modified.color(ColorRole::Window), Color::RED);
        // Spot-check that an unrelated role is untouched.
        assert_eq!(modified.color(ColorRole::Text), base.color(ColorRole::Text));
        assert_eq!(
            modified.color(ColorRole::Highlight),
            base.color(ColorRole::Highlight)
        );
    }

    #[test]
    fn with_role_chains() {
        let palette = Palette::default()
            .with_role(ColorRole::Window, Color::RED)
            .with_role(ColorRole::Highlight, Color::GREEN);
        assert_eq!(palette.color(ColorRole::Window), Color::RED);
        assert_eq!(palette.color(ColorRole::Highlight), Color::GREEN);
    }

    #[test]
    fn clone_round_trip_preserves_equality() {
        let palette = Palette::default().with_role(ColorRole::Button, Color::BLUE);
        assert_eq!(palette, palette.clone());
    }
}
