//! Extension trait for accessing parent/child relationships from any object.
use quartzite_core::{AsObject, ObjectId};

use crate::{
    application::{TreeAccessError, try_with_tree},
    object_tree::ObjectTree,
};

/// Provides ergonomic access to parent and child relationships stored in the
/// process-wide [`ObjectTree`].
///
/// Automatically implemented for every type that implements [`AsObject`].
///
/// Methods without a `_in` suffix use the process-global tree registered by
/// [`Application::new`](crate::Application::new) and return
/// [`Err`]`(`[`TreeAccessError`]`)` when called outside an active
/// [`Application`](crate::Application).
///
/// # Examples
///
/// ```no_run
/// use quartzite_core::AsObject;
/// use quartzite_runtime::{Application, ObjectTreeExt};
///
/// let _app = Application::new().unwrap();
/// # fn example(obj: &impl AsObject) -> Result<(), quartzite_runtime::TreeAccessError> {
/// let _parent = obj.parent()?;
/// let _children = obj.children()?;
/// # Ok(()) }
/// ```
#[allow(
    clippy::doc_link_code,
    reason = "adjacency-to-(args) pattern: renders Err(TreeAccessError) with both identifiers intra-doc-linked; flattening to [Err]([TreeAccessError]) would drop the surrounding code styling"
)]
pub trait ObjectTreeExt: AsObject {
    /// Returns the [`ObjectId`] of this object's parent in the active tree, or
    /// `Ok(None)` when this object is a root.
    ///
    /// # Errors
    ///
    /// Returns [`TreeAccessError`] when no [`Application`](crate::Application) is live.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::AsObject;
    /// use quartzite_runtime::ObjectTreeExt;
    ///
    /// # fn example(obj: &impl AsObject) -> Result<(), quartzite_runtime::TreeAccessError> {
    /// let _parent = obj.parent()?;
    /// # Ok(()) }
    /// ```
    fn parent(&self) -> Result<Option<ObjectId>, TreeAccessError> {
        let id = self.object_base().id();
        try_with_tree(|tree| tree.parent_of(id))
    }

    /// Returns the [`ObjectId`] of this object's parent in `tree`, or `None`
    /// when this object is a root.
    ///
    /// _Simple._
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
    fn parent_in(&self, tree: &ObjectTree) -> Option<ObjectId> {
        tree.parent_of(self.object_base().id())
    }

    /// Returns the [`ObjectId`]s of this object's children in insertion order.
    ///
    /// # Errors
    ///
    /// Returns [`TreeAccessError`] when no [`Application`](crate::Application) is live.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::AsObject;
    /// use quartzite_runtime::ObjectTreeExt;
    ///
    /// # fn example(obj: &impl AsObject) -> Result<(), quartzite_runtime::TreeAccessError> {
    /// let _children = obj.children()?;
    /// # Ok(()) }
    /// ```
    fn children(&self) -> Result<Vec<ObjectId>, TreeAccessError> {
        let id = self.object_base().id();
        try_with_tree(|tree| tree.children_of(id).to_vec())
    }

    /// Returns a slice of this object's children in insertion order, with
    /// lifetime tied to `tree`.
    ///
    /// _Simple._
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
    fn children_in<'t>(&self, tree: &'t ObjectTree) -> &'t [ObjectId] {
        tree.children_of(self.object_base().id())
    }
}

impl<T: AsObject> ObjectTreeExt for T {}
