use std::sync::{Arc, Mutex};

use quartzite_core::{FromValue, ObjectBase, Signal, Value};
use quartzite_macros::{Extend, Object};

#[derive(Extend, Object)]
#[root]
struct Counter {
    #[base]
    object_base: ObjectBase,
    #[prop(notify = count_changed)]
    pub count: i32,
    #[signal]
    pub count_changed: Signal<(i32,)>,
    #[prop(read_only)]
    pub version: i32,
}

// AC6: read_property returns correct Value; write_property updates field and emits notify.
#[test]
fn ac6_read_write_property_with_notify() {
    let mut c = Counter {
        object_base: ObjectBase::new(),
        count: 0,
        count_changed: Signal::default(),
        version: 1,
    };

    // read_property
    let val = __quartzite_Counter::__read_property_Counter(&c, "count");
    assert_eq!(val, Some(Value::Int(0)));

    // write_property triggers notify
    let notified = Arc::new(Mutex::new(false));
    let notified_clone = Arc::clone(&notified);
    c.count_changed.connect(move |_args: &(i32,)| {
        *notified_clone.lock().unwrap() = true;
    });

    let written = __quartzite_Counter::__write_property_Counter(&mut c, "count", Value::Int(42));
    assert!(written);
    assert_eq!(c.count, 42);
    assert!(*notified.lock().unwrap());
}

// AC7: read_only property returns false from write_property.
#[test]
fn ac7_read_only_write_returns_false() {
    let mut c = Counter {
        object_base: ObjectBase::new(),
        count: 0,
        count_changed: Signal::default(),
        version: 1,
    };
    let written = __quartzite_Counter::__write_property_Counter(&mut c, "version", Value::Int(99));
    assert!(!written);
    assert_eq!(c.version, 1); // unchanged
}

// AC10: connect_signal registers a callback that fires on emit.
#[test]
fn ac10_connect_signal_and_emit() {
    let mut c = Counter {
        object_base: ObjectBase::new(),
        count: 0,
        count_changed: Signal::default(),
        version: 1,
    };

    let received = Arc::new(Mutex::new(None::<i32>));
    let received_clone = Arc::clone(&received);
    let cb = Box::new(move |args: &[Value]| {
        if let Some(v) = args.first() {
            *received_clone.lock().unwrap() = i32::from_value(v.clone()).ok();
        }
    });

    let id = __quartzite_Counter::__connect_signal_dynamic_Counter(&mut c, "count_changed", cb);
    assert!(id.is_some());

    c.count_changed.emit(&(7,));
    assert_eq!(*received.lock().unwrap(), Some(7));
}
