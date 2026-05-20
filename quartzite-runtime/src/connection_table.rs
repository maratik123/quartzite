//! Process-wide store of active signal–slot connections.
use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;

use quartzite_core::{
    ConnectionId, ObjectId,
    signal::{DispatcherAlreadySet, QueuedDispatcher, set_queued_dispatcher},
};

use crate::loop_registry::LoopRegistry;

/// Index of which connection ids belong to a given signal.
type SignalIndex = usize;

/// Record of a single active signal → slot connection.
///
/// # Examples
///
/// ```
/// use quartzite_core::ObjectId;
/// use quartzite_runtime::connection_table::ConnectionRecord;
///
/// let record = ConnectionRecord {
///     sender_id: ObjectId::new(),
///     signal_index: 0,
///     receiver_id: ObjectId::new(),
/// };
/// assert_eq!(record.signal_index, 0);
/// ```
pub struct ConnectionRecord {
    /// [`ObjectId`] of the object that owns the signal.
    pub sender_id: ObjectId,
    /// Index of the signal in the sender's [`MetaObject::signals`](quartzite_core::MetaObject::signals) slice.
    pub signal_index: SignalIndex,
    /// [`ObjectId`] of the object that owns the slot.
    pub receiver_id: ObjectId,
}

/// Process-wide store of active connections.
///
/// Two secondary indices allow O(m) cleanup when an object is destroyed.
/// Locks are released before invoking slots to prevent deadlock on re-entrant emit.
/// Queued dispatch routes invocations to the per-thread [`EventLoop`](crate::EventLoop)
/// registered in `LoopRegistry`.
///
/// # Examples
///
/// ```
/// use quartzite_runtime::ConnectionTable;
///
/// let table = ConnectionTable::new();
/// assert!(table.receivers_for_signal(quartzite_core::ObjectId::new(), 0).is_empty());
/// ```
pub struct ConnectionTable {
    connections: RwLock<HashMap<ConnectionId, ConnectionRecord>>,
    by_receiver: RwLock<HashMap<ObjectId, Vec<ConnectionId>>>,
    by_signal: RwLock<HashMap<(ObjectId, SignalIndex), Vec<ConnectionId>>>,
}

impl ConnectionTable {
    /// Creates a new, empty [`ConnectionTable`].
    ///
    /// Returns an `Arc` because the table is shared between the application and the
    /// queued-dispatcher registration. Queued dispatch routes to per-thread event loops
    /// via `LoopRegistry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_runtime::ConnectionTable;
    ///
    /// let table = ConnectionTable::new();
    /// assert!(table.receivers_for_signal(quartzite_core::ObjectId::new(), 0).is_empty());
    /// ```
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            connections: RwLock::new(HashMap::new()),
            by_receiver: RwLock::new(HashMap::new()),
            by_signal: RwLock::new(HashMap::new()),
        })
    }

    /// Registers this table as the process-wide [`QueuedDispatcher`].
    ///
    /// # Errors
    ///
    /// Returns [`DispatcherAlreadySet`] if a queued dispatcher has already been registered
    /// in this process.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ConnectionTable;
    ///
    /// let table = ConnectionTable::new();
    /// table.install_as_dispatcher().expect("no dispatcher registered yet");
    /// ```
    pub fn install_as_dispatcher(self: &Arc<Self>) -> Result<(), DispatcherAlreadySet> {
        set_queued_dispatcher(Arc::clone(self) as Arc<dyn QueuedDispatcher>)
    }

    /// Registers a new connection from `(sender_id, signal_index)` to `receiver_id`.
    ///
    /// # Parameters
    ///
    /// - `sender_id`: id of the object owning the signal.
    /// - `signal_index`: index of the signal in the sender's `MetaObject::signals` slice.
    /// - `receiver_id`: id of the object owning the slot.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::ConnectionTable;
    ///
    /// let table = ConnectionTable::new();
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
        self.connections.write().insert(id, record);
        self.by_receiver
            .write()
            .entry(receiver_id)
            .or_default()
            .push(id);
        self.by_signal
            .write()
            .entry((sender_id, signal_index))
            .or_default()
            .push(id);
        id
    }

    /// Removes a connection by id. Has no effect if `id` is not registered.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier returned by a previous [`register`](Self::register) call.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::ConnectionTable;
    ///
    /// let table = ConnectionTable::new();
    /// let id = table.register(ObjectId::new(), 0, ObjectId::new());
    /// table.remove(id);
    /// ```
    pub fn remove(&self, id: ConnectionId) {
        #[allow(
            clippy::significant_drop_in_scrutinee,
            reason = "connections write-guard must remain live across record extraction and the two by_receiver / by_signal lookups so a concurrent insert cannot observe a half-removed entry"
        )]
        if let Some(record) = self.connections.write().remove(&id) {
            if let Some(v) = self.by_receiver.write().get_mut(&record.receiver_id) {
                v.retain(|&c| c != id);
            }
            if let Some(v) = self
                .by_signal
                .write()
                .get_mut(&(record.sender_id, record.signal_index))
            {
                v.retain(|&c| c != id);
            }
        }
    }

    /// Removes all connections where `id` is the receiver. Called on object destroy.
    ///
    /// # Parameters
    ///
    /// - `id`: id of the receiver object whose slots are all being torn down.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::ConnectionTable;
    ///
    /// let table = ConnectionTable::new();
    /// let receiver_id = ObjectId::new();
    /// table.register(ObjectId::new(), 0, receiver_id);
    /// table.remove_by_receiver(receiver_id); // removes all slots for this receiver
    /// ```
    pub fn remove_by_receiver(&self, id: ObjectId) {
        let ids: Vec<ConnectionId> = self.by_receiver.write().remove(&id).unwrap_or_default();
        let mut conns = self.connections.write();
        let mut by_signal = self.by_signal.write();
        for cid in ids {
            if let Some(record) = conns.remove(&cid)
                && let Some(v) = by_signal.get_mut(&(record.sender_id, record.signal_index))
            {
                v.retain(|&c| c != cid);
            }
        }
    }

    /// Returns connection ids for a given `(sender, signal)` pair.
    ///
    /// # Parameters
    ///
    /// - `sender_id`: id of the object owning the signal.
    /// - `signal_index`: index of the signal in the sender's `MetaObject::signals` slice.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::ConnectionTable;
    ///
    /// let table = ConnectionTable::new();
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
            .get(&(sender_id, signal_index))
            .cloned()
            .unwrap_or_default()
    }
}

impl QueuedDispatcher for ConnectionTable {
    fn post(&self, target: std::thread::ThreadId, f: Box<dyn FnOnce() + Send + 'static>) {
        if let Some(el) = LoopRegistry::get(target) {
            el.post(f);
        } else {
            tracing::warn!(
                thread_id = ?target,
                "no EventLoop registered for target thread; queued invocation dropped"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_core::ObjectId;

    fn make_table() -> Arc<ConnectionTable> {
        ConnectionTable::new()
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
