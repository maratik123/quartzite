use quartzite_core::ObjectId;

/// The kind of keyboard event.
///
/// # Examples
///
/// ```
/// use quartzite_events::KeyEventKind;
///
/// let kind = KeyEventKind::Press;
/// assert_eq!(kind, KeyEventKind::Press);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyEventKind {
    /// A key was pressed down.
    Press,
    /// A key was released.
    Release,
}

/// The kind of mouse event.
///
/// # Examples
///
/// ```
/// use quartzite_events::MouseEventKind;
///
/// let kind = MouseEventKind::Move;
/// assert_eq!(kind, MouseEventKind::Move);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MouseEventKind {
    /// A mouse button was pressed.
    Press,
    /// A mouse button was released.
    Release,
    /// The mouse cursor moved.
    Move,
}

/// Discriminant describing which kind of event occurred.
///
/// `T` is the application-level user event payload type. Use the default `T = ()` for
/// applications with no custom events.
///
/// `EventType<T>` is `Copy` when `T: Copy` and `Clone` when `T: Clone`.
///
/// # Examples
///
/// ```
/// use quartzite_events::{EventType, KeyEventKind, MouseEventKind};
///
/// let et: EventType<()> = EventType::Key(KeyEventKind::Press);
/// assert_eq!(et, EventType::Key(KeyEventKind::Press));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventType<T: 'static + Send + Sync = ()> {
    /// A keyboard event.
    Key(KeyEventKind),
    /// A mouse event.
    Mouse(MouseEventKind),
    /// The widget/window was resized.
    Resize,
    /// A close request was received.
    Close,
    /// A timer fired.
    Timer,
    /// A user-defined event with application-specific payload.
    User(T),
}

impl<T: 'static + Send + Sync + Copy> Copy for EventType<T> {}

/// Object-safe trait implemented by all event types.
///
/// `T` must match the application's chosen user event type; use `T = ()` for the common case.
/// `dyn Event<T>` is a valid trait object for any fixed `T: 'static + Send + Sync`.
///
/// # Examples
///
/// ```
/// use quartzite_events::{Event, EventType, KeyEventKind};
///
/// struct MyEvent;
/// impl Event for MyEvent {
///     fn event_type(&self) -> EventType {
///         EventType::Key(KeyEventKind::Press)
///     }
/// }
///
/// let e: &dyn Event = &MyEvent;
/// assert_eq!(e.event_type(), EventType::Key(KeyEventKind::Press));
/// ```
pub trait Event<T: 'static + Send + Sync = ()> {
    /// Returns the discriminant describing which kind of event this is.
    fn event_type(&self) -> EventType<T>;
}

/// Filter installed on an object to intercept events before they reach their target.
///
/// Return `true` to consume the event (stop propagation); `false` to continue.
///
/// # Examples
///
/// ```
/// use quartzite_events::{Event, EventFilter, EventType};
/// use quartzite_core::ObjectId;
///
/// struct LogFilter;
/// impl EventFilter for LogFilter {
///     fn event_filter(&mut self, _obj: ObjectId, _event: &dyn Event) -> bool {
///         false
///     }
/// }
/// ```
pub trait EventFilter<T: 'static + Send + Sync = ()> {
    /// Called for each event dispatched to `obj`.
    ///
    /// Return `true` to consume the event; `false` to let it propagate.
    fn event_filter(&mut self, obj: ObjectId, event: &dyn Event<T>) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_default_user_unit() {
        let et: EventType<()> = EventType::User(());
        assert_eq!(et, EventType::User(()));
    }

    #[test]
    fn event_type_custom_enum() {
        #[derive(Debug, PartialEq, Eq)]
        enum AppEvent {
            Foo,
        }
        let et: EventType<AppEvent> = EventType::User(AppEvent::Foo);
        assert_eq!(et, EventType::User(AppEvent::Foo));
    }

    #[test]
    fn event_type_key_nested_match() {
        let et: EventType<()> = EventType::Key(KeyEventKind::Press);
        match et {
            EventType::Key(KeyEventKind::Press) => {}
            _ => panic!("expected Key(Press)"),
        }
    }

    #[test]
    fn event_type_mouse_nested_match() {
        let et: EventType<()> = EventType::Mouse(MouseEventKind::Move);
        match et {
            EventType::Mouse(MouseEventKind::Move) => {}
            _ => panic!("expected Mouse(Move)"),
        }
    }

    #[test]
    fn event_type_copy_when_t_is_copy() {
        let et: EventType<()> = EventType::Key(KeyEventKind::Release);
        let et2 = et;
        let _ = et;
        assert_eq!(et, et2);
    }

    #[test]
    fn dyn_event_compiles() {
        struct DummyEvent;
        impl Event<()> for DummyEvent {
            fn event_type(&self) -> EventType<()> {
                EventType::Timer
            }
        }
        let boxed: alloc::boxed::Box<dyn Event<()>> = alloc::boxed::Box::new(DummyEvent);
        assert_eq!(boxed.event_type(), EventType::Timer);
    }
}
