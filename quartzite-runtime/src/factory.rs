//! Object factory: creates objects by class name for scripting and serialization.
use std::collections::HashMap;

use quartzite_core::traits::Object;

type Constructor = Box<dyn Fn() -> Box<dyn Object> + Send + Sync>;

/// Creates objects by class name string — used by scripting and serialization.
pub struct ObjectFactory {
    registry: HashMap<String, Constructor>,
}

impl ObjectFactory {
    /// Create an empty `ObjectFactory` with no registered constructors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ObjectFactory;
    ///
    /// let factory = ObjectFactory::new();
    /// assert!(factory.create("Unknown").is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Register a constructor for `class_name`. Overwrites any existing entry.
    pub fn register<F>(&mut self, class_name: impl Into<String>, ctor: F)
    where
        F: Fn() -> Box<dyn Object> + Send + Sync + 'static,
    {
        self.registry.insert(class_name.into(), Box::new(ctor));
    }

    /// Create an instance of `class_name`. Returns `None` if not registered.
    pub fn create(&self, class_name: &str) -> Option<Box<dyn Object>> {
        self.registry.get(class_name).map(|ctor| ctor())
    }
}

impl Default for ObjectFactory {
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

    static TEST_META: MetaObject = MetaObject::new("TestObj", &[], &[], &[], &[]);

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
        fn connect_signal(&mut self, _: &str, _: SignalCallback) -> Option<ConnectionId> {
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
