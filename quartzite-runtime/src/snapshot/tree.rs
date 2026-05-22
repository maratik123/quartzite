//! Tree-layer snapshot: capture and restore an entire `ObjectTree`.
use std::collections::HashMap;

use quartzite_core::{
    ObjectId,
    snapshot::{DeserializeError, ObjectNode, SerializeError, TreeSnapshot},
    value::{Value, WeakObjectRef},
};

use crate::object_tree::ObjectTree;

use super::object::{capture_object, restore_object};

/// Captures an entire subtree rooted at `root_id` from `tree` into a
/// [`TreeSnapshot`].
///
/// The walk is depth-first. Only
/// [`Stored`](quartzite_core::meta::PropertyFlag::Stored) properties are
/// included in each node's snapshot.
///
/// # Parameters
///
/// - `tree`: the tree to capture from.
/// - `root_id`: the root of the subtree to snapshot.
///
/// # Errors
///
/// - [`SerializeError::ObjectNotInTree`] if `root_id` (or any descendant id)
///   is not present in `tree`.
/// - [`SerializeError::PropertyMissing`] if any object's meta lists a
///   `Stored` property that `read_property` does not return a value for.
///
/// # Examples
///
/// ```no_run
/// use quartzite_core::ObjectId;
/// use quartzite_runtime::{ObjectTree, snapshot::capture_tree};
///
/// // let (tree, root_id): (ObjectTree, ObjectId) = ...;
/// // let snap = capture_tree(&tree, root_id).unwrap();
/// ```
pub fn capture_tree(tree: &ObjectTree, root_id: ObjectId) -> Result<TreeSnapshot, SerializeError> {
    let root = capture_node(tree, root_id)?;
    Ok(TreeSnapshot {
        schema_version: quartzite_core::snapshot::CURRENT_SCHEMA_VERSION,
        root,
    })
}

fn capture_node(tree: &ObjectTree, id: ObjectId) -> Result<ObjectNode, SerializeError> {
    let snapshot = tree
        .with(id, |obj| capture_object(obj))
        .ok_or(SerializeError::ObjectNotInTree { id: id.raw() })??;
    let children = tree
        .children_of(id)
        .iter()
        .map(|&child_id| capture_node(tree, child_id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ObjectNode {
        snapshot,
        children,
        object_id: id.raw(),
    })
}

/// Restores an entire [`ObjectTree`] from a [`TreeSnapshot`].
///
/// Returns the new tree and the [`ObjectId`] of the root object. The restored
/// tree is completely fresh — it does not merge into or mutate any existing tree.
///
/// After restore:
/// - Each object is constructed via the process-wide [`ObjectFactory`](crate::factory::ObjectFactory).
/// - `Stored` properties are written back; non-`Stored` properties retain their defaults.
/// - Signal connections are **dropped** — the restored objects start with empty connection
///   tables and `signals_blocked = false`.
/// - Intra-tree [`WeakObjectRef`] payloads in `Stored` `Value::Object` properties are
///   **remapped** to the new IDs via an `OldObjectId → NewObjectId` table built during restore.
///   `WeakObjectRef`s embedded inside `Value::Custom` payloads are opaque and are **not**
///   remapped — they will dangle if the custom payload stores an object id.
///
/// # Parameters
///
/// - `snap`: the snapshot to restore.
///
/// # Errors
///
/// - [`DeserializeError::UnsupportedVersion`] — schema version too new.
/// - [`DeserializeError::FactoryMissing`] — `ObjectFactory::global()` is `None`.
/// - [`DeserializeError::UnknownClass`] — a class name is not registered.
/// - [`DeserializeError::WriteRejected`] — a property value was rejected.
///
/// # Examples
///
/// ```no_run
/// use quartzite_core::snapshot::TreeSnapshot;
/// use quartzite_runtime::snapshot::restore_tree;
///
/// // let snap: TreeSnapshot = ...;
/// // let (tree, root_id) = restore_tree(&snap).unwrap();
/// ```
pub fn restore_tree(snap: &TreeSnapshot) -> Result<(ObjectTree, ObjectId), DeserializeError> {
    snap.validate_version()?;

    let mut tree = ObjectTree::new();
    let mut remap: HashMap<u64, u64> = HashMap::new();

    let root_id = restore_node(&snap.root, &mut tree, None, &mut remap)?;

    // Second pass: rewrite Value::Object payloads in every stored property.
    remap_tree_refs(&mut tree, root_id, &remap);

    Ok((tree, root_id))
}

fn restore_node(
    node: &ObjectNode,
    tree: &mut ObjectTree,
    parent_id: Option<ObjectId>,
    remap: &mut HashMap<u64, u64>,
) -> Result<ObjectId, DeserializeError> {
    let obj = restore_object(&node.snapshot)?;
    let new_id = obj.object_base().id();

    // Build old→new mapping so the second pass can rewrite Value::Object payloads.
    // object_id == 0 means the node was not captured with tree context (no remap needed).
    if node.object_id != 0 {
        remap.insert(node.object_id, new_id.raw());
    }

    let insert_id = tree.insert(obj, parent_id);
    debug_assert_eq!(insert_id, new_id);

    for child_node in &node.children {
        restore_node(child_node, tree, Some(new_id), remap)?;
    }

    Ok(new_id)
}

/// Walks every Stored property of every object in the tree (depth-first from
/// `root_id`) and rewrites `Value::Object(WeakObjectRef(old))` payloads to
/// `Value::Object(WeakObjectRef(remap[old]))`. Only `List` and `Map` container
/// variants are recursed into; `Custom` payloads are opaque and not walked.
fn remap_tree_refs(tree: &mut ObjectTree, root_id: ObjectId, remap: &HashMap<u64, u64>) {
    if remap.is_empty() {
        return;
    }
    // Collect all ids in pre-order so we can mutate via with_mut.
    let ids = collect_ids(tree, root_id);
    for id in ids {
        tree.with_mut(id, |obj| {
            let meta = obj.meta_object();
            let prop_names: Vec<&'static str> = meta
                .properties
                .iter()
                .filter(|p| p.flags.contains(quartzite_core::meta::PropertyFlag::Stored))
                .map(|p| p.name)
                .collect();
            for name in prop_names {
                if let Some(mut val) = obj.read_property(name)
                    && remap_value(&mut val, remap)
                {
                    obj.write_property(name, val);
                }
            }
        });
    }
}

#[inline]
fn collect_ids(tree: &ObjectTree, root_id: ObjectId) -> Vec<ObjectId> {
    let mut out = Vec::new();
    collect_ids_inner(tree, root_id, &mut out);
    out
}

fn collect_ids_inner(tree: &ObjectTree, id: ObjectId, out: &mut Vec<ObjectId>) {
    out.push(id);
    let children: Vec<ObjectId> = tree.children_of(id).to_vec();
    for child in children {
        collect_ids_inner(tree, child, out);
    }
}

/// Rewrites `Value::Object` arms in-place. Returns `true` if a rewrite occurred.
fn remap_value(val: &mut Value, remap: &HashMap<u64, u64>) -> bool {
    match val {
        Value::Object(WeakObjectRef(id)) => {
            if let Some(&new_id) = remap.get(id) {
                *id = new_id;
                return true;
            }
        }
        Value::List(list) => {
            let mut changed = false;
            for item in list.iter_mut() {
                changed |= remap_value(item, remap);
            }
            return changed;
        }
        Value::Map(map) => {
            let mut changed = false;
            for item in map.values_mut() {
                changed |= remap_value(item, remap);
            }
            return changed;
        }
        _ => {}
    }
    false
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use quartzite_core::{
        enumflags2,
        id::ConnectionId,
        meta::{
            MetaObject, PropertyFlag, PropertyMeta, noop_lookup_enum, noop_lookup_method,
            noop_lookup_property, noop_lookup_signal,
        },
        object_base::ObjectBase,
        signal::ConnectionType,
        snapshot::{CURRENT_SCHEMA_VERSION, DeserializeError},
        traits::{AsObject, Object, SignalCallback},
        value::{Value, WeakObjectRef},
    };

    use super::*;
    use crate::factory::ObjectFactory;

    // --- Test fixture ---

    struct TreeSample {
        base: ObjectBase,
        count: i64,
        link: Value,
    }

    impl TreeSample {
        fn new_boxed() -> Box<dyn Object> {
            Box::new(TreeSample {
                base: ObjectBase::new(),
                count: 0,
                link: Value::Null,
            })
        }
    }

    impl AsObject for TreeSample {
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

    static TREE_SAMPLE_PROPS: &[PropertyMeta] = &[
        PropertyMeta::new(
            "count",
            "i64",
            enumflags2::make_bitflags!(PropertyFlag::{Readable | Writable | Stored | Designable}),
        ),
        PropertyMeta::new(
            "link",
            "Value",
            enumflags2::make_bitflags!(PropertyFlag::{Readable | Writable | Stored | Designable}),
        ),
    ];

    static TREE_SAMPLE_META: MetaObject = MetaObject::new(
        "TreeSample",
        TREE_SAMPLE_PROPS,
        &[],
        &[],
        &[],
        noop_lookup_property,
        noop_lookup_signal,
        noop_lookup_method,
        noop_lookup_enum,
    );

    impl Object for TreeSample {
        fn meta_object(&self) -> &'static MetaObject {
            &TREE_SAMPLE_META
        }

        fn read_property(&self, name: &str) -> Option<Value> {
            match name {
                "count" => Some(Value::Int(self.count)),
                "link" => Some(self.link.clone()),
                _ => None,
            }
        }

        fn write_property(&mut self, name: &str, value: Value) -> bool {
            match name {
                "count" => {
                    if let Value::Int(n) = value {
                        self.count = n;
                        true
                    } else {
                        false
                    }
                }
                "link" => {
                    self.link = value;
                    true
                }
                _ => false,
            }
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

        fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
            None
        }
    }

    static FACTORY_INSTALLED: OnceLock<()> = OnceLock::new();

    fn install_factory() {
        FACTORY_INSTALLED.get_or_init(|| {
            let mut factory = ObjectFactory::new();
            factory.register("TreeSample", TreeSample::new_boxed);
            if ObjectFactory::install(factory).is_err() {
                // Factory already installed (shared process); register into existing one.
                if let Some(arc) = ObjectFactory::global() {
                    arc.write().register("TreeSample", TreeSample::new_boxed);
                }
            }
        });
    }

    #[test]
    fn tree_round_trips_structure() {
        let _lock = quartzite_test_helpers::test_lock();
        install_factory();

        let mut tree = ObjectTree::new();
        let mut root = TreeSample::new_boxed();
        root.write_property("count", Value::Int(1));
        let root_id = tree.insert(root, None);

        let mut child = TreeSample::new_boxed();
        child.write_property("count", Value::Int(2));
        let child_id = tree.insert(child, Some(root_id));

        let mut grand = TreeSample::new_boxed();
        grand.write_property("count", Value::Int(3));
        tree.insert(grand, Some(child_id));

        let snap = capture_tree(&tree, root_id).unwrap();
        let (restored, new_root_id) = restore_tree(&snap).unwrap();

        assert_eq!(
            restored.with(new_root_id, |o| o.read_property("count")),
            Some(Some(Value::Int(1)))
        );
        let children = restored.children_of(new_root_id).to_vec();
        assert_eq!(children.len(), 1);
        let new_child_id = children[0];
        assert_eq!(
            restored.with(new_child_id, |o| o.read_property("count")),
            Some(Some(Value::Int(2)))
        );
        let grandchildren = restored.children_of(new_child_id);
        assert_eq!(grandchildren.len(), 1);
        assert_eq!(
            restored.with(grandchildren[0], |o| o.read_property("count")),
            Some(Some(Value::Int(3)))
        );
    }

    #[test]
    fn weakobjectref_link_remapped_after_restore() {
        let _lock = quartzite_test_helpers::test_lock();
        // AC3: a Value::Object payload holding the old child id must be rewritten
        // to point at the new child id after restore_tree.
        install_factory();

        let mut tree = ObjectTree::new();
        // Insert root first so child gets a fresh id.
        let mut root = TreeSample::new_boxed();
        root.write_property("count", Value::Int(10));
        let root_id = tree.insert(root, None);
        let mut child = TreeSample::new_boxed();
        child.write_property("count", Value::Int(20));
        let child_id = tree.insert(child, Some(root_id));
        let old_child_raw = child_id.raw();

        // Give root a "link" property pointing at the child's old id.
        tree.with_mut(root_id, |o| {
            o.write_property("link", Value::Object(WeakObjectRef(old_child_raw)));
        });

        let snap = capture_tree(&tree, root_id).unwrap();
        let (restored, new_root_id) = restore_tree(&snap).unwrap();

        let new_children = restored.children_of(new_root_id).to_vec();
        assert_eq!(new_children.len(), 1);
        let new_child_raw = new_children[0].raw();

        // Fresh restore mints new ObjectIds — old and new must differ.
        assert_ne!(old_child_raw, new_child_raw);

        // Root's "link" must now reference the NEW child id.
        assert_eq!(
            restored.with(new_root_id, |o| o.read_property("link")),
            Some(Some(Value::Object(WeakObjectRef(new_child_raw)))),
            "link must be remapped from old child id to new child id"
        );
    }

    #[test]
    fn schema_version_rejected() {
        let snap = quartzite_core::snapshot::TreeSnapshot {
            schema_version: u32::MAX,
            root: quartzite_core::snapshot::ObjectNode {
                snapshot: quartzite_core::snapshot::ObjectSnapshot {
                    class_name: "X".into(),
                    properties: Default::default(),
                    signals_blocked: false,
                },
                children: vec![],
                object_id: 0,
            },
        };
        assert!(matches!(
            restore_tree(&snap),
            Err(DeserializeError::UnsupportedVersion {
                found: u32::MAX,
                supported: CURRENT_SCHEMA_VERSION,
            })
        ));
    }

    #[test]
    fn remap_value_rewrites_object() {
        let mut remap = HashMap::new();
        remap.insert(10u64, 99u64);

        let mut val = Value::Object(WeakObjectRef(10));
        assert!(remap_value(&mut val, &remap));
        assert_eq!(val, Value::Object(WeakObjectRef(99)));
    }

    #[test]
    fn remap_value_rewrites_nested_list() {
        let mut remap = HashMap::new();
        remap.insert(5u64, 42u64);

        let mut val = Value::List(vec![Value::Object(WeakObjectRef(5)), Value::Int(1)]);
        assert!(remap_value(&mut val, &remap));
        match val {
            Value::List(items) => assert_eq!(items[0], Value::Object(WeakObjectRef(42))),
            _ => panic!(),
        }
    }

    #[test]
    fn remap_value_rewrites_nested_map() {
        let mut remap = HashMap::new();
        remap.insert(7u64, 77u64);

        let mut map = std::collections::BTreeMap::new();
        map.insert("k".into(), Value::Object(WeakObjectRef(7)));
        let mut val = Value::Map(map);
        assert!(remap_value(&mut val, &remap));
    }

    #[test]
    fn remap_value_unknown_id_unchanged() {
        let remap: HashMap<u64, u64> = HashMap::new();
        let mut val = Value::Object(WeakObjectRef(99));
        assert!(!remap_value(&mut val, &remap));
        assert_eq!(val, Value::Object(WeakObjectRef(99)));
    }

    #[test]
    fn signals_blocked_persists_across_restore() {
        let _lock = quartzite_test_helpers::test_lock();
        install_factory();
        let mut tree = ObjectTree::new();
        let mut root = TreeSample::new_boxed();
        root.object_base_mut().block_signals();
        assert!(root.object_base().signals_blocked());
        let root_id = tree.insert(root, None);

        let snap = capture_tree(&tree, root_id).unwrap();
        let (restored, new_root) = restore_tree(&snap).unwrap();

        // signals_blocked IS persisted — must be true after restore (AC4).
        // The fixture's connect_signal always returns None so no connections are
        // ever formed; the ConnectionTable is therefore vacuously empty after restore.
        assert_eq!(
            restored.with(new_root, |o| o.object_base().signals_blocked()),
            Some(true)
        );
    }
}
