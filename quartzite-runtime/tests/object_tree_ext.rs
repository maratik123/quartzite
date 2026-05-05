// Each tests/*.rs file compiles as a separate binary — this file gets its own
// OnceLock, required for Application singleton tests.
//
// All AC1–AC9 scenarios run in a single #[test] to guarantee sequential
// execution: the after-drop assertions (AC3, AC6, AC9) must not race with
// live-Application scenarios that run concurrently in other test functions.

use quartzite_core::{
    ObjectId,
    id::ConnectionId,
    meta::MetaObject,
    object_base::ObjectBase,
    traits::{AsObject, Object, SignalCallback},
    value::Value,
};
use quartzite_runtime::{Application, ObjectTreeExt, try_with_tree};

// Minimal stub implementing Object — mirrors the pattern in tests/object_tree.rs.
struct Stub {
    base: ObjectBase,
}

impl Stub {
    fn named(name: &str) -> Box<dyn Object> {
        Box::new(Stub {
            base: ObjectBase::named(name),
        })
    }

    /// Creates a query-only stub with a specific `id`, not inserted into the tree.
    fn with_id(id: ObjectId) -> Self {
        Stub {
            base: ObjectBase::new_with_id(id),
        }
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

/// All AC1–AC9 scenarios in a single sequential test.
#[test]
fn parent_children_accessors_all_acs() {
    // ──────────────────────────────────────────────────────────────────────────
    // Phase 0 — before Application::new() (AC9 pre-new)
    // ──────────────────────────────────────────────────────────────────────────
    assert!(
        try_with_tree(|_| ()).is_none(),
        "AC9: try_with_tree must return None before Application::new()"
    );

    // ──────────────────────────────────────────────────────────────────────────
    // Phase 1 — live Application
    // ──────────────────────────────────────────────────────────────────────────
    let app = Application::new().expect("Application::new() must succeed");

    // AC9 — tree is accessible after new()
    assert!(
        try_with_tree(|_| ()).is_some(),
        "AC9: try_with_tree must return Some after Application::new()"
    );

    // Build a small tree: root -> child_a, child_b; child_a -> grandchild
    let (root_id, child_a_id, child_b_id, grandchild_id) = {
        let mut tree = app.object_tree().lock().unwrap();
        let root_id = tree.insert(Stub::named("root"), None);
        let child_a_id = tree.insert(Stub::named("child_a"), Some(root_id));
        let child_b_id = tree.insert(Stub::named("child_b"), Some(root_id));
        let grandchild_id = tree.insert(Stub::named("grandchild"), Some(child_a_id));
        (root_id, child_a_id, child_b_id, grandchild_id)
    };

    // Query stubs: same IDs as the tree entries; used to call ObjectTreeExt methods.
    let root_q = Stub::with_id(root_id);
    let child_a_q = Stub::with_id(child_a_id);
    let child_b_q = Stub::with_id(child_b_id);
    let grandchild_q = Stub::with_id(grandchild_id);

    // AC1 — parent of root is None
    assert_eq!(root_q.parent(), None, "AC1: root.parent() must be None");

    // AC2 — parent of child returns Some(parent_id)
    assert_eq!(
        child_a_q.parent(),
        Some(root_id),
        "AC2: child_a.parent() must be Some(root_id)"
    );
    assert_eq!(
        grandchild_q.parent(),
        Some(child_a_id),
        "AC2: grandchild.parent() must be Some(child_a_id)"
    );

    // AC4 — children in insertion order
    assert_eq!(
        root_q.children(),
        vec![child_a_id, child_b_id],
        "AC4: root.children() must be [child_a, child_b] in insertion order"
    );

    // AC5 — leaf returns empty Vec
    assert_eq!(
        grandchild_q.children(),
        Vec::<ObjectId>::new(),
        "AC5: grandchild.children() must be empty"
    );

    // AC7 / AC8 — _in variants match global variants.
    // Compute global results first (outside any lock), then compare inside the lock.
    // Calling parent()/children() while holding the tree mutex would deadlock.
    let parent_b_global = child_b_q.parent();
    let children_root_global = root_q.children();
    {
        let tree = app.object_tree().lock().unwrap();
        assert_eq!(
            child_b_q.parent_in(&tree),
            parent_b_global,
            "AC7: parent_in(&tree) must match parent()"
        );
        assert_eq!(
            root_q.children_in(&tree),
            children_root_global.as_slice(),
            "AC8: children_in(&tree) must match children()"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Phase 2 — drop Application handle; query stubs stay in scope
    // ──────────────────────────────────────────────────────────────────────────
    drop(app);

    // AC3 — parent() returns None after drop
    assert_eq!(
        root_q.parent(),
        None,
        "AC3: parent() must return None after Application is dropped"
    );

    // AC6 — children() returns empty Vec after drop
    assert_eq!(
        root_q.children(),
        Vec::<ObjectId>::new(),
        "AC6: children() must return empty Vec after Application is dropped"
    );

    // AC9 post — try_with_tree returns None after drop
    assert!(
        try_with_tree(|_| ()).is_none(),
        "AC9: try_with_tree must return None after Application is dropped"
    );
}
