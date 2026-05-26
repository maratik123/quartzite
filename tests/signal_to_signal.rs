// Test structs intentionally lack `///` docs; suppress the undocumented-item diagnostic.
#![allow(deprecated)]
//! Integration tests for signal-to-signal connections through the `quartzite` facade crate (AC1–AC11).

// Integration tests for signal-to-signal connections (AC1–AC11).
use std::sync::{
    Arc,
    atomic::{AtomicI32, AtomicU32, Ordering},
};

use quartzite::prelude::*;

// ----- Shared test objects -----

#[derive(Extend, Object)]
#[root]
struct Emitter {
    #[base]
    object_base: ObjectBase,
    #[signal]
    pub value_sent: Signal<(i32,)>,
    #[signal]
    pub clicked: Signal<()>,
}

#[object_impl]
impl Emitter {}

#[derive(Extend, Object)]
#[root]
struct Relay {
    #[base]
    object_base: ObjectBase,
    #[signal]
    pub value_received: Signal<(i32,)>,
    #[signal]
    pub clicked: Signal<()>,
}

#[object_impl]
impl Relay {}

fn new_emitter() -> Emitter {
    Emitter {
        object_base: ObjectBase::new(),
        value_sent: Signal::default(),
        clicked: Signal::default(),
    }
}

fn new_relay() -> Relay {
    Relay {
        object_base: ObjectBase::new(),
        value_received: Signal::default(),
        clicked: Signal::default(),
    }
}

// ----- AC1: emit_signal on a known signal with correct arity returns Some(()). -----

#[test]
fn emit_signal_known_signal_returns_some() {
    let mut e = new_emitter();
    assert_eq!(e.emit_signal("value_sent", &[Value::Int(1)]), Some(()));
    assert_eq!(e.emit_signal("clicked", &[]), Some(()));
}

// ----- AC2: emit_signal for unknown signal or wrong arity returns None. -----

#[test]
fn emit_signal_unknown_signal_returns_none() {
    let mut e = new_emitter();
    assert_eq!(e.emit_signal("no_such_signal", &[]), None);
}

#[test]
fn emit_signal_wrong_arity_returns_none() {
    let mut e = new_emitter();
    // value_sent expects 1 arg; pass 0.
    assert_eq!(e.emit_signal("value_sent", &[]), None);
}

// ----- AC3: unknown from-signal returns UnknownFromSignal. -----

#[test]
fn connect_signal_to_signal_unknown_from_signal_returns_error() {
    let mut emitter = new_emitter();
    let relay: Arc<Mutex<dyn Object>> = Arc::new(Mutex::new(new_relay()));
    let err = connect_signal_to_signal(
        &mut emitter,
        "unknown",
        &relay,
        "value_received",
        ConnectionType::Direct,
    )
    .unwrap_err();
    assert!(matches!(err, SignalConnectionError::UnknownFromSignal(_)));
}

// ----- AC4: arity mismatch returns ArityMismatch (from < to). -----

#[test]
fn connect_signal_to_signal_arity_mismatch_returns_error() {
    let mut emitter = new_emitter();
    let relay: Arc<Mutex<dyn Object>> = Arc::new(Mutex::new(new_relay()));
    // clicked has 0 params (from); value_received has 1 (to) → from < to triggers ArityMismatch.
    let err = connect_signal_to_signal(
        &mut emitter,
        "clicked",
        &relay,
        "value_received",
        ConnectionType::Direct,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        SignalConnectionError::ArityMismatch { from: 0, to: 1 }
    ));
}

// ----- AC4b: from >= to (1-arg → 0-arg): connection succeeds and forwarding fires the target. -----

#[test]
fn connect_signal_to_signal_truncates_extras() {
    let mut emitter = new_emitter();
    let relay = Arc::new(Mutex::new(new_relay()));
    // Counter incremented each time relay.clicked fires.
    let counter = Arc::new(AtomicU32::new(0));
    {
        let c = Arc::clone(&counter);
        relay.lock().clicked.connect(move |_args: &()| {
            c.fetch_add(1, Ordering::Relaxed);
        });
    }
    // Wire value_sent (1-arg i32) → clicked (0-arg): excess arg is dropped at emit time.
    let to: Arc<Mutex<dyn Object>> = Arc::clone(&relay) as Arc<Mutex<dyn Object>>;
    connect_signal_to_signal(
        &mut emitter,
        "value_sent",
        &to,
        "clicked",
        ConnectionType::Direct,
    )
    .expect("1-arg → 0-arg must succeed under from >= to rule");

    emitter.value_sent.emit_unconditionally(&(42,));
    assert_eq!(
        counter.load(Ordering::Relaxed),
        1,
        "clicked must have fired exactly once after truncated forwarding"
    );
}

// ----- AC5: Direct connection forwards signal synchronously. -----

#[test]
fn direct_connection_forwards_synchronously() {
    let mut emitter = new_emitter();
    let relay = Arc::new(Mutex::new(new_relay()));
    let captured = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&captured);
        relay
            .lock()
            .value_received
            .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
    }
    let to: Arc<Mutex<dyn Object>> = Arc::clone(&relay) as Arc<Mutex<dyn Object>>;
    connect_signal_to_signal(
        &mut emitter,
        "value_sent",
        &to,
        "value_received",
        ConnectionType::Direct,
    )
    .unwrap();

    emitter.value_sent.emit_unconditionally(&(77,));
    assert_eq!(
        captured.load(Ordering::Relaxed),
        77,
        "AC5: value must be forwarded"
    );
}

// ----- AC7: Liveness — target released → forwarding silently skipped. -----

#[test]
fn liveness_after_target_drop() {
    let mut emitter = new_emitter();
    let relay = Arc::new(Mutex::new(new_relay()));
    let captured = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&captured);
        relay
            .lock()
            .value_received
            .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
    }
    let to: Arc<Mutex<dyn Object>> = Arc::clone(&relay) as Arc<Mutex<dyn Object>>;
    connect_signal_to_signal(
        &mut emitter,
        "value_sent",
        &to,
        "value_received",
        ConnectionType::Direct,
    )
    .unwrap();

    emitter.value_sent.emit_unconditionally(&(1,));
    assert_eq!(captured.load(Ordering::Relaxed), 1);

    drop(to);
    drop(relay);
    emitter.value_sent.emit_unconditionally(&(2,));
    assert_eq!(
        captured.load(Ordering::Relaxed),
        1,
        "AC7: no forwarding after target drop"
    );
}

// ----- AC8: disconnect stops forwarding. -----

#[test]
fn disconnect_stops_forwarding() {
    let mut emitter = new_emitter();
    let relay = Arc::new(Mutex::new(new_relay()));
    let captured = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&captured);
        relay
            .lock()
            .value_received
            .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
    }
    let to: Arc<Mutex<dyn Object>> = Arc::clone(&relay) as Arc<Mutex<dyn Object>>;
    let id = connect_signal_to_signal(
        &mut emitter,
        "value_sent",
        &to,
        "value_received",
        ConnectionType::Direct,
    )
    .unwrap();

    emitter.value_sent.emit_unconditionally(&(5,));
    assert_eq!(captured.load(Ordering::Relaxed), 5);

    emitter.value_sent.disconnect(id);
    emitter.value_sent.emit_unconditionally(&(6,));
    assert_eq!(
        captured.load(Ordering::Relaxed),
        5,
        "AC8: disconnect must stop forwarding"
    );
}

// ----- AC9: Chains — Emitter → Relay A → Relay B works naturally. -----

#[test]
fn chain_emitter_to_relay_a_to_relay_b() {
    let mut emitter = new_emitter();
    let relay_a = Arc::new(Mutex::new(new_relay()));
    let relay_b = Arc::new(Mutex::new(new_relay()));
    let captured = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&captured);
        relay_b
            .lock()
            .value_received
            .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
    }

    // Emitter → Relay A
    {
        let a: Arc<Mutex<dyn Object>> = Arc::clone(&relay_a) as Arc<Mutex<dyn Object>>;
        connect_signal_to_signal(
            &mut emitter,
            "value_sent",
            &a,
            "value_received",
            ConnectionType::Direct,
        )
        .unwrap();
    }

    // Relay A → Relay B (lock relay_a, connect while held, then release)
    {
        let mut guard = relay_a.lock();
        let b: Arc<Mutex<dyn Object>> = Arc::clone(&relay_b) as Arc<Mutex<dyn Object>>;
        connect_signal_to_signal(
            &mut *guard,
            "value_received",
            &b,
            "value_received",
            ConnectionType::Direct,
        )
        .unwrap();
    }

    emitter.value_sent.emit_unconditionally(&(99,));
    assert_eq!(
        captured.load(Ordering::Relaxed),
        99,
        "AC9: chain Emitter→A→B must forward"
    );
}

// ----- AC10: connect_signals typed API works. -----

#[test]
fn connect_signals_typed_api_direct() {
    let mut emitter = new_emitter();
    let relay = Arc::new(Mutex::new(new_relay()));
    let captured = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&captured);
        relay
            .lock()
            .value_received
            .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
    }
    connect_signals(
        &mut emitter,
        "value_sent",
        |obj: &mut Emitter| &mut obj.value_sent,
        &relay,
        "value_received",
        ConnectionType::Direct,
    )
    .unwrap();

    emitter.value_sent.emit_unconditionally(&(33,));
    assert_eq!(
        captured.load(Ordering::Relaxed),
        33,
        "AC10: typed API must forward"
    );
}

// ----- AC6: Auto cross-thread — callback is posted to the queued dispatcher. -----

struct TestDispatcher {
    posted: Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>,
}

impl QueuedDispatcher for TestDispatcher {
    fn post(&self, _target: std::thread::ThreadId, f: Box<dyn FnOnce() + Send + 'static>) {
        self.posted.lock().push(f);
    }
}

// Process-wide singleton so multiple tests share the same dispatcher.
fn install_dispatcher() -> Arc<TestDispatcher> {
    use std::sync::OnceLock;
    static D: OnceLock<Arc<TestDispatcher>> = OnceLock::new();
    Arc::clone(D.get_or_init(|| {
        let d = Arc::new(TestDispatcher {
            posted: Mutex::new(vec![]),
        });
        let _ = set_queued_dispatcher(Arc::clone(&d) as Arc<dyn QueuedDispatcher>);
        d
    }))
}

#[test]
fn auto_cross_thread_posts_to_dispatcher() {
    let _lock = quartzite_test_helpers::test_lock();
    let dispatcher = install_dispatcher();
    dispatcher.posted.lock().clear();

    let mut emitter = new_emitter();
    // Build a relay whose ObjectBase was created on another thread, so its thread_id
    // differs from the current test thread — triggering the Queued path in Auto dispatch.
    let (tx, rx) = std::sync::mpsc::channel::<Relay>();
    std::thread::spawn(move || tx.send(new_relay()).unwrap())
        .join()
        .unwrap();
    let relay = Arc::new(Mutex::new(rx.recv().unwrap()));
    let captured = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&captured);
        relay
            .lock()
            .value_received
            .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
    }

    let to: Arc<Mutex<dyn Object>> = Arc::clone(&relay) as Arc<Mutex<dyn Object>>;
    connect_signal_to_signal(
        &mut emitter,
        "value_sent",
        &to,
        "value_received",
        ConnectionType::Auto,
    )
    .unwrap();

    emitter.value_sent.emit_unconditionally(&(88,));
    // Must NOT have fired synchronously.
    assert_eq!(
        captured.load(Ordering::Relaxed),
        0,
        "AC6: Auto cross-thread must not fire synchronously"
    );
    // Drain and run posted tasks.
    let tasks: Vec<_> = dispatcher.posted.lock().drain(..).collect();
    assert!(
        !tasks.is_empty(),
        "AC6: at least one task must have been queued"
    );
    for task in tasks {
        task();
    }
    assert_eq!(
        captured.load(Ordering::Relaxed),
        88,
        "AC6: Auto cross-thread must forward after dispatcher drain"
    );
}

// ----- AC11: all connection API symbols accessible from quartzite::prelude::*. -----
// Covered implicitly: every test above imports only `quartzite::prelude::*` and uses
// `connect_signal_to_signal`, `connect_signals`, `SignalConnectionError`, `ConnectionType`,
// `ConnectionId`, `Object` — all resolved from the prelude without explicit path qualifiers.
