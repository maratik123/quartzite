//! Extension trait for accessing parent/child relationships from any object.
use quartzite_core::{AsObject, ObjectId};

use crate::{global_tree, object_tree::ObjectTree};

/// Provides ergonomic access to parent and child relationships stored in the
/// process-wide [`ObjectTree`].
///
/// Automatically implemented for every type that implements [`AsObject`].
///
/// Methods without a `_in` suffix use the process-global tree registered by
/// [`Application::new`](crate::Application::new) and return `None` or an
/// empty collection when called outside an active [`Application`](crate::Application).
///
/// # Examples
///
/// ```no_run
/// use quartzite_core::AsObject;
/// use quartzite_runtime::{Application, ObjectTreeExt};
///
/// let _app = Application::new().unwrap();
/// # fn example(obj: &impl AsObject) {
/// let _parent = obj.parent();
/// let _children = obj.children();
/// # }
/// ```
pub trait ObjectTreeExt: AsObject {
    /// Returns the [`ObjectId`] of this object's parent in the active tree, or
    /// `None` when this object is a root or no [`Application`](crate::Application) is live.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::AsObject;
    /// use quartzite_runtime::ObjectTreeExt;
    ///
    /// # fn example(obj: &impl AsObject) {
    /// let _parent = obj.parent();
    /// # }
    /// ```
    fn parent(&self) -> Option<ObjectId> {
        let id = self.object_base().id();
        global_tree::try_with_tree(|tree| tree.parent_of(id)).flatten()
    }

    /// Returns the [`ObjectId`] of this object's parent in `tree`, or `None`
    /// when this object is a root.
    ///
    /// # Parameters
    ///
    /// - `tree`: the [`ObjectTree`] to query.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::AsObject;
    /// use quartzite_runtime::{ObjectTree, ObjectTreeExt};
    ///
    /// # fn example(obj: &impl AsObject, tree: &ObjectTree) {
    /// let _parent = obj.parent_in(tree);
    /// # }
    /// ```
    #[inline]
    fn parent_in(&self, tree: &ObjectTree) -> Option<ObjectId> {
        tree.parent_of(self.object_base().id())
    }

    /// Returns the [`ObjectId`]s of this object's children in insertion order,
    /// or an empty [`Vec`] when this object is a leaf or no [`Application`](crate::Application) is live.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::AsObject;
    /// use quartzite_runtime::ObjectTreeExt;
    ///
    /// # fn example(obj: &impl AsObject) {
    /// let _children = obj.children();
    /// # }
    /// ```
    fn children(&self) -> Vec<ObjectId> {
        let id = self.object_base().id();
        global_tree::try_with_tree(|tree| tree.children_of(id).to_vec()).unwrap_or_default()
    }

    /// Returns a slice of this object's children in insertion order, with
    /// lifetime tied to `tree`.
    ///
    /// # Parameters
    ///
    /// - `tree`: the [`ObjectTree`] to query.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::{AsObject, ObjectId};
    /// use quartzite_runtime::{ObjectTree, ObjectTreeExt};
    ///
    /// # fn example(obj: &impl AsObject, tree: &ObjectTree) {
    /// let _children: &[ObjectId] = obj.children_in(tree);
    /// # }
    /// ```
    #[inline]
    fn children_in<'t>(&self, tree: &'t ObjectTree) -> &'t [ObjectId] {
        tree.children_of(self.object_base().id())
    }
}

impl<T: AsObject> ObjectTreeExt for T {}
