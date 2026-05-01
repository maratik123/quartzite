#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::{sync::Arc, vec::Vec};

use crate::id::ConnectionId;
#[cfg(feature = "std")]
use crate::receiver_guard::ReceiverGuard;

/// Determines how a slot is invoked when a signal is emitted.
///
/// `Queued` (cross-thread delivery) requires the `std` feature and an active
/// `QueuedDispatcher` registered via `set_queued_dispatcher`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnectionType {
    /// Invoke the slot immediately in the emitting call stack.
    Direct,
    /// Invoke the slot exactly once, then automatically disconnect.
    SingleShot,
    /// Post the slot to the event loop; requires `std` and an active dispatcher.
    #[cfg(feature = "std")]
    Queued,
}

/// Internal slot record stored inside a `Signal`.
/// Callbacks require `Send` so that `Signal<Args>` can be wrapped in
/// `Arc<Mutex<Signal<Args>>>` for cross-thread posting (e.g. `Timer`).
struct SlotEntry<Args: 'static> {
    id: ConnectionId,
    callback: Box<dyn Fn(&Args) + Send>,
    conn_type: ConnectionType,
}

/// Receives closures posted by queued signal connections and executes them on
/// the event-loop thread. Implemented by `ConnectionTable` in `quartzite-runtime`.
#[cfg(feature = "std")]
pub trait QueuedDispatcher: Send + Sync {
    fn post(&self, f: Box<dyn FnOnce() + Send>);
}

#[cfg(feature = "std")]
static QUEUED_DISPATCHER: std::sync::OnceLock<Arc<dyn QueuedDispatcher>> =
    std::sync::OnceLock::new();

/// Error returned by `set_queued_dispatcher` when a dispatcher is already registered.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatcherAlreadySet;

/// Register the process-wide queued dispatcher. Called by `Application::new()`.
/// Returns `Ok(())` on the first call; `Err(DispatcherAlreadySet)` on subsequent calls.
#[cfg(feature = "std")]
pub fn set_queued_dispatcher(d: Arc<dyn QueuedDispatcher>) -> Result<(), DispatcherAlreadySet> {
    QUEUED_DISPATCHER.set(d).map_err(|_| DispatcherAlreadySet)
}

/// Returns a reference to the registered dispatcher, or `None` before
/// `Application` has been created.
#[cfg(feature = "std")]
pub fn queued_dispatcher() -> Option<&'static Arc<dyn QueuedDispatcher>> {
    QUEUED_DISPATCHER.get()
}

/// Object-safe interface for a queued slot stored inside a `Signal<Args>`.
#[cfg(feature = "std")]
trait DynQueuedSlot<Args: 'static>: Send + Sync {
    fn id(&self) -> ConnectionId;
    fn post_if_alive(&self, args: &Args);
}

/// Concrete queued-slot entry. `Args: Clone + Send` is required to package
/// the arguments into a cross-thread closure.
#[cfg(feature = "std")]
struct QueuedSlotInner<Args: Clone + Send + 'static> {
    id: ConnectionId,
    callback: Arc<dyn Fn(Args) + Send + Sync>,
    guard: std::sync::Weak<ReceiverGuard>,
}

#[cfg(feature = "std")]
impl<Args: Clone + Send + 'static> DynQueuedSlot<Args> for QueuedSlotInner<Args> {
    fn id(&self) -> ConnectionId {
        self.id
    }

    fn post_if_alive(&self, args: &Args) {
        if self.guard.upgrade().is_none() {
            return;
        }
        let args_owned = args.clone();
        let cb = Arc::clone(&self.callback);
        if let Some(dispatcher) = queued_dispatcher() {
            dispatcher.post(Box::new(move || cb(args_owned)));
        }
    }
}

/// A typed signal that can have multiple slots connected.
///
/// `Args` is typically a tuple (e.g. `Signal<(i32, bool)>`). Slots receive a
/// shared reference to the args tuple.
///
/// `emit` takes `&mut self` to allow `SingleShot` cleanup without interior
/// mutability. This means a slot cannot call `emit` on the *same* signal
/// instance during emission (the borrow checker prevents it). Re-entrant
/// emission across *different* signals is fine.
///
/// Slot callbacks require `Send` so that `Signal<Args>` can be safely wrapped
/// in `Arc<Mutex<Signal<Args>>>` for cross-thread emission (e.g. `Timer`).
/// The signal itself is not `Sync` — concurrent emission is not supported.
pub struct Signal<Args: 'static> {
    slots: Vec<SlotEntry<Args>>,
    #[cfg(feature = "std")]
    queued_slots: Vec<Box<dyn DynQueuedSlot<Args>>>,
}

impl<Args: 'static> Default for Signal<Args> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Args: 'static> Signal<Args> {
    pub fn new() -> Self {
        Signal {
            slots: Vec::new(),
            #[cfg(feature = "std")]
            queued_slots: Vec::new(),
        }
    }

    /// Connect a `Direct` slot. Returns the `ConnectionId` that can be used to
    /// disconnect later.
    pub fn connect<F: Fn(&Args) + Send + 'static>(&mut self, f: F) -> ConnectionId {
        self.connect_typed(f, ConnectionType::Direct)
    }

    /// Connect a slot with an explicit `ConnectionType`.
    pub fn connect_typed<F: Fn(&Args) + Send + 'static>(
        &mut self,
        f: F,
        ct: ConnectionType,
    ) -> ConnectionId {
        let id = ConnectionId::new();
        self.slots.push(SlotEntry {
            id,
            callback: Box::new(f),
            conn_type: ct,
        });
        id
    }

    /// Connect a `Queued` slot that is invoked on the event-loop thread.
    ///
    /// The receiver guard is checked before posting: if the guard has expired
    /// (receiver destroyed), the closure is silently discarded. Requires
    /// `Args: Clone + Send` so the arguments can be moved across threads.
    #[cfg(feature = "std")]
    pub fn connect_queued<F>(&mut self, f: F, guard: std::sync::Weak<ReceiverGuard>) -> ConnectionId
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: Clone + Send,
    {
        let id = ConnectionId::new();
        self.queued_slots.push(Box::new(QueuedSlotInner {
            id,
            callback: Arc::new(f),
            guard,
        }));
        id
    }

    /// Remove the slot identified by `id`. No-op if `id` is not found.
    pub fn disconnect(&mut self, id: ConnectionId) {
        self.slots.retain(|s| s.id != id);
        #[cfg(feature = "std")]
        self.queued_slots.retain(|s| s.id() != id);
    }

    /// Invoke all connected slots with `args`.
    ///
    /// `SingleShot` slots are called once and then removed in-place.
    /// `Queued` slots are posted to the event-loop thread via the registered
    /// `QueuedDispatcher` (if any).
    /// Because `emit` takes `&mut self`, no slot can call `connect`, `disconnect`,
    /// or `emit` on the *same* signal instance during emission — the borrow checker
    /// prevents it. Cross-signal mutation from within a slot is fine.
    pub fn emit(&mut self, args: &Args) {
        let mut i = 0;
        while i < self.slots.len() {
            (self.slots[i].callback)(args);
            if self.slots[i].conn_type == ConnectionType::SingleShot {
                self.slots.remove(i);
            } else {
                i += 1;
            }
        }
        #[cfg(feature = "std")]
        for slot in &self.queued_slots {
            slot.post_if_alive(args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering},
    };

    // --- AC4: emit calls all connected Direct slots ---

    #[test]
    #[cfg(feature = "std")]
    fn emit_calls_connected_direct_slots() {
        let mut sig: Signal<(i32,)> = Signal::new();
        let a = Arc::new(AtomicI32::new(0));
        let b = Arc::new(AtomicI32::new(0));

        let a2 = Arc::clone(&a);
        let b2 = Arc::clone(&b);

        sig.connect(move |args| a2.store(args.0, Ordering::Relaxed));
        sig.connect(move |args| b2.store(args.0, Ordering::Relaxed));

        sig.emit(&(42,));

        assert_eq!(a.load(Ordering::Relaxed), 42);
        assert_eq!(b.load(Ordering::Relaxed), 42);
    }

    // --- AC5: disconnect removes slot ---

    #[test]
    #[cfg(feature = "std")]
    fn disconnect_removes_slot() {
        let mut sig: Signal<(i32,)> = Signal::new();
        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);

        let id = sig.connect(move |_args| called2.store(true, Ordering::Relaxed));
        sig.disconnect(id);
        sig.emit(&(1,));

        assert!(
            !called.load(Ordering::Relaxed),
            "disconnected slot must not be called"
        );
    }

    // --- AC6: SingleShot called exactly once ---

    #[test]
    #[cfg(feature = "std")]
    fn single_shot_called_once() {
        let mut sig: Signal<()> = Signal::new();
        let count = Arc::new(AtomicU32::new(0));
        let count2 = Arc::clone(&count);

        sig.connect_typed(
            move |_| {
                count2.fetch_add(1, Ordering::Relaxed);
            },
            ConnectionType::SingleShot,
        );

        sig.emit(&());
        sig.emit(&());
        sig.emit(&());

        assert_eq!(
            count.load(Ordering::Relaxed),
            1,
            "SingleShot slot must fire exactly once"
        );
    }

    // --- emit on empty signal must not panic ---

    #[test]
    fn emit_with_no_slots_does_not_panic() {
        let mut sig: Signal<(bool,)> = Signal::new();
        sig.emit(&(true,)); // must not panic
    }

    #[test]
    #[cfg(feature = "std")]
    fn multiple_disconnects_do_not_panic() {
        let mut sig: Signal<()> = Signal::new();
        let id = sig.connect(|_| {});
        sig.disconnect(id);
        sig.disconnect(id); // second disconnect must be a no-op
        sig.emit(&());
    }

    #[test]
    #[cfg(feature = "std")]
    fn all_pre_connected_direct_slots_fire() {
        let mut sig: Signal<()> = Signal::new();
        let count = Arc::new(AtomicU32::new(0));
        let c1 = Arc::clone(&count);
        let c2 = Arc::clone(&count);

        sig.connect(move |_| {
            c1.fetch_add(1, Ordering::Relaxed);
        });
        sig.connect(move |_| {
            c2.fetch_add(1, Ordering::Relaxed);
        });
        sig.emit(&());

        assert_eq!(count.load(Ordering::Relaxed), 2);
    }

    #[test]
    #[cfg(feature = "std")]
    fn disconnect_nonexistent_id_is_noop() {
        let mut sig: Signal<()> = Signal::new();
        let fake_id = ConnectionId::new();
        sig.disconnect(fake_id); // must not panic
        sig.emit(&()); // must not panic
    }

    // --- Queued slot: guard check ---

    #[test]
    #[cfg(feature = "std")]
    fn queued_slot_not_posted_after_receiver_destroyed() {
        use crate::receiver_guard::ReceiverGuard;

        struct MockDispatcher {
            call_count: Mutex<usize>,
        }
        impl QueuedDispatcher for MockDispatcher {
            fn post(&self, f: Box<dyn FnOnce() + Send>) {
                *self.call_count.lock().unwrap() += 1;
                f();
            }
        }

        let mock = Arc::new(MockDispatcher {
            call_count: Mutex::new(0),
        });
        // Only set if not already set (other tests may have set it).
        let _ = QUEUED_DISPATCHER.set(Arc::clone(&mock) as Arc<dyn QueuedDispatcher>);

        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();

        let mut sig: Signal<i32> = Signal::new();
        let posted_values: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
        let pv = Arc::clone(&posted_values);
        sig.connect_queued(move |v| pv.lock().unwrap().push(v), guard_weak.clone());

        // Drop receiver guard to invalidate.
        drop(guard_arc);

        // Emit after receiver is destroyed — must NOT post.
        let count_before = *mock.call_count.lock().unwrap();
        sig.emit(&99);
        let count_after = *mock.call_count.lock().unwrap();
        assert_eq!(
            count_before, count_after,
            "no post must occur after receiver is destroyed"
        );
    }

    // --- WeakObjectRef tests ---

    #[test]
    fn weak_object_ref_wraps_u64() {
        use crate::value::WeakObjectRef;
        let r = WeakObjectRef(42);
        assert_eq!(r.0, 42);
        let r2 = r; // Copy
        assert_eq!(r2, r); // PartialEq
    }

    #[test]
    fn value_object_variant_round_trips() {
        use crate::value::{Value, WeakObjectRef};
        let v = Value::Object(WeakObjectRef(7));
        assert_eq!(v.type_name(), "Object");
        let v2 = v.clone();
        assert_eq!(v, v2);
    }
}
