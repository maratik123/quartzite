#[cfg(not(feature = "std"))]
use alloc::sync::{Arc, Weak};
#[cfg(feature = "std")]
use std::sync::{Arc, Weak};

/// Zero-sized lifetime token. Every `ObjectBase` holds one `Arc<ReceiverGuard>`.
/// Incoming signal connections hold a `Weak<ReceiverGuard>`. When the object is
/// dropped, the `Arc` drops and all `Weak::upgrade()` calls return `None`, causing
/// queued slot calls targeting that object to be silently discarded.
pub struct ReceiverGuard;

impl ReceiverGuard {
    pub fn new_pair() -> (Arc<Self>, Weak<Self>) {
        let arc = Arc::new(ReceiverGuard);
        let weak = Arc::downgrade(&arc);
        (arc, weak)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_upgrades_while_arc_alive() {
        let (arc, weak) = ReceiverGuard::new_pair();
        assert!(weak.upgrade().is_some());
        drop(arc);
    }

    #[test]
    fn weak_returns_none_after_arc_dropped() {
        let (arc, weak) = ReceiverGuard::new_pair();
        drop(arc);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    #[cfg(feature = "std")]
    fn concurrent_drop_and_upgrade() {
        use std::thread;
        let (arc, weak) = ReceiverGuard::new_pair();
        let weak2 = weak.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let _ = weak2.upgrade();
            }
        });
        drop(arc);
        handle.join().unwrap();
        assert!(weak.upgrade().is_none());
    }
}
