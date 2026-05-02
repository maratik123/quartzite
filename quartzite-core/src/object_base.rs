//! Base state shared by every quartzite object.
#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};

use crate::{
    id::{ConnectionId, ObjectId},
    receiver_guard::ReceiverGuard,
    value::Value,
};

/// Core data carried by every object in the quartzite object tree.
///
/// `ObjectBase` provides identity ([`ObjectId`]), optional name, thread-affinity
/// tracking, a lifetime token for safe signal delivery ([`ReceiverGuard`]), and
/// storage for dynamic properties and outgoing connection bookkeeping.
///
/// Objects are typically created through a higher-level type that includes an
/// `ObjectBase` field and derives `Extend` (from `quartzite-macros`).
///
/// # Examples
///
/// ```
/// use quartzite_core::ObjectBase;
///
/// let base = ObjectBase::named("my-button");
/// assert_eq!(base.name(), Some("my-button"));
/// assert!(!base.signals_blocked);
/// ```
pub struct ObjectBase {
    /// Private: uniqueness invariant — must never be overwritten after construction.
    id: ObjectId,
    /// Human-readable name used for debugging and lookup in the object tree.
    /// `None` for anonymous objects; access via [`ObjectBase::name`].
    name: Option<String>,
    /// Private: lifetime token — Arc is dropped when the object is dropped, invalidating
    /// all `Weak<ReceiverGuard>` held by queued connections.
    receiver_guard: Arc<ReceiverGuard>,
    /// Tracks outgoing signal connections; populated and managed by `quartzite-runtime`.
    ///
    /// `ObjectBase` itself does not modify this field.
    pub outgoing_connections: Vec<ConnectionId>,
    /// Runtime property bag for properties added outside the compile-time schema.
    pub dynamic_properties: BTreeMap<String, Value>,
    /// When `true`, emitting any signal on this object is a no-op.
    pub signals_blocked: bool,
    /// The thread on which this object was created; used by `connect_auto` to decide
    /// between direct and queued delivery.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub thread_id: std::thread::ThreadId,
}

impl ObjectBase {
    /// Creates an anonymous `ObjectBase` with a freshly allocated [`ObjectId`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectBase;
    ///
    /// let base = ObjectBase::new();
    /// assert!(base.name().is_none());
    /// assert!(!base.signals_blocked);
    /// ```
    pub fn new() -> Self {
        let (guard, _) = ReceiverGuard::new_pair();
        Self {
            id: ObjectId::new(),
            name: None,
            receiver_guard: guard,
            outgoing_connections: Vec::new(),
            dynamic_properties: BTreeMap::new(),
            signals_blocked: false,
            #[cfg(feature = "std")]
            thread_id: std::thread::current().id(),
        }
    }

    /// Creates an `ObjectBase` with the given name and a freshly allocated [`ObjectId`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectBase;
    ///
    /// let base = ObjectBase::named("sensor-1");
    /// assert_eq!(base.name(), Some("sensor-1"));
    /// ```
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Self::new()
        }
    }

    /// Returns the name of this object, or `None` if it is anonymous.
    ///
    /// To rename or clear the name at runtime, use `ObjectTree::rename` or
    /// `ObjectTree::clear_name` (from `quartzite-runtime`).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectBase;
    ///
    /// assert!(ObjectBase::new().name().is_none());
    /// assert_eq!(ObjectBase::named("btn").name(), Some("btn"));
    /// ```
    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name directly on this base (low-level, used by `ObjectTree::rename`/`clear_name`).
    ///
    /// Prefer calling `ObjectTree::rename` or `ObjectTree::clear_name` in production code
    /// so the name index stays consistent.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectBase;
    ///
    /// let mut base = ObjectBase::new();
    /// base.set_name_raw(Some("widget".into()));
    /// assert_eq!(base.name(), Some("widget"));
    /// base.set_name_raw(None);
    /// assert!(base.name().is_none());
    /// ```
    #[inline]
    pub fn set_name_raw(&mut self, name: Option<String>) {
        self.name = name;
    }

    /// Returns the unique identifier for this object.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectBase;
    ///
    /// let base = ObjectBase::new();
    /// assert!(base.id().raw() > 0);
    /// ```
    #[inline]
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Returns a reference to the receiver guard (lifetime token for signal delivery).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectBase;
    ///
    /// let base = ObjectBase::new();
    /// let guard = base.receiver_guard();
    /// assert_eq!(std::sync::Arc::strong_count(guard), 1);
    /// ```
    #[inline]
    pub fn receiver_guard(&self) -> &Arc<ReceiverGuard> {
        &self.receiver_guard
    }

    /// Returns `true` if this object was created on the calling thread.
    ///
    /// Used by `connect_auto` to decide between direct and queued signal delivery.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectBase;
    ///
    /// let base = ObjectBase::new();
    /// assert!(base.is_on_current_thread());
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn is_on_current_thread(&self) -> bool {
        self.thread_id == std::thread::current().id()
    }
}

impl Default for ObjectBase {
    #[inline]
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
    fn new_object_name_is_none() {
        let base = ObjectBase::new();
        assert!(base.name().is_none());
    }

    #[test]
    fn named_sets_name() {
        let base = ObjectBase::named("foo");
        assert_eq!(base.name(), Some("foo"));
    }

    #[test]
    fn each_new_gets_unique_id() {
        let a = ObjectBase::new();
        let b = ObjectBase::new();
        assert_ne!(a.id(), b.id());
    }
}
