//! AC22 — `connect_signal_to_slot` integration test.
//!
//! Verifies that [`quartzite_core::connect_signal_to_slot`] can wire a source
//! object's `"click"` signal to `Application::quit` via the meta-system slot
//! path, and that emitting the signal stops a running event loop.
//!
//! Separated from other test binaries so this binary gets a fresh `OnceLock`
//! and there are no singleton races with `tests/application.rs` or
//! `tests/application_signal_to_quit.rs`.

use std::assert_matches;
use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use parking_lot::Mutex;
use quartzite_core::{
    ConnectionId, Object, ObjectBase, SignalCallback,
    connect::{SignalConnectionError, connect_signal_to_slot},
    meta::{MetaObject, SignalMeta},
    signal::{ConnectionType, Signal},
    value::Value,
};
use quartzite_runtime::Application;

// ── Minimal source object with a zero-arg "click" signal ────────────────────

/// Hand-rolled source object carrying a single zero-argument `"click"` signal.
struct ClickSource {
    base: ObjectBase,
    click: Signal<()>,
}

impl ClickSource {
    fn new() -> Self {
        Self {
            base: ObjectBase::new(),
            click: Signal::new(),
        }
    }

    /// Emit the `"click"` signal.
    fn fire(&mut self) {
        quartzite_core::emit!(self.click, &());
    }
}

impl quartzite_core::AsObject for ClickSource {
    fn object_base(&self) -> &ObjectBase {
        &self.base
    }

    fn object_base_mut(&mut self) -> &mut ObjectBase {
        &mut self.base
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

static CLICK_SIGNALS: [SignalMeta; 1] = [SignalMeta::new("click", &[])];
static CLICK_META: MetaObject = MetaObject::new(
    "ClickSource",
    &[],
    &CLICK_SIGNALS,
    &[],
    &[],
    quartzite_core::meta::noop_lookup_property,
    |name| {
        if name == "click" {
            Some(CLICK_SIGNALS[0])
        } else {
            None
        }
    },
    quartzite_core::meta::noop_lookup_method,
    quartzite_core::meta::noop_lookup_enum,
);

impl Object for ClickSource {
    fn meta_object(&self) -> &'static MetaObject {
        &CLICK_META
    }

    fn read_property(&self, _name: &str) -> Option<Value> {
        None
    }

    fn write_property(&mut self, _name: &str, _val: Value) -> bool {
        false
    }

    fn invoke_method(&mut self, _name: &str, _args: &[Value]) -> Option<Value> {
        None
    }

    fn connect_signal(
        &mut self,
        signal: &str,
        callback: SignalCallback,
        conn_type: ConnectionType,
    ) -> Option<ConnectionId> {
        if signal == "click" {
            let cb = Arc::new(callback);
            Some(
                self.click
                    .connect_typed(move |_args: &()| (*cb)(&[]), conn_type),
            )
        } else {
            None
        }
    }

    fn emit_signal(&mut self, signal: &str, args: &[Value]) -> Option<()> {
        if signal == "click" && args.is_empty() {
            quartzite_core::emit!(self.click, &());
            Some(())
        } else {
            None
        }
    }
}

// ── Test ────────────────────────────────────────────────────────────────────

/// AC22: wire `ClickSource::click` → `Application::quit` via
/// `connect_signal_to_slot`; assert `exec()` exits within 200 ms.
#[test]
fn ac22_connect_signal_to_slot_stops_application() {
    let app = Application::new().unwrap();

    let mut source = ClickSource::new();

    // Wrap Application in Arc<Mutex<dyn Object>> so it can be held weakly
    // inside the connection callback.
    let target: Arc<Mutex<dyn Object>> = Arc::new(Mutex::new(app));

    connect_signal_to_slot(&mut source, "click", &target, "quit")
        .expect("connect_signal_to_slot must succeed for valid signal name");

    // Run the event loop on a background thread.
    let (tx, rx) = mpsc::channel::<()>();
    let handle = thread::spawn(move || {
        Application::global().unwrap().exec();
        let _ = tx.send(());
    });

    // Give exec() time to start blocking.
    thread::sleep(Duration::from_millis(5));

    // Emit "click" — must trigger quit → exec() returns.
    source.fire();

    rx.recv_timeout(Duration::from_millis(200))
        .expect("connect_signal_to_slot must stop the Application event loop within 200 ms");
    handle.join().expect("exec() thread must exit cleanly");
}

/// AC22 (error path): `connect_signal_to_slot` returns `UnknownFromSignal` for
/// a non-existent signal name.  Uses a second `ClickSource` as the target so
/// this test does not need its own `Application` singleton (which would clash
/// with the singleton already created in `ac22_connect_signal_to_slot_stops_application`
/// within this same binary).
#[test]
fn ac22_unknown_signal_returns_error() {
    let mut source = ClickSource::new();
    let target: Arc<Mutex<dyn Object>> = Arc::new(Mutex::new(ClickSource::new()));

    let result = connect_signal_to_slot(&mut source, "nonexistent", &target, "quit");
    assert_matches!(result, Err(SignalConnectionError::UnknownFromSignal(_)));
}
