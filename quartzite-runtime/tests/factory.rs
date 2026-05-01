use quartzite_core::{
    id::ConnectionId,
    meta::MetaObject,
    object_base::ObjectBase,
    traits::{AsObject, Object, SignalCallback},
    value::Value,
};
use quartzite_runtime::ObjectFactory;

struct FooObj {
    base: ObjectBase,
}

impl FooObj {
    fn new_boxed() -> Box<dyn Object> {
        Box::new(FooObj {
            base: ObjectBase::new(),
        })
    }
}

impl AsObject for FooObj {
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

static FOO_META: MetaObject = MetaObject::new("FooObj", &[], &[], &[], &[]);

impl Object for FooObj {
    fn meta_object(&self) -> &'static MetaObject {
        &FOO_META
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

// AC8 — registered class creates an instance.
#[test]
fn registered_class_creates_instance() {
    let mut factory = ObjectFactory::new();
    factory.register("FooObj", FooObj::new_boxed);
    assert!(
        factory.create("FooObj").is_some(),
        "registered class must produce an instance"
    );
}

// Unregistered class returns None.
#[test]
fn unregistered_class_returns_none() {
    let factory = ObjectFactory::new();
    assert!(factory.create("Nonexistent").is_none());
}

// Re-registering a class name overwrites the previous constructor.
#[test]
fn duplicate_registration_overwrites() {
    let mut factory = ObjectFactory::new();
    factory.register("FooObj", FooObj::new_boxed);
    factory.register("FooObj", FooObj::new_boxed);
    assert!(factory.create("FooObj").is_some());
}

// Multiple classes can be registered independently.
#[test]
fn multiple_classes_independent() {
    let mut factory = ObjectFactory::new();
    factory.register("A", FooObj::new_boxed);
    factory.register("B", FooObj::new_boxed);
    assert!(factory.create("A").is_some());
    assert!(factory.create("B").is_some());
    assert!(factory.create("C").is_none());
}
