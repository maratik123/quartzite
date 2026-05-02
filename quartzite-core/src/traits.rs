//! Core object traits: `AsObject`, `Object`, `ObjectExt`, and `SignalCallback`.
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

use crate::{
    id::{ConnectionId, ObjectId},
    meta::MetaObject,
    object_base::ObjectBase,
    value::Value,
};

/// Boxed callback type used by `Object::connect_signal`.
///
/// The `Send` bound future-proofs the API for queued (cross-thread) signal delivery,
/// which is handled by `quartzite-runtime`.
pub type SignalCallback = Box<dyn Fn(&[Value]) + Send + Sync>;

/// Object-safe accessor trait. Every concrete object type implements this to expose
/// its `ObjectBase` and allow `Any`-based downcasting.
///
/// This trait is deliberately minimal so that `Box<dyn AsObject>` remains valid.
pub trait AsObject {
    /// Returns a shared reference to this object's `ObjectBase`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::AsObject;
    /// # fn example(obj: &impl AsObject) {
    /// let base = obj.object_base();
    /// println!("object name: {}", base.name);
    /// # }
    /// ```
    fn object_base(&self) -> &ObjectBase;

    /// Returns a mutable reference to this object's `ObjectBase`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::AsObject;
    /// # fn example(obj: &mut impl AsObject) {
    /// obj.object_base_mut().name = "renamed".into();
    /// # }
    /// ```
    fn object_base_mut(&mut self) -> &mut ObjectBase;

    /// Upcast to `&dyn Any` to enable checked downcasting via `downcast_ref`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::AsObject;
    /// # fn example<T: AsObject + 'static>(obj: &impl AsObject) {
    /// if let Some(concrete) = obj.as_any().downcast_ref::<T>() {
    ///     // use concrete
    /// }
    /// # }
    /// ```
    fn as_any(&self) -> &dyn core::any::Any;

    /// Upcast to `&mut dyn Any` to enable checked downcasting via `downcast_mut`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::AsObject;
    /// # fn example<T: AsObject + 'static>(obj: &mut impl AsObject) {
    /// if let Some(concrete) = obj.as_any_mut().downcast_mut::<T>() {
    ///     // mutate concrete
    /// }
    /// # }
    /// ```
    fn as_any_mut(&mut self) -> &mut dyn core::any::Any;
}

/// Full meta-system trait. Extends `AsObject` with property access, method invocation,
/// and signal connection. Also, object-safe: all methods use `&str`, `&[Value]`, and
/// boxed closures — no generics that would break object safety.
pub trait Object: AsObject + Send {
    /// Returns the static `MetaObject` descriptor for this type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::Object;
    /// # fn example(obj: &impl Object) {
    /// let meta = obj.meta_object();
    /// println!("class: {}", meta.class_name);
    /// # }
    /// ```
    fn meta_object(&self) -> &'static MetaObject;

    /// Returns the current value of `name`, or `None` if `name` is not a known property.
    fn read_property(&self, name: &str) -> Option<Value>;

    /// Sets property `name` to `val`. Returns `true` on success.
    ///
    /// Returns `false` if `name` is unknown, the property is read-only, or `val` has
    /// the wrong type.
    fn write_property(&mut self, name: &str, val: Value) -> bool;

    /// Invokes method `name` with `args`. Returns `Some(result)` on success.
    ///
    /// Returns `None` if `name` is unknown, the argument count is wrong, or any
    /// argument fails type conversion.
    fn invoke_method(&mut self, name: &str, args: &[Value]) -> Option<Value>;

    /// Connects a dynamic callback to signal `name`. Returns `Some(id)` on success.
    ///
    /// Returns `None` if `name` does not match any signal on this object.
    fn connect_signal(&mut self, signal: &str, callback: SignalCallback) -> Option<ConnectionId>;
}

/// Ergonomic blanket-impl extension. Methods here are not object-safe (e.g., generics
/// in `downcast_ref`), so they live in a separate trait that is not dyn-compatible.
///
/// Automatically implemented for every type that implements `AsObject`.
pub trait ObjectExt: AsObject {
    /// Returns the unique `ObjectId` of this object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::traits::ObjectExt;
    /// // obj is any type that implements AsObject
    /// # fn example(obj: &impl ObjectExt) {
    /// let id = obj.id();
    /// # }
    /// ```
    fn id(&self) -> ObjectId {
        self.object_base().id()
    }

    /// Returns the current name of this object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::traits::ObjectExt;
    /// # fn example(obj: &impl ObjectExt) {
    /// let name: &str = obj.name();
    /// # }
    /// ```
    fn name(&self) -> &str {
        &self.object_base().name
    }

    /// Replaces the object's name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::traits::ObjectExt;
    /// # fn example(obj: &mut impl ObjectExt) {
    /// obj.set_name("new-name");
    /// assert_eq!(obj.name(), "new-name");
    /// # }
    /// ```
    fn set_name(&mut self, name: impl Into<String>) {
        self.object_base_mut().name = name.into();
    }

    /// Returns `true` when called on the same thread that created this object.
    /// Only available with the `std` feature (requires `std::thread`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::traits::ObjectExt;
    /// # fn example(obj: &impl ObjectExt) {
    /// assert!(obj.is_on_current_thread());
    /// # }
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn is_on_current_thread(&self) -> bool {
        self.object_base().is_on_current_thread()
    }

    /// Attempts a checked downcast to a concrete type. Returns `Some(&T)` when the
    /// underlying type is exactly `T`, `None` otherwise (AC8).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::traits::{AsObject, ObjectExt};
    /// # fn example<T: AsObject + 'static>(obj: &impl ObjectExt) {
    /// if let Some(concrete) = obj.downcast_ref::<T>() {
    ///     // use concrete
    /// }
    /// # }
    /// ```
    fn downcast_ref<T: AsObject + 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    /// Mutable variant of `downcast_ref`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::traits::{AsObject, ObjectExt};
    /// # fn example<T: AsObject + 'static>(obj: &mut impl ObjectExt) {
    /// if let Some(concrete) = obj.downcast_mut::<T>() {
    ///     // mutate concrete
    /// }
    /// # }
    /// ```
    fn downcast_mut<T: AsObject + 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }

    /// Returns `true` when the underlying concrete type is `T`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::traits::{AsObject, ObjectExt};
    /// # fn example<T: AsObject + 'static>(obj: &impl ObjectExt) {
    /// let matched: bool = obj.is::<T>();
    /// # }
    /// ```
    fn is<T: AsObject + 'static>(&self) -> bool {
        self.as_any().is::<T>()
    }
}

/// Blanket implementation — every `AsObject` automatically gets `ObjectExt`.
impl<T: AsObject> ObjectExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{id::ConnectionId, meta::MetaObject, object_base::ObjectBase, value::Value};

    // --- Minimal concrete object for testing ---

    struct DummyObject {
        base: ObjectBase,
    }

    impl DummyObject {
        fn new() -> Self {
            Self {
                base: ObjectBase::new(),
            }
        }
    }

    impl AsObject for DummyObject {
        fn object_base(&self) -> &ObjectBase {
            &self.base
        }

        fn object_base_mut(&mut self) -> &mut ObjectBase {
            &mut self.base
        }

        fn as_any(&self) -> &dyn core::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
            self
        }
    }

    static DUMMY_META: MetaObject = MetaObject::new(
        "DummyObject",
        &[],
        &[],
        &[],
        &[],
        crate::meta::noop_lookup_property,
        crate::meta::noop_lookup_signal,
        crate::meta::noop_lookup_method,
        crate::meta::noop_lookup_enum,
    );

    impl Object for DummyObject {
        fn meta_object(&self) -> &'static MetaObject {
            &DUMMY_META
        }

        fn read_property(&self, _name: &str) -> Option<Value> {
            None
        }

        fn write_property(&mut self, _name: &str, _val: Value) -> bool {
            false
        }

        fn invoke_method(&mut self, _name: &str, _args: &[Value]) -> Option<Value> {
            None
        }

        fn connect_signal(
            &mut self,
            _signal: &str,
            _callback: SignalCallback,
        ) -> Option<ConnectionId> {
            None
        }
    }

    /// Another type to verify cross-type downcast returns None.
    struct OtherObject {
        base: ObjectBase,
    }

    impl AsObject for OtherObject {
        fn object_base(&self) -> &ObjectBase {
            &self.base
        }

        fn object_base_mut(&mut self) -> &mut ObjectBase {
            &mut self.base
        }

        fn as_any(&self) -> &dyn core::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
            self
        }
    }

    // --- Tests ---

    #[test]
    fn downcast_ref_correct_type() {
        let obj = DummyObject::new();
        // Downcast via trait object (exercises the Any path).
        let boxed: Box<dyn AsObject> = Box::new(DummyObject::new());
        assert!(boxed.as_any().downcast_ref::<DummyObject>().is_some());
        // Also via ObjectExt directly.
        assert!(obj.downcast_ref::<DummyObject>().is_some());
    }

    #[test]
    fn downcast_ref_wrong_type() {
        let obj = DummyObject::new();
        assert!(obj.downcast_ref::<OtherObject>().is_none());
    }

    #[test]
    fn object_ext_name_round_trip() {
        let mut obj = DummyObject::new();
        assert_eq!(obj.name(), "");
        obj.set_name("test-name");
        assert_eq!(obj.name(), "test-name");
    }

    #[test]
    fn object_ext_id_matches_base() {
        let obj = DummyObject::new();
        assert_eq!(obj.id(), obj.object_base().id());
    }

    #[test]
    fn is_returns_true_for_correct_type() {
        let obj = DummyObject::new();
        assert!(obj.is::<DummyObject>());
        assert!(!obj.is::<OtherObject>());
    }

    #[test]
    #[cfg(feature = "std")]
    fn is_on_current_thread_true_for_creator() {
        let obj = DummyObject::new();
        assert!(obj.is_on_current_thread());
    }
}
