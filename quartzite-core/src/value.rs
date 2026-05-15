//! Dynamic value type and type-conversion traits.
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Trait for user-defined value types stored inside `Value::Custom`.
///
/// Implementors must support cloning (`clone_box`), downcast (`as_any`), and
/// `Debug` formatting (required as a supertrait so that `Box<dyn CustomValue>`
/// and `Arc<dyn CustomValue>` are automatically `Debug`).
///
/// When the `serde` feature is enabled, every concrete implementation of this
/// trait must additionally be annotated with `#[typetag::serde]` on its `impl`
/// block so that `Value::Custom` can be serialized and deserialized correctly.
/// Without the annotation, the impl will fail to compile when `serde` is active.
///
/// # Examples
///
/// ```
/// use quartzite_core::value::CustomValue;
///
/// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// #[derive(Debug, Clone)]
/// struct MyVal(i32);
///
/// #[cfg_attr(feature = "serde", typetag::serde)]
/// impl CustomValue for MyVal {
///     fn type_name(&self) -> &'static str { "MyVal" }
///     fn clone_box(&self) -> Box<dyn CustomValue> { Box::new(self.clone()) }
///     fn as_any(&self) -> &dyn core::any::Any { self }
/// }
///
/// let v: Box<dyn CustomValue> = Box::new(MyVal(42));
/// assert_eq!(v.type_name(), "MyVal");
/// ```
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait CustomValue: core::any::Any + core::fmt::Debug + Send + Sync {
    /// Returns a static string identifying the concrete type (e.g. `"MyVal"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::value::CustomValue;
    ///
    /// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// #[derive(Debug, Clone)]
    /// struct MyVal;
    ///
    /// #[cfg_attr(feature = "serde", typetag::serde)]
    /// impl CustomValue for MyVal {
    ///     fn type_name(&self) -> &'static str { "MyVal" }
    ///     fn clone_box(&self) -> Box<dyn CustomValue> { Box::new(self.clone()) }
    ///     fn as_any(&self) -> &dyn core::any::Any { self }
    /// }
    ///
    /// assert_eq!(MyVal.type_name(), "MyVal");
    /// ```
    fn type_name(&self) -> &'static str;

    /// Clones this value into a new `Box<dyn CustomValue>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::value::CustomValue;
    ///
    /// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// #[derive(Debug, Clone)]
    /// struct MyVal;
    ///
    /// #[cfg_attr(feature = "serde", typetag::serde)]
    /// impl CustomValue for MyVal {
    ///     fn type_name(&self) -> &'static str { "MyVal" }
    ///     fn clone_box(&self) -> Box<dyn CustomValue> { Box::new(self.clone()) }
    ///     fn as_any(&self) -> &dyn core::any::Any { self }
    /// }
    ///
    /// let cloned: Box<dyn CustomValue> = MyVal.clone_box();
    /// assert_eq!(cloned.type_name(), "MyVal");
    /// ```
    fn clone_box(&self) -> Box<dyn CustomValue>;

    /// Upcasts to `&dyn Any` for checked downcasting.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::value::CustomValue;
    ///
    /// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// #[derive(Debug, Clone)]
    /// struct MyVal(i32);
    ///
    /// #[cfg_attr(feature = "serde", typetag::serde)]
    /// impl CustomValue for MyVal {
    ///     fn type_name(&self) -> &'static str { "MyVal" }
    ///     fn clone_box(&self) -> Box<dyn CustomValue> { Box::new(self.clone()) }
    ///     fn as_any(&self) -> &dyn core::any::Any { self }
    /// }
    ///
    /// let v = MyVal(7);
    /// assert!(v.as_any().downcast_ref::<MyVal>().is_some());
    /// ```
    fn as_any(&self) -> &dyn core::any::Any;
}

impl Clone for Box<dyn CustomValue> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Weak reference to a runtime object. Wraps the `ObjectId` inner `u64`.
///
/// Liveness is confirmed by `ObjectTree::is_valid(ref)` in `quartzite-runtime`.
/// The type is intentionally opaque here so that `quartzite-core` stays `no_std`-safe
/// and runtime-agnostic.
///
/// # Examples
///
/// ```
/// use quartzite_core::value::WeakObjectRef;
///
/// let r = WeakObjectRef(42);
/// assert_eq!(r.0, 42);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WeakObjectRef(pub u64);

/// A closed, dynamically-typed value container.
///
/// The `Custom` variant allows storing arbitrary user types via `Arc<dyn CustomValue>`.
/// `Value::Null` is the default.
///
/// # Examples
///
/// ```
/// use quartzite_core::value::Value;
///
/// assert_eq!(Value::default(), Value::Null);
/// assert_eq!(Value::Int(7).type_name(), "Int");
/// ```
#[derive(Clone, Debug, Default)]
pub enum Value {
    /// Absence of a value; the default variant.
    #[default]
    Null,
    /// A boolean value.
    Bool(bool),
    /// A 64-bit signed integer.
    Int(i64),
    /// A 64-bit floating-point number.
    Float(f64),
    /// A UTF-8 string.
    String(String),
    /// An ordered list of values.
    List(Vec<Value>),
    /// A string-keyed map of values (ordered by key).
    Map(BTreeMap<String, Value>),
    /// Raw byte sequence.
    Bytes(Vec<u8>),
    /// An arbitrary user-defined value implementing [`CustomValue`].
    Custom(Arc<dyn CustomValue>),
    /// A weak reference to a runtime object (see [`WeakObjectRef`]).
    Object(WeakObjectRef),
    /// A time duration.
    Duration(core::time::Duration),
}

// `Value` intentionally does not implement `Eq`: the `Float` variant uses
// IEEE 754 semantics where NaN != NaN, making reflexivity unsound.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            // Custom values are compared by Arc pointer identity, not deep equality.
            (Value::Custom(a), Value::Custom(b)) => Arc::ptr_eq(a, b),
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Duration(a), Value::Duration(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    /// Returns a static string naming the active variant (for use in error messages).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::value::Value;
    ///
    /// assert_eq!(Value::Int(42).type_name(), "Int");
    /// assert_eq!(Value::Null.type_name(), "Null");
    /// assert_eq!(Value::Bool(true).type_name(), "Bool");
    /// assert_eq!(Value::Duration(core::time::Duration::from_secs(1)).type_name(), "Duration");
    /// ```
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Bool(_) => "Bool",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
            Value::Bytes(_) => "Bytes",
            Value::Custom(_) => "Custom",
            Value::Object(_) => "Object",
            Value::Duration(_) => "Duration",
        }
    }
}

/// Error returned when a `FromValue` conversion fails due to a type mismatch.
///
/// # Examples
///
/// ```
/// use quartzite_core::value::{FromValue, TypeError, Value};
///
/// let err = i64::from_value(Value::Bool(true)).unwrap_err();
/// assert_eq!(err.expected, "Int");
/// assert_eq!(err.got, "Bool");
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("type error: expected {expected}, got {got}")]
pub struct TypeError {
    /// The type name that was expected (e.g. `"Int"`).
    pub expected: &'static str,
    /// The type name that was actually found (e.g. `"String"`).
    pub got: &'static str,
}

/// Converts a [`Value`] into a concrete Rust type.
///
/// # Examples
///
/// ```
/// use quartzite_core::value::{FromValue, Value};
///
/// assert_eq!(i64::from_value(Value::Int(7)), Ok(7i64));
/// assert!(bool::from_value(Value::Int(0)).is_err());
/// ```
pub trait FromValue: Sized {
    /// Attempts to convert `val` into `Self`.
    ///
    /// # Parameters
    ///
    /// - `val`: the dynamic value to convert; consumed by the call.
    ///
    /// # Errors
    ///
    /// Returns `Err(TypeError)` when `val` is not the variant expected by `Self`
    /// (for example, calling `i64::from_value(Value::Bool(_))`), or when the
    /// payload does not fit the target type (e.g. `u32::from_value(Value::Int(-1))`).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::value::{FromValue, Value};
    ///
    /// assert_eq!(i64::from_value(Value::Int(42)), Ok(42i64));
    /// ```
    fn from_value(val: Value) -> Result<Self, TypeError>;
}

/// Converts a concrete Rust type into a [`Value`].
///
/// # Examples
///
/// ```
/// use quartzite_core::value::{IntoValue, Value};
///
/// assert_eq!(true.into_value(), Value::Bool(true));
/// assert_eq!(1.5f64.into_value(), Value::Float(1.5));
/// ```
pub trait IntoValue {
    /// Wraps `self` in the appropriate `Value` variant.
    ///
    /// _Simple._
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::value::{IntoValue, Value};
    ///
    /// assert_eq!(42i64.into_value(), Value::Int(42));
    /// ```
    fn into_value(self) -> Value;
}

// `i64` is the native Int storage type — conversion is infallible.
impl FromValue for i64 {
    fn from_value(val: Value) -> Result<Self, TypeError> {
        match val {
            Value::Int(n) => Ok(n),
            _ => Err(TypeError {
                expected: "Int",
                got: val.type_name(),
            }),
        }
    }
}

impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Value::Int(self)
    }
}

// For other integer types, use a checked conversion so that out-of-range values
// produce a `TypeError` instead of silently wrapping or truncating.
macro_rules! impl_from_value_checked {
    ($t:ty) => {
        impl FromValue for $t {
            fn from_value(val: Value) -> Result<Self, TypeError> {
                match val {
                    Value::Int(n) => {
                        <$t as ::core::convert::TryFrom<i64>>::try_from(n).map_err(|_| TypeError {
                            expected: concat!("Int fitting ", stringify!($t)),
                            got: "Int",
                        })
                    }
                    _ => Err(TypeError {
                        expected: "Int",
                        got: val.type_name(),
                    }),
                }
            }
        }
    };
}

// i32 and u32 always fit in i64, so `as i64` is lossless.
macro_rules! impl_int_checked {
    ($t:ty) => {
        impl_from_value_checked!($t);
        impl IntoValue for $t {
            #[inline]
            fn into_value(self) -> Value {
                Value::Int(self as i64)
            }
        }
    };
}

impl_int_checked!(i32);
impl_int_checked!(u32);

// u64 and usize can exceed i64::MAX; saturate rather than wrap silently.
impl_from_value_checked!(u64);
impl IntoValue for u64 {
    #[inline]
    fn into_value(self) -> Value {
        Value::Int(i64::try_from(self).unwrap_or(i64::MAX))
    }
}

impl_from_value_checked!(usize);
impl IntoValue for usize {
    #[inline]
    fn into_value(self) -> Value {
        Value::Int(i64::try_from(self).unwrap_or(i64::MAX))
    }
}

impl FromValue for f64 {
    fn from_value(val: Value) -> Result<Self, TypeError> {
        match val {
            Value::Float(f) => Ok(f),
            _ => Err(TypeError {
                expected: "Float",
                got: val.type_name(),
            }),
        }
    }
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Value::Float(self)
    }
}

impl FromValue for f32 {
    fn from_value(val: Value) -> Result<Self, TypeError> {
        match val {
            Value::Float(f) => Ok(f as f32),
            _ => Err(TypeError {
                expected: "Float",
                got: val.type_name(),
            }),
        }
    }
}

impl IntoValue for f32 {
    fn into_value(self) -> Value {
        Value::Float(self as f64)
    }
}

impl FromValue for bool {
    fn from_value(val: Value) -> Result<Self, TypeError> {
        match val {
            Value::Bool(b) => Ok(b),
            _ => Err(TypeError {
                expected: "Bool",
                got: val.type_name(),
            }),
        }
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

impl FromValue for String {
    fn from_value(val: Value) -> Result<Self, TypeError> {
        match val {
            Value::String(s) => Ok(s),
            _ => Err(TypeError {
                expected: "String",
                got: val.type_name(),
            }),
        }
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::String(self)
    }
}

impl FromValue for Option<String> {
    /// Attempts to convert `val` into an `Option<String>`.
    ///
    /// # Parameters
    ///
    /// - `val`: must be `Value::Null` (yields `None`) or `Value::String` (yields `Some`);
    ///   any other variant returns `Err`.
    ///
    /// # Errors
    ///
    /// Returns `Err(TypeError)` when `val` is neither `Value::Null` nor `Value::String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::value::{FromValue, Value};
    ///
    /// assert_eq!(Option::<String>::from_value(Value::Null), Ok(None));
    /// assert_eq!(
    ///     Option::<String>::from_value(Value::String("hi".into())),
    ///     Ok(Some("hi".to_owned()))
    /// );
    /// assert!(Option::<String>::from_value(Value::Int(0)).is_err());
    /// ```
    fn from_value(val: Value) -> Result<Self, TypeError> {
        match val {
            Value::Null => Ok(None),
            Value::String(s) => Ok(Some(s)),
            _ => Err(TypeError {
                expected: "String or Null",
                got: val.type_name(),
            }),
        }
    }
}

impl IntoValue for Option<String> {
    /// Wraps `self` in `Value::String` when `Some`, or `Value::Null` when `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::value::{IntoValue, Value};
    ///
    /// assert_eq!(None::<String>.into_value(), Value::Null);
    /// assert_eq!(Some("hi".to_owned()).into_value(), Value::String("hi".into()));
    /// ```
    #[inline]
    fn into_value(self) -> Value {
        match self {
            None => Value::Null,
            Some(s) => Value::String(s),
        }
    }
}

impl FromValue for core::time::Duration {
    /// Attempts to convert `val` into a [`core::time::Duration`].
    ///
    /// # Parameters
    ///
    /// - `val`: must be `Value::Duration`; any other variant returns `Err`.
    ///
    /// # Errors
    ///
    /// Returns `Err(TypeError)` when `val` is not `Value::Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::time::Duration;
    /// use quartzite_core::value::{FromValue, Value};
    ///
    /// let d = Duration::from_secs(1);
    /// assert_eq!(Duration::from_value(Value::Duration(d)), Ok(d));
    /// assert!(Duration::from_value(Value::Int(0)).is_err());
    /// ```
    fn from_value(val: Value) -> Result<Self, TypeError> {
        match val {
            Value::Duration(d) => Ok(d),
            _ => Err(TypeError {
                expected: "Duration",
                got: val.type_name(),
            }),
        }
    }
}

impl IntoValue for core::time::Duration {
    /// Wraps `self` in `Value::Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::time::Duration;
    /// use quartzite_core::value::{IntoValue, Value};
    ///
    /// assert_eq!(Duration::from_secs(2).into_value(), Value::Duration(Duration::from_secs(2)));
    /// ```
    #[inline]
    fn into_value(self) -> Value {
        Value::Duration(self)
    }
}

// --- Serde impls (feature-gated) ---

#[cfg(feature = "serde")]
impl Serialize for WeakObjectRef {
    #[inline]
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for WeakObjectRef {
    #[inline]
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(WeakObjectRef(u64::deserialize(d)?))
    }
}

/// Proxy enum used by [`Deserialize`] for [`Value`].
///
/// Must match the variant order and names of [`Value`] exactly so that
/// backends that use the variant index (e.g. bincode) round-trip correctly.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
enum ValueProxy {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
    Bytes(Vec<u8>),
    Custom(Box<dyn CustomValue>),
    Object(u64),
    Duration(u64, u32),
}

#[cfg(feature = "serde")]
impl From<ValueProxy> for Value {
    fn from(p: ValueProxy) -> Self {
        match p {
            ValueProxy::Null => Value::Null,
            ValueProxy::Bool(v) => Value::Bool(v),
            ValueProxy::Int(v) => Value::Int(v),
            ValueProxy::Float(v) => Value::Float(v),
            ValueProxy::String(v) => Value::String(v),
            ValueProxy::List(v) => Value::List(v),
            ValueProxy::Map(v) => Value::Map(v),
            ValueProxy::Bytes(v) => Value::Bytes(v),
            ValueProxy::Custom(b) => Value::Custom(Arc::from(b)),
            ValueProxy::Object(id) => Value::Object(WeakObjectRef(id)),
            ValueProxy::Duration(secs, nanos) => {
                Value::Duration(core::time::Duration::new(secs, nanos))
            }
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTupleVariant;
        match self {
            Value::Null => s.serialize_unit_variant("Value", 0, "Null"),
            Value::Bool(v) => s.serialize_newtype_variant("Value", 1, "Bool", v),
            Value::Int(v) => s.serialize_newtype_variant("Value", 2, "Int", v),
            Value::Float(v) => s.serialize_newtype_variant("Value", 3, "Float", v),
            Value::String(v) => s.serialize_newtype_variant("Value", 4, "String", v),
            Value::List(v) => s.serialize_newtype_variant("Value", 5, "List", v),
            Value::Map(v) => s.serialize_newtype_variant("Value", 6, "Map", v),
            Value::Bytes(v) => s.serialize_newtype_variant("Value", 7, "Bytes", v),
            Value::Custom(arc) => {
                // typetag provides Serialize for Box<dyn CustomValue>; clone_box is the
                // minimal cost path to avoid double-borrowing the Arc's inner dyn.
                let boxed: Box<dyn CustomValue> = arc.as_ref().clone_box();
                s.serialize_newtype_variant("Value", 8, "Custom", &boxed)
            }
            Value::Object(WeakObjectRef(id)) => {
                s.serialize_newtype_variant("Value", 9, "Object", id)
            }
            Value::Duration(d) => {
                let mut tv = s.serialize_tuple_variant("Value", 10, "Duration", 2)?;
                tv.serialize_field(&d.as_secs())?;
                tv.serialize_field(&d.subsec_nanos())?;
                tv.end()
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(ValueProxy::deserialize(d)?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // --- Minimal CustomValue for testing (non-serde path) ---

    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone)]
    struct MyCustom;

    #[cfg_attr(feature = "serde", typetag::serde)]
    impl CustomValue for MyCustom {
        fn type_name(&self) -> &'static str {
            "MyCustom"
        }

        fn clone_box(&self) -> Box<dyn CustomValue> {
            Box::new(self.clone())
        }

        fn as_any(&self) -> &dyn core::any::Any {
            self
        }
    }

    // --- Basic variant tests ---

    #[test]
    fn null_is_default() {
        assert_eq!(Value::default(), Value::Null);
    }

    #[test]
    fn list_round_trip() {
        let items = vec![Value::Int(1), Value::Bool(true), Value::Null];
        let val = Value::List(items.clone());
        match val {
            Value::List(v) => assert_eq!(v, items),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn map_round_trip() {
        let mut m = BTreeMap::new();
        m.insert(String::from("key"), Value::Int(99));
        let val = Value::Map(m);
        match val {
            Value::Map(ref map) => {
                assert_eq!(map.get("key"), Some(&Value::Int(99)));
            }
            _ => panic!("expected Map"),
        }
    }

    #[test]
    fn custom_clone_no_panic() {
        let arc: Arc<dyn CustomValue> = Arc::new(MyCustom);
        let val = Value::Custom(arc);
        let cloned = val.clone();
        // Both should be Custom variants (pointer may differ due to clone_box).
        assert!(matches!(cloned, Value::Custom(_)));
    }

    // --- FromValue / IntoValue round-trips via rstest ---

    #[rstest]
    #[case(Value::Int(0), 0i32)]
    #[case(Value::Int(42), 42i32)]
    #[case(Value::Int(-1), -1i32)]
    fn int_from_value_ok(#[case] input: Value, #[case] expected: i32) {
        assert_eq!(i32::from_value(input), Ok(expected));
    }

    #[test]
    fn int_try_into_i32_ok() {
        assert_eq!(i32::from_value(Value::Int(42)), Ok(42i32));
    }

    #[test]
    fn string_try_into_i32_err() {
        let result = i32::from_value(Value::String(String::from("x")));
        assert_eq!(
            result,
            Err(TypeError {
                expected: "Int",
                got: "String",
            })
        );
    }

    #[rstest]
    #[case(true)]
    #[case(false)]
    fn bool_round_trip(#[case] b: bool) {
        let val = b.into_value();
        assert_eq!(bool::from_value(val), Ok(b));
    }

    #[rstest]
    #[case(String::from("hello"))]
    #[case(String::from(""))]
    fn string_round_trip(#[case] s: String) {
        let val = s.clone().into_value();
        assert_eq!(String::from_value(val), Ok(s));
    }

    #[rstest]
    #[case(0.0f64)]
    // Non-trivial neutral value — avoids `clippy::approx_constant` matches for PI/E.
    #[case(1.5f64)]
    fn f64_round_trip(#[case] f: f64) {
        let val = f.into_value();
        assert_eq!(f64::from_value(val), Ok(f));
    }

    // --- Checked integer conversion boundary tests ---

    #[test]
    fn u32_rejects_negative() {
        assert!(u32::from_value(Value::Int(-1)).is_err());
    }

    #[test]
    fn u32_rejects_out_of_range() {
        assert!(u32::from_value(Value::Int(i64::from(u32::MAX) + 1)).is_err());
    }

    #[test]
    fn u32_accepts_max() {
        assert_eq!(
            u32::from_value(Value::Int(i64::from(u32::MAX))),
            Ok(u32::MAX)
        );
    }

    #[test]
    fn u64_rejects_negative() {
        assert!(u64::from_value(Value::Int(-1)).is_err());
    }

    #[test]
    fn u64_accepts_zero() {
        assert_eq!(u64::from_value(Value::Int(0)), Ok(0u64));
    }

    #[test]
    fn u64_max_saturates_to_i64_max() {
        assert_eq!(u64::MAX.into_value(), Value::Int(i64::MAX));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn usize_max_saturates_to_i64_max() {
        assert_eq!(usize::MAX.into_value(), Value::Int(i64::MAX));
    }

    #[test]
    fn i32_rejects_out_of_range() {
        assert!(i32::from_value(Value::Int(i64::MAX)).is_err());
    }

    #[test]
    fn duration_round_trip() {
        let d = core::time::Duration::from_millis(500);
        assert_eq!(core::time::Duration::from_value(d.into_value()), Ok(d));
    }

    #[test]
    fn duration_type_name() {
        assert_eq!(
            Value::Duration(core::time::Duration::ZERO).type_name(),
            "Duration"
        );
    }

    #[test]
    fn duration_rejects_int() {
        assert!(core::time::Duration::from_value(Value::Int(0)).is_err());
    }

    #[test]
    fn usize_round_trip() {
        assert_eq!(usize::from_value(42usize.into_value()), Ok(42usize));
    }

    #[test]
    fn usize_rejects_negative() {
        assert!(usize::from_value(Value::Int(-1)).is_err());
    }

    // --- Option<String> ---

    #[test]
    fn option_string_none_into_value_is_null() {
        assert_eq!(None::<String>.into_value(), Value::Null);
    }

    #[test]
    fn option_string_some_into_value_is_string() {
        assert_eq!(
            Some("hello".to_owned()).into_value(),
            Value::String("hello".into())
        );
    }

    #[test]
    fn option_string_from_null_is_none() {
        assert_eq!(Option::<String>::from_value(Value::Null), Ok(None));
    }

    #[test]
    fn option_string_from_string_is_some() {
        assert_eq!(
            Option::<String>::from_value(Value::String("world".into())),
            Ok(Some("world".to_owned()))
        );
    }

    #[test]
    fn option_string_from_int_is_err() {
        let err = Option::<String>::from_value(Value::Int(1)).unwrap_err();
        assert_eq!(err.expected, "String or Null");
        assert_eq!(err.got, "Int");
    }

    #[rstest]
    #[case(None)]
    #[case(Some("round-trip".to_owned()))]
    fn option_string_round_trip(#[case] v: Option<String>) {
        assert_eq!(Option::<String>::from_value(v.clone().into_value()), Ok(v));
    }

    #[test]
    fn i32_accepts_min_max() {
        assert_eq!(
            i32::from_value(Value::Int(i64::from(i32::MIN))),
            Ok(i32::MIN)
        );
        assert_eq!(
            i32::from_value(Value::Int(i64::from(i32::MAX))),
            Ok(i32::MAX)
        );
    }

    #[test]
    fn i64_round_trip() {
        assert_eq!(i64::from_value(Value::Int(i64::MAX)), Ok(i64::MAX));
        assert_eq!(i64::from_value(Value::Int(i64::MIN)), Ok(i64::MIN));
    }

    // --- FromValue error arms for each implementor ---

    #[test]
    fn i64_rejects_wrong_type() {
        let err = i64::from_value(Value::Bool(true)).unwrap_err();
        assert_eq!(err.expected, "Int");
        assert_eq!(err.got, "Bool");
    }

    #[test]
    fn f64_rejects_wrong_type() {
        let err = f64::from_value(Value::Int(1)).unwrap_err();
        assert_eq!(err.expected, "Float");
        assert_eq!(err.got, "Int");
    }

    #[test]
    fn f32_rejects_wrong_type() {
        let err = f32::from_value(Value::Int(1)).unwrap_err();
        assert_eq!(err.expected, "Float");
        assert_eq!(err.got, "Int");
    }

    #[test]
    fn bool_rejects_wrong_type() {
        let err = bool::from_value(Value::Int(0)).unwrap_err();
        assert_eq!(err.expected, "Bool");
        assert_eq!(err.got, "Int");
    }

    #[test]
    fn string_rejects_wrong_type() {
        let err = String::from_value(Value::Null).unwrap_err();
        assert_eq!(err.expected, "String");
        assert_eq!(err.got, "Null");
    }

    // --- Clone for Box<dyn CustomValue> ---

    #[test]
    fn box_custom_value_clone() {
        let b: Box<dyn CustomValue> = Box::new(MyCustom);
        let cloned = b.clone();
        assert!(cloned.as_any().downcast_ref::<MyCustom>().is_some());
    }

    // --- Value::type_name for all variants ---

    #[test]
    fn type_name_all_variants() {
        assert_eq!(Value::Null.type_name(), "Null");
        assert_eq!(Value::Bool(false).type_name(), "Bool");
        assert_eq!(Value::Int(0).type_name(), "Int");
        assert_eq!(Value::Float(0.0).type_name(), "Float");
        assert_eq!(Value::String(String::new()).type_name(), "String");
        assert_eq!(Value::List(vec![]).type_name(), "List");
        assert_eq!(Value::Map(Default::default()).type_name(), "Map");
        assert_eq!(Value::Bytes(vec![]).type_name(), "Bytes");
        assert_eq!(
            Value::Custom(Arc::new(MyCustom)).type_name(),
            "Custom"
        );
        assert_eq!(Value::Object(WeakObjectRef(0)).type_name(), "Object");
        assert_eq!(
            Value::Duration(core::time::Duration::ZERO).type_name(),
            "Duration"
        );
    }

    // --- Value PartialEq: mismatched-variant arm ---

    #[test]
    fn value_ne_different_variants() {
        assert_ne!(Value::Int(1), Value::Bool(true));
        assert_ne!(Value::String("x".into()), Value::Int(0));
        assert_ne!(Value::Null, Value::Int(0));
    }

    // --- Serde round-trip tests ---

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;
        use std::collections::BTreeMap;

        fn rt_json(v: &Value) -> Value {
            let s = serde_json::to_string(v).expect("serialize failed");
            serde_json::from_str(&s).expect("deserialize failed")
        }

        fn rt_bincode(v: &Value) -> Value {
            let bytes =
                bincode::serde::encode_to_vec(v, bincode::config::standard()).expect("encode");
            let (val, _) =
                bincode::serde::decode_from_slice::<Value, _>(&bytes, bincode::config::standard())
                    .expect("decode");
            val
        }

        fn assert_eq_json(v: &Value) {
            assert_eq!(rt_json(v), *v);
        }

        fn assert_eq_bincode(v: &Value) {
            assert_eq!(rt_bincode(v), *v);
        }

        #[test]
        fn null_round_trips() {
            assert_eq_json(&Value::Null);
            assert_eq_bincode(&Value::Null);
        }

        #[rstest]
        #[case(Value::Bool(true))]
        #[case(Value::Bool(false))]
        #[case(Value::Int(0))]
        #[case(Value::Int(i64::MIN))]
        #[case(Value::Int(i64::MAX))]
        #[case(Value::Float(0.0))]
        #[case(Value::Float(1.5))]
        #[case(Value::Float(-3.0))]
        #[case(Value::String(String::new()))]
        #[case(Value::String("ascii".into()))]
        #[case(Value::String("üñíçødé".into()))]
        #[case(Value::Bytes(vec![]))]
        #[case(Value::Bytes(vec![0, 128, 255]))]
        #[case(Value::Object(WeakObjectRef(0)))]
        #[case(Value::Object(WeakObjectRef(1)))]
        #[case(Value::Object(WeakObjectRef(u64::MAX)))]
        #[case(Value::Duration(core::time::Duration::ZERO))]
        #[case(Value::Duration(core::time::Duration::from_secs(1)))]
        fn scalar_round_trips_json(#[case] v: Value) {
            assert_eq_json(&v);
        }

        #[rstest]
        #[case(Value::Bool(true))]
        #[case(Value::Bool(false))]
        #[case(Value::Int(0))]
        #[case(Value::Int(i64::MIN))]
        #[case(Value::Int(i64::MAX))]
        #[case(Value::Float(0.0))]
        #[case(Value::Float(1.5))]
        #[case(Value::Float(-3.0))]
        #[case(Value::String(String::new()))]
        #[case(Value::String("ascii".into()))]
        #[case(Value::Bytes(vec![]))]
        #[case(Value::Bytes(vec![0, 128, 255]))]
        #[case(Value::Object(WeakObjectRef(0)))]
        #[case(Value::Object(WeakObjectRef(u64::MAX)))]
        #[case(Value::Duration(core::time::Duration::ZERO))]
        #[case(Value::Duration(core::time::Duration::from_secs(1)))]
        fn scalar_round_trips_bincode(#[case] v: Value) {
            assert_eq_bincode(&v);
        }

        #[test]
        fn list_round_trips() {
            let v = Value::List(vec![Value::Int(1), Value::Bool(true), Value::Null]);
            assert_eq_json(&v);
            assert_eq_bincode(&v);
        }

        #[test]
        fn map_round_trips() {
            let mut m = BTreeMap::new();
            m.insert("k1".into(), Value::Int(42));
            m.insert("k2".into(), Value::Bool(false));
            let v = Value::Map(m);
            assert_eq_json(&v);
            assert_eq_bincode(&v);
        }

        #[test]
        fn float_nan_round_trips_bincode() {
            let nan = Value::Float(f64::NAN);
            let out = rt_bincode(&nan);
            assert!(matches!(out, Value::Float(f) if f.is_nan()));
        }

        #[test]
        fn malformed_bincode_returns_err() {
            let res = bincode::serde::decode_from_slice::<Value, _>(
                &[0xff; 16],
                bincode::config::standard(),
            );
            assert!(res.is_err());
        }

        // --- Custom (typetag) round-trip ---

        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        struct MyTypedCustom {
            v: i64,
        }

        #[typetag::serde]
        impl CustomValue for MyTypedCustom {
            fn type_name(&self) -> &'static str {
                "MyTypedCustom"
            }

            fn clone_box(&self) -> Box<dyn CustomValue> {
                Box::new(self.clone())
            }

            fn as_any(&self) -> &dyn core::any::Any {
                self
            }
        }

        #[test]
        fn custom_round_trips_json() {
            let v = Value::Custom(Arc::new(MyTypedCustom { v: 42 }));
            let out = rt_json(&v);
            match out {
                Value::Custom(arc) => {
                    let c = arc.as_any().downcast_ref::<MyTypedCustom>().unwrap();
                    assert_eq!(c.v, 42);
                }
                other => panic!("expected Custom, got {other:?}"),
            }
        }

        #[test]
        fn custom_round_trips_bincode() {
            let v = Value::Custom(Arc::new(MyTypedCustom { v: -7 }));
            let out = rt_bincode(&v);
            match out {
                Value::Custom(arc) => {
                    let c = arc.as_any().downcast_ref::<MyTypedCustom>().unwrap();
                    assert_eq!(c.v, -7);
                }
                other => panic!("expected Custom, got {other:?}"),
            }
        }
    }
}
