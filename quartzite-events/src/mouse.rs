use quartzite_geometry::Point;

use crate::{
    event::{Event, EventType, MouseEventKind},
    keyboard::KeyModifiers,
};

bitflags::bitflags! {
    /// A set of mouse buttons, usable as a single button or a pressed-buttons bitmask.
    ///
    /// Individual constants (`LEFT`, `RIGHT`, `MIDDLE`, `BACK`, `FORWARD`) are single-bit values.
    /// The `buttons` field on [`MouseEvent`] combines multiple pressed buttons via bitwise OR.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::MouseButton;
    ///
    /// let combined = MouseButton::LEFT | MouseButton::RIGHT;
    /// assert!(combined.contains(MouseButton::LEFT));
    /// assert!(combined.contains(MouseButton::RIGHT));
    /// ```
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct MouseButton: u8 {
        /// The primary (left) mouse button.
        const LEFT    = 0b0000_0001;
        /// The secondary (right) mouse button.
        const RIGHT   = 0b0000_0010;
        /// The middle mouse button (scroll wheel click).
        const MIDDLE  = 0b0000_0100;
        /// The back side button.
        const BACK    = 0b0000_1000;
        /// The forward side button.
        const FORWARD = 0b0001_0000;
    }
}

/// A mouse input event carrying position, button state, and keyboard modifiers.
///
/// # Examples
///
/// ```
/// use quartzite_events::{MouseButton, MouseEvent, MouseEventKind};
/// use quartzite_geometry::Point;
///
/// let e = MouseEvent::new(
///     Point::new(10, 20),
///     Point::new(110, 220),
///     MouseButton::LEFT,
///     MouseButton::LEFT,
///     Default::default(),
///     MouseEventKind::Press,
/// );
/// assert_eq!(e.button(), MouseButton::LEFT);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    position: Point,
    global_position: Point,
    button: MouseButton,
    buttons: MouseButton,
    modifiers: KeyModifiers,
    kind: MouseEventKind,
}

impl MouseEvent {
    /// Creates a new mouse event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseButton, MouseEvent, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(
    ///     Point::new(0, 0),
    ///     Point::new(0, 0),
    ///     MouseButton::empty(),
    ///     MouseButton::empty(),
    ///     Default::default(),
    ///     MouseEventKind::Move,
    /// );
    /// assert_eq!(e.kind(), MouseEventKind::Move);
    /// ```
    #[inline]
    pub const fn new(
        position: Point,
        global_position: Point,
        button: MouseButton,
        buttons: MouseButton,
        modifiers: KeyModifiers,
        kind: MouseEventKind,
    ) -> Self {
        Self {
            position,
            global_position,
            button,
            buttons,
            modifiers,
            kind,
        }
    }

    /// Returns the cursor position in widget-local coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseButton, MouseEvent, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(5, 10), Point::new(0, 0), MouseButton::empty(), MouseButton::empty(), Default::default(), MouseEventKind::Move);
    /// assert_eq!(e.position(), Point::new(5, 10));
    /// ```
    #[inline]
    pub const fn position(&self) -> Point {
        self.position
    }

    /// Returns the cursor position in screen coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseButton, MouseEvent, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(100, 200), MouseButton::empty(), MouseButton::empty(), Default::default(), MouseEventKind::Move);
    /// assert_eq!(e.global_position(), Point::new(100, 200));
    /// ```
    #[inline]
    pub const fn global_position(&self) -> Point {
        self.global_position
    }

    /// Returns the button that triggered this event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseButton, MouseEvent, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(0, 0), MouseButton::RIGHT, MouseButton::RIGHT, Default::default(), MouseEventKind::Press);
    /// assert_eq!(e.button(), MouseButton::RIGHT);
    /// ```
    #[inline]
    pub const fn button(&self) -> MouseButton {
        self.button
    }

    /// Returns a bitmask of all currently pressed buttons.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseButton, MouseEvent, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let pressed = MouseButton::LEFT | MouseButton::RIGHT;
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(0, 0), MouseButton::LEFT, pressed, Default::default(), MouseEventKind::Press);
    /// assert!(e.buttons().contains(MouseButton::RIGHT));
    /// ```
    #[inline]
    pub const fn buttons(&self) -> MouseButton {
        self.buttons
    }

    /// Returns the active keyboard modifiers at the time of the event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(0, 0), MouseButton::empty(), MouseButton::empty(), KeyModifiers::CTRL, MouseEventKind::Move);
    /// assert!(e.modifiers().contains(KeyModifiers::CTRL));
    /// ```
    #[inline]
    pub const fn modifiers(&self) -> KeyModifiers {
        self.modifiers
    }

    /// Returns the specific kind of mouse event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseButton, MouseEvent, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(0, 0), MouseButton::empty(), MouseButton::empty(), Default::default(), MouseEventKind::Release);
    /// assert_eq!(e.kind(), MouseEventKind::Release);
    /// ```
    #[inline]
    pub const fn kind(&self) -> MouseEventKind {
        self.kind
    }
}

impl<T: 'static + Send + Sync> Event<T> for MouseEvent {
    #[inline]
    fn event_type(&self) -> EventType<T> {
        EventType::Mouse(self.kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(kind: MouseEventKind, button: MouseButton) -> MouseEvent {
        MouseEvent::new(
            Point::new(10, 20),
            Point::new(110, 220),
            button,
            button,
            KeyModifiers::empty(),
            kind,
        )
    }

    #[test]
    fn mouse_event_button_left() {
        let e = make_event(MouseEventKind::Press, MouseButton::LEFT);
        assert_eq!(e.button(), MouseButton::LEFT);
    }

    #[test]
    fn mouse_event_event_type_press() {
        let e = make_event(MouseEventKind::Press, MouseButton::LEFT);
        assert_eq!(
            e.event_type(),
            EventType::<()>::Mouse(MouseEventKind::Press)
        );
    }

    #[test]
    fn mouse_event_event_type_move() {
        let e = make_event(MouseEventKind::Move, MouseButton::empty());
        assert_eq!(e.event_type(), EventType::<()>::Mouse(MouseEventKind::Move));
    }

    #[test]
    fn mouse_button_bitmask() {
        let combined = MouseButton::LEFT | MouseButton::RIGHT;
        assert!(combined.contains(MouseButton::LEFT));
        assert!(combined.contains(MouseButton::RIGHT));
        assert!(!combined.contains(MouseButton::MIDDLE));
    }

    #[test]
    fn mouse_event_multi_button() {
        let pressed = MouseButton::LEFT | MouseButton::RIGHT;
        let e = MouseEvent::new(
            Point::new(0, 0),
            Point::new(0, 0),
            MouseButton::LEFT,
            pressed,
            KeyModifiers::empty(),
            MouseEventKind::Press,
        );
        assert!(e.buttons().contains(MouseButton::RIGHT));
    }
}
