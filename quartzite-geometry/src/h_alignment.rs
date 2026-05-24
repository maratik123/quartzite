//! [`HAlignment`] enum — horizontal content alignment within a bounding box.

use quartzite_macros::MetaEnum;

/// Controls how text or content is aligned horizontally within a bounding box.
///
/// Used both by widget layout (for content positioning inside a widget) and by
/// `quartzite-paint`'s `Painter::draw_text_in` (for horizontal text alignment
/// within a destination rectangle).
///
/// For vertical alignment, see [`VAlignment`](crate::VAlignment).
///
/// # Examples
///
/// ```
/// use quartzite_geometry::HAlignment;
///
/// assert_eq!(HAlignment::Left as i64, 0);
/// assert_eq!(HAlignment::default(), HAlignment::Left);
/// ```
#[derive(MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum HAlignment {
    /// Align content to the left.
    #[default]
    Left = 0,
    /// Center content within the available horizontal space.
    Center = 1,
    /// Align content to the right.
    Right = 2,
    /// Justify content to fill the available horizontal space.
    Justify = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_core::{FromValue, IntoValue, Value};

    #[test]
    fn default_is_left() {
        assert_eq!(HAlignment::default(), HAlignment::Left);
    }

    #[test]
    fn discriminants_match_legacy_widget_alignment() {
        assert_eq!(HAlignment::Left as i64, 0);
        assert_eq!(HAlignment::Center as i64, 1);
        assert_eq!(HAlignment::Right as i64, 2);
        assert_eq!(HAlignment::Justify as i64, 3);
    }

    #[test]
    fn into_value_round_trip() {
        let v = HAlignment::Center.into_value();
        assert_eq!(v, Value::Int(1));
        assert_eq!(HAlignment::from_value(v), Ok(HAlignment::Center));
    }
}
