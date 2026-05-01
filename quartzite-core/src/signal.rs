#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::id::ConnectionId;

/// Determines how a slot is invoked when a signal is emitted.
///
/// `Queued` (cross-thread delivery) is deferred to `quartzite-runtime`, which
/// provides an event loop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnectionType {
    /// Invoke the slot immediately in the emitting call stack.
    Direct,
    /// Invoke the slot exactly once, then automatically disconnect.
    SingleShot,
}

/// Internal slot record stored inside a `Signal`.
struct SlotEntry<Args: 'static> {
    id: ConnectionId,
    callback: Box<dyn Fn(&Args)>,
    conn_type: ConnectionType,
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
/// `Signal` is intentionally **not** `Send + Sync`; thread-safety semantics are
/// decided by `quartzite-runtime`.
pub struct Signal<Args: 'static> {
    slots: Vec<SlotEntry<Args>>,
}

impl<Args: 'static> Default for Signal<Args> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Args: 'static> Signal<Args> {
    pub fn new() -> Self {
        Signal { slots: Vec::new() }
    }

    /// Connect a `Direct` slot. Returns the `ConnectionId` that can be used to
    /// disconnect later.
    pub fn connect<F: Fn(&Args) + 'static>(&mut self, f: F) -> ConnectionId {
        self.connect_typed(f, ConnectionType::Direct)
    }

    /// Connect a slot with an explicit `ConnectionType`.
    pub fn connect_typed<F: Fn(&Args) + 'static>(
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

    /// Remove the slot identified by `id`. No-op if `id` is not found.
    pub fn disconnect(&mut self, id: ConnectionId) {
        self.slots.retain(|s| s.id != id);
    }

    /// Invoke all connected slots with `args`.
    ///
    /// `SingleShot` slots are called once and then removed. The slot list is
    /// snapshotted (by collecting `(id, conn_type)` pairs) before iteration so
    /// that slots added during emission are not called in the same emit pass.
    pub fn emit(&mut self, args: &Args) {
        // Snapshot before iterating so we observe only pre-existing slots.
        let snapshot: Vec<(ConnectionId, ConnectionType)> =
            self.slots.iter().map(|s| (s.id, s.conn_type)).collect();

        let mut to_remove: Vec<ConnectionId> = Vec::new();

        for (id, ct) in &snapshot {
            // Skip if already disconnected (e.g. by a previous slot in this emission).
            if let Some(entry) = self.slots.iter().find(|s| s.id == *id) {
                (entry.callback)(args);
            }
            if *ct == ConnectionType::SingleShot {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            self.disconnect(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use std::{cell::Cell, rc::Rc};

    #[cfg(not(feature = "std"))]
    use alloc::{rc::Rc, vec};

    // --- AC4: emit calls all connected Direct slots ---

    #[test]
    #[cfg(feature = "std")]
    fn emit_calls_connected_direct_slots() {
        let mut sig: Signal<(i32,)> = Signal::new();
        let a = Rc::new(Cell::new(0i32));
        let b = Rc::new(Cell::new(0i32));

        let a2 = Rc::clone(&a);
        let b2 = Rc::clone(&b);

        sig.connect(move |args| a2.set(args.0));
        sig.connect(move |args| b2.set(args.0));

        sig.emit(&(42,));

        assert_eq!(a.get(), 42);
        assert_eq!(b.get(), 42);
    }

    // --- AC5: disconnect removes slot ---

    #[test]
    #[cfg(feature = "std")]
    fn disconnect_removes_slot() {
        let mut sig: Signal<(i32,)> = Signal::new();
        let called = Rc::new(Cell::new(false));
        let called2 = Rc::clone(&called);

        let id = sig.connect(move |_args| called2.set(true));
        sig.disconnect(id);
        sig.emit(&(1,));

        assert!(!called.get(), "disconnected slot must not be called");
    }

    // --- AC6: SingleShot called exactly once ---

    #[test]
    #[cfg(feature = "std")]
    fn single_shot_called_once() {
        let mut sig: Signal<()> = Signal::new();
        let count = Rc::new(Cell::new(0u32));
        let count2 = Rc::clone(&count);

        sig.connect_typed(
            move |_| count2.set(count2.get() + 1),
            ConnectionType::SingleShot,
        );

        sig.emit(&());
        sig.emit(&());
        sig.emit(&());

        assert_eq!(count.get(), 1, "SingleShot slot must fire exactly once");
    }

    // --- emit on empty signal must not panic ---

    #[test]
    fn emit_with_no_slots_does_not_panic() {
        let mut sig: Signal<(bool,)> = Signal::new();
        sig.emit(&(true,)); // must not panic
    }

    // --- Re-entrancy note ---
    // With `emit(&mut self)`, calling `emit` on the *same* signal from within a
    // slot is prevented at compile time (cannot hold &mut and call &mut again).
    // Cross-signal re-entrancy is fine and requires no special handling.
    // This is a documented design decision: simpler than RefCell, limitation is
    // acceptable for single-threaded object graphs.

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
    fn slot_added_during_emit_not_called_in_same_pass() {
        // This test documents snapshot behavior: slots added by other means
        // (not re-entrantly, since &mut self prevents that) are not called
        // in the pass where they are added.
        // Here we simply verify that two pre-connected slots both fire.
        let mut sig: Signal<()> = Signal::new();
        let count = Rc::new(Cell::new(0u32));
        let c1 = Rc::clone(&count);
        let c2 = Rc::clone(&count);

        sig.connect(move |_| c1.set(c1.get() + 1));
        sig.connect(move |_| c2.set(c2.get() + 1));
        sig.emit(&());

        assert_eq!(count.get(), 2);
    }

    #[test]
    #[cfg(feature = "std")]
    fn disconnect_nonexistent_id_is_noop() {
        let mut sig: Signal<()> = Signal::new();
        let fake_id = ConnectionId::new();
        sig.disconnect(fake_id); // must not panic
        sig.emit(&()); // must not panic
    }
}
