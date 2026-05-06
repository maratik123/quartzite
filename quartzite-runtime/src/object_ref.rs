//! Typed handles to runtime objects: `ObjectRef<T>` (live) and `WeakRef<T>` (unvalidated).
use std::marker::PhantomData;

use quartzite_core::ObjectId;

use crate::object_tree::ObjectTree;

/// Typed, guaranteed-live handle to an object in the tree.
///
/// The lifetime guarantee is by convention — the `ObjectId` must still be
/// present in the tree when the ref is used. Use [`WeakRef::is_valid`] to
/// confirm liveness before use.
///
/// # Examples
///
/// ```
/// use quartzite_core::ObjectId;
/// use quartzite_runtime::ObjectRef;
///
/// let id = ObjectId::new();
/// let r: ObjectRef<()> = ObjectRef::new(id);
/// assert_eq!(r.id(), id);
/// ```
#[derive(Debug)]
pub struct ObjectRef<T> {
    id: ObjectId,
    _marker: PhantomData<fn() -> T>,
}

impl<T> ObjectRef<T> {
    /// Wraps `id` in a typed `ObjectRef`.
    ///
    /// No liveness check is performed here; the caller must ensure the object
    /// exists in the tree.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the live object this ref refers to.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::ObjectRef;
    ///
    /// let id = ObjectId::new();
    /// let r: ObjectRef<()> = ObjectRef::new(id);
    /// assert_eq!(r.id(), id);
    /// ```
    #[inline]
    pub fn new(id: ObjectId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Returns the underlying [`ObjectId`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::ObjectRef;
    ///
    /// let id = ObjectId::new();
    /// let r: ObjectRef<()> = ObjectRef::new(id);
    /// assert_eq!(r.id(), id);
    /// ```
    #[inline]
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Converts this `ObjectRef` into a [`WeakRef`] with no liveness guarantee.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::ObjectRef;
    ///
    /// let id = ObjectId::new();
    /// let r: ObjectRef<()> = ObjectRef::new(id);
    /// let w = r.downgrade();
    /// assert_eq!(w.id(), id);
    /// ```
    #[inline]
    pub fn downgrade(self) -> WeakRef<T> {
        WeakRef {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for ObjectRef<T> {}
impl<T> Clone for ObjectRef<T> {
    // _Simple._
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for ObjectRef<T> {
    // _Simple._
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for ObjectRef<T> {}

impl<T> std::hash::Hash for ObjectRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Typed handle with no liveness guarantee.
///
/// Call [`is_valid`](Self::is_valid) to check whether the referenced object still
/// exists in the tree before use.
///
/// # Examples
///
/// ```
/// use quartzite_core::ObjectId;
/// use quartzite_runtime::WeakRef;
///
/// let id = ObjectId::new();
/// let w: WeakRef<()> = WeakRef::new(id);
/// assert_eq!(w.id(), id);
/// ```
#[derive(Debug)]
pub struct WeakRef<T> {
    id: ObjectId,
    _marker: PhantomData<fn() -> T>,
}

impl<T> WeakRef<T> {
    /// Wraps `id` in a typed `WeakRef`.
    ///
    /// No liveness check is performed here. Use [`is_valid`](Self::is_valid) before
    /// accessing the object.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the (possibly dropped) object this ref refers to.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::WeakRef;
    ///
    /// let id = ObjectId::new();
    /// let w: WeakRef<()> = WeakRef::new(id);
    /// assert_eq!(w.id(), id);
    /// ```
    #[inline]
    pub fn new(id: ObjectId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Returns the underlying [`ObjectId`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::WeakRef;
    ///
    /// let id = ObjectId::new();
    /// let w: WeakRef<()> = WeakRef::new(id);
    /// assert_eq!(w.id(), id);
    /// ```
    #[inline]
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Returns `true` if the referenced object is still present in `tree`.
    ///
    /// # Parameters
    ///
    /// - `tree`: object tree to query for liveness of this ref's id.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::{ObjectTree, WeakRef};
    ///
    /// let tree = ObjectTree::new();
    /// let id = ObjectId::new();
    /// let w: WeakRef<()> = WeakRef::new(id);
    /// assert!(!w.is_valid(&tree)); // id was never inserted
    /// ```
    #[inline]
    pub fn is_valid(&self, tree: &ObjectTree) -> bool {
        tree.contains(self.id)
    }
}

impl<T> Copy for WeakRef<T> {}
impl<T> Clone for WeakRef<T> {
    // _Simple._
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for WeakRef<T> {
    // _Simple._
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for WeakRef<T> {}

impl<T> std::hash::Hash for WeakRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_core::ObjectId;

    #[test]
    fn object_ref_copy_eq_hash() {
        let id = ObjectId::new();
        let r: ObjectRef<()> = ObjectRef::new(id);
        let r2 = r;
        assert_eq!(r, r2);
        assert_eq!(r.id(), id);
    }

    #[test]
    fn weak_ref_copy_eq_hash() {
        let id = ObjectId::new();
        let w: WeakRef<()> = WeakRef::new(id);
        let w2 = w;
        assert_eq!(w, w2);
        assert_eq!(w.id(), id);
    }

    #[test]
    fn downgrade_preserves_id() {
        let id = ObjectId::new();
        let r: ObjectRef<()> = ObjectRef::new(id);
        let w = r.downgrade();
        assert_eq!(w.id(), id);
    }
}
