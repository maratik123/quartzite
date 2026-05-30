//! Integration test verifying macros accessed via the `quartzite::macros` facade emit `::quartzite::core` paths (no direct `quartzite_macros` import needed).
// Test fixtures opt out of the undocumented-item diagnostic via per-block
// `undocumented = "allow"` attrs (doc prose on internal fixtures is noise).
#![allow(deprecated)]

// Verifies that macros accessed via `quartzite::macros` emit `::quartzite::core` paths —
// no direct `quartzite_macros` import is needed.
use quartzite::core::{AsObject, Object, ObjectBase, Signal, Value};
use quartzite::macros::{Extend, Object, object_impl};

#[derive(Extend, Object)]
#[root]
#[extend(undocumented = "allow")]
#[object(undocumented = "allow")]
struct Sensor {
    #[base]
    object_base: ObjectBase,
    #[prop]
    pub reading: i32,
    #[signal]
    pub reading_changed: Signal<(i32,)>,
}

#[object_impl]
impl Sensor {}

#[test]
fn via_facade_property_read_write() {
    let mut s = Sensor {
        object_base: ObjectBase::new(),
        reading: 0,
        reading_changed: Signal::default(),
    };
    assert_eq!(s.read_property("reading"), Some(Value::Int(0)));
    assert!(s.write_property("reading", Value::Int(7)));
    assert_eq!(s.reading, 7);
    let _: &ObjectBase = s.object_base();
}
