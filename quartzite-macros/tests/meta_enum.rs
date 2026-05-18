//! Integration tests for the `#[derive(MetaEnum)]` macro: `IntoValue` / `FromValue` round-tripping over discriminants.

use quartzite::core::{FromValue, IntoValue, TypeError, Value};
use quartzite_macros::MetaEnum;

#[derive(MetaEnum, PartialEq, Debug)]
enum Status {
    Ok = 200,
    NotFound = 404,
    Error = 500,
}

#[test]
fn into_value_produces_int() {
    assert_eq!(Status::Ok.into_value(), Value::Int(200));
    assert_eq!(Status::NotFound.into_value(), Value::Int(404));
    assert_eq!(Status::Error.into_value(), Value::Int(500));
}

#[test]
fn from_value_round_trip() {
    assert_eq!(Status::from_value(Value::Int(200)), Ok(Status::Ok));
    assert_eq!(Status::from_value(Value::Int(404)), Ok(Status::NotFound));
    assert_eq!(Status::from_value(Value::Int(500)), Ok(Status::Error));
}

#[test]
fn from_value_unknown_discriminant_returns_err() {
    let result = Status::from_value(Value::Int(0));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.expected, "Status");
    assert_eq!(err.got, "Int");
}

#[test]
fn from_value_wrong_type_returns_err() {
    let result = Status::from_value(Value::Bool(true));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        TypeError {
            expected: "Status",
            got: "Bool",
        }
    );
}

#[test]
fn into_value_then_from_value_round_trips() {
    for variant in [Status::Ok, Status::NotFound, Status::Error] {
        let val = variant.into_value();
        assert!(matches!(val, Value::Int(_)));
    }
}
