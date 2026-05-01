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
}

impl ObjectTree {
    pub fn new() -> Self {
        Self {
            store: SlotMap::new(),
            forward: HashMap::new(),
            reverse: HashMap::new(),
            parent_map: HashMap::new(),
            children_map: HashMap::new(),
        }
    }

    /// Insert `obj` into the tree, optionally under `parent_id`.
    /// Returns the `ObjectId` of the inserted object.
    pub fn insert(&mut self, obj: Box<dyn Object>, parent_id: Option<ObjectId>) -> ObjectId {
        let id = obj.object_base().id();
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
    pub fn contains(&self, id: ObjectId) -> bool {
        self.forward.contains_key(&id)
    }

    /// Run `f` with a shared reference to the object identified by `id`.
    /// Returns `None` if `id` is not in the tree.
    pub fn with<R, F>(&self, id: ObjectId, f: F) -> Option<R>
    where
        F: FnOnce(&dyn Object) -> R,
    {
        let slot = self.forward.get(&id)?;
        self.store.get(slot.0).map(|obj| f(obj.as_ref()))
    }

    /// Run `f` with an exclusive reference to the object identified by `id`.
    /// Returns `None` if `id` is not in the tree.
    pub fn with_mut<R, F>(&mut self, id: ObjectId, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Object) -> R,
    {
        let slot = self.forward.get(&id)?;
        self.store.get_mut(slot.0).map(|obj| f(obj.as_mut()))
    }

    /// Returns the parent `ObjectId` of `id`, or `None` if `id` is a root.
    pub fn parent_of(&self, id: ObjectId) -> Option<ObjectId> {
        self.parent_map.get(&id).copied()
    }

    /// Returns the ordered list of children of `id`.
    pub fn children_of(&self, id: ObjectId) -> &[ObjectId] {
        self.children_map
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Move `id` to a new parent. If `new_parent` is `None`, `id` becomes a root.
    pub fn reparent(&mut self, id: ObjectId, new_parent: Option<ObjectId>) {
        if let Some(old_parent) = self.parent_map.remove(&id)
            && let Some(siblings) = self.children_map.get_mut(&old_parent)
        {
            siblings.retain(|&c| c != id);
        }
        if let Some(pid) = new_parent {
            self.parent_map.insert(id, pid);
            self.children_map.entry(pid).or_default().push(id);
        }
    }

    /// Find an object by `name` field. Returns the first match.
    pub fn find_by_name(&self, name: &str) -> Option<ObjectId> {
        for (id, slot) in &self.forward {
            if let Some(obj) = self.store.get(slot.0)
                && obj.object_base().name == name
            {
                return Some(*id);
            }
        }
        None
    }

    /// Remove `id` and all its descendants from the tree (depth-first post-order).
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
            self.store.remove(slot.0);
            self.reverse.remove(&slot);
        }
        if let Some(parent_id) = self.parent_map.remove(&id)
            && let Some(siblings) = self.children_map.get_mut(&parent_id)
        {
            siblings.retain(|&c| c != id);
        }
        self.children_map.remove(&id);
    }
}

impl Default for ObjectTree {
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

    static STUB_META: MetaObject = MetaObject::new("StubObject", &[], &[], &[], &[]);

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
        let found = tree.with(id, |o| o.object_base().name.clone());
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
        assert_eq!(tree.find_by_name("foo"), Some(id));
    }

    #[test]
    fn find_by_name_returns_none_when_absent() {
        let tree = ObjectTree::new();
        assert!(tree.find_by_name("missing").is_none());
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
}
