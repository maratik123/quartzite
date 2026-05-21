//! [`Palette`] — colour lookup table indexed by [`ColorRole`] and [`ColorGroup`].

use quartzite_paint_api::Color;

use crate::{ColorGroup, ColorRole};

/// Number of distinct [`ColorRole`] variants and therefore the first axis length
/// used by [`Palette`].
const ROLE_COUNT: usize = ColorRole::ALL.len();

/// Number of distinct [`ColorGroup`] variants and therefore the second axis length
/// used by [`Palette`].
const GROUP_COUNT: usize = ColorGroup::ALL.len();

/// Blend factor for the `Hover` derived cells: 6 % toward `WindowText × Normal`.
const HOVER_BLEND_FACTOR: f32 = 0.06;

/// Blend factor for the `Pressed` derived cells: 16 % toward `WindowText × Normal`.
const PRESSED_BLEND_FACTOR: f32 = 0.16;

/// A colour lookup table keyed by [`ColorRole`] and [`ColorGroup`].
///
/// `Palette` stores one [`Color`] per `(role, group)` cell in a fixed-size 2D
/// array indexed by `colors[role as usize][group as usize]`. Lookups via
/// [`color`](Self::color) are constant-time array reads; updates via
/// [`with_role`](Self::with_role) and
/// [`with_role_all_groups`](Self::with_role_all_groups) are builder-style
/// (consume `self`, return the modified palette).
///
/// [`Palette::new`] installs a sensible non-transparent value for every
/// `(role, group)` cell — concrete styles override the slots they care about
/// via [`with_role`](Self::with_role) or
/// [`with_role_all_groups`](Self::with_role_all_groups).
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::Color;
/// use quartzite_style_types::{ColorGroup, ColorRole, Palette};
///
/// let palette = Palette::new()
///     .with_role(ColorRole::Window, ColorGroup::Normal, Color::RED);
/// assert_eq!(palette.color(ColorRole::Window, ColorGroup::Normal), Color::RED);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    colors: [[Color; GROUP_COUNT]; ROLE_COUNT],
}

impl Palette {
    /// Returns a palette where every `(role, group)` cell resolves to a
    /// non-transparent colour.
    ///
    /// Every `Normal` cell is seeded to a sensible resting value; `Hover` and
    /// `Pressed` cells are derived from the `Normal` value by blending toward
    /// `WindowText × Normal`:
    ///
    /// - `Hover(c) = c.blend(WindowText × Normal, 0.06)`
    /// - `Pressed(c) = c.blend(WindowText × Normal, 0.16)`
    ///
    /// After the derivation, `FocusRing × Hover` and `FocusRing × Pressed` are
    /// forced to mirror `FocusRing × Normal` — the focus-ring colour has no
    /// meaningful state variant in v1.
    ///
    /// This is a `const fn` so a `Palette` can be used as a compile-time constant.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// use quartzite_style_types::{ColorGroup, ColorRole, Palette};
    ///
    /// const PAL: Palette = Palette::new();
    /// assert_ne!(
    ///     PAL.color(ColorRole::Window, ColorGroup::Normal),
    ///     Color::TRANSPARENT,
    /// );
    /// ```
    #[inline]
    pub const fn new() -> Self {
        // Seed every cell to White as the baseline; overrides follow.
        let mut colors = [[Color::WHITE; GROUP_COUNT]; ROLE_COUNT];

        // Seed Normal cells (same values as the previous single-axis Palette).
        colors[ColorRole::WindowText as usize][ColorGroup::Normal as usize] = Color::BLACK;
        colors[ColorRole::ButtonText as usize][ColorGroup::Normal as usize] = Color::BLACK;
        colors[ColorRole::Text as usize][ColorGroup::Normal as usize] = Color::BLACK;
        colors[ColorRole::HighlightedText as usize][ColorGroup::Normal as usize] = Color::WHITE;
        colors[ColorRole::Highlight as usize][ColorGroup::Normal as usize] = Color::SKY_BLUE;
        colors[ColorRole::Link as usize][ColorGroup::Normal as usize] = Color::BLUE;
        colors[ColorRole::LinkVisited as usize][ColorGroup::Normal as usize] = Color::BLUE;
        colors[ColorRole::BrightText as usize][ColorGroup::Normal as usize] = Color::WHITE;
        colors[ColorRole::FocusRing as usize][ColorGroup::Normal as usize] = Color::SKY_BLUE;

        // Derive Hover and Pressed cells from Normal by blending toward WindowText × Normal.
        let windowtext_normal = colors[ColorRole::WindowText as usize][ColorGroup::Normal as usize];

        let mut r = 0;
        while r < ROLE_COUNT {
            let normal = colors[r][ColorGroup::Normal as usize];
            colors[r][ColorGroup::Hover as usize] =
                normal.blend(windowtext_normal, HOVER_BLEND_FACTOR);
            colors[r][ColorGroup::Pressed as usize] =
                normal.blend(windowtext_normal, PRESSED_BLEND_FACTOR);
            r += 1;
        }

        // FocusRing special-case: Hover and Pressed cells mirror Normal (spec § Out of scope).
        let focus_normal = colors[ColorRole::FocusRing as usize][ColorGroup::Normal as usize];
        colors[ColorRole::FocusRing as usize][ColorGroup::Hover as usize] = focus_normal;
        colors[ColorRole::FocusRing as usize][ColorGroup::Pressed as usize] = focus_normal;

        Self { colors }
    }

    /// Returns the colour assigned to `(role, group)`.
    ///
    /// # Parameters
    ///
    /// - `role`: which semantic role to look up.
    /// - `group`: which interaction-state group to look up.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// use quartzite_style_types::{ColorGroup, ColorRole, Palette};
    ///
    /// let palette = Palette::default();
    /// assert_ne!(
    ///     palette.color(ColorRole::Window, ColorGroup::Normal),
    ///     Color::TRANSPARENT,
    /// );
    /// ```
    #[inline]
    pub const fn color(&self, role: ColorRole, group: ColorGroup) -> Color {
        self.colors[role as usize][group as usize]
    }

    /// Returns a new palette with the `(role, group)` cell replaced by `color`.
    ///
    /// Builder-style: consumes `self` and returns the modified palette so
    /// callers can chain customisations.
    ///
    /// # Parameters
    ///
    /// - `role`: which semantic role to update.
    /// - `group`: which interaction-state group cell to replace.
    /// - `color`: the new value for that cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// use quartzite_style_types::{ColorGroup, ColorRole, Palette};
    ///
    /// let palette = Palette::default()
    ///     .with_role(ColorRole::Window, ColorGroup::Normal, Color::RED)
    ///     .with_role(ColorRole::Window, ColorGroup::Hover, Color::GREEN);
    /// assert_eq!(palette.color(ColorRole::Window, ColorGroup::Normal), Color::RED);
    /// assert_eq!(palette.color(ColorRole::Window, ColorGroup::Hover), Color::GREEN);
    /// ```
    #[inline]
    pub const fn with_role(mut self, role: ColorRole, group: ColorGroup, color: Color) -> Self {
        self.colors[role as usize][group as usize] = color;
        self
    }

    /// Returns a new palette with all three group cells of `role` replaced by `color`.
    ///
    /// Equivalent to three consecutive calls to [`with_role`](Self::with_role)
    /// for `Normal`, `Hover`, and `Pressed`. Useful for seeding a role whose
    /// state cells do not differ (e.g. non-stateful roles in `DARK_PALETTE`).
    ///
    /// Builder-style: consumes `self` and returns the modified palette.
    ///
    /// # Parameters
    ///
    /// - `role`: which semantic role to update.
    /// - `color`: the value to write into all three group cells of `role`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// use quartzite_style_types::{ColorGroup, ColorRole, Palette};
    ///
    /// let palette = Palette::default().with_role_all_groups(ColorRole::Window, Color::RED);
    /// for group in ColorGroup::ALL {
    ///     assert_eq!(palette.color(ColorRole::Window, *group), Color::RED);
    /// }
    /// ```
    #[inline]
    pub const fn with_role_all_groups(self, role: ColorRole, color: Color) -> Self {
        self.with_role(role, ColorGroup::Normal, color)
            .with_role(role, ColorGroup::Hover, color)
            .with_role(role, ColorGroup::Pressed, color)
    }
}

impl Default for Palette {
    /// Returns [`Self::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// use quartzite_style_types::{ColorGroup, ColorRole, Palette};
    ///
    /// let palette = Palette::default();
    /// for role in ColorRole::ALL {
    ///     for group in ColorGroup::ALL {
    ///         assert_ne!(palette.color(*role, *group), Color::TRANSPARENT);
    ///     }
    /// }
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Const-eval guard: verifies `color` stays callable in a const context.
    const _: Color = Palette::new().color(ColorRole::Button, ColorGroup::Normal);

    #[test]
    fn every_cell_of_default_is_non_transparent() {
        let palette = Palette::default();
        for role in ColorRole::ALL {
            for group in ColorGroup::ALL {
                assert_ne!(
                    palette.color(*role, *group),
                    Color::TRANSPARENT,
                    "role {role:?} group {group:?} is transparent"
                );
            }
        }
    }

    #[test]
    fn light_palette_meaningful_state_cells() {
        let palette = Palette::default();
        // Button × Hover = WHITE.blend(BLACK, 0.06) = #F0F0F0
        assert_eq!(
            palette.color(ColorRole::Button, ColorGroup::Hover),
            Color::WHITE.blend(Color::BLACK, HOVER_BLEND_FACTOR),
        );
        // Button × Pressed = WHITE.blend(BLACK, 0.16) = #D6D6D6
        assert_eq!(
            palette.color(ColorRole::Button, ColorGroup::Pressed),
            Color::WHITE.blend(Color::BLACK, PRESSED_BLEND_FACTOR),
        );
        // Highlight × Hover = SKY_BLUE.blend(BLACK, 0.06) ≈ #0078F0
        assert_eq!(
            palette.color(ColorRole::Highlight, ColorGroup::Hover),
            Color::SKY_BLUE.blend(Color::BLACK, HOVER_BLEND_FACTOR),
        );
        // Highlight × Pressed = SKY_BLUE.blend(BLACK, 0.16) ≈ #006CD6
        assert_eq!(
            palette.color(ColorRole::Highlight, ColorGroup::Pressed),
            Color::SKY_BLUE.blend(Color::BLACK, PRESSED_BLEND_FACTOR),
        );
        // FocusRing × Normal = SKY_BLUE (#0080FF)
        assert_eq!(
            palette.color(ColorRole::FocusRing, ColorGroup::Normal),
            Color::SKY_BLUE,
        );
    }

    #[test]
    fn light_palette_focus_ring_per_group_mirrors_normal() {
        let palette = Palette::default();
        let normal = palette.color(ColorRole::FocusRing, ColorGroup::Normal);
        assert_eq!(
            palette.color(ColorRole::FocusRing, ColorGroup::Hover),
            normal,
            "FocusRing Hover must mirror Normal",
        );
        assert_eq!(
            palette.color(ColorRole::FocusRing, ColorGroup::Pressed),
            normal,
            "FocusRing Pressed must mirror Normal",
        );
    }

    #[test]
    fn light_palette_derivation_applies_to_all_cells() {
        let palette = Palette::default();
        // Window × Normal = WHITE; WindowText × Normal = BLACK.
        // Window × Hover = WHITE.blend(BLACK, 0.06)
        assert_eq!(
            palette.color(ColorRole::Window, ColorGroup::Hover),
            Color::WHITE.blend(Color::BLACK, HOVER_BLEND_FACTOR),
        );
        // Text × Normal = BLACK = WindowText × Normal; derivation produces BLACK.
        assert_eq!(
            palette.color(ColorRole::Text, ColorGroup::Hover),
            Color::BLACK,
        );
    }

    #[test]
    fn default_highlight_differs_from_highlighted_text() {
        let palette = Palette::default();
        assert_ne!(
            palette.color(ColorRole::Highlight, ColorGroup::Normal),
            palette.color(ColorRole::HighlightedText, ColorGroup::Normal),
            "Highlight must be visually distinct from HighlightedText so that \
             white HighlightedText remains legible against the Highlight background"
        );
    }

    #[test]
    fn with_role_replaces_single_cell() {
        let base = Palette::default();
        let modified = base
            .clone()
            .with_role(ColorRole::Window, ColorGroup::Normal, Color::RED);
        assert_eq!(
            modified.color(ColorRole::Window, ColorGroup::Normal),
            Color::RED
        );
        // Spot-check that an unrelated cell is untouched.
        assert_eq!(
            modified.color(ColorRole::Text, ColorGroup::Normal),
            base.color(ColorRole::Text, ColorGroup::Normal)
        );
        assert_eq!(
            modified.color(ColorRole::Highlight, ColorGroup::Normal),
            base.color(ColorRole::Highlight, ColorGroup::Normal)
        );
        // Hover cell of Window is unchanged by the Normal override.
        assert_eq!(
            modified.color(ColorRole::Window, ColorGroup::Hover),
            base.color(ColorRole::Window, ColorGroup::Hover)
        );
    }

    #[test]
    fn with_role_all_groups_replaces_all_three_cells() {
        let palette = Palette::default().with_role_all_groups(ColorRole::Button, Color::RED);
        for group in ColorGroup::ALL {
            assert_eq!(
                palette.color(ColorRole::Button, *group),
                Color::RED,
                "Button group {group:?} should be RED"
            );
        }
        // Spot-check that an unrelated role is untouched.
        let base = Palette::default();
        for group in ColorGroup::ALL {
            assert_eq!(
                palette.color(ColorRole::Window, *group),
                base.color(ColorRole::Window, *group),
                "Window group {group:?} should be unchanged"
            );
        }
    }

    #[test]
    fn with_role_chains() {
        let palette = Palette::default()
            .with_role(ColorRole::Window, ColorGroup::Normal, Color::RED)
            .with_role(ColorRole::Highlight, ColorGroup::Normal, Color::GREEN);
        assert_eq!(
            palette.color(ColorRole::Window, ColorGroup::Normal),
            Color::RED
        );
        assert_eq!(
            palette.color(ColorRole::Highlight, ColorGroup::Normal),
            Color::GREEN
        );
    }

    #[test]
    fn clone_round_trip_preserves_equality() {
        let palette =
            Palette::default().with_role(ColorRole::Button, ColorGroup::Normal, Color::BLUE);
        assert_eq!(palette, palette.clone());
    }

    #[test]
    fn dark_palette_has_non_transparent_color_for_every_role() {
        use crate::DARK_PALETTE;
        for role in ColorRole::ALL {
            for group in ColorGroup::ALL {
                assert_ne!(
                    DARK_PALETTE.color(*role, *group),
                    Color::TRANSPARENT,
                    "role {role:?} group {group:?} is transparent in DARK_PALETTE"
                );
            }
        }
    }

    #[test]
    fn dark_palette_highlight_differs_from_highlighted_text() {
        use crate::DARK_PALETTE;
        assert_ne!(
            DARK_PALETTE.color(ColorRole::Highlight, ColorGroup::Normal),
            DARK_PALETTE.color(ColorRole::HighlightedText, ColorGroup::Normal),
            "Highlight must be visually distinct from HighlightedText so that \
             HighlightedText remains legible against the Highlight background"
        );
    }
}
