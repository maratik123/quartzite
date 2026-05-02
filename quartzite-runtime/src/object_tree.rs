//! Hierarchical store for all runtime objects.
use std::collections::HashMap;

use quartzite_core::{ObjectId, traits::Object};
use slotmap::{DefaultKey, SlotMap};

use crate::object_id::SlotKey;

/// Central store for all runtime objects.
///
/// Owns objects via a `SlotMap`. Maintains two identity maps (forward and
/// reverse) and two relationship maps (parent and children). All mutations
/// go through the methods here — no caller ever modifies the maps directly.
///
/// `ObjectTree: Send` follows automatically from `Object: Send`. Callers
/// requiring shared access should wrap the tree in `Mutex<ObjectTree>`.
pub struct ObjectTree {
    store: SlotMap<DefaultKey, Box<dyn Object>>,
    /// ObjectId → arena slot (forward index)
    forward: HashMap<ObjectId, SlotKey>,
    /// arena slot → ObjectId (reverse index, for destroy traversal)
    reverse: HashMap<SlotKey, ObjectId>,
    /// child ObjectId → parent ObjectId
    parent_map: HashMap<ObjectId, ObjectId>,
    /// parent ObjectId → ordered children list
    children_map: HashMap<ObjectId, Vec<ObjectId>>,
    /// name → list of ObjectIds with that name (supports multiple objects sharing a name)
    by_name: HashMap<String, Vec<ObjectId>>,
}

impl ObjectTree {
    /// Create an empty `ObjectTree`.
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
    pub fn new() -> Self {
        Self {
            store: SlotMap::new(),
            forward: HashMap::new(),
            reverse: HashMap::new(),
            parent_map: HashMap::new(),
            children_map: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    /// Insert `obj` into the tree, optionally under `parent_id`.
    /// Returns the `ObjectId` of the inserted object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// // tree.insert(Box::new(my_object), None);
    /// ```
    pub fn insert(&mut self, obj: Box<dyn Object>, parent_id: Option<ObjectId>) -> ObjectId {
        let id = obj.object_base().id();
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
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &ObjectTree, id: quartzite_core::ObjectId) {
    /// assert!(!tree.contains(id));
    /// # }
    /// ```
    pub fn contains(&self, id: ObjectId) -> bool {
        self.forward.contains_key(&id)
    }

    /// Run `f` with a shared reference to the object identified by `id`.
    /// Returns `None` if `id` is not in the tree.
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

    /// Run `f` with an exclusive reference to the object identified by `id`.
    /// Returns `None` if `id` is not in the tree.
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

    /// Returns the parent `ObjectId` of `id`, or `None` if `id` is a root.
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
    pub fn parent_of(&self, id: ObjectId) -> Option<ObjectId> {
        self.parent_map.get(&id).copied()
    }

    /// Returns the ordered list of children of `id`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &ObjectTree, parent_id: quartzite_core::ObjectId) {
    /// let children: &[quartzite_core::ObjectId] = tree.children_of(parent_id);
    /// # }
    /// ```
    pub fn children_of(&self, id: ObjectId) -> &[ObjectId] {
        self.children_map
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Move `id` to a new parent. If `new_parent` is `None`, `id` becomes a root.
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

    /// Find all objects with the given `name`. Returns a slice of `ObjectId`s
    /// (empty if no object has that name).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectTree;
    /// # fn example(tree: &ObjectTree) {
    /// let ids: &[quartzite_core::ObjectId] = tree.find_by_name("my-button");
    /// # }
    /// ```
    pub fn find_by_name(&self, name: &str) -> &[ObjectId] {
        self.by_name
            .get(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Rename the object `id` to `new_name`, updating the name index.
    ///
    /// Has no effect if `id` is not in the tree.
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
        let new_name = new_name.into();
        // Remove from old name bucket.
        if let Some(old_name) = self.with(id, |obj| obj.object_base().name().map(str::to_owned)) {
            if let Some(old_name) = old_name {
                Self::remove_from_by_name(&mut self.by_name, &old_name, id);
            }
        } else {
            return; // id not in tree
        }
        // Update the object's name.
        self.with_mut(id, |obj| {
            obj.object_base_mut().set_name_raw(Some(new_name.clone()))
        });
        // Insert into new name bucket.
        self.by_name.entry(new_name).or_default().push(id);
    }

    /// Clear the name of object `id`, making it anonymous and removing it from the name index.
    ///
    /// Has no effect if `id` is not in the tree.
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
        // Remove from old name bucket.
        let old_name = self.with(id, |obj| obj.object_base().name().map(str::to_owned));
        match old_name {
            None => return, // id not in tree
            Some(Some(old_name)) => {
                Self::remove_from_by_name(&mut self.by_name, &old_name, id);
            }
            Some(None) => {} // already anonymous
        }
        self.with_mut(id, |obj| obj.object_base_mut().set_name_raw(None));
    }

    /// Remove `id` and all its descendants from the tree (depth-first post-order).
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
        // Collect subtree in depth-first post-order (leaves first).
        let mut order: Vec<ObjectId> = Vec::new();
        self.collect_post_order(id, &mut order);

        for node_id in order {
            self.remove_node(node_id);
        }
    }

    fn collect_post_order(&self, id: ObjectId, out: &mut Vec<ObjectId>) {
        if let Some(children) = self.children_map.get(&id) {
            for &child in children.iter() {
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

impl Default for ObjectTree {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_core::{
        id::ConnectionId,
        meta::MetaObject,
        object_base::ObjectBase,
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
        fn connect_signal(&mut self, _: &str, _: SignalCallback) -> Option<ConnectionId> {
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
        assert!(ids.contains(&a), "missing a: {ids:?}");
        assert!(ids.contains(&b), "missing b: {ids:?}");
        assert_eq!(ids.len(), 2);
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
}
