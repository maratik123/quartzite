//! Enumerations used throughout the widget system.

use quartzite_macros::MetaEnum;

/// Determines whether and how a widget accepts keyboard focus.
///
/// # Examples
///
/// ```
/// use quartzite_widgets::FocusPolicy;
///
/// assert_eq!(FocusPolicy::NoFocus as i64, 0);
/// ```
#[derive(MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(i64)]
pub enum FocusPolicy {
    /// The widget never receives keyboard focus.
    #[default]
    NoFocus = 0,
    /// The widget receives focus only via mouse click.
    ClickFocus = 1,
    /// The widget receives focus via Tab key navigation.
    TabFocus = 2,
    /// The widget accepts focus by any means.
    StrongFocus = 3,
}

/// How a widget grows or shrinks when the parent layout resizes.
///
/// # Examples
///
/// ```
/// use quartzite_widgets::SizePolicy;
///
/// assert_eq!(SizePolicy::Fixed as i64, 0);
/// ```
#[derive(MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(i64)]
pub enum SizePolicy {
    /// The size hint is the only acceptable size.
    #[default]
    Fixed = 0,
    /// The widget can be any size but prefers its size hint.
    Preferred = 1,
    /// The widget should grow to fill available space.
    Expanding = 2,
    /// The widget can shrink below its size hint.
    Minimum = 3,
    /// The widget can grow or shrink in any direction.
    MinimumExpanding = 4,
}

/// The shape of the mouse cursor displayed over a widget.
///
/// # Examples
///
/// ```
/// use quartzite_widgets::CursorShape;
///
/// assert_eq!(CursorShape::Arrow as i64, 0);
/// ```
#[derive(MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(i64)]
pub enum CursorShape {
    /// Standard arrow pointer.
    #[default]
    Arrow = 0,
    /// I-beam cursor for text entry fields.
    IBeam = 1,
    /// Crosshair cursor for precise positioning.
    Crosshair = 2,
    /// Open-hand cursor for draggable content.
    OpenHand = 3,
    /// Closed-hand cursor while dragging.
    ClosedHand = 4,
    /// Hourglass / spinning-wheel busy cursor.
    Wait = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_policy_default_is_no_focus() {
        assert_eq!(FocusPolicy::default(), FocusPolicy::NoFocus);
    }

    #[test]
    fn size_policy_default_is_fixed() {
        assert_eq!(SizePolicy::default(), SizePolicy::Fixed);
    }

    #[test]
    fn cursor_shape_default_is_arrow() {
        assert_eq!(CursorShape::default(), CursorShape::Arrow);
    }
}
