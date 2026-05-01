#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};

/// Trait for user-defined value types stored inside `Value::Custom`.
///
/// Implementors must support cloning (`clone_box`) and downcast (`as_any`).
/// The `fmt_debug` method is required to allow `Debug` formatting on the trait object.
pub trait CustomValue: core::any::Any {
    fn type_name(&self) -> &'static str;
    fn clone_box(&self) -> Box<dyn CustomValue>;
    fn as_any(&self) -> &dyn core::any::Any;
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;
}

impl Clone for Box<dyn CustomValue> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl core::fmt::Debug for dyn CustomValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt_debug(f)
    }
}

/// A closed, dynamically-typed value container.
///
/// The `Custom` variant allows storing arbitrary user types via `Arc<dyn CustomValue>`.
/// `Value::Null` is the default.
#[derive(Clone, Debug, Default)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
    Bytes(Vec<u8>),
    Custom(Arc<dyn CustomValue>),
    // Object(WeakObjectRef) deferred until quartzite-runtime decides ownership model
}

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
            _ => false,
        }
    }
}

impl Value {
    /// Returns a static string naming the active variant (for use in error messages).
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
        }
    }
}

/// Error returned when a `FromValue` conversion fails due to a type mismatch.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub expected: &'static str,
    pub got: &'static str,
}

impl core::fmt::Display for TypeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "type error: expected {}, got {}",
            self.expected, self.got
        )
    }
}

/// Convert a `Value` into a concrete Rust type.
pub trait FromValue: Sized {
    fn from_value(val: Value) -> Result<Self, TypeError>;
}

/// Convert a concrete Rust type into a `Value`.
pub trait IntoValue {
    fn into_value(self) -> Value;
}

macro_rules! impl_int {
    ($t:ty) => {
        impl FromValue for $t {
            fn from_value(val: Value) -> Result<Self, TypeError> {
                match val {
                    Value::Int(n) => Ok(n as $t),
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

impl_int!(i32);
impl_int!(i64);
impl_int!(u32);
impl_int!(u64);

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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // --- Minimal CustomValue for testing ---

    #[derive(Debug, Clone)]
    struct MyCustom {
        label: &'static str,
    }

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

        fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("MyCustom")
                .field("label", &self.label)
                .finish()
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
        let arc: Arc<dyn CustomValue> = Arc::new(MyCustom { label: "hello" });
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
    #[case(3.14f64)]
    fn f64_round_trip(#[case] f: f64) {
        let val = f.into_value();
        assert_eq!(f64::from_value(val), Ok(f));
    }
}
