use quartzite_geometry::Size;

use crate::event::{Event, EventType};

/// Event fired when a widget or window is resized.
///
/// # Examples
///
/// ```
/// use quartzite_events::ResizeEvent;
/// use quartzite_geometry::Size;
///
/// let e = ResizeEvent::new(Size::new(800, 600), Size::new(1024, 768));
/// assert_eq!(e.old_size(), Size::new(800, 600));
/// assert_eq!(e.new_size(), Size::new(1024, 768));
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ResizeEvent {
    old_size: Size,
    new_size: Size,
}

impl ResizeEvent {
    /// Creates a new resize event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::ResizeEvent;
    /// use quartzite_geometry::Size;
    ///
    /// let e = ResizeEvent::new(Size::new(100, 100), Size::new(200, 200));
    /// assert_eq!(e.new_size(), Size::new(200, 200));
    /// ```
    #[inline]
    pub const fn new(old_size: Size, new_size: Size) -> Self {
        Self { old_size, new_size }
    }

    /// Returns the size before the resize.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::ResizeEvent;
    /// use quartzite_geometry::Size;
    ///
    /// let e = ResizeEvent::new(Size::new(400, 300), Size::new(800, 600));
    /// assert_eq!(e.old_size(), Size::new(400, 300));
    /// ```
    #[inline]
    pub const fn old_size(&self) -> Size {
        self.old_size
    }

    /// Returns the size after the resize.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::ResizeEvent;
    /// use quartzite_geometry::Size;
    ///
    /// let e = ResizeEvent::new(Size::new(400, 300), Size::new(800, 600));
    /// assert_eq!(e.new_size(), Size::new(800, 600));
    /// ```
    #[inline]
    pub const fn new_size(&self) -> Size {
        self.new_size
    }
}

impl<T: 'static + Send + Sync> Event<T> for ResizeEvent {
    #[inline]
    fn event_type(&self) -> EventType<T> {
        EventType::Resize
    }
}

/// A close request event. Call [`accept`](CloseEvent::accept) to allow the close.
///
/// By default the close is not accepted; the handler must explicitly call `accept()`.
///
/// # Examples
///
/// ```
/// use quartzite_events::CloseEvent;
///
/// let mut e = CloseEvent::new();
/// assert!(!e.accepted());
/// e.accept();
/// assert!(e.accepted());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CloseEvent {
    accepted: bool,
}

impl CloseEvent {
    /// Creates a new close event in the non-accepted state.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::CloseEvent;
    ///
    /// let e = CloseEvent::new();
    /// assert!(!e.accepted());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self { accepted: false }
    }

    /// Returns `true` if the close has been accepted.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::CloseEvent;
    ///
    /// let e = CloseEvent::new();
    /// assert!(!e.accepted());
    /// ```
    #[inline]
    pub const fn accepted(&self) -> bool {
        self.accepted
    }

    /// Accepts the close request.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::CloseEvent;
    ///
    /// let mut e = CloseEvent::new();
    /// e.accept();
    /// assert!(e.accepted());
    /// ```
    #[inline]
    pub fn accept(&mut self) {
        self.accepted = true;
    }
}

impl<T: 'static + Send + Sync> Event<T> for CloseEvent {
    #[inline]
    fn event_type(&self) -> EventType<T> {
        EventType::Close
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_geometry::Size;

    #[test]
    fn close_event_default_not_accepted() {
        let e = CloseEvent::new();
        assert!(!e.accepted());
    }

    #[test]
    fn close_event_accept() {
        let mut e = CloseEvent::new();
        e.accept();
        assert!(e.accepted());
    }

    #[test]
    fn resize_event_stores_sizes() {
        let e = ResizeEvent::new(Size::new(800, 600), Size::new(1024, 768));
        assert_eq!(e.old_size(), Size::new(800, 600));
        assert_eq!(e.new_size(), Size::new(1024, 768));
    }

    #[test]
    fn resize_event_type() {
        let e = ResizeEvent::new(Size::new(0, 0), Size::new(100, 100));
        assert_eq!(e.event_type(), EventType::<()>::Resize);
    }

    #[test]
    fn close_event_type() {
        let e = CloseEvent::new();
        assert_eq!(e.event_type(), EventType::<()>::Close);
    }
}
