//! Process-wide store of active signal–slot connections.
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use quartzite_core::{
    ConnectionId, ObjectId,
    signal::{DispatcherAlreadySet, QueuedDispatcher, set_queued_dispatcher},
};

use crate::event_loop::EventLoop;

/// Index of which connection ids belong to a given signal.
type SignalIndex = usize;

/// Record of a single active signal → slot connection.
pub struct ConnectionRecord {
    /// `ObjectId` of the object that owns the signal.
    pub sender_id: ObjectId,
    /// Index of the signal in the sender's `MetaObject::signals` slice.
    pub signal_index: SignalIndex,
    /// `ObjectId` of the object that owns the slot.
    pub receiver_id: ObjectId,
}

/// Process-wide store of active connections.
///
/// Two secondary indices allow O(m) cleanup when an object is destroyed.
/// Locks are released before invoking slots to prevent deadlock on re-entrant emit.
pub struct ConnectionTable {
    connections: RwLock<HashMap<ConnectionId, ConnectionRecord>>,
    by_receiver: RwLock<HashMap<ObjectId, Vec<ConnectionId>>>,
    by_signal: RwLock<HashMap<(ObjectId, SignalIndex), Vec<ConnectionId>>>,
    event_loop: Arc<EventLoop>,
}

impl ConnectionTable {
    /// Create a new, empty `ConnectionTable` backed by `event_loop` for queued dispatch.
    ///
    /// Returns an `Arc` because `ConnectionTable` is shared between the application and
    /// the `QueuedDispatcher` registration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_runtime::{ConnectionTable, EventLoop};
    ///
    /// let event_loop = Arc::new(EventLoop::new());
    /// let table = ConnectionTable::new(event_loop);
    /// ```
    pub fn new(event_loop: Arc<EventLoop>) -> Arc<Self> {
        Arc::new(Self {
            connections: RwLock::new(HashMap::new()),
            by_receiver: RwLock::new(HashMap::new()),
            by_signal: RwLock::new(HashMap::new()),
            event_loop,
        })
    }

    /// Register this table as the process-wide `QueuedDispatcher`.
    /// Returns `Err(DispatcherAlreadySet)` if a dispatcher is already registered.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_runtime::{ConnectionTable, EventLoop};
    ///
    /// let el = Arc::new(EventLoop::new());
    /// let table = ConnectionTable::new(el);
    /// table.install_as_dispatcher().expect("no dispatcher registered yet");
    /// ```
    pub fn install_as_dispatcher(self: &Arc<Self>) -> Result<(), DispatcherAlreadySet> {
        set_queued_dispatcher(Arc::clone(self) as Arc<dyn QueuedDispatcher>)
    }

    /// Register a new connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::{ConnectionTable, EventLoop};
    ///
    /// let el = Arc::new(EventLoop::new());
    /// let table = ConnectionTable::new(el);
    /// let _id = table.register(ObjectId::new(), 0, ObjectId::new());
    /// ```
    pub fn register(
        &self,
        sender_id: ObjectId,
        signal_index: SignalIndex,
        receiver_id: ObjectId,
    ) -> ConnectionId {
        let id = ConnectionId::new();
        let record = ConnectionRecord {
            sender_id,
            signal_index,
            receiver_id,
        };
        self.connections.write().unwrap().insert(id, record);
        self.by_receiver
            .write()
            .unwrap()
            .entry(receiver_id)
            .or_default()
            .push(id);
        self.by_signal
            .write()
            .unwrap()
            .entry((sender_id, signal_index))
            .or_default()
            .push(id);
        id
    }

    /// Remove a connection by id.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::{ConnectionTable, EventLoop};
    ///
    /// let el = Arc::new(EventLoop::new());
    /// let table = ConnectionTable::new(el);
    /// let id = table.register(ObjectId::new(), 0, ObjectId::new());
    /// table.remove(id);
    /// ```
    pub fn remove(&self, id: ConnectionId) {
        if let Some(record) = self.connections.write().unwrap().remove(&id) {
            if let Some(v) = self
                .by_receiver
                .write()
                .unwrap()
                .get_mut(&record.receiver_id)
            {
                v.retain(|&c| c != id);
            }
            if let Some(v) = self
                .by_signal
                .write()
                .unwrap()
                .get_mut(&(record.sender_id, record.signal_index))
            {
                v.retain(|&c| c != id);
            }
        }
    }

    /// Remove all connections where `id` is the receiver (called on object destroy).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::{ConnectionTable, EventLoop};
    ///
    /// let el = Arc::new(EventLoop::new());
    /// let table = ConnectionTable::new(el);
    /// let receiver_id = ObjectId::new();
    /// table.register(ObjectId::new(), 0, receiver_id);
    /// table.remove_by_receiver(receiver_id); // removes all slots for this receiver
    /// ```
    pub fn remove_by_receiver(&self, id: ObjectId) {
        let ids: Vec<ConnectionId> = self
            .by_receiver
            .write()
            .unwrap()
            .remove(&id)
            .unwrap_or_default();
        let mut conns = self.connections.write().unwrap();
        let mut by_signal = self.by_signal.write().unwrap();
        for cid in ids {
            if let Some(record) = conns.remove(&cid)
                && let Some(v) = by_signal.get_mut(&(record.sender_id, record.signal_index))
            {
                v.retain(|&c| c != cid);
            }
        }
    }

    /// Returns connection ids for a given (sender, signal) pair.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::{ConnectionTable, EventLoop};
    ///
    /// let el = Arc::new(EventLoop::new());
    /// let table = ConnectionTable::new(el);
    /// let ids = table.receivers_for_signal(ObjectId::new(), 0);
    /// assert!(ids.is_empty());
    /// ```
    pub fn receivers_for_signal(
        &self,
        sender_id: ObjectId,
        signal_index: SignalIndex,
    ) -> Vec<ConnectionId> {
        self.by_signal
            .read()
            .unwrap()
            .get(&(sender_id, signal_index))
            .cloned()
            .unwrap_or_default()
    }
}

impl QueuedDispatcher for ConnectionTable {
    fn post(&self, f: Box<dyn FnOnce() + Send>) {
        self.event_loop.post(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_core::ObjectId;

    fn make_table() -> Arc<ConnectionTable> {
        let el = Arc::new(EventLoop::new());
        ConnectionTable::new(el)
    }

    #[test]
    fn register_and_remove() {
        let table = make_table();
        let sender = ObjectId::new();
        let receiver = ObjectId::new();

        let id = table.register(sender, 0, receiver);

        assert!(table.receivers_for_signal(sender, 0).contains(&id));
        table.remove(id);
        assert!(!table.receivers_for_signal(sender, 0).contains(&id));
    }

    #[test]
    fn remove_by_receiver_cleans_up() {
        let table = make_table();
        let sender = ObjectId::new();
        let receiver = ObjectId::new();

        table.register(sender, 0, receiver);

        table.remove_by_receiver(receiver);
        assert!(table.receivers_for_signal(sender, 0).is_empty());
    }
}
