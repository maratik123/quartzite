use quartzite_core::{
    id::ConnectionId,
    meta::MetaObject,
    object_base::ObjectBase,
    traits::{AsObject, Object, SignalCallback},
    value::Value,
};
use quartzite_runtime::{FactoryAlreadySet, ObjectFactory};

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

static FOO_META: MetaObject = MetaObject::new(
    "FooObj",
    &[],
    &[],
    &[],
    &[],
    quartzite_core::meta::noop_lookup_property,
    quartzite_core::meta::noop_lookup_signal,
    quartzite_core::meta::noop_lookup_method,
    quartzite_core::meta::noop_lookup_enum,
);

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

    fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
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

// --- Singleton tests ---
//
// OnceLock is irrevocable per process. We coordinate with a helper that calls
// install exactly once per binary, regardless of test execution order.

static FIRST_INSTALL: std::sync::OnceLock<Result<(), FactoryAlreadySet>> =
    std::sync::OnceLock::new();

fn install_once() -> &'static Result<(), FactoryAlreadySet> {
    FIRST_INSTALL.get_or_init(|| ObjectFactory::install(ObjectFactory::new()))
}

#[test]
fn first_install_returns_ok() {
    assert_eq!(install_once(), &Ok(()));
}

#[test]
fn install_second_call_returns_factory_already_set() {
    install_once();
    let result = ObjectFactory::install(ObjectFactory::new());
    assert_eq!(result, Err(FactoryAlreadySet));
}

#[test]
fn global_returns_some_after_install() {
    install_once();
    assert!(ObjectFactory::global().is_some());
}

#[test]
fn create_through_global_unregistered_returns_none() {
    install_once();
    let factory = ObjectFactory::global().expect("factory not installed");
    assert!(
        factory
            .read()
            .expect("lock poisoned")
            .create("NoSuchClass")
            .is_none()
    );
}

#[test]
fn register_and_create_through_global() {
    install_once();
    let factory = ObjectFactory::global().expect("factory not installed");
    factory
        .write()
        .expect("lock poisoned")
        .register("GlobalFoo", FooObj::new_boxed);
    assert!(
        factory
            .read()
            .expect("lock poisoned")
            .create("GlobalFoo")
            .is_some()
    );
}
