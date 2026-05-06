use quartzite_core::ObjectId;

use crate::event::{Event, EventType};

/// Event fired when a timer expires.
///
/// Carries the [`ObjectId`] of the timer that fired and the 0-indexed [`fire_count`](Self::fire_count)
/// for this timer run.
///
/// # Examples
///
/// ```
/// use quartzite_core::ObjectId;
/// use quartzite_event_types::TimerEvent;
///
/// let id = ObjectId::default();
/// let e = TimerEvent::new(id, 0);
/// assert_eq!(e.timer_id(), id);
/// assert_eq!(e.fire_count(), 0);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TimerEvent {
    timer_id: ObjectId,
    fire_count: usize,
}

impl TimerEvent {
    /// Creates a new timer event for the given timer object and fire count.
    ///
    /// # Parameters
    ///
    /// - `timer_id`: identifier of the timer object that fired this event.
    /// - `fire_count`: 0-indexed count of how many times this timer has fired in the current run.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_event_types::TimerEvent;
    ///
    /// let id = ObjectId::default();
    /// let e = TimerEvent::new(id, 3);
    /// assert_eq!(e.timer_id(), id);
    /// assert_eq!(e.fire_count(), 3);
    /// ```
    #[inline]
    pub const fn new(timer_id: ObjectId, fire_count: usize) -> Self {
        Self {
            timer_id,
            fire_count,
        }
    }

    /// Returns the id of the timer that fired.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_event_types::TimerEvent;
    ///
    /// let id = ObjectId::default();
    /// assert_eq!(TimerEvent::new(id, 0).timer_id(), id);
    /// ```
    #[inline]
    pub const fn timer_id(&self) -> ObjectId {
        self.timer_id
    }

    /// Returns the 0-indexed fire count for this timer run.
    ///
    /// The first fire in a run returns `0`, the second `1`, and so on.
    /// The counter resets to `0` each time the timer is (re-)started.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_event_types::TimerEvent;
    ///
    /// let e = TimerEvent::new(ObjectId::default(), 7);
    /// assert_eq!(e.fire_count(), 7);
    /// ```
    #[inline]
    pub const fn fire_count(&self) -> usize {
        self.fire_count
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
        let e = TimerEvent::new(id, 0);
        assert_eq!(e.timer_id(), id);
    }

    #[test]
    fn timer_event_stores_fire_count() {
        let e = TimerEvent::new(ObjectId::default(), 7);
        assert_eq!(e.fire_count(), 7);
    }

    #[test]
    fn timer_event_fire_count_zero() {
        let e = TimerEvent::new(ObjectId::default(), 0);
        assert_eq!(e.fire_count(), 0);
    }

    #[test]
    fn timer_event_type() {
        let e = TimerEvent::new(ObjectId::default(), 0);
        assert_eq!(e.event_type(), EventType::<()>::Timer);
    }
}
