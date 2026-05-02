#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};

use crate::{
    id::{ConnectionId, ObjectId},
    receiver_guard::ReceiverGuard,
    value::Value,
};

pub struct ObjectBase {
    /// Private: uniqueness invariant — must never be overwritten after construction.
    id: ObjectId,
    pub name: String,
    /// Private: lifetime token — Arc is dropped when the object is dropped, invalidating
    /// all `Weak<ReceiverGuard>` held by queued connections.
    receiver_guard: Arc<ReceiverGuard>,
    /// Reserved for use by `quartzite-runtime` to track outgoing signal connections.
    /// `ObjectBase` itself does not populate this field.
    pub outgoing_connections: Vec<ConnectionId>,
    pub dynamic_properties: BTreeMap<String, Value>,
    pub signals_blocked: bool,
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub thread_id: std::thread::ThreadId,
}

impl ObjectBase {
    pub fn new() -> Self {
        let (guard, _) = ReceiverGuard::new_pair();
        Self {
            id: ObjectId::new(),
            name: String::new(),
            receiver_guard: guard,
            outgoing_connections: Vec::new(),
            dynamic_properties: BTreeMap::new(),
            signals_blocked: false,
            #[cfg(feature = "std")]
            thread_id: std::thread::current().id(),
        }
    }

    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::new()
        }
    }

    /// Returns the unique identifier for this object.
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Returns a reference to the receiver guard (lifetime token for signal delivery).
    pub fn receiver_guard(&self) -> &Arc<ReceiverGuard> {
        &self.receiver_guard
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn is_on_current_thread(&self) -> bool {
        self.thread_id == std::thread::current().id()
    }
}

impl Default for ObjectBase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn new_records_thread_id() {
        let base = ObjectBase::new();
        assert!(base.is_on_current_thread());
    }

    #[test]
    fn signals_blocked_default_false() {
        let base = ObjectBase::new();
        assert!(!base.signals_blocked);
    }

    #[test]
    fn dynamic_properties_empty_on_new() {
        let base = ObjectBase::new();
        assert!(base.dynamic_properties.is_empty());
    }

    #[test]
    fn named_sets_name() {
        let base = ObjectBase::named("foo");
        assert_eq!(base.name, "foo");
    }

    #[test]
    fn each_new_gets_unique_id() {
        let a = ObjectBase::new();
        let b = ObjectBase::new();
        assert_ne!(a.id(), b.id());
    }
}
