use enumflags2::{BitFlags, bitflags};
use quartzite_geometry::Point;

use crate::{
    event::{Event, EventType, MouseEventKind},
    keyboard::KeyModifiers,
};

/// An individual mouse button.
///
/// Combine multiple buttons into a [`MouseButtons`] set with `|`.
/// Use [`MouseButtons::empty()`] to represent no button (e.g. for move events).
///
/// # Examples
///
/// ```
/// use quartzite_events::{MouseButton, MouseButtons};
///
/// let pressed: MouseButtons = MouseButton::Left | MouseButton::Right;
/// assert!(pressed.contains(MouseButton::Left));
/// assert!(pressed.contains(MouseButton::Right));
/// assert!(!pressed.contains(MouseButton::Middle));
/// ```
#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// The primary (left) mouse button.
    Left = 0b0000_0001,
    /// The secondary (right) mouse button.
    Right = 0b0000_0010,
    /// The middle mouse button (scroll wheel click).
    Middle = 0b0000_0100,
    /// The back side button.
    Back = 0b0000_1000,
    /// The forward side button.
    Forward = 0b0001_0000,
}

/// A set of mouse buttons.
///
/// Constructed by OR-ing [`MouseButton`] variants. Use [`BitFlags::empty()`] for no buttons.
///
/// # Examples
///
/// ```
/// use quartzite_events::{MouseButton, MouseButtons};
///
/// let pressed: MouseButtons = MouseButton::Left | MouseButton::Right;
/// assert!(pressed.contains(MouseButton::Left));
/// ```
pub type MouseButtons = BitFlags<MouseButton>;

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
///     MouseButton::Left.into(),
///     MouseButton::Left.into(),
///     Default::default(),
///     MouseEventKind::Press,
/// );
/// assert!(e.button().contains(MouseButton::Left));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    position: Point,
    global_position: Point,
    button: MouseButtons,
    buttons: MouseButtons,
    modifiers: KeyModifiers,
    kind: MouseEventKind,
}

impl MouseEvent {
    /// Creates a new mouse event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseEvent, MouseButtons, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(
    ///     Point::new(0, 0),
    ///     Point::new(0, 0),
    ///     MouseButtons::empty(),
    ///     MouseButtons::empty(),
    ///     Default::default(),
    ///     MouseEventKind::Move,
    /// );
    /// assert_eq!(e.kind(), MouseEventKind::Move);
    /// ```
    #[inline]
    pub fn new(
        position: Point,
        global_position: Point,
        button: MouseButtons,
        buttons: MouseButtons,
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
    /// use quartzite_events::{MouseEvent, MouseButtons, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(5, 10), Point::new(0, 0), MouseButtons::empty(), MouseButtons::empty(), Default::default(), MouseEventKind::Move);
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
    /// use quartzite_events::{MouseEvent, MouseButtons, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(100, 200), MouseButtons::empty(), MouseButtons::empty(), Default::default(), MouseEventKind::Move);
    /// assert_eq!(e.global_position(), Point::new(100, 200));
    /// ```
    #[inline]
    pub const fn global_position(&self) -> Point {
        self.global_position
    }

    /// Returns the button that triggered this event, or empty for move events.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseButton, MouseEvent, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(0, 0), MouseButton::Right.into(), MouseButton::Right.into(), Default::default(), MouseEventKind::Press);
    /// assert!(e.button().contains(MouseButton::Right));
    /// ```
    #[inline]
    pub fn button(&self) -> MouseButtons {
        self.button
    }

    /// Returns a bitmask of all currently pressed buttons.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseButton, MouseButtons, MouseEvent, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let pressed: MouseButtons = MouseButton::Left | MouseButton::Right;
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(0, 0), MouseButton::Left.into(), pressed, Default::default(), MouseEventKind::Press);
    /// assert!(e.buttons().contains(MouseButton::Right));
    /// ```
    #[inline]
    pub fn buttons(&self) -> MouseButtons {
        self.buttons
    }

    /// Returns the active keyboard modifiers at the time of the event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{KeyModifier, MouseEvent, MouseButtons, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(0, 0), MouseButtons::empty(), MouseButtons::empty(), KeyModifier::Ctrl.into(), MouseEventKind::Move);
    /// assert!(e.modifiers().contains(KeyModifier::Ctrl));
    /// ```
    #[inline]
    pub fn modifiers(&self) -> KeyModifiers {
        self.modifiers
    }

    /// Returns the specific kind of mouse event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{MouseEvent, MouseButtons, MouseEventKind};
    /// use quartzite_geometry::Point;
    ///
    /// let e = MouseEvent::new(Point::new(0, 0), Point::new(0, 0), MouseButtons::empty(), MouseButtons::empty(), Default::default(), MouseEventKind::Release);
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

    fn make_event(kind: MouseEventKind, button: MouseButtons) -> MouseEvent {
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
        let e = make_event(MouseEventKind::Press, MouseButton::Left.into());
        assert!(e.button().contains(MouseButton::Left));
    }

    #[test]
    fn mouse_event_event_type_press() {
        let e = make_event(MouseEventKind::Press, MouseButton::Left.into());
        assert_eq!(
            e.event_type(),
            EventType::<()>::Mouse(MouseEventKind::Press)
        );
    }

    #[test]
    fn mouse_event_event_type_move() {
        let e = make_event(MouseEventKind::Move, MouseButtons::empty());
        assert_eq!(e.event_type(), EventType::<()>::Mouse(MouseEventKind::Move));
    }

    #[test]
    fn mouse_button_bitmask() {
        let combined: MouseButtons = MouseButton::Left | MouseButton::Right;
        assert!(combined.contains(MouseButton::Left));
        assert!(combined.contains(MouseButton::Right));
        assert!(!combined.contains(MouseButton::Middle));
    }

    #[test]
    fn mouse_event_multi_button() {
        let pressed: MouseButtons = MouseButton::Left | MouseButton::Right;
        let e = MouseEvent::new(
            Point::new(0, 0),
            Point::new(0, 0),
            MouseButton::Left.into(),
            pressed,
            KeyModifiers::empty(),
            MouseEventKind::Press,
        );
        assert!(e.buttons().contains(MouseButton::Right));
    }
}
