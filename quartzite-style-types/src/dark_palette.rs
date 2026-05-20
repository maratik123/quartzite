//! Dark-theme [`Palette`] constant for quartzite.
//!
//! Exports [`DARK_PALETTE`], a compile-time [`Palette`] whose eleven
//! [`ColorRole`] slots are seeded from the dark-theme values defined in
//! `design-system/README.md` § *Dark theme*.

use crate::{ColorRole, Palette};
use quartzite_paint_api::Color;

/// Dark-theme color seed for the quartzite styling system.
///
/// Every [`ColorRole`] slot is set to the RGBA value that `design-system/README.md`
/// § *Dark theme* specifies for that role, converted from sRGB hex to 3-decimal
/// linear floats. The light-theme equivalent is [`Palette::default()`].
///
/// `DARK_PALETTE` is a compile-time constant; it can be used anywhere a
/// `const` [`Palette`] is required.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::Color;
/// use quartzite_style_types::{ColorRole, DARK_PALETTE};
///
/// assert_eq!(
///     DARK_PALETTE.color(ColorRole::Highlight),
///     Color::new(0.118, 0.564, 1.000, 1.0),
/// );
/// ```
pub const DARK_PALETTE: Palette = Palette::new()
    .with_role(ColorRole::Window, Color::MINE_SHAFT)
    .with_role(ColorRole::WindowText, Color::MERCURY)
    .with_role(ColorRole::Button, Color::ECLIPSE)
    .with_role(ColorRole::ButtonText, Color::MERCURY)
    .with_role(ColorRole::Base, Color::NERO)
    .with_role(ColorRole::Text, Color::MERCURY)
    .with_role(ColorRole::Highlight, Color::DODGER_BLUE)
    .with_role(ColorRole::HighlightedText, Color::WHITE)
    .with_role(ColorRole::Link, Color::LIGHT_DODGER_BLUE)
    .with_role(ColorRole::LinkVisited, Color::CHAROITE)
    .with_role(ColorRole::BrightText, Color::PASTEL_RED);

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time regression guard: verifies that `DARK_PALETTE` evaluates in a
    /// `const` context.
    const _: Palette = DARK_PALETTE;
}
