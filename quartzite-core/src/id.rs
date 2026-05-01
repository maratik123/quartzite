use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ObjectId(u64);

impl ObjectId {
    // Relaxed is enough: we only need uniqueness, not cross-thread ordering.
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ConnectionId(u64);

impl ConnectionId {
    // Relaxed is enough: we only need uniqueness, not cross-thread ordering.
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl Default for ConnectionId {
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
    fn connection_id_new_returns_distinct_sequential() {
        let a = ConnectionId::new();
        let b = ConnectionId::new();
        assert_ne!(a, b);
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
