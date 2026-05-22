//! Integration tests for object and tree snapshot capture/restore via the `serde` feature.
#![cfg(feature = "serde")]

use std::{collections::BTreeMap, sync::OnceLock};

use quartzite_core::{
    id::ConnectionId,
    meta::{
        MetaObject, PropertyFlag, PropertyMeta, noop_lookup_enum, noop_lookup_method,
        noop_lookup_property, noop_lookup_signal,
    },
    object_base::ObjectBase,
    signal::ConnectionType,
    snapshot::{
        CURRENT_SCHEMA_VERSION, DeserializeError, ObjectNode, ObjectSnapshot, TreeSnapshot,
    },
    traits::{AsObject, Object, SignalCallback},
    value::{CustomValue, Value, WeakObjectRef},
};
use quartzite_runtime::{
    ObjectTree,
    factory::ObjectFactory,
    snapshot::{capture_object, capture_tree, restore_object, restore_tree},
};

// --- CustomValue for round-trip test ---

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MyCustom {
    v: i64,
}

#[typetag::serde]
impl CustomValue for MyCustom {
    fn type_name(&self) -> &'static str {
        "MyCustom"
    }

    fn clone_box(&self) -> Box<dyn CustomValue> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- Test fixture ---

struct SerdeFixture {
    base: ObjectBase,
    count: i64,
    name: String,
    link: Value,
    cache: i64, // NOT Stored
}

impl SerdeFixture {
    fn new_boxed() -> Box<dyn Object> {
        Box::new(SerdeFixture {
            base: ObjectBase::new(),
            count: 0,
            name: String::new(),
            link: Value::Null,
            cache: 0,
        })
    }
}

impl AsObject for SerdeFixture {
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

// Stored props use PropertyFlag::read_write() (Readable|Writable|Stored|Designable).
// Non-stored uses Readable|Writable|Designable without Stored.
fn make_fixture_meta() -> &'static MetaObject {
    static META: OnceLock<MetaObject> = OnceLock::new();
    META.get_or_init(|| {
        let stored = PropertyFlag::read_write();
        let transient = PropertyFlag::Readable | PropertyFlag::Writable | PropertyFlag::Designable;
        let props: &'static [PropertyMeta] = Box::leak(
            vec![
                PropertyMeta::new("count", "i64", stored),
                PropertyMeta::new("name", "String", stored),
                PropertyMeta::new("link", "Value", stored),
                PropertyMeta::new("cache", "i64", transient),
            ]
            .into_boxed_slice(),
        );
        MetaObject::new(
            "SerdeFixture",
            props,
            &[],
            &[],
            &[],
            noop_lookup_property,
            noop_lookup_signal,
            noop_lookup_method,
            noop_lookup_enum,
        )
    })
}

impl Object for SerdeFixture {
    fn meta_object(&self) -> &'static MetaObject {
        make_fixture_meta()
    }

    fn read_property(&self, name: &str) -> Option<Value> {
        match name {
            "count" => Some(Value::Int(self.count)),
            "name" => Some(Value::String(self.name.clone())),
            "link" => Some(self.link.clone()),
            "cache" => Some(Value::Int(self.cache)),
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
            "name" => {
                if let Value::String(s) = value {
                    self.name = s;
                    true
                } else {
                    false
                }
            }
            "link" => {
                self.link = value;
                true
            }
            "cache" => {
                if let Value::Int(n) = value {
                    self.cache = n;
                    true
                } else {
                    false
                }
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
        factory.register("SerdeFixture", SerdeFixture::new_boxed);
        if ObjectFactory::install(factory).is_err() {
            // Factory already installed (shared process); register into existing one.
            if let Some(arc) = ObjectFactory::global() {
                arc.write()
                    .register("SerdeFixture", SerdeFixture::new_boxed);
            }
        }
    });
}

// --- Helpers ---

fn primitive_values() -> Vec<Value> {
    vec![
        Value::Null,
        Value::Bool(true),
        Value::Bool(false),
        Value::Int(42),
        Value::Int(i64::MIN),
        Value::Float(1.5),
        Value::String("hello".into()),
        Value::List(vec![Value::Int(1), Value::Null]),
        {
            let mut map = BTreeMap::new();
            map.insert("k".into(), Value::Bool(true));
            Value::Map(map)
        },
        Value::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        Value::Object(WeakObjectRef(123)),
        Value::Duration(std::time::Duration::from_millis(500)),
    ]
}

// --- Value round-trip tests (property layer) ---

#[test]
fn value_json_round_trips_primitives() {
    for val in primitive_values() {
        let json = serde_json::to_string(&val).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(val, back, "json round-trip failed for {json}");
    }
}

#[test]
fn value_bincode_round_trips_primitives() {
    let config = bincode::config::standard();
    for val in primitive_values() {
        let bytes = bincode::serde::encode_to_vec(&val, config).expect("serialize");
        let (back, _): (Value, _) =
            bincode::serde::decode_from_slice(&bytes, config).expect("deserialize");
        assert_eq!(val, back);
    }
}

#[test]
fn value_bincode_round_trips_nan() {
    let config = bincode::config::standard();
    let val = Value::Float(f64::NAN);
    let bytes = bincode::serde::encode_to_vec(&val, config).expect("serialize");
    let (back, _): (Value, _) =
        bincode::serde::decode_from_slice(&bytes, config).expect("deserialize");
    let Value::Float(f) = back else {
        panic!("expected Float")
    };
    assert!(f.is_nan());
}

#[test]
fn custom_value_json_round_trip() {
    use std::sync::Arc;
    let val = Value::Custom(Arc::new(MyCustom { v: 42 }));
    let json = serde_json::to_string(&val).expect("serialize");
    let back: Value = serde_json::from_str(&json).expect("deserialize");
    let Value::Custom(arc) = back else {
        panic!("expected Custom")
    };
    let inner = arc.as_any().downcast_ref::<MyCustom>().expect("downcast");
    assert_eq!(inner.v, 42);
}

#[test]
fn custom_value_bincode_round_trip() {
    use std::sync::Arc;
    let val = Value::Custom(Arc::new(MyCustom { v: 99 }));
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&val, config).expect("serialize");
    let (back, _): (Value, _) =
        bincode::serde::decode_from_slice(&bytes, config).expect("deserialize");
    let Value::Custom(arc) = back else {
        panic!("expected Custom")
    };
    let inner = arc.as_any().downcast_ref::<MyCustom>().expect("downcast");
    assert_eq!(inner.v, 99);
}

// --- Object layer tests ---

#[test]
fn capture_object_excludes_non_stored() {
    let _lock = quartzite_test_helpers::test_lock();
    install_factory();
    let mut obj = SerdeFixture::new_boxed();
    obj.write_property("count", Value::Int(7));
    obj.write_property("cache", Value::Int(99));

    let snap = capture_object(obj.as_ref()).unwrap();
    assert_eq!(snap.class_name, "SerdeFixture");
    assert!(snap.properties.contains_key("count"));
    assert!(
        !snap.properties.contains_key("cache"),
        "non-Stored 'cache' must not appear in snapshot"
    );
}

#[test]
fn restore_object_round_trips_stored_keeps_default_for_non_stored() {
    let _lock = quartzite_test_helpers::test_lock();
    install_factory();
    let mut obj = SerdeFixture::new_boxed();
    obj.write_property("count", Value::Int(42));
    obj.write_property("name", Value::String("world".into()));
    obj.write_property("cache", Value::Int(999));

    let snap = capture_object(obj.as_ref()).unwrap();
    let restored = restore_object(&snap).unwrap();

    assert_eq!(restored.read_property("count"), Some(Value::Int(42)));
    assert_eq!(
        restored.read_property("name"),
        Some(Value::String("world".into()))
    );
    // non-Stored: must keep default (0), not the captured value (999)
    assert_eq!(restored.read_property("cache"), Some(Value::Int(0)));
}

#[test]
fn restore_object_json_round_trip() {
    let _lock = quartzite_test_helpers::test_lock();
    install_factory();
    let mut obj = SerdeFixture::new_boxed();
    obj.write_property("count", Value::Int(7));
    obj.write_property("name", Value::String("snap".into()));

    let snap = capture_object(obj.as_ref()).unwrap();
    let json = serde_json::to_string(&snap).expect("serialize");
    let snap2: ObjectSnapshot = serde_json::from_str(&json).expect("deserialize");
    let restored = restore_object(&snap2).unwrap();

    assert_eq!(restored.read_property("count"), Some(Value::Int(7)));
    assert_eq!(
        restored.read_property("name"),
        Some(Value::String("snap".into()))
    );
}

// --- Tree layer tests ---

#[test]
fn tree_round_trips_parent_child_structure() {
    let _lock = quartzite_test_helpers::test_lock();
    install_factory();

    let mut tree = ObjectTree::new();
    let mut root = SerdeFixture::new_boxed();
    root.write_property("count", Value::Int(1));
    let root_id = tree.insert(root, None);

    let mut child = SerdeFixture::new_boxed();
    child.write_property("count", Value::Int(2));
    let child_id = tree.insert(child, Some(root_id));

    let mut grand = SerdeFixture::new_boxed();
    grand.write_property("count", Value::Int(3));
    tree.insert(grand, Some(child_id));

    let snap = capture_tree(&tree, root_id).unwrap();
    let (restored, new_root) = restore_tree(&snap).unwrap();

    assert_eq!(
        restored.with(new_root, |o| o.read_property("count")),
        Some(Some(Value::Int(1)))
    );
    let children = restored.children_of(new_root).to_vec();
    assert_eq!(children.len(), 1);
    assert_eq!(
        restored.with(children[0], |o| o.read_property("count")),
        Some(Some(Value::Int(2)))
    );
    let grandchildren = restored.children_of(children[0]).to_vec();
    assert_eq!(grandchildren.len(), 1);
    assert_eq!(
        restored.with(grandchildren[0], |o| o.read_property("count")),
        Some(Some(Value::Int(3)))
    );
}

#[test]
fn tree_json_round_trip_end_to_end() {
    let _lock = quartzite_test_helpers::test_lock();
    install_factory();

    let mut tree = ObjectTree::new();
    let mut root = SerdeFixture::new_boxed();
    root.write_property("count", Value::Int(100));
    root.write_property("name", Value::String("root".into()));
    let root_id = tree.insert(root, None);

    let snap = capture_tree(&tree, root_id).unwrap();
    let json = serde_json::to_string(&snap).expect("serialize tree");
    let snap2: TreeSnapshot = serde_json::from_str(&json).expect("deserialize tree");
    let (restored, new_root) = restore_tree(&snap2).unwrap();

    assert_eq!(
        restored.with(new_root, |o| o.read_property("count")),
        Some(Some(Value::Int(100)))
    );
    assert_eq!(
        restored.with(new_root, |o| o.read_property("name")),
        Some(Some(Value::String("root".into())))
    );
}

#[test]
fn tree_bincode_round_trip_end_to_end() {
    let _lock = quartzite_test_helpers::test_lock();
    install_factory();

    let mut tree = ObjectTree::new();
    let mut root = SerdeFixture::new_boxed();
    root.write_property("count", Value::Int(55));
    let root_id = tree.insert(root, None);

    let snap = capture_tree(&tree, root_id).unwrap();
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&snap, config).expect("serialize");
    let (snap2, _): (TreeSnapshot, _) =
        bincode::serde::decode_from_slice(&bytes, config).expect("deserialize");
    let (restored, new_root) = restore_tree(&snap2).unwrap();

    assert_eq!(
        restored.with(new_root, |o| o.read_property("count")),
        Some(Some(Value::Int(55)))
    );
}

#[test]
fn schema_version_rejected() {
    let snap = TreeSnapshot {
        schema_version: u32::MAX,
        root: ObjectNode {
            snapshot: ObjectSnapshot {
                class_name: "SerdeFixture".into(),
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
fn signals_blocked_persists_across_restore() {
    let _lock = quartzite_test_helpers::test_lock();
    install_factory();

    let mut tree = ObjectTree::new();
    let mut root = SerdeFixture::new_boxed();
    root.object_base_mut().block_signals();
    assert!(root.object_base().signals_blocked());
    let root_id = tree.insert(root, None);

    let snap = capture_tree(&tree, root_id).unwrap();
    let (restored, new_root) = restore_tree(&snap).unwrap();

    assert_eq!(
        restored.with(new_root, |o| o.object_base().signals_blocked()),
        Some(true),
        "signals_blocked must be preserved across restore"
    );
}
