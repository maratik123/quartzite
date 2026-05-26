//! Integration test verifying single-dep usage of the `quartzite` facade crate: `quartzite::prelude::*` is sufficient (no direct `quartzite-core` or `quartzite-macros` import needed).

// Verifies single-dep usage: only `quartzite::prelude::*` — no direct `quartzite-core` or
// `quartzite-macros` import is needed.
use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct Counter {
    #[base]
    object_base: ObjectBase,
    #[property]
    pub count: i32,
    #[signal]
    pub count_changed: Signal<(i32,)>,
}

#[object_impl]
impl Counter {}

#[test]
fn single_dep_property_access() {
    let mut c = Counter {
        object_base: ObjectBase::new(),
        count: 0,
        count_changed: Signal::default(),
    };
    assert_eq!(c.read_property("count"), Some(Value::Int(0)));
    assert!(c.write_property("count", Value::Int(5)));
    assert_eq!(c.count, 5);
    let _: &ObjectBase = c.object_base();
}
