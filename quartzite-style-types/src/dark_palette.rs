//! Dark-theme [`Palette`] constant for quartzite.
//!
//! Exports [`DARK_PALETTE`], a compile-time [`Palette`] whose twelve
//! [`ColorRole`] slots are seeded from the dark-theme values defined by
//! this crate's constants.

use crate::{ColorRole, Palette};
use quartzite_paint_api::Color;

/// Dark-theme color seed for the quartzite styling system.
///
/// Every [`ColorRole`] slot is set to a dark-theme RGBA value converted from
/// sRGB hex to 3-decimal linear floats. Non-stateful roles use
/// [`Palette::with_role_all_groups`] to seed all three [`ColorGroup`] cells
/// identically; stateful roles (`Button`, `Highlight`) use per-group setters
/// for their meaningful state cells. The light-theme equivalent is
/// [`Palette::default()`].
///
/// `DARK_PALETTE` is a compile-time constant; it can be used anywhere a
/// `const` [`Palette`] is required.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::Color;
/// use quartzite_style_types::{ColorGroup, ColorRole, DARK_PALETTE};
///
/// assert_eq!(
///     DARK_PALETTE.color(ColorRole::Highlight, ColorGroup::Normal),
///     Color::new(0.118, 0.564, 1.000, 1.0),
/// );
/// ```
pub const DARK_PALETTE: Palette = Palette::new()
    .with_role_all_groups(ColorRole::Window, Color::MINE_SHAFT)
    .with_role_all_groups(ColorRole::WindowText, Color::MERCURY)
    .with_role_all_groups(ColorRole::Button, Color::ECLIPSE)
    .with_role_all_groups(ColorRole::ButtonText, Color::MERCURY)
    .with_role_all_groups(ColorRole::Base, Color::NERO)
    .with_role_all_groups(ColorRole::Text, Color::MERCURY)
    .with_role_all_groups(ColorRole::Highlight, Color::DODGER_BLUE)
    .with_role_all_groups(ColorRole::HighlightedText, Color::WHITE)
    .with_role_all_groups(ColorRole::Link, Color::LIGHT_DODGER_BLUE)
    .with_role_all_groups(ColorRole::LinkVisited, Color::CHAROITE)
    .with_role_all_groups(ColorRole::BrightText, Color::PASTEL_RED)
    .with_role_all_groups(ColorRole::FocusRing, Color::DODGER_BLUE);

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time regression guard: verifies that `DARK_PALETTE` evaluates in a
    /// `const` context.
    const _: Palette = DARK_PALETTE;
}
