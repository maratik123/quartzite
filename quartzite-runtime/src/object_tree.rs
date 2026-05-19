//! Hierarchical store for all runtime objects.
use std::collections::{HashMap, VecDeque};

use quartzite_core::{ObjectId, Value, traits::Object};
use slotmap::{DefaultKey, SlotMap};
use tracing::{debug_span, trace_span};

use crate::object_id::SlotKey;

/// Central store for all runtime objects.
///
/// Owns objects via a `SlotMap`. Maintains two identity maps (forward and
/// reverse) and two relationship maps (parent and children). All mutations
/// go through the methods here — no caller ever modifies the maps directly.
///
/// `ObjectTree: Send` follows automatically from `Object: Send`. Callers
/// requiring shared access should wrap the tree in `Mutex<ObjectTree>`.
///
/// # Examples
///
/// ```
/// use quartzite_runtime::ObjectTree;
///
/// let tree = ObjectTree::new();
/// assert!(!tree.contains(quartzite_core::ObjectId::new()));
/// ```
#[derive(Default)]
pub struct ObjectTree {
    store: SlotMap<DefaultKey, Box<dyn Object>>,
    /// `ObjectId` → arena slot (forward index)
    forward: HashMap<ObjectId, SlotKey>,
    /// arena slot → `ObjectId` (reverse index, for destroy traversal)
    reverse: HashMap<SlotKey, ObjectId>,
    /// child `ObjectId` → parent `ObjectId`
    parent_map: HashMap<ObjectId, ObjectId>,
    /// parent `ObjectId` → ordered children list
    children_map: HashMap<ObjectId, Vec<ObjectId>>,
    /// name → list of `ObjectId`s with that name (supports multiple objects sharing a name)
    by_name: HashMap<String, Vec<ObjectId>>,
}

impl ObjectTree {
    /// Creates an empty `ObjectTree`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_runtime::ObjectTree;
    ///
    /// let tree = ObjectTree::new();
    /// assert!(!tree.contains(ObjectId::new()));
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts `obj` into the tree, optionally under `parent_id`. Returns the
    /// [`ObjectId`] of the inserted object.
    ///
    /// # Parameters
    ///
    /// - `obj`: object to insert; ownership is transferred to the tree.
    /// - `parent_id`: optional parent under which to insert; `None` makes a root node.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// // tree.insert(Box::new(my_object), None);
    /// ```
    pub fn insert(&mut self, obj: Box<dyn Object>, parent_id: Option<ObjectId>) -> ObjectId {
        let id = obj.object_base().id();
        let _span =
            debug_span!("object_tree::insert", object_id = ?id, parent_id = ?parent_id).entered();
        // Register in the by_name index if the object has a name.
        if let Some(name) = obj.object_base().name() {
            self.by_name.entry(name.to_owned()).or_default().push(id);
        }
        let slot = SlotKey(self.store.insert(obj));
        self.forward.insert(id, slot);
        self.reverse.insert(slot, id);
        self.children_map.entry(id).or_default();
        if let Some(pid) = parent_id {
            self.parent_map.insert(id, pid);
            self.children_map.entry(pid).or_default().push(id);
        }
        id
    }

    /// Returns `true` if `id` is present in the tree.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier to look up in the forward index.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_runtime::ObjectTree;
    ///
    /// let tree = ObjectTree::new();
    /// assert!(!tree.contains(quartzite_core::ObjectId::new()));
    /// ```
    #[inline]
    pub fn contains(&self, id: ObjectId) -> bool {
        self.forward.contains_key(&id)
    }

    /// Runs `f` with a shared reference to the object identified by `id`. Returns
    /// `None` if `id` is not in the tree.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the object to access.
    /// - `f`: closure invoked with a shared reference to the object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &ObjectTree, id: quartzite_core::ObjectId) {
    /// let name = tree.with(id, |obj| obj.object_base().name().unwrap_or("").to_owned());
    /// # }
    /// ```
    pub fn with<R, F>(&self, id: ObjectId, f: F) -> Option<R>
    where
        F: FnOnce(&dyn Object) -> R,
    {
        let slot = self.forward.get(&id)?;
        self.store.get(slot.0).map(|obj| f(obj.as_ref()))
    }

    /// Runs `f` with an exclusive reference to the object identified by `id`. Returns
    /// `None` if `id` is not in the tree.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the object to access.
    /// - `f`: closure invoked with an exclusive reference to the object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// use quartzite_core::value::Value;
    /// # fn example(tree: &mut ObjectTree, id: quartzite_core::ObjectId) {
    /// tree.with_mut(id, |obj| { obj.write_property("count", Value::Int(0)); });
    /// # }
    /// ```
    pub fn with_mut<R, F>(&mut self, id: ObjectId, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Object) -> R,
    {
        let slot = self.forward.get(&id)?;
        self.store.get_mut(slot.0).map(|obj| f(obj.as_mut()))
    }

    /// Returns the parent [`ObjectId`] of `id`, or `None` if `id` is a root.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the child whose parent is requested.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &ObjectTree, child_id: quartzite_core::ObjectId) {
    /// if let Some(parent_id) = tree.parent_of(child_id) {
    ///     // use parent_id
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn parent_of(&self, id: ObjectId) -> Option<ObjectId> {
        self.parent_map.get(&id).copied()
    }

    /// Returns the ordered list of children of `id`.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the parent whose children are requested.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &ObjectTree, parent_id: quartzite_core::ObjectId) {
    /// let children: &[quartzite_core::ObjectId] = tree.children_of(parent_id);
    /// # }
    /// ```
    #[inline]
    pub fn children_of(&self, id: ObjectId) -> &[ObjectId] {
        self.children_map
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Moves `id` to a new parent. If `new_parent` is `None`, `id` becomes a root.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the object being moved.
    /// - `new_parent`: optional new parent; `None` detaches `id` to become a root.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &mut ObjectTree, id: quartzite_core::ObjectId) {
    /// tree.reparent(id, None); // detach from its current parent
    /// # }
    /// ```
    pub fn reparent(&mut self, id: ObjectId, new_parent: Option<ObjectId>) {
        let _span =
            debug_span!("object_tree::reparent", object_id = ?id, new_parent_id = ?new_parent)
                .entered();
        self.detach_from_parent(id);
        if let Some(pid) = new_parent {
            self.parent_map.insert(id, pid);
            self.children_map.entry(pid).or_default().push(id);
        }
    }

    fn detach_from_parent(&mut self, id: ObjectId) {
        if let Some(old_parent) = self.parent_map.remove(&id)
            && let Some(siblings) = self.children_map.get_mut(&old_parent)
        {
            siblings.retain(|&c| c != id);
        }
    }

    /// Returns the slice of `ObjectId`s for objects with the given `name`. The slice
    /// is empty if no object has that name.
    ///
    /// # Parameters
    ///
    /// - `name`: name to look up in the by-name index; matched exactly.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &ObjectTree) {
    /// let ids: &[quartzite_core::ObjectId] = tree.find_by_name("my-button");
    /// # }
    /// ```
    #[inline]
    pub fn find_by_name(&self, name: &str) -> &[ObjectId] {
        self.by_name
            .get(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Returns all [`ObjectId`]s within the subtree rooted at `root` whose name equals
    /// `name`, sorted shallowest first (ascending depth from `root`). `root` itself is
    /// included if its name matches (depth 0). Ties at the same depth preserve
    /// children-insertion order.
    ///
    /// Returns an empty `Vec` when `root` is not in the tree or no matching object exists
    /// within the subtree.
    ///
    /// # Parameters
    ///
    /// - `root`: the subtree root to search within (inclusive).
    /// - `name`: name to match exactly.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &ObjectTree, root: quartzite_core::ObjectId) {
    /// let ids = tree.find_by_name_in(root, "my-button");
    /// // ids[0] is the shallowest match; ids.last() is the deepest.
    /// # }
    /// ```
    pub fn find_by_name_in(&self, root: ObjectId, name: &str) -> Vec<ObjectId> {
        if !self.contains(root) {
            return Vec::new();
        }
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(root);
        while let Some(id) = queue.pop_front() {
            if self.with(id, |obj| obj.object_base().name() == Some(name)) == Some(true) {
                result.push(id);
            }
            if let Some(children) = self.children_map.get(&id) {
                queue.extend(children.iter().copied());
            }
        }
        result
    }

    /// Renames the object `id` to `new_name`, updating the name index.
    ///
    /// Has no effect if `id` is not in the tree.
    ///
    /// _Simple._
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the object to rename.
    /// - `new_name`: new name; replaces any existing name and updates the by-name index.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &mut ObjectTree, id: quartzite_core::ObjectId) {
    /// tree.rename(id, "new-name");
    /// assert_eq!(tree.find_by_name("new-name"), &[id]);
    /// # }
    /// ```
    pub fn rename(&mut self, id: ObjectId, new_name: impl Into<String>) {
        fn inner(this: &mut ObjectTree, id: ObjectId, new_name: String) {
            let _span =
                trace_span!("object_tree::rename", object_id = ?id, new_name = %new_name).entered();
            // Remove from old name bucket; capture old name for signal payload.
            let old_name_opt: Option<String>;
            if let Some(old_name) = this.with(id, |obj| obj.object_base().name().map(str::to_owned))
            {
                if let Some(ref old_name) = old_name {
                    if *old_name == new_name {
                        return; // no-op: name unchanged
                    }
                    ObjectTree::remove_from_by_name(&mut this.by_name, old_name, id);
                }
                old_name_opt = old_name;
            } else {
                return; // id not in tree
            }
            // Update the object's name.
            this.with_mut(id, |obj| {
                obj.object_base_mut().set_name_raw(Some(new_name.clone()));
            });
            // Insert into new name bucket.
            this.by_name.entry(new_name.clone()).or_default().push(id);
            // Emit name_changed after index is consistent. old = None means was anonymous.
            let old_val = old_name_opt.map_or(Value::Null, Value::String);
            this.with_mut(id, |obj| {
                obj.emit_signal("name_changed", &[old_val, Value::String(new_name)]);
            });
        }
        inner(self, id, new_name.into());
    }

    /// Clears the name of object `id`, making it anonymous and removing it from the
    /// name index.
    ///
    /// Has no effect if `id` is not in the tree.
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the object whose name is being cleared.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &mut ObjectTree, id: quartzite_core::ObjectId) {
    /// tree.clear_name(id);
    /// # }
    /// ```
    pub fn clear_name(&mut self, id: ObjectId) {
        let _span = trace_span!("object_tree::clear_name", object_id = ?id).entered();
        // Remove from old name bucket.
        let old_name = self.with(id, |obj| obj.object_base().name().map(str::to_owned));
        match old_name {
            // None: id not in tree. Some(None): already anonymous — no state change, no signal.
            None | Some(None) => (),
            Some(Some(old_name)) => {
                Self::remove_from_by_name(&mut self.by_name, &old_name, id);
                self.with_mut(id, |obj| obj.object_base_mut().set_name_raw(None));
                // Emit name_changed after index is consistent.
                self.with_mut(id, |obj| {
                    obj.emit_signal("name_changed", &[Value::String(old_name), Value::Null]);
                });
            }
        }
    }

    /// Removes `id` and all its descendants from the tree (depth-first post-order).
    ///
    /// # Parameters
    ///
    /// - `id`: identifier of the root of the subtree to destroy.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &mut ObjectTree, id: quartzite_core::ObjectId) {
    /// tree.destroy(id);
    /// assert!(!tree.contains(id));
    /// # }
    /// ```
    pub fn destroy(&mut self, id: ObjectId) {
        let _span = debug_span!("object_tree::destroy", object_id = ?id).entered();
        // Collect subtree in depth-first post-order (leaves first).
        let mut order: Vec<ObjectId> = Vec::new();
        self.collect_post_order(id, &mut order);

        for node_id in order {
            self.remove_node(node_id);
        }
    }

    fn collect_post_order(&self, id: ObjectId, out: &mut Vec<ObjectId>) {
        if let Some(children) = self.children_map.get(&id) {
            for &child in children {
                self.collect_post_order(child, out);
            }
        }
        out.push(id);
    }

    fn remove_node(&mut self, id: ObjectId) {
        if let Some(slot) = self.forward.remove(&id) {
            // Remove from by_name index before dropping the object.
            if let Some(obj) = self.store.get(slot.0)
                && let Some(name) = obj.object_base().name()
            {
                Self::remove_from_by_name(&mut self.by_name, name, id);
            }
            self.store.remove(slot.0);
            self.reverse.remove(&slot);
        }
        self.detach_from_parent(id);
        self.children_map.remove(&id);
    }

    /// Remove `id` from the `by_name` bucket for `name`. Removes the bucket if empty.
    fn remove_from_by_name(by_name: &mut HashMap<String, Vec<ObjectId>>, name: &str, id: ObjectId) {
        if let Some(ids) = by_name.get_mut(name) {
            ids.retain(|&x| x != id);
            if ids.is_empty() {
                by_name.remove(name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use itertools::{Itertools, assert_equal};
    use quartzite_core::{
        id::ConnectionId,
        meta::MetaObject,
        object_base::ObjectBase,
        signal::ConnectionType,
        traits::{AsObject, Object, SignalCallback},
        value::Value,
    };

    struct StubObject {
        base: ObjectBase,
    }

    impl StubObject {
        fn named(name: &str) -> Box<dyn Object> {
            Box::new(Self {
                base: ObjectBase::named(name),
            })
        }
    }

    impl AsObject for StubObject {
        fn object_base(&self) -> &ObjectBase {
            &self.base
        }
        fn object_base_mut(&mut self) -> &mut ObjectBase {
            &mut self.base
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    static STUB_META: MetaObject = MetaObject::new(
        "StubObject",
        &[],
        &[],
        &[],
        &[],
        quartzite_core::meta::noop_lookup_property,
        quartzite_core::meta::noop_lookup_signal,
        quartzite_core::meta::noop_lookup_method,
        quartzite_core::meta::noop_lookup_enum,
    );

    impl Object for StubObject {
        fn meta_object(&self) -> &'static MetaObject {
            &STUB_META
        }
        fn read_property(&self, _: &str) -> Option<Value> {
            None
        }
        fn write_property(&mut self, _: &str, _: Value) -> bool {
            false
        }
        fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
            None
        }
        fn connect_signal(
            &mut self,
            _: &str,
            _: SignalCallback,
            _: ConnectionType,
        ) -> Option<ConnectionId> {
            None
        }

        fn emit_signal(&mut self, _: &str, _: &[quartzite_core::Value]) -> Option<()> {
            None
        }
    }

    #[test]
    fn insert_returns_id_and_with_finds_it() {
        let mut tree = ObjectTree::new();
        let obj = StubObject::named("alpha");
        let id = tree.insert(obj, None);
        let found = tree.with(id, |o| o.object_base().name().unwrap_or("").to_owned());
        assert_eq!(found, Some("alpha".to_string()));
    }

    #[test]
    fn contains_true_after_insert_false_after_destroy() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("x"), None);
        assert!(tree.contains(id));
        tree.destroy(id);
        assert!(!tree.contains(id));
    }

    #[test]
    fn with_returns_none_for_unknown_id() {
        let tree = ObjectTree::new();
        let id = ObjectId::new();
        assert!(tree.with(id, |_| ()).is_none());
    }

    #[test]
    fn destroy_removes_all_descendants() {
        let mut tree = ObjectTree::new();
        let root = tree.insert(StubObject::named("root"), None);
        let child = tree.insert(StubObject::named("child"), Some(root));
        let grandchild = tree.insert(StubObject::named("gc"), Some(child));
        tree.destroy(root);
        assert!(!tree.contains(root));
        assert!(!tree.contains(child));
        assert!(!tree.contains(grandchild));
    }

    #[test]
    fn parent_child_reflected_in_accessors() {
        let mut tree = ObjectTree::new();
        let parent = tree.insert(StubObject::named("p"), None);
        let child = tree.insert(StubObject::named("c"), Some(parent));
        assert_eq!(tree.parent_of(child), Some(parent));
        assert!(tree.children_of(parent).contains(&child));
    }

    #[test]
    fn find_by_name_returns_correct_id() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("foo"), None);
        tree.insert(StubObject::named("bar"), None);
        assert_eq!(tree.find_by_name("foo"), &[id]);
    }

    #[test]
    fn find_by_name_returns_empty_when_absent() {
        let tree = ObjectTree::new();
        assert!(tree.find_by_name("missing").is_empty());
    }

    #[test]
    fn stale_id_after_destroy_returns_none() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("gone"), None);
        tree.destroy(id);
        assert!(tree.with(id, |_| ()).is_none());
    }

    #[test]
    fn reparent_updates_both_parent_and_children() {
        let mut tree = ObjectTree::new();
        let p1 = tree.insert(StubObject::named("p1"), None);
        let p2 = tree.insert(StubObject::named("p2"), None);
        let child = tree.insert(StubObject::named("c"), Some(p1));

        tree.reparent(child, Some(p2));

        assert_eq!(tree.parent_of(child), Some(p2));
        assert!(!tree.children_of(p1).contains(&child));
        assert!(tree.children_of(p2).contains(&child));
    }

    // --- by_name index tests ---

    fn new_unnamed() -> Box<dyn Object> {
        use quartzite_core::object_base::ObjectBase;
        Box::new(StubObject {
            base: ObjectBase::new(),
        })
    }

    #[test]
    fn new_object_name_is_none() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(new_unnamed(), None);
        let name = tree.with(id, |o| o.object_base().name().map(str::to_owned));
        assert_eq!(name, Some(None));
    }

    #[test]
    fn named_object_name_is_some() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("btn"), None);
        let name = tree.with(id, |o| o.object_base().name().map(str::to_owned));
        assert_eq!(name, Some(Some("btn".to_owned())));
    }

    #[test]
    fn rename_updates_index() {
        let mut tree = ObjectTree::new();
        // Start from an unnamed object (None) to exercise the None→Some path.
        let id = tree.insert(new_unnamed(), None);
        tree.rename(id, "new");
        assert_eq!(tree.find_by_name("new"), &[id]);
    }

    #[test]
    fn rename_old_name_removed() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("alpha"), None);
        tree.rename(id, "beta");
        assert!(!tree.find_by_name("alpha").contains(&id));
    }

    #[test]
    fn rename_same_name_is_noop_object_still_found() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("same"), None);
        tree.rename(id, "same");
        assert_eq!(tree.find_by_name("same"), &[id]);
    }

    #[test]
    fn rename_same_name_does_not_duplicate_index_entry() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("dup-check"), None);
        tree.rename(id, "dup-check");
        tree.rename(id, "dup-check");
        assert_eq!(tree.find_by_name("dup-check").len(), 1);
    }

    #[test]
    fn rename_empty_string_to_same_empty_string_is_noop() {
        // Object with name Some("") renamed to "" must be a no-op.
        // Distinct from anonymous (name = None) renamed to "" which is a real rename.
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named(""), None);
        tree.rename(id, "");
        assert_eq!(tree.find_by_name("").len(), 1);
        assert_eq!(tree.find_by_name(""), &[id]);
    }

    #[test]
    fn rename_anonymous_to_empty_string_is_real_rename() {
        // name = None → "" is not a no-op.
        let mut tree = ObjectTree::new();
        let id = tree.insert(new_unnamed(), None);
        tree.rename(id, "");
        assert_eq!(tree.find_by_name(""), &[id]);
    }

    #[test]
    fn rename_same_name_preserves_shared_bucket_order() {
        // Without the no-op guard, rename(id1, "shared") would remove id1 from the
        // bucket then re-append it, producing [id2, id1] instead of [id1, id2].
        let mut tree = ObjectTree::new();
        let id1 = tree.insert(StubObject::named("shared"), None);
        let id2 = tree.insert(StubObject::named("shared"), None);
        tree.rename(id1, "shared"); // no-op
        let ids = tree.find_by_name("shared");
        assert_eq!(
            ids,
            &[id1, id2],
            "insertion order must not change on no-op rename"
        );
    }

    #[test]
    fn rename_different_name_still_works_after_noop_guard() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("before"), None);
        tree.rename(id, "before"); // no-op
        tree.rename(id, "after");
        assert!(tree.find_by_name("before").is_empty());
        assert_eq!(tree.find_by_name("after"), &[id]);
    }

    #[test]
    fn rename_unknown_id_is_noop() {
        let mut tree = ObjectTree::new();
        let unknown = ObjectId::new();
        tree.rename(unknown, "ghost"); // must not panic
        assert!(tree.find_by_name("ghost").is_empty());
    }

    #[test]
    fn clear_name_removes_from_index() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("named"), None);
        tree.clear_name(id);
        assert!(tree.find_by_name("named").is_empty());
        let name = tree.with(id, |o| o.object_base().name().map(str::to_owned));
        assert_eq!(name, Some(None));
    }

    #[test]
    fn find_by_name_returns_all_with_same_name() {
        let mut tree = ObjectTree::new();
        let a = tree.insert(StubObject::named("dup"), None);
        let b = tree.insert(StubObject::named("dup"), None);
        let ids = tree.find_by_name("dup");
        assert_equal(
            ids.iter().sorted_unstable(),
            [a, b].iter().sorted_unstable(),
        );
    }

    #[test]
    fn find_by_name_empty_string_vs_unnamed() {
        let mut tree = ObjectTree::new();
        // An object named "" (empty string) is different from an anonymous object.
        let named_empty = tree.insert(StubObject::named(""), None);
        let _anon = tree.insert(new_unnamed(), None);
        let ids = tree.find_by_name("");
        assert_eq!(ids, &[named_empty]);
    }

    #[test]
    fn destroy_removes_from_by_name() {
        let mut tree = ObjectTree::new();
        let id = tree.insert(StubObject::named("doomed"), None);
        tree.destroy(id);
        assert!(tree.find_by_name("doomed").is_empty());
    }

    // --- RecordingObject: captures emit_signal calls for signal emission tests ---

    struct RecordingObject {
        base: ObjectBase,
        emissions: std::sync::Arc<parking_lot::Mutex<Vec<(String, Vec<Value>)>>>,
    }

    impl RecordingObject {
        fn named(
            name: &str,
        ) -> (
            Box<dyn Object>,
            std::sync::Arc<parking_lot::Mutex<Vec<(String, Vec<Value>)>>>,
        ) {
            let log = std::sync::Arc::new(parking_lot::Mutex::new(Vec::new()));
            let obj = Box::new(Self {
                base: ObjectBase::named(name),
                emissions: Arc::clone(&log),
            });
            (obj, log)
        }

        fn anonymous() -> (
            Box<dyn Object>,
            std::sync::Arc<parking_lot::Mutex<Vec<(String, Vec<Value>)>>>,
        ) {
            let log = std::sync::Arc::new(parking_lot::Mutex::new(Vec::new()));
            let obj = Box::new(Self {
                base: ObjectBase::new(),
                emissions: Arc::clone(&log),
            });
            (obj, log)
        }
    }

    impl AsObject for RecordingObject {
        fn object_base(&self) -> &ObjectBase {
            &self.base
        }
        fn object_base_mut(&mut self) -> &mut ObjectBase {
            &mut self.base
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    static RECORDING_META: MetaObject = MetaObject::new(
        "RecordingObject",
        &[],
        &[],
        &[],
        &[],
        quartzite_core::meta::noop_lookup_property,
        quartzite_core::meta::noop_lookup_signal,
        quartzite_core::meta::noop_lookup_method,
        quartzite_core::meta::noop_lookup_enum,
    );

    impl Object for RecordingObject {
        fn meta_object(&self) -> &'static MetaObject {
            &RECORDING_META
        }
        fn read_property(&self, _: &str) -> Option<Value> {
            None
        }
        fn write_property(&mut self, _: &str, _: Value) -> bool {
            false
        }
        fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
            None
        }
        fn connect_signal(
            &mut self,
            _: &str,
            _: SignalCallback,
            _: ConnectionType,
        ) -> Option<ConnectionId> {
            None
        }
        fn emit_signal(&mut self, name: &str, args: &[Value]) -> Option<()> {
            self.emissions.lock().push((name.to_owned(), args.to_vec()));
            Some(())
        }
    }

    // --- find_by_name_in tests (AC1, AC2, AC3, AC3b) ---

    #[test]
    fn find_by_name_in_returns_only_subtree_matches() {
        // AC1: only descendants-or-self of root are returned.
        let mut tree = ObjectTree::new();
        let root = tree.insert(StubObject::named("root"), None);
        let child = tree.insert(StubObject::named("target"), Some(root));
        let outside = tree.insert(StubObject::named("target"), None);
        let found = tree.find_by_name_in(root, "target");
        assert!(
            found.contains(&child),
            "should find in-subtree match: {found:?}"
        );
        assert!(
            !found.contains(&outside),
            "should not find out-of-subtree match: {found:?}"
        );
    }

    #[test]
    fn find_by_name_in_excludes_outside_subtree() {
        // AC2: name outside subtree returns empty.
        let mut tree = ObjectTree::new();
        let root = tree.insert(StubObject::named("root"), None);
        let _outside = tree.insert(StubObject::named("elsewhere"), None);
        let found = tree.find_by_name_in(root, "elsewhere");
        assert!(
            found.is_empty(),
            "should not find name outside subtree: {found:?}"
        );
    }

    #[test]
    fn find_by_name_in_unknown_root_returns_empty() {
        // AC3: unknown root returns empty.
        let tree = ObjectTree::new();
        let found = tree.find_by_name_in(ObjectId::new(), "any");
        assert!(found.is_empty());
    }

    #[test]
    fn find_by_name_in_includes_root_itself() {
        // AC1: root itself is included if its name matches.
        let mut tree = ObjectTree::new();
        let root = tree.insert(StubObject::named("match"), None);
        let found = tree.find_by_name_in(root, "match");
        assert_eq!(found, vec![root]);
    }

    #[test]
    fn find_by_name_in_shallowest_first_ordering() {
        // AC3b: BFS order — shallower matches appear before deeper ones.
        //
        // Tree:
        //   root (depth 0)
        //   └── child  (depth 1, named "target")
        //       └── grandchild (depth 2, named "target")
        let mut tree = ObjectTree::new();
        let root = tree.insert(StubObject::named("root"), None);
        let child = tree.insert(StubObject::named("target"), Some(root));
        let grandchild = tree.insert(StubObject::named("target"), Some(child));
        let found = tree.find_by_name_in(root, "target");
        assert_eq!(
            found,
            vec![child, grandchild],
            "shallowest must come first: {found:?}"
        );
    }

    // --- name_changed signal emission tests (AC4–AC9) ---

    #[test]
    #[allow(
        clippy::significant_drop_tightening,
        reason = "MutexGuard held intentionally to keep critical section atomic"
    )]
    fn rename_emits_name_changed_with_old_and_new() {
        // AC5: rename(id, new) emits (Some(old), Some(new)).
        let mut tree = ObjectTree::new();
        let (obj, log) = RecordingObject::named("old");
        let id = tree.insert(obj, None);
        tree.rename(id, "new");
        let emissions = log.lock();
        assert_eq!(emissions.len(), 1, "expected one emission: {emissions:?}");
        assert_eq!(emissions[0].0, "name_changed");
        assert_eq!(
            emissions[0].1,
            vec![Value::String("old".into()), Value::String("new".into())]
        );
    }

    #[test]
    #[allow(
        clippy::significant_drop_tightening,
        reason = "MutexGuard held intentionally to keep critical section atomic"
    )]
    fn rename_noop_does_not_emit() {
        // AC7: same-name rename does not emit.
        let mut tree = ObjectTree::new();
        let (obj, log) = RecordingObject::named("same");
        let id = tree.insert(obj, None);
        tree.rename(id, "same");
        let emissions = log.lock();
        assert!(
            emissions.is_empty(),
            "no-op rename must not emit: {emissions:?}"
        );
    }

    #[test]
    #[allow(
        clippy::significant_drop_tightening,
        reason = "MutexGuard held intentionally to keep critical section atomic"
    )]
    fn rename_anonymous_emits_null_old() {
        // AC8: anonymous → named emits (None, Some(new)).
        let mut tree = ObjectTree::new();
        let (obj, log) = RecordingObject::anonymous();
        let id = tree.insert(obj, None);
        tree.rename(id, "named");
        let emissions = log.lock();
        assert_eq!(emissions.len(), 1, "expected one emission: {emissions:?}");
        assert_eq!(
            emissions[0].1,
            vec![Value::Null, Value::String("named".into())]
        );
    }

    #[test]
    #[allow(
        clippy::significant_drop_tightening,
        reason = "MutexGuard held intentionally to keep critical section atomic"
    )]
    fn clear_name_emits_name_changed_with_old_and_null() {
        // AC6: clear_name emits (Some(old), None).
        let mut tree = ObjectTree::new();
        let (obj, log) = RecordingObject::named("old");
        let id = tree.insert(obj, None);
        tree.clear_name(id);
        let emissions = log.lock();
        assert_eq!(emissions.len(), 1, "expected one emission: {emissions:?}");
        assert_eq!(emissions[0].0, "name_changed");
        assert_eq!(
            emissions[0].1,
            vec![Value::String("old".into()), Value::Null]
        );
    }

    #[test]
    #[allow(
        clippy::significant_drop_tightening,
        reason = "MutexGuard held intentionally to keep critical section atomic"
    )]
    fn clear_name_already_anonymous_does_not_emit() {
        // Already-anonymous clear_name must not emit.
        let mut tree = ObjectTree::new();
        let (obj, log) = RecordingObject::anonymous();
        let id = tree.insert(obj, None);
        tree.clear_name(id);
        let emissions = log.lock();
        assert!(
            emissions.is_empty(),
            "anonymous clear_name must not emit: {emissions:?}"
        );
    }

    #[test]
    #[allow(
        clippy::significant_drop_tightening,
        reason = "MutexGuard held intentionally to keep critical section atomic"
    )]
    fn destroy_does_not_emit_name_changed() {
        // AC9: destroy must not emit name_changed.
        let mut tree = ObjectTree::new();
        let (obj, log) = RecordingObject::named("target");
        let id = tree.insert(obj, None);
        tree.destroy(id);
        let emissions = log.lock();
        assert!(
            emissions.iter().all(|(name, _)| name != "name_changed"),
            "destroy must not emit name_changed: {emissions:?}"
        );
    }
}
