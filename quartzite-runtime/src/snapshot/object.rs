//! Object-layer snapshot: capture and restore a single object.
use quartzite_core::{
    meta::PropertyFlag,
    snapshot::{DeserializeError, ObjectSnapshot, SerializeError},
    traits::Object,
};

use crate::factory::ObjectFactory;

/// Captures all [`Stored`](PropertyFlag::Stored) properties of `obj` into an
/// [`ObjectSnapshot`].
///
/// Properties without the `Stored` flag are silently skipped.
///
/// # Parameters
///
/// - `obj`: the object whose stored properties are captured.
///
/// # Errors
///
/// Returns [`SerializeError::PropertyMissing`] when `read_property` returns `None`
/// for a property that is declared as `Stored` in the object's meta — this
/// indicates a meta-system invariant violation.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::snapshot::capture_object;
/// // let obj: Box<dyn quartzite_core::Object> = ...;
/// // let snap = capture_object(obj.as_ref()).unwrap();
/// ```
pub fn capture_object(obj: &dyn Object) -> Result<ObjectSnapshot, SerializeError> {
    let meta = obj.meta_object();
    let class_name = meta.class_name.to_owned();
    let mut properties = std::collections::BTreeMap::new();
    for prop in meta.properties {
        if !prop.flags.contains(PropertyFlag::Stored) {
            continue;
        }
        let value =
            obj.read_property(prop.name)
                .ok_or_else(|| SerializeError::PropertyMissing {
                    class_name: class_name.clone(),
                    property: prop.name.to_owned(),
                })?;
        properties.insert(prop.name.to_owned(), value);
    }
    Ok(ObjectSnapshot {
        class_name,
        properties,
    })
}

/// Restores an object from an [`ObjectSnapshot`] using the process-wide
/// [`ObjectFactory`].
///
/// Constructs a fresh instance of `snap.class_name`, then writes each property
/// from the snapshot back into it. Non-`Stored` properties retain their
/// default-constructed values.
///
/// Signal connections are **not** restored — the restored object starts with an
/// empty connection table and `signals_blocked = false`.
///
/// # Parameters
///
/// - `snap`: the snapshot to restore from.
///
/// # Errors
///
/// - [`DeserializeError::FactoryMissing`] — `ObjectFactory::global()` is `None`.
/// - [`DeserializeError::UnknownClass`] — no constructor is registered for
///   `snap.class_name`.
/// - [`DeserializeError::WriteRejected`] — `write_property` returned `false`
///   for a property value in the snapshot.
///
/// # Examples
///
/// ```no_run
/// use quartzite_core::snapshot::ObjectSnapshot;
/// use quartzite_runtime::snapshot::restore_object;
///
/// // let snap: ObjectSnapshot = ...;
/// // let obj = restore_object(&snap).unwrap();
/// ```
pub fn restore_object(snap: &ObjectSnapshot) -> Result<Box<dyn Object>, DeserializeError> {
    let factory_arc = ObjectFactory::global().ok_or(DeserializeError::FactoryMissing)?;
    let factory = factory_arc.read();
    let mut obj =
        factory
            .create(&snap.class_name)
            .ok_or_else(|| DeserializeError::UnknownClass {
                name: snap.class_name.clone(),
            })?;
    drop(factory);

    for (name, value) in &snap.properties {
        if !obj.write_property(name, value.clone()) {
            return Err(DeserializeError::WriteRejected {
                class_name: snap.class_name.clone(),
                property: name.clone(),
            });
        }
    }
    Ok(obj)
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
        traits::{AsObject, Object, SignalCallback},
        value::Value,
    };

    use super::*;

    // --- Test fixture ---

    struct Sample {
        base: ObjectBase,
        count: i64,
        name: String,
        cache: i64, // not Stored
    }

    impl Sample {
        fn new_boxed() -> Box<dyn Object> {
            Box::new(Sample {
                base: ObjectBase::new(),
                count: 0,
                name: String::new(),
                cache: 0,
            })
        }
    }

    impl AsObject for Sample {
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

    static SAMPLE_PROPS: &[PropertyMeta] = &[
        PropertyMeta::new(
            "count",
            "i64",
            enumflags2::make_bitflags!(PropertyFlag::{Readable | Writable | Stored | Designable}),
        ),
        PropertyMeta::new(
            "name",
            "String",
            enumflags2::make_bitflags!(PropertyFlag::{Readable | Writable | Stored | Designable}),
        ),
        // Stored NOT set
        PropertyMeta::new(
            "cache",
            "i64",
            enumflags2::make_bitflags!(PropertyFlag::{Readable | Writable | Designable}),
        ),
    ];

    static SAMPLE_META: MetaObject = MetaObject::new(
        "SnapshotSample",
        SAMPLE_PROPS,
        &[],
        &[],
        &[],
        noop_lookup_property,
        noop_lookup_signal,
        noop_lookup_method,
        noop_lookup_enum,
    );

    impl Object for Sample {
        fn meta_object(&self) -> &'static MetaObject {
            &SAMPLE_META
        }

        fn read_property(&self, name: &str) -> Option<Value> {
            match name {
                "count" => Some(Value::Int(self.count)),
                "name" => Some(Value::String(self.name.clone())),
                "cache" => Some(Value::Int(self.cache)),
                _ => None,
            }
        }

        fn write_property(&mut self, name: &str, value: Value) -> bool {
            match (name, value) {
                ("count", Value::Int(n)) => {
                    self.count = n;
                    true
                }
                ("name", Value::String(s)) => {
                    self.name = s;
                    true
                }
                ("cache", Value::Int(n)) => {
                    self.cache = n;
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

    // Install factory once per test binary.
    static FACTORY_INSTALLED: OnceLock<()> = OnceLock::new();

    fn install_factory() {
        FACTORY_INSTALLED.get_or_init(|| {
            let mut factory = ObjectFactory::new();
            factory.register("SnapshotSample", Sample::new_boxed);
            if ObjectFactory::install(factory).is_err() {
                // Factory already installed (shared process); register into existing one.
                if let Some(arc) = ObjectFactory::global() {
                    arc.write().register("SnapshotSample", Sample::new_boxed);
                }
            }
        });
    }

    #[test]
    fn capture_includes_stored_excludes_non_stored() {
        let _lock = quartzite_test_helpers::test_lock();
        let mut s = Sample::new_boxed();
        s.write_property("count", Value::Int(7));
        s.write_property("name", Value::String("hello".into()));
        s.write_property("cache", Value::Int(99));

        let snap = capture_object(s.as_ref()).unwrap();
        assert_eq!(snap.class_name, "SnapshotSample");
        assert_eq!(snap.properties.get("count"), Some(&Value::Int(7)));
        assert_eq!(
            snap.properties.get("name"),
            Some(&Value::String("hello".into()))
        );
        // 'cache' is not Stored — must be absent
        assert!(!snap.properties.contains_key("cache"));
    }

    #[test]
    fn restore_object_round_trips_stored_props() {
        let _lock = quartzite_test_helpers::test_lock();
        install_factory();

        let mut s = Sample::new_boxed();
        s.write_property("count", Value::Int(42));
        s.write_property("name", Value::String("world".into()));
        s.write_property("cache", Value::Int(999));

        let snap = capture_object(s.as_ref()).unwrap();
        let restored = restore_object(&snap).unwrap();

        // Stored props match
        assert_eq!(restored.read_property("count"), Some(Value::Int(42)));
        assert_eq!(
            restored.read_property("name"),
            Some(Value::String("world".into()))
        );
        // Non-stored prop keeps default
        assert_eq!(restored.read_property("cache"), Some(Value::Int(0)));
    }

    #[test]
    fn restore_object_unknown_class_returns_error() {
        let _lock = quartzite_test_helpers::test_lock();
        install_factory();

        let snap = ObjectSnapshot {
            class_name: "DoesNotExist".into(),
            properties: Default::default(),
        };
        assert!(matches!(
            restore_object(&snap),
            Err(DeserializeError::UnknownClass { name }) if name == "DoesNotExist"
        ));
    }

    #[test]
    fn restore_object_type_mismatch_returns_write_rejected() {
        let _lock = quartzite_test_helpers::test_lock();
        install_factory();

        let snap = ObjectSnapshot {
            class_name: "SnapshotSample".into(),
            properties: [("count".into(), Value::Bool(true))].into_iter().collect(),
        };
        assert!(matches!(
            restore_object(&snap),
            Err(DeserializeError::WriteRejected { .. })
        ));
    }

    // --- BrokenObject: meta declares "broken" as Stored but read_property returns None for it ---

    struct BrokenObject {
        base: ObjectBase,
    }

    impl BrokenObject {
        fn new_boxed() -> Box<dyn Object> {
            Box::new(BrokenObject {
                base: ObjectBase::new(),
            })
        }
    }

    impl AsObject for BrokenObject {
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

    static BROKEN_PROPS: &[PropertyMeta] = &[PropertyMeta::new(
        "broken",
        "i64",
        enumflags2::make_bitflags!(PropertyFlag::{Readable | Writable | Stored | Designable}),
    )];

    static BROKEN_META: MetaObject = MetaObject::new(
        "BrokenObject",
        BROKEN_PROPS,
        &[],
        &[],
        &[],
        noop_lookup_property,
        noop_lookup_signal,
        noop_lookup_method,
        noop_lookup_enum,
    );

    impl Object for BrokenObject {
        fn meta_object(&self) -> &'static MetaObject {
            &BROKEN_META
        }

        // Intentionally returns None for "broken" even though it's Stored — simulates
        // a meta-invariant violation so capture_object returns PropertyMissing.
        fn read_property(&self, _name: &str) -> Option<Value> {
            None
        }

        fn write_property(&mut self, _name: &str, _value: Value) -> bool {
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

        fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
            None
        }
    }

    #[test]
    fn capture_object_returns_property_missing_when_read_returns_none() {
        let obj = BrokenObject::new_boxed();
        assert!(matches!(
            capture_object(obj.as_ref()),
            Err(SerializeError::PropertyMissing { property, .. }) if property == "broken"
        ));
    }
}
