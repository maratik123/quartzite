//! Object factory: creates objects by class name for scripting and serialization.
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use quartzite_core::traits::Object;

type Constructor = Box<dyn Fn() -> Box<dyn Object> + Send + Sync>;

static FACTORY: OnceLock<Arc<RwLock<ObjectFactory>>> = OnceLock::new();

/// Error returned by [`ObjectFactory::install`] when a factory is already registered.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::{FactoryAlreadySet, ObjectFactory};
///
/// ObjectFactory::install(ObjectFactory::new()).unwrap();
/// let err: FactoryAlreadySet = ObjectFactory::install(ObjectFactory::new()).unwrap_err();
/// assert_eq!(err, FactoryAlreadySet);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("ObjectFactory is already installed")]
pub struct FactoryAlreadySet;

/// Creates objects by class name string — used by scripting and serialization.
///
/// # Examples
///
/// ```
/// use quartzite_runtime::ObjectFactory;
///
/// let factory = ObjectFactory::new();
/// assert!(factory.create("Unknown").is_none());
/// ```
#[derive(Default)]
pub struct ObjectFactory {
    registry: HashMap<String, Constructor>,
}

impl ObjectFactory {
    /// Creates an empty `ObjectFactory` with no registered constructors.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_runtime::ObjectFactory;
    ///
    /// let factory = ObjectFactory::new();
    /// assert!(factory.create("Unknown").is_none());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Installs `factory` as the process-wide singleton.
    ///
    /// Called by [`Application::new`](crate::Application::new) automatically — explicit
    /// calls are needed only when using the factory without an `Application`.
    ///
    /// # Parameters
    ///
    /// - `factory`: factory instance to install as the global singleton.
    ///
    /// # Errors
    ///
    /// Returns [`FactoryAlreadySet`] if a process-wide factory has already been installed.
    /// Only one factory may exist per process.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectFactory;
    ///
    /// ObjectFactory::install(ObjectFactory::new()).expect("factory already set");
    /// ```
    pub fn install(factory: ObjectFactory) -> Result<(), FactoryAlreadySet> {
        FACTORY
            .set(Arc::new(RwLock::new(factory)))
            .map_err(|_| FactoryAlreadySet)
    }

    /// Returns a reference-counted handle to the process-wide factory, or `None` if
    /// [`install`](ObjectFactory::install) has not been called yet.
    ///
    /// Callers must lock the returned `RwLock` to read or mutate the factory:
    /// - `global().unwrap().read().expect("poisoned").create("Foo")`,
    /// - `global().unwrap().write().expect("poisoned").register("Foo", ctor)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectFactory;
    ///
    /// if let Some(factory) = ObjectFactory::global() {
    ///     let _obj = factory.read().expect("poisoned").create("MyClass");
    /// }
    /// ```
    #[inline]
    pub fn global() -> Option<Arc<RwLock<ObjectFactory>>> {
        FACTORY.get().cloned()
    }

    /// Registers a constructor for `class_name`. Overwrites any existing entry.
    ///
    /// _Simple._
    ///
    /// # Parameters
    ///
    /// - `class_name`: the class name string used to look up this constructor.
    /// - `ctor`: closure returning a freshly boxed object instance on each call.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectFactory;
    ///
    /// let mut factory = ObjectFactory::new();
    /// // factory.register("MyObject", || Box::new(MyObject::new()));
    /// ```
    pub fn register<F>(&mut self, class_name: impl Into<String>, ctor: F)
    where
        F: Fn() -> Box<dyn Object> + Send + Sync + 'static,
    {
        fn inner(this: &mut ObjectFactory, class_name: String, ctor: Constructor) {
            this.registry.insert(class_name, ctor);
        }
        inner(self, class_name.into(), Box::new(ctor))
    }

    /// Creates an instance of `class_name`. Returns `None` if not registered.
    ///
    /// # Parameters
    ///
    /// - `class_name`: the class name string to look up; matched against
    ///   [`register`](Self::register) calls.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_runtime::ObjectFactory;
    ///
    /// let factory = ObjectFactory::new();
    /// assert!(factory.create("Unknown").is_none());
    /// ```
    pub fn create(&self, class_name: &str) -> Option<Box<dyn Object>> {
        self.registry.get(class_name).map(|ctor| ctor())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_core::{
        id::ConnectionId,
        meta::MetaObject,
        object_base::ObjectBase,
        signal::ConnectionType,
        traits::{AsObject, Object, SignalCallback},
        value::Value,
    };

    struct TestObj {
        base: ObjectBase,
    }

    impl TestObj {
        fn new_boxed() -> Box<dyn Object> {
            Box::new(TestObj {
                base: ObjectBase::new(),
            })
        }
    }

    impl AsObject for TestObj {
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

    static TEST_META: MetaObject = MetaObject::new(
        "TestObj",
        &[],
        &[],
        &[],
        &[],
        quartzite_core::meta::noop_lookup_property,
        quartzite_core::meta::noop_lookup_signal,
        quartzite_core::meta::noop_lookup_method,
        quartzite_core::meta::noop_lookup_enum,
    );

    impl Object for TestObj {
        fn meta_object(&self) -> &'static MetaObject {
            &TEST_META
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

        fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
            None
        }
    }

    #[test]
    fn registered_class_creates_instance() {
        let mut factory = ObjectFactory::new();
        factory.register("TestObj", TestObj::new_boxed);
        assert!(factory.create("TestObj").is_some());
    }

    #[test]
    fn unregistered_class_returns_none() {
        let factory = ObjectFactory::new();
        assert!(factory.create("Unknown").is_none());
    }

    #[test]
    fn duplicate_registration_overwrites() {
        let mut factory = ObjectFactory::new();
        factory.register("TestObj", TestObj::new_boxed);
        factory.register("TestObj", TestObj::new_boxed);
        assert!(factory.create("TestObj").is_some());
    }
}
