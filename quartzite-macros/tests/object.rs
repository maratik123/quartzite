//! Integration tests for the `#[derive(Object)]` macro: property read/write, notify signals, and read-only flags.
// Test structs intentionally lack `///` docs; suppress the undocumented-item diagnostic.
#![allow(deprecated)]

use std::sync::Arc;

use quartzite::core::Mutex;

use quartzite::core::{
    AsObject, FromValue, Object, ObjectBase, PropertyFlag, Signal, Value, signal::ConnectionType,
};
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
        *notified_clone.lock() = true;
    });

    let written = c.write_property("count", Value::Int(42));
    assert!(written);
    assert_eq!(c.count, 42);
    assert!(*notified.lock());
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
            *received_clone.lock() = i32::from_value(v.clone()).ok();
        }
    });

    let id = c.connect_signal("count_changed", cb, ConnectionType::Direct);
    assert!(id.is_some());

    c.count_changed.emit_unconditionally(&(7,));
    assert_eq!(*received.lock(), Some(7));
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
        .connect(move |_: &(i32,)| *called_clone.lock() = true);

    c.object_base_mut().block_signals();
    c.emit_count_changed(42);

    assert!(!*called.lock(), "slot must not fire when blocked");
}

// AC4: emit_<signal> wrapper delivers normally when not blocked.
#[test]
fn emit_wrapper_delivers_when_unblocked() {
    let mut c = make_counter();
    let received = Arc::new(Mutex::new(None::<i32>));
    let received_clone = Arc::clone(&received);
    c.count_changed
        .connect(move |args: &(i32,)| *received_clone.lock() = Some(args.0));

    c.emit_count_changed(7);

    assert_eq!(*received.lock(), Some(7));
}

// AC5: write_property does not emit notify when signals are blocked, but still writes value.
#[test]
fn write_property_notify_suppressed_when_blocked() {
    let mut c = make_counter();
    let notified = Arc::new(Mutex::new(false));
    let notified_clone = Arc::clone(&notified);
    c.count_changed
        .connect(move |_: &(i32,)| *notified_clone.lock() = true);

    c.object_base_mut().block_signals();
    let written = c.write_property("count", Value::Int(99));

    assert!(written, "write_property must return true");
    assert_eq!(c.count, 99, "value must be updated even when blocked");
    assert!(!*notified.lock(), "notify must not fire when blocked");
}

// Regression: write_property notify fires normally when not blocked.
#[test]
fn write_property_notify_fires_when_unblocked() {
    let mut c = make_counter();
    let notified = Arc::new(Mutex::new(false));
    let notified_clone = Arc::clone(&notified);
    c.count_changed
        .connect(move |_: &(i32,)| *notified_clone.lock() = true);

    c.write_property("count", Value::Int(5));

    assert!(*notified.lock(), "notify must fire when unblocked");
}

// Property flag values: readable/writable/notify/constant from generated metadata.
#[test]
fn property_flags_in_generated_meta() {
    let c = make_counter();
    let meta = c.meta_object();

    let count_flags = meta.property("count").expect("count property").flags;
    assert!(count_flags.contains(PropertyFlag::Readable));
    assert!(count_flags.contains(PropertyFlag::Writable));
    assert!(count_flags.contains(PropertyFlag::Notify));
    assert!(!count_flags.contains(PropertyFlag::Constant));

    let version_flags = meta.property("version").expect("version property").flags;
    assert!(version_flags.contains(PropertyFlag::Readable));
    assert!(!version_flags.contains(PropertyFlag::Writable));
    assert!(!version_flags.contains(PropertyFlag::Notify));
}

// AC1+AC3: unblock_signals restores delivery after block.
#[test]
fn unblock_restores_emit_wrapper() {
    let mut c = make_counter();
    let called = Arc::new(Mutex::new(false));
    let called_clone = Arc::clone(&called);
    c.count_changed
        .connect(move |_: &(i32,)| *called_clone.lock() = true);

    c.object_base_mut().block_signals();
    c.emit_count_changed(1);
    assert!(!*called.lock(), "must be suppressed while blocked");

    c.object_base_mut().unblock_signals();
    c.emit_count_changed(2);
    assert!(*called.lock(), "must fire after unblock");
}

// SingleShot: slot is removed after first delivery via macro-generated connect_signal path.
// emit_unconditionally is used directly (not emit_count_changed) to bypass the signals_blocked
// check — we want to observe slot removal in isolation without block/unblock interactions.
#[test]
fn single_shot_fires_once_via_object_trait() {
    let mut c = make_counter();
    let call_count = Arc::new(Mutex::new(0u32));
    let call_count_clone = Arc::clone(&call_count);
    let cb = Box::new(move |_args: &[Value]| {
        *call_count_clone.lock() += 1;
    });

    let id = c.connect_signal("count_changed", cb, ConnectionType::SingleShot);
    assert!(
        id.is_some(),
        "connect_signal must return Some for a known signal"
    );

    c.count_changed.emit_unconditionally(&(1,));
    assert_eq!(*call_count.lock(), 1, "slot must fire on first emit");

    c.count_changed.emit_unconditionally(&(2,));
    assert_eq!(
        *call_count.lock(),
        1,
        "slot must not fire after SingleShot removal"
    );
}
