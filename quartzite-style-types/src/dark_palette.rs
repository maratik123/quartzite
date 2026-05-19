//! Dark-theme [`Palette`] constant for quartzite.
//!
//! Exports [`DARK_PALETTE`], a compile-time [`Palette`] whose eleven
//! [`ColorRole`] slots are seeded from the dark-theme values defined in
//! `design-system/README.md` § *Dark theme* and
//! `design-system/colors_and_type.css` `[data-theme="dark"]`.

use quartzite_paint_api::Color;

use crate::{ColorRole, Palette};

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
    // Window: #2B2B2B
    .with_role(ColorRole::Window, Color::new(0.169, 0.169, 0.169, 1.0))
    // WindowText: #E8E8E8
    .with_role(ColorRole::WindowText, Color::new(0.910, 0.910, 0.910, 1.0))
    // Button: #3C3C3C
    .with_role(ColorRole::Button, Color::new(0.235, 0.235, 0.235, 1.0))
    // ButtonText: #E8E8E8
    .with_role(ColorRole::ButtonText, Color::new(0.910, 0.910, 0.910, 1.0))
    // Base: #1E1E1E
    .with_role(ColorRole::Base, Color::new(0.118, 0.118, 0.118, 1.0))
    // Text: #E8E8E8
    .with_role(ColorRole::Text, Color::new(0.910, 0.910, 0.910, 1.0))
    // Highlight: #1E90FF (DodgerBlue)
    .with_role(ColorRole::Highlight, Color::new(0.118, 0.564, 1.000, 1.0))
    // HighlightedText: #FFFFFF
    .with_role(ColorRole::HighlightedText, Color::WHITE)
    // Link: #5BB0FF
    .with_role(ColorRole::Link, Color::new(0.357, 0.690, 1.000, 1.0))
    // LinkVisited: #C58AFF
    .with_role(ColorRole::LinkVisited, Color::new(0.773, 0.541, 1.000, 1.0))
    // BrightText: #FF6B6B
    .with_role(ColorRole::BrightText, Color::new(1.000, 0.420, 0.420, 1.0));

/// Compile-time regression guard: verifies that `DARK_PALETTE` evaluates in a
/// `const` context.
const _: Palette = DARK_PALETTE;
