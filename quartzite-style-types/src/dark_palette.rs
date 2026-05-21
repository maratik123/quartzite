//! Dark-theme [`Palette`] constant for quartzite.
//!
//! Exports [`DARK_PALETTE`], a compile-time [`Palette`] whose twelve
//! [`ColorRole`] × three [`ColorGroup`] cells are seeded from the dark-theme
//! values defined by this crate's constants. Every `(role, group)` cell is
//! non-transparent: `Normal` cells carry the dark-theme resting colour;
//! `Hover` / `Pressed` cells are derived by blending toward `MERCURY` at
//! 6 % / 16 % respectively. [`ColorRole::FocusRing`] Hover / Pressed cells
//! mirror Normal per spec § *Out of scope*.

use crate::{ColorGroup, ColorRole, Palette};
use quartzite_paint_api::Color;

/// Blend factor for the `Hover` derived cells in [`DARK_PALETTE`]: 6 % toward MERCURY.
///
/// Mirrors the value in `palette.rs`; repeated here so the per-group
/// overrides in [`DARK_PALETTE`] use the same formula constant.
const HOVER_BLEND_FACTOR: f32 = 0.06;

/// Blend factor for the `Pressed` derived cells in [`DARK_PALETTE`]: 16 % toward MERCURY.
///
/// Mirrors the value in `palette.rs`; repeated here so the per-group
/// overrides in [`DARK_PALETTE`] use the same formula constant.
const PRESSED_BLEND_FACTOR: f32 = 0.16;

/// Seeds `(role, Normal)` to `color`, then derives `Hover` and `Pressed` by
/// blending `color` toward [`Color::MERCURY`] at 6 % / 16 % respectively.
///
/// This helper is used to build [`DARK_PALETTE`] in a `const` context,
/// applying the dark-theme derivation formula (blend toward `MERCURY`) for
/// every role whose `Normal` value differs from the light-theme default.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::Color;
/// use quartzite_style_types::{ColorGroup, ColorRole, Palette};
///
/// // with_role_dark is not public; this demonstrates the equivalent chain:
/// let p = Palette::default()
///     .with_role(ColorRole::Window, ColorGroup::Normal, Color::MINE_SHAFT)
///     .with_role(ColorRole::Window, ColorGroup::Hover,
///         Color::MINE_SHAFT.blend(Color::MERCURY, 0.06))
///     .with_role(ColorRole::Window, ColorGroup::Pressed,
///         Color::MINE_SHAFT.blend(Color::MERCURY, 0.16));
/// assert_eq!(
///     p.color(ColorRole::Window, ColorGroup::Hover),
///     Color::MINE_SHAFT.blend(Color::MERCURY, 0.06),
/// );
/// ```
#[inline]
const fn with_role_dark(palette: Palette, role: ColorRole, color: Color) -> Palette {
    palette
        .with_role(role, ColorGroup::Normal, color)
        .with_role(
            role,
            ColorGroup::Hover,
            color.blend(Color::MERCURY, HOVER_BLEND_FACTOR),
        )
        .with_role(
            role,
            ColorGroup::Pressed,
            color.blend(Color::MERCURY, PRESSED_BLEND_FACTOR),
        )
}

/// Dark-theme color seed for the quartzite styling system.
///
/// Every `(role, group)` cell is non-transparent. `Normal` cells are seeded
/// to dark-theme RGBA values; `Hover` / `Pressed` cells are derived by
/// blending each role's `Normal` value toward [`Color::MERCURY`] at 6 % /
/// 16 % respectively (the same derivation formula used by [`Palette::new`]
/// for the light theme, but with `MERCURY` as the blend target instead of
/// `BLACK`). [`ColorRole::FocusRing`] uses
/// [`Palette::with_role_all_groups`] to force `Hover = Pressed = Normal`
/// per spec § *Out of scope*. The light-theme equivalent is
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
pub const DARK_PALETTE: Palette = {
    let p = Palette::new();
    // Non-stateful roles: Normal + derived Hover/Pressed toward MERCURY.
    let p = with_role_dark(p, ColorRole::Window, Color::MINE_SHAFT);
    let p = with_role_dark(p, ColorRole::WindowText, Color::MERCURY);
    let p = with_role_dark(p, ColorRole::ButtonText, Color::MERCURY);
    let p = with_role_dark(p, ColorRole::Base, Color::NERO);
    let p = with_role_dark(p, ColorRole::Text, Color::MERCURY);
    let p = with_role_dark(p, ColorRole::HighlightedText, Color::WHITE);
    let p = with_role_dark(p, ColorRole::Link, Color::LIGHT_DODGER_BLUE);
    let p = with_role_dark(p, ColorRole::LinkVisited, Color::CHAROITE);
    let p = with_role_dark(p, ColorRole::BrightText, Color::PASTEL_RED);
    // Stateful roles: Normal + Hover/Pressed = derivation toward MERCURY.
    let p = with_role_dark(p, ColorRole::Button, Color::ECLIPSE);
    let p = with_role_dark(p, ColorRole::Highlight, Color::DODGER_BLUE);
    // FocusRing: all three group cells seeded identically (Hover/Pressed mirror Normal).
    p.with_role_all_groups(ColorRole::FocusRing, Color::DODGER_BLUE)
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time regression guard: verifies that `DARK_PALETTE` evaluates in a
    /// `const` context.
    const _: Palette = DARK_PALETTE;

    /// AC8 — assert the five meaningful dark-theme cells are set correctly.
    ///
    /// Button and Highlight Hover/Pressed are derived from their Normal
    /// cells blended toward MERCURY at 6 % / 16 % respectively.
    #[test]
    fn dark_palette_meaningful_state_cells() {
        // Button × Hover = ECLIPSE.blend(MERCURY, 0.06) ≈ #464646
        assert_eq!(
            DARK_PALETTE.color(ColorRole::Button, ColorGroup::Hover),
            Color::ECLIPSE.blend(Color::MERCURY, HOVER_BLEND_FACTOR),
        );
        // Button × Pressed = ECLIPSE.blend(MERCURY, 0.16) ≈ #585858
        assert_eq!(
            DARK_PALETTE.color(ColorRole::Button, ColorGroup::Pressed),
            Color::ECLIPSE.blend(Color::MERCURY, PRESSED_BLEND_FACTOR),
        );
        // Highlight × Hover = DODGER_BLUE.blend(MERCURY, 0.06) ≈ #2A95FE
        assert_eq!(
            DARK_PALETTE.color(ColorRole::Highlight, ColorGroup::Hover),
            Color::DODGER_BLUE.blend(Color::MERCURY, HOVER_BLEND_FACTOR),
        );
        // Highlight × Pressed = DODGER_BLUE.blend(MERCURY, 0.16) ≈ #3E9EFB
        assert_eq!(
            DARK_PALETTE.color(ColorRole::Highlight, ColorGroup::Pressed),
            Color::DODGER_BLUE.blend(Color::MERCURY, PRESSED_BLEND_FACTOR),
        );
        // FocusRing × Normal = DODGER_BLUE (#1E90FF)
        assert_eq!(
            DARK_PALETTE.color(ColorRole::FocusRing, ColorGroup::Normal),
            Color::DODGER_BLUE,
        );
    }

    /// AC8 — `FocusRing` Hover and Pressed must mirror Normal in the dark theme.
    ///
    /// Enforces spec § *Out of scope*: `FocusRing` has no meaningful state variant in v1.
    #[test]
    fn dark_palette_focus_ring_per_group_mirrors_normal() {
        let normal = DARK_PALETTE.color(ColorRole::FocusRing, ColorGroup::Normal);
        assert_eq!(
            DARK_PALETTE.color(ColorRole::FocusRing, ColorGroup::Hover),
            normal,
            "FocusRing Hover must mirror Normal in DARK_PALETTE",
        );
        assert_eq!(
            DARK_PALETTE.color(ColorRole::FocusRing, ColorGroup::Pressed),
            normal,
            "FocusRing Pressed must mirror Normal in DARK_PALETTE",
        );
    }

    /// AC8 — derivation applies to non-stateful roles in the dark theme.
    ///
    /// `Window × Pressed` is derived from `MINE_SHAFT` blended toward `MERCURY`
    /// at 16 %, confirming the derivation formula ran for a non-stateful role.
    #[test]
    fn dark_palette_derivation_applies_to_non_stateful_roles() {
        assert_eq!(
            DARK_PALETTE.color(ColorRole::Window, ColorGroup::Pressed),
            Color::MINE_SHAFT.blend(Color::MERCURY, PRESSED_BLEND_FACTOR),
        );
    }
}
