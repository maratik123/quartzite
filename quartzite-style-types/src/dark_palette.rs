//! Dark-theme [`Palette`] constant for quartzite.
//!
//! Exports [`DARK_PALETTE`], a compile-time [`Palette`] whose eleven
//! [`ColorRole`] slots are seeded from the dark-theme values defined in
//! `design-system/README.md` § *Dark theme* and
//! `design-system/colors_and_type.css` `[data-theme="dark"]`.

use crate::color::{
    BITTERSWEET, DODGER_BLUE, HELIOTROPE, MAYA_BLUE, NERO, ONYX, OUTER_SPACE, PLATINUM,
};
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
    .with_role(ColorRole::Window, OUTER_SPACE)
    .with_role(ColorRole::WindowText, PLATINUM)
    .with_role(ColorRole::Button, ONYX)
    .with_role(ColorRole::ButtonText, PLATINUM)
    .with_role(ColorRole::Base, NERO)
    .with_role(ColorRole::Text, PLATINUM)
    .with_role(ColorRole::Highlight, DODGER_BLUE)
    .with_role(ColorRole::HighlightedText, Color::WHITE)
    .with_role(ColorRole::Link, MAYA_BLUE)
    .with_role(ColorRole::LinkVisited, HELIOTROPE)
    .with_role(ColorRole::BrightText, BITTERSWEET);

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time regression guard: verifies that `DARK_PALETTE` evaluates in a
    /// `const` context.
    const _: Palette = DARK_PALETTE;
}
