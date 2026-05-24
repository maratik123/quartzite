//! [`VAlignment`] enum — vertical content alignment within a bounding box.

use quartzite_macros::MetaEnum;

/// Controls how text or content is aligned vertically within a bounding box.
///
/// Used by `quartzite-paint`'s `Painter::draw_text_in` for the vertical axis.
/// The default is [`Top`](Self::Top), which anchors text to the top of the
/// destination rectangle.
///
/// For horizontal alignment, see [`HAlignment`](crate::HAlignment).
///
/// # Examples
///
/// ```
/// use quartzite_geometry::VAlignment;
///
/// assert_eq!(VAlignment::default(), VAlignment::Top);
/// assert_eq!(format!("{:?}", VAlignment::Center), "Center");
/// ```
#[derive(MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum VAlignment {
    /// Align content to the top of the bounding box.
    #[default]
    Top,
    /// Center content vertically within the bounding box.
    Center,
    /// Align content to the bottom of the bounding box.
    Bottom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_top() {
        assert_eq!(VAlignment::default(), VAlignment::Top);
    }

    #[test]
    #[cfg(feature = "std")]
    fn debug_round_trip() {
        use std::format;
        assert_eq!(format!("{:?}", VAlignment::Top), "Top");
        assert_eq!(format!("{:?}", VAlignment::Center), "Center");
        assert_eq!(format!("{:?}", VAlignment::Bottom), "Bottom");
    }

    #[test]
    fn eq_and_copy() {
        let a = VAlignment::Center;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(VAlignment::Top, VAlignment::Bottom);
    }
}
