//! Dynamic value type and type-conversion traits.
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};

/// Trait for user-defined value types stored inside `Value::Custom`.
///
/// Implementors must support cloning (`clone_box`), downcast (`as_any`), and
/// `Debug` formatting (required as a supertrait so that `Box<dyn CustomValue>`
/// and `Arc<dyn CustomValue>` are automatically `Debug`).
///
/// # Examples
///
/// ```
/// use quartzite_core::value::CustomValue;
///
/// #[derive(Debug, Clone)]
/// struct MyVal(i32);
///
/// impl CustomValue for MyVal {
///     fn type_name(&self) -> &'static str { "MyVal" }
///     fn clone_box(&self) -> Box<dyn CustomValue> { Box::new(self.clone()) }
///     fn as_any(&self) -> &dyn core::any::Any { self }
/// }
///
/// let v: Box<dyn CustomValue> = Box::new(MyVal(42));
/// assert_eq!(v.type_name(), "MyVal");
/// ```
pub trait CustomValue: core::any::Any + core::fmt::Debug + Send + Sync {
    /// Returns a static string identifying the concrete type (e.g. `"MyVal"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::value::CustomValue;
    ///
    /// #[derive(Debug, Clone)]
    /// struct MyVal;
    ///
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
    /// #[derive(Debug, Clone)]
    /// struct MyVal;
    ///
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
    /// #[derive(Debug, Clone)]
    /// struct MyVal(i32);
    ///
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
macro_rules! impl_int_checked {
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
        impl IntoValue for $t {
            fn into_value(self) -> Value {
                Value::Int(self as i64)
            }
        }
    };
}

impl_int_checked!(i32);
impl_int_checked!(u32);
impl_int_checked!(u64);
impl_int_checked!(usize);

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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // --- Minimal CustomValue for testing ---

    #[derive(Debug, Clone)]
    struct MyCustom;

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
}
