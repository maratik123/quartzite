//! Unique identifier types for objects and signal connections.
use core::sync::atomic::{AtomicU64, Ordering};

/// Process-unique identifier for an object in the quartzite object tree.
///
/// Each [`ObjectBase`](crate::ObjectBase) allocates one `ObjectId` at construction.
/// IDs are monotonically increasing within a process and are never reused.
///
/// # Examples
///
/// ```
/// use quartzite_core::ObjectId;
///
/// let a = ObjectId::new();
/// let b = ObjectId::new();
/// assert_ne!(a, b);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct ObjectId(u64);

impl ObjectId {
    /// Allocates a fresh, process-unique [`ObjectId`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    ///
    /// let id = ObjectId::new();
    /// assert_ne!(id.raw(), 0);
    /// ```
    // Relaxed is enough: we only need uniqueness, not cross-thread ordering.
    #[inline]
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the underlying `u64` discriminant.
    ///
    /// Useful for serialisation or logging. The value is stable for the lifetime of
    /// the process but should not be persisted across restarts.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    ///
    /// let id = ObjectId::new();
    /// assert!(id.raw() > 0);
    /// ```
    #[inline]
    pub fn raw(self) -> u64 {
        self.0
    }
}

impl Default for ObjectId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Process-unique identifier for a signal-slot connection.
///
/// Returned by `Signal::connect*` methods and used to [`disconnect`](crate::Signal::disconnect)
/// a specific slot. IDs are monotonically increasing within a process and are never reused.
///
/// # Examples
///
/// ```
/// use quartzite_core::ConnectionId;
///
/// let a = ConnectionId::new();
/// let b = ConnectionId::new();
/// assert_ne!(a, b);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct ConnectionId(u64);

impl ConnectionId {
    /// Allocates a fresh, process-unique [`ConnectionId`].
    ///
    /// Normally called internally by `Signal::connect*`. Exposed publicly so that
    /// runtimes can create synthetic connection records.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ConnectionId;
    ///
    /// let id = ConnectionId::new();
    /// assert_ne!(id.raw(), 0);
    /// ```
    // Relaxed is enough: we only need uniqueness, not cross-thread ordering.
    #[inline]
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the underlying `u64` discriminant.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ConnectionId;
    ///
    /// let id = ConnectionId::new();
    /// assert!(id.raw() > 0);
    /// ```
    #[inline]
    pub fn raw(self) -> u64 {
        self.0
    }
}

impl Default for ConnectionId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use std::{collections::HashSet, sync::Mutex, thread};

    #[test]
    fn object_id_new_returns_distinct_sequential() {
        let a = ObjectId::new();
        let b = ObjectId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn object_id_later_allocation_is_greater() {
        let a = ObjectId::new();
        let b = ObjectId::new();
        assert!(a < b);
    }

    #[test]
    fn connection_id_new_returns_distinct_sequential() {
        let a = ConnectionId::new();
        let b = ConnectionId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn connection_id_later_allocation_is_greater() {
        let a = ConnectionId::new();
        let b = ConnectionId::new();
        assert!(a < b);
    }

    #[test]
    #[cfg(feature = "std")]
    fn object_id_new_returns_distinct_concurrent() {
        const N: usize = 64;
        let ids: Mutex<HashSet<u64>> = Mutex::new(HashSet::new());
        thread::scope(|s| {
            for _ in 0..N {
                s.spawn(|| {
                    let id = ObjectId::new();
                    ids.lock().unwrap().insert(id.raw());
                });
            }
        });
        assert_eq!(ids.lock().unwrap().len(), N);
    }

    #[test]
    #[cfg(feature = "std")]
    fn connection_id_new_returns_distinct_concurrent() {
        const N: usize = 64;
        let ids: Mutex<HashSet<u64>> = Mutex::new(HashSet::new());
        thread::scope(|s| {
            for _ in 0..N {
                s.spawn(|| {
                    let id = ConnectionId::new();
                    ids.lock().unwrap().insert(id.raw());
                });
            }
        });
        assert_eq!(ids.lock().unwrap().len(), N);
    }
}
