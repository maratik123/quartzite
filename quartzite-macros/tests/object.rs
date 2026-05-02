use std::sync::{Arc, Mutex};

use quartzite::core::{AsObject, FromValue, Object, ObjectBase, Signal, Value};
use quartzite_macros::{Extend, Object, object_impl};

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

#[object_impl]
impl Counter {}

// AC6: read_property returns correct Value; write_property updates field and emits notify.
#[test]
fn ac6_read_write_property_with_notify() {
    let mut c = Counter {
        object_base: ObjectBase::new(),
        count: 0,
        count_changed: Signal::default(),
        version: 1,
    };

    let val = c.read_property("count");
    assert_eq!(val, Some(Value::Int(0)));

    let notified = Arc::new(Mutex::new(false));
    let notified_clone = Arc::clone(&notified);
    c.count_changed.connect(move |_args: &(i32,)| {
        *notified_clone.lock().unwrap() = true;
    });

    let written = c.write_property("count", Value::Int(42));
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
    let written = c.write_property("version", Value::Int(99));
    assert!(!written);
    assert_eq!(c.version, 1);
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

    let id = c.connect_signal("count_changed", cb);
    assert!(id.is_some());

    c.count_changed.emit(&(7,));
    assert_eq!(*received.lock().unwrap(), Some(7));
}

fn make_counter() -> Counter {
    Counter {
        object_base: ObjectBase::new(),
        count: 0,
        count_changed: Signal::default(),
        version: 1,
    }
}

// AC3: emit_<signal> wrapper suppresses slot calls when signals are blocked.
#[test]
fn emit_wrapper_suppressed_when_blocked() {
    let mut c = make_counter();
    let called = Arc::new(Mutex::new(false));
    let called_clone = Arc::clone(&called);
    c.count_changed
        .connect(move |_: &(i32,)| *called_clone.lock().unwrap() = true);

    c.object_base_mut().block_signals();
    c.emit_count_changed(42);

    assert!(!*called.lock().unwrap(), "slot must not fire when blocked");
}

// AC4: emit_<signal> wrapper delivers normally when not blocked.
#[test]
fn emit_wrapper_delivers_when_unblocked() {
    let mut c = make_counter();
    let received = Arc::new(Mutex::new(None::<i32>));
    let received_clone = Arc::clone(&received);
    c.count_changed
        .connect(move |args: &(i32,)| *received_clone.lock().unwrap() = Some(args.0));

    c.emit_count_changed(7);

    assert_eq!(*received.lock().unwrap(), Some(7));
}

// AC5: write_property does not emit notify when signals are blocked, but still writes value.
#[test]
fn write_property_notify_suppressed_when_blocked() {
    let mut c = make_counter();
    let notified = Arc::new(Mutex::new(false));
    let notified_clone = Arc::clone(&notified);
    c.count_changed
        .connect(move |_: &(i32,)| *notified_clone.lock().unwrap() = true);

    c.object_base_mut().block_signals();
    let written = c.write_property("count", Value::Int(99));

    assert!(written, "write_property must return true");
    assert_eq!(c.count, 99, "value must be updated even when blocked");
    assert!(
        !*notified.lock().unwrap(),
        "notify must not fire when blocked"
    );
}

// Regression: write_property notify fires normally when not blocked.
#[test]
fn write_property_notify_fires_when_unblocked() {
    let mut c = make_counter();
    let notified = Arc::new(Mutex::new(false));
    let notified_clone = Arc::clone(&notified);
    c.count_changed
        .connect(move |_: &(i32,)| *notified_clone.lock().unwrap() = true);

    c.write_property("count", Value::Int(5));

    assert!(*notified.lock().unwrap(), "notify must fire when unblocked");
}

// AC1+AC3: unblock_signals restores delivery after block.
#[test]
fn unblock_restores_emit_wrapper() {
    let mut c = make_counter();
    let called = Arc::new(Mutex::new(false));
    let called_clone = Arc::clone(&called);
    c.count_changed
        .connect(move |_: &(i32,)| *called_clone.lock().unwrap() = true);

    c.object_base_mut().block_signals();
    c.emit_count_changed(1);
    assert!(!*called.lock().unwrap(), "must be suppressed while blocked");

    c.object_base_mut().unblock_signals();
    c.emit_count_changed(2);
    assert!(*called.lock().unwrap(), "must fire after unblock");
}
