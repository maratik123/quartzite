//! [`Alignment`] enum — content alignment within a bounding box.

use quartzite_macros::MetaEnum;

/// Controls how text or content is aligned within a bounding box.
///
/// Used both by widget layout (for content positioning inside a widget) and by
/// `quartzite-paint`'s `Painter::draw_text_in` (for text alignment within a
/// destination rectangle).
///
/// # Examples
///
/// ```
/// use quartzite_geometry::Alignment;
///
/// assert_eq!(Alignment::Left as i64, 0);
/// assert_eq!(Alignment::default(), Alignment::Left);
/// ```
#[derive(MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(i64)]
pub enum Alignment {
    /// Align to the left (horizontal) or top (vertical).
    #[default]
    Left = 0,
    /// Center content within the available space.
    Center = 1,
    /// Align to the right (horizontal) or bottom (vertical).
    Right = 2,
    /// Justify content to fill the available space.
    Justify = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_core::{FromValue, IntoValue, Value};

    #[test]
    fn default_is_left() {
        assert_eq!(Alignment::default(), Alignment::Left);
    }

    #[test]
    fn discriminants_match_legacy_widget_alignment() {
        assert_eq!(Alignment::Left as i64, 0);
        assert_eq!(Alignment::Center as i64, 1);
        assert_eq!(Alignment::Right as i64, 2);
        assert_eq!(Alignment::Justify as i64, 3);
    }

    #[test]
    fn into_value_round_trip() {
        let v = Alignment::Center.into_value();
        assert_eq!(v, Value::Int(1));
        assert_eq!(Alignment::from_value(v), Ok(Alignment::Center));
    }
}
