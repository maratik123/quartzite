//! Dark-theme [`Palette`] constant for quartzite.
//!
//! Exports [`DARK_PALETTE`], a compile-time [`Palette`] whose eleven
//! [`ColorRole`] slots are seeded from the dark-theme values defined in
//! `design-system/README.md` § *Dark theme* and
//! `design-system/colors_and_type.css` `[data-theme="dark"]`.

use quartzite_paint_api::Color;

use crate::{ColorRole, Palette};

/// Dark `#2B2B2B` — window background.
const DARK_WINDOW: Color = Color::new(0.169, 0.169, 0.169, 1.0);
/// Dark `#E8E8E8` — general text and button text on dark backgrounds.
const DARK_TEXT_ON_DARK: Color = Color::new(0.910, 0.910, 0.910, 1.0);
/// Dark `#3C3C3C` — button background.
const DARK_BUTTON: Color = Color::new(0.235, 0.235, 0.235, 1.0);
/// Dark `#1E1E1E` — base (editor/input) background.
const DARK_BASE: Color = Color::new(0.118, 0.118, 0.118, 1.0);
/// Dark `#1E90FF` — `DodgerBlue` highlight / selection color.
const DARK_HIGHLIGHT: Color = Color::new(0.118, 0.564, 1.000, 1.0);
/// Dark `#5BB0FF` — hyperlink color.
const DARK_LINK: Color = Color::new(0.357, 0.690, 1.000, 1.0);
/// Dark `#C58AFF` — visited hyperlink color.
const DARK_LINK_VISITED: Color = Color::new(0.773, 0.541, 1.000, 1.0);
/// Dark `#FF6B6B` — bright / warning text color.
const DARK_BRIGHT_TEXT: Color = Color::new(1.000, 0.420, 0.420, 1.0);

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
    .with_role(ColorRole::Window, DARK_WINDOW)
    .with_role(ColorRole::WindowText, DARK_TEXT_ON_DARK)
    .with_role(ColorRole::Button, DARK_BUTTON)
    .with_role(ColorRole::ButtonText, DARK_TEXT_ON_DARK)
    .with_role(ColorRole::Base, DARK_BASE)
    .with_role(ColorRole::Text, DARK_TEXT_ON_DARK)
    .with_role(ColorRole::Highlight, DARK_HIGHLIGHT)
    .with_role(ColorRole::HighlightedText, Color::WHITE)
    .with_role(ColorRole::Link, DARK_LINK)
    .with_role(ColorRole::LinkVisited, DARK_LINK_VISITED)
    .with_role(ColorRole::BrightText, DARK_BRIGHT_TEXT);

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time regression guard: verifies that `DARK_PALETTE` evaluates in a
    /// `const` context.
    const _: Palette = DARK_PALETTE;
}
