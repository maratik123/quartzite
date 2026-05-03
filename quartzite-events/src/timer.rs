use quartzite_core::ObjectId;

use crate::event::{Event, EventType};

/// Event fired when a timer expires.
///
/// # Examples
///
/// ```
/// use quartzite_core::ObjectId;
/// use quartzite_events::TimerEvent;
///
/// // ObjectId is used as an opaque handle; construct one for illustration via Default.
/// let id = ObjectId::default();
/// let e = TimerEvent::new(id);
/// assert_eq!(e.timer_id(), id);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TimerEvent {
    timer_id: ObjectId,
}

impl TimerEvent {
    /// Creates a new timer event for the given timer object.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_events::TimerEvent;
    ///
    /// let id = ObjectId::default();
    /// let e = TimerEvent::new(id);
    /// assert_eq!(e.timer_id(), id);
    /// ```
    #[inline]
    pub const fn new(timer_id: ObjectId) -> Self {
        Self { timer_id }
    }

    /// Returns the id of the timer that fired.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_events::TimerEvent;
    ///
    /// let id = ObjectId::default();
    /// assert_eq!(TimerEvent::new(id).timer_id(), id);
    /// ```
    #[inline]
    pub const fn timer_id(&self) -> ObjectId {
        self.timer_id
    }
}

impl<T: 'static + Send + Sync> Event<T> for TimerEvent {
    #[inline]
    fn event_type(&self) -> EventType<T> {
        EventType::Timer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_event_stores_id() {
        let id = ObjectId::default();
        let e = TimerEvent::new(id);
        assert_eq!(e.timer_id(), id);
    }

    #[test]
    fn timer_event_type() {
        let e = TimerEvent::new(ObjectId::default());
        assert_eq!(e.event_type(), EventType::<()>::Timer);
    }
}
