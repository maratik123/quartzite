use std::sync::{Arc, Mutex};

use quartzite_core::{
    ObjectId,
    id::ConnectionId,
    meta::MetaObject,
    object_base::ObjectBase,
    traits::{AsObject, Object, SignalCallback},
    value::Value,
};
use quartzite_runtime::ObjectTree;

struct Stub {
    base: ObjectBase,
}

impl Stub {
    fn named(name: &str) -> Box<dyn Object> {
        Box::new(Stub {
            base: ObjectBase::named(name),
        })
    }
}

impl AsObject for Stub {
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
    "Stub",
    &[],
    &[],
    &[],
    &[],
    quartzite_core::meta::noop_lookup_property,
    quartzite_core::meta::noop_lookup_signal,
    quartzite_core::meta::noop_lookup_method,
    quartzite_core::meta::noop_lookup_enum,
);

impl Object for Stub {
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

/// Records its own name into a shared log when dropped — used to verify destruction order.
struct LogObj {
    base: ObjectBase,
    log: Arc<Mutex<Vec<String>>>,
}

impl LogObj {
    fn new(name: &str, log: Arc<Mutex<Vec<String>>>) -> Box<dyn Object> {
        Box::new(LogObj {
            base: ObjectBase::named(name),
            log,
        })
    }
}

impl Drop for LogObj {
    fn drop(&mut self) {
        self.log.lock().unwrap().push(self.base.name.clone());
    }
}

impl AsObject for LogObj {
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

static LOG_META: MetaObject = MetaObject::new(
    "LogObj",
    &[],
    &[],
    &[],
    &[],
    quartzite_core::meta::noop_lookup_property,
    quartzite_core::meta::noop_lookup_signal,
    quartzite_core::meta::noop_lookup_method,
    quartzite_core::meta::noop_lookup_enum,
);

impl Object for LogObj {
    fn meta_object(&self) -> &'static MetaObject {
        &LOG_META
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

fn make_tree() -> ObjectTree {
    ObjectTree::new()
}

// AC1 — insert returns an id; subsequent get (via `with`) finds the object.
#[test]
fn insert_returns_id_and_get_finds_it() {
    let mut tree = make_tree();
    let id = tree.insert(Stub::named("alpha"), None);
    let name = tree.with(id, |o| o.object_base().name.clone());
    assert_eq!(name, Some("alpha".to_string()));
}

// AC2 — destroy removes the object and all descendants.
#[test]
fn destroy_removes_object_returns_none() {
    let mut tree = make_tree();
    let id = tree.insert(Stub::named("x"), None);
    tree.destroy(id);
    assert!(tree.with(id, |_| ()).is_none());
}

// AC2 extended — depth-3 tree; destroy root removes all.
#[test]
fn destroy_removes_all_descendants() {
    let mut tree = make_tree();
    let root = tree.insert(Stub::named("root"), None);
    let child = tree.insert(Stub::named("child"), Some(root));
    let grandchild = tree.insert(Stub::named("gc"), Some(child));

    tree.destroy(root);

    assert!(tree.with(root, |_| ()).is_none());
    assert!(tree.with(child, |_| ()).is_none());
    assert!(tree.with(grandchild, |_| ()).is_none());
}

// AC3 — after destroy, get(id) returns None.
#[test]
fn stale_id_after_destroy_returns_none() {
    let mut tree = make_tree();
    let id = tree.insert(Stub::named("gone"), None);
    tree.destroy(id);
    assert!(tree.with(id, |_| ()).is_none());
}

// AC4 — parent/child relationship reflected in accessors.
#[test]
fn parent_child_reflected_in_accessors() {
    let mut tree = make_tree();
    let parent = tree.insert(Stub::named("p"), None);
    let child = tree.insert(Stub::named("c"), Some(parent));

    assert_eq!(tree.parent_of(child), Some(parent));
    assert!(tree.children_of(parent).contains(&child));
}

// AC5 — find_by_name returns the correct id.
#[test]
fn find_by_name_returns_correct_id() {
    let mut tree = make_tree();
    let foo = tree.insert(Stub::named("foo"), None);
    tree.insert(Stub::named("bar"), None);

    assert_eq!(tree.find_by_name("foo"), Some(foo));
}

// AC5 edge — absent name returns None.
#[test]
fn find_by_name_returns_none_when_absent() {
    let tree = make_tree();
    assert!(tree.find_by_name("nope").is_none());
}

// Verify reparent updates both old and new parent's children lists.
#[test]
fn reparent_updates_both_parent_and_children() {
    let mut tree = make_tree();
    let p1 = tree.insert(Stub::named("p1"), None);
    let p2 = tree.insert(Stub::named("p2"), None);
    let child = tree.insert(Stub::named("c"), Some(p1));

    tree.reparent(child, Some(p2));

    assert_eq!(tree.parent_of(child), Some(p2));
    assert!(!tree.children_of(p1).contains(&child));
    assert!(tree.children_of(p2).contains(&child));
}

// Verify that destruction order is depth-first post-order (leaves before parents).
#[test]
fn destroy_is_depth_first_post_order() {
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let mut tree = make_tree();

    // Build: root → c1 → gc
    //             → c2
    let root = tree.insert(LogObj::new("root", Arc::clone(&log)), None);
    let c1 = tree.insert(LogObj::new("c1", Arc::clone(&log)), Some(root));
    let _c2 = tree.insert(LogObj::new("c2", Arc::clone(&log)), Some(root));
    let _gc = tree.insert(LogObj::new("gc", Arc::clone(&log)), Some(c1));

    tree.destroy(root);

    let order = log.lock().unwrap().clone();
    assert_eq!(order.len(), 4, "all 4 nodes must be destroyed");

    let gc_pos = order.iter().position(|n| n == "gc").unwrap();
    let c1_pos = order.iter().position(|n| n == "c1").unwrap();
    let root_pos = order.iter().position(|n| n == "root").unwrap();

    assert!(gc_pos < c1_pos, "gc must be destroyed before its parent c1");
    assert!(c1_pos < root_pos, "c1 must be destroyed before root");
    assert_eq!(order.last().unwrap(), "root", "root must be destroyed last");

    let unknown = ObjectId::new();
    assert!(!tree.contains(unknown));
}
