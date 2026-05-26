//! Integration tests for `quartzite_core::connect` — all paths except
//! `auto_cross_thread_posts_to_dispatcher` (which needs a `#[cfg(test)]`-private
//! dispatcher helper from `signal::tests` and must live in the inline unit-test module).

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use parking_lot::Mutex;

use quartzite_core::{
    AsObject, FromValue, IntoValue, Object,
    connect::{
        SignalConnectionError, connect_signal_to_signal, connect_signal_to_slot, connect_signals,
    },
    id::ConnectionId,
    meta::{
        MetaObject, MethodMeta, ParamMeta, SignalMeta, noop_lookup_enum, noop_lookup_method,
        noop_lookup_property, noop_lookup_signal,
    },
    object_base::ObjectBase,
    signal::{ConnectionType, Signal},
    traits::SignalCallback,
    value::Value,
};

// ── Sender (1-arg i32 signal) ────────────────────────────────────────────────

struct Sender {
    base: ObjectBase,
    pub sig_a: Signal<(i32,)>,
}

impl Sender {
    fn new() -> Self {
        Self {
            base: ObjectBase::new(),
            sig_a: Signal::new(),
        }
    }
}

impl AsObject for Sender {
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

static SENDER_PARAMS: [ParamMeta; 1] = [ParamMeta::new("arg0", "i32")];
static SENDER_SIGNALS: [SignalMeta; 1] = [SignalMeta::new("sig_a", &SENDER_PARAMS)];
static SENDER_META: MetaObject = MetaObject::new(
    "Sender",
    &[],
    &SENDER_SIGNALS,
    &[],
    &[],
    noop_lookup_property,
    |name| {
        if name == "sig_a" {
            Some(SENDER_SIGNALS[0])
        } else {
            None
        }
    },
    noop_lookup_method,
    noop_lookup_enum,
);

impl Object for Sender {
    fn meta_object(&self) -> &'static MetaObject {
        &SENDER_META
    }
    fn read_property(&self, _: &str) -> Option<Value> {
        None
    }
    fn write_property(&mut self, _: &str, _: Value) -> bool {
        false
    }
    fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
        None
    }
    fn connect_signal(
        &mut self,
        signal: &str,
        callback: SignalCallback,
        conn_type: ConnectionType,
    ) -> Option<ConnectionId> {
        match signal {
            "sig_a" => {
                let cb = Arc::new(callback);
                Some(self.sig_a.connect_typed(
                    move |args: &(i32,)| (*cb)(&[IntoValue::into_value(args.0)]),
                    conn_type,
                ))
            }
            _ => None,
        }
    }
    fn emit_signal(&mut self, signal: &str, args: &[Value]) -> Option<()> {
        match signal {
            "sig_a" => {
                if args.len() != 1 {
                    return None;
                }
                let v: i32 = FromValue::from_value(args[0].clone()).ok()?;
                quartzite_core::emit!(self.sig_a, &(v,));
                Some(())
            }
            _ => None,
        }
    }
}

// ── Receiver (1-arg i32 signal) ──────────────────────────────────────────────

struct Receiver {
    base: ObjectBase,
    pub sig_b: Signal<(i32,)>,
}

impl Receiver {
    fn new() -> Self {
        Self {
            base: ObjectBase::new(),
            sig_b: Signal::new(),
        }
    }
}

impl AsObject for Receiver {
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

static RECV_PARAMS: [ParamMeta; 1] = [ParamMeta::new("arg0", "i32")];
static RECV_SIGNALS: [SignalMeta; 1] = [SignalMeta::new("sig_b", &RECV_PARAMS)];
static RECV_META: MetaObject = MetaObject::new(
    "Receiver",
    &[],
    &RECV_SIGNALS,
    &[],
    &[],
    noop_lookup_property,
    |name| {
        if name == "sig_b" {
            Some(RECV_SIGNALS[0])
        } else {
            None
        }
    },
    noop_lookup_method,
    noop_lookup_enum,
);

impl Object for Receiver {
    fn meta_object(&self) -> &'static MetaObject {
        &RECV_META
    }
    fn read_property(&self, _: &str) -> Option<Value> {
        None
    }
    fn write_property(&mut self, _: &str, _: Value) -> bool {
        false
    }
    fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
        None
    }
    fn connect_signal(
        &mut self,
        signal: &str,
        callback: SignalCallback,
        conn_type: ConnectionType,
    ) -> Option<ConnectionId> {
        match signal {
            "sig_b" => {
                let cb = Arc::new(callback);
                Some(self.sig_b.connect_typed(
                    move |args: &(i32,)| (*cb)(&[IntoValue::into_value(args.0)]),
                    conn_type,
                ))
            }
            _ => None,
        }
    }
    fn emit_signal(&mut self, signal: &str, args: &[Value]) -> Option<()> {
        match signal {
            "sig_b" => {
                if args.len() != 1 {
                    return None;
                }
                let v: i32 = FromValue::from_value(args[0].clone()).ok()?;
                quartzite_core::emit!(self.sig_b, &(v,));
                Some(())
            }
            _ => None,
        }
    }
}

// ── NullRecv (0-arg signal) ───────────────────────────────────────────────────

struct NullRecv {
    base: ObjectBase,
    pub sig: Signal<()>,
}

static NULL_SIGS: [SignalMeta; 1] = [SignalMeta::new("sig", &[])];
static NULL_META: MetaObject = MetaObject::new(
    "NullRecv",
    &[],
    &NULL_SIGS,
    &[],
    &[],
    noop_lookup_property,
    |name| {
        if name == "sig" {
            Some(NULL_SIGS[0])
        } else {
            None
        }
    },
    noop_lookup_method,
    noop_lookup_enum,
);

impl AsObject for NullRecv {
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

impl Object for NullRecv {
    fn meta_object(&self) -> &'static MetaObject {
        &NULL_META
    }
    fn read_property(&self, _: &str) -> Option<Value> {
        None
    }
    fn write_property(&mut self, _: &str, _: Value) -> bool {
        false
    }
    fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
        None
    }
    fn connect_signal(
        &mut self,
        signal: &str,
        callback: SignalCallback,
        conn_type: ConnectionType,
    ) -> Option<ConnectionId> {
        match signal {
            "sig" => {
                let cb = Arc::new(callback);
                Some(
                    self.sig
                        .connect_typed(move |_args: &()| (*cb)(&[]), conn_type),
                )
            }
            _ => None,
        }
    }
    fn emit_signal(&mut self, signal: &str, args: &[Value]) -> Option<()> {
        match signal {
            "sig" => {
                if args.is_empty() {
                    quartzite_core::emit!(self.sig, &());
                    Some(())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ── RecordingNullRecv (0-arg, sets AtomicBool on emit) ───────────────────────

struct RecordingNullRecv {
    base: ObjectBase,
    pub sig: Signal<()>,
    pub fired: Arc<AtomicBool>,
}

impl AsObject for RecordingNullRecv {
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

impl Object for RecordingNullRecv {
    fn meta_object(&self) -> &'static MetaObject {
        &NULL_META
    }
    fn read_property(&self, _: &str) -> Option<Value> {
        None
    }
    fn write_property(&mut self, _: &str, _: Value) -> bool {
        false
    }
    fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
        None
    }
    fn connect_signal(
        &mut self,
        signal: &str,
        callback: SignalCallback,
        conn_type: ConnectionType,
    ) -> Option<ConnectionId> {
        match signal {
            "sig" => {
                let cb = Arc::new(callback);
                Some(
                    self.sig
                        .connect_typed(move |_args: &()| (*cb)(&[]), conn_type),
                )
            }
            _ => None,
        }
    }
    fn emit_signal(&mut self, signal: &str, args: &[Value]) -> Option<()> {
        match signal {
            "sig" => {
                if args.is_empty() {
                    self.fired.store(true, Ordering::Relaxed);
                    quartzite_core::emit!(self.sig, &());
                    Some(())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ── Sender2 (2-arg (i32, i32) signal) ────────────────────────────────────────

struct Sender2 {
    base: ObjectBase,
    pub sig: Signal<(i32, i32)>,
}

impl Sender2 {
    fn new() -> Self {
        Self {
            base: ObjectBase::new(),
            sig: Signal::new(),
        }
    }
}

static S2_PARAMS: [ParamMeta; 2] = [ParamMeta::new("a", "i32"), ParamMeta::new("b", "i32")];
static S2_SIGS: [SignalMeta; 1] = [SignalMeta::new("sig", &S2_PARAMS)];
static S2_META: MetaObject = MetaObject::new(
    "Sender2",
    &[],
    &S2_SIGS,
    &[],
    &[],
    noop_lookup_property,
    |name| {
        if name == "sig" {
            Some(S2_SIGS[0])
        } else {
            None
        }
    },
    noop_lookup_method,
    noop_lookup_enum,
);

impl AsObject for Sender2 {
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

impl Object for Sender2 {
    fn meta_object(&self) -> &'static MetaObject {
        &S2_META
    }
    fn read_property(&self, _: &str) -> Option<Value> {
        None
    }
    fn write_property(&mut self, _: &str, _: Value) -> bool {
        false
    }
    fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
        None
    }
    fn connect_signal(
        &mut self,
        signal: &str,
        callback: SignalCallback,
        conn_type: ConnectionType,
    ) -> Option<ConnectionId> {
        match signal {
            "sig" => {
                let cb = Arc::new(callback);
                Some(self.sig.connect_typed(
                    move |args: &(i32, i32)| {
                        (*cb)(&[IntoValue::into_value(args.0), IntoValue::into_value(args.1)]);
                    },
                    conn_type,
                ))
            }
            _ => None,
        }
    }
    fn emit_signal(&mut self, signal: &str, args: &[Value]) -> Option<()> {
        match signal {
            "sig" => {
                if args.len() != 2 {
                    return None;
                }
                let a: i32 = FromValue::from_value(args[0].clone()).ok()?;
                let b: i32 = FromValue::from_value(args[1].clone()).ok()?;
                quartzite_core::emit!(self.sig, &(a, b));
                Some(())
            }
            _ => None,
        }
    }
}

// ── BigReceiver (2-arg, receives into the void) ───────────────────────────────

struct BigReceiver {
    base: ObjectBase,
    pub _sig: Signal<(i32, i32)>,
}

static BIG_RECV_PARAMS: [ParamMeta; 2] = [ParamMeta::new("a", "i32"), ParamMeta::new("b", "i32")];
static BIG_RECV_SIGS: [SignalMeta; 1] = [SignalMeta::new("sig", &BIG_RECV_PARAMS)];
static BIG_RECV_META: MetaObject = MetaObject::new(
    "BigReceiver",
    &[],
    &BIG_RECV_SIGS,
    &[],
    &[],
    noop_lookup_property,
    |name| {
        if name == "sig" {
            Some(BIG_RECV_SIGS[0])
        } else {
            None
        }
    },
    noop_lookup_method,
    noop_lookup_enum,
);

impl AsObject for BigReceiver {
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

impl Object for BigReceiver {
    fn meta_object(&self) -> &'static MetaObject {
        &BIG_RECV_META
    }
    fn read_property(&self, _: &str) -> Option<Value> {
        None
    }
    fn write_property(&mut self, _: &str, _: Value) -> bool {
        false
    }
    fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
        None
    }
    fn connect_signal(
        &mut self,
        _: &str,
        _: SignalCallback,
        _: ConnectionType,
    ) -> Option<ConnectionId> {
        None
    }
    fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
        None
    }
}

// ── RecordingSlotRecv ─────────────────────────────────────────────────────────

struct RecordingSlotRecv {
    base: ObjectBase,
    meta: &'static MetaObject,
    pub recorded: Arc<Mutex<Vec<Value>>>,
}

impl RecordingSlotRecv {
    fn new(meta: &'static MetaObject, recorded: Arc<Mutex<Vec<Value>>>) -> Self {
        Self {
            base: ObjectBase::new(),
            meta,
            recorded,
        }
    }
}

static RSR_PARAMS_1ARG: [ParamMeta; 1] = [ParamMeta::new("v", "i32")];
static RSR_METHODS_1ARG: [MethodMeta; 1] = [MethodMeta::new("on_click", &RSR_PARAMS_1ARG, "()")];
static RECV_META_VALIDATED_1ARG: MetaObject = MetaObject::new(
    "RecordingSlotRecv",
    &[],
    &[],
    &RSR_METHODS_1ARG,
    &[],
    noop_lookup_property,
    noop_lookup_signal,
    |name| {
        if name == "on_click" {
            Some(RSR_METHODS_1ARG[0])
        } else {
            None
        }
    },
    noop_lookup_enum,
);

static RSR_PARAMS_BOOL: [ParamMeta; 1] = [ParamMeta::new("v", "bool")];
static RSR_METHODS_BOOL: [MethodMeta; 1] = [MethodMeta::new("on_click", &RSR_PARAMS_BOOL, "()")];
static RECV_META_VALIDATED_BOOL: MetaObject = MetaObject::new(
    "RecordingSlotRecv",
    &[],
    &[],
    &RSR_METHODS_BOOL,
    &[],
    noop_lookup_property,
    noop_lookup_signal,
    |name| {
        if name == "on_click" {
            Some(RSR_METHODS_BOOL[0])
        } else {
            None
        }
    },
    noop_lookup_enum,
);

static RECV_META_FALLBACK: MetaObject = MetaObject::new(
    "RecordingSlotRecv",
    &[],
    &[],
    &[],
    &[],
    noop_lookup_property,
    noop_lookup_signal,
    noop_lookup_method,
    noop_lookup_enum,
);

impl AsObject for RecordingSlotRecv {
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

impl Object for RecordingSlotRecv {
    fn meta_object(&self) -> &'static MetaObject {
        self.meta
    }
    fn read_property(&self, _: &str) -> Option<Value> {
        None
    }
    fn write_property(&mut self, _: &str, _: Value) -> bool {
        false
    }
    fn invoke_method(&mut self, _method: &str, args: &[Value]) -> Option<Value> {
        *self.recorded.lock() = args.to_vec();
        None
    }
    fn connect_signal(
        &mut self,
        _: &str,
        _: SignalCallback,
        _: ConnectionType,
    ) -> Option<ConnectionId> {
        None
    }
    fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
        None
    }
}

// ── Tests: connect_signal_to_signal ──────────────────────────────────────────

#[test]
fn unknown_from_signal_returns_error() {
    let mut sender = Sender::new();
    let to = Arc::new(Mutex::new(Receiver::new())) as Arc<Mutex<dyn Object>>;
    let err = connect_signal_to_signal(
        &mut sender,
        "nonexistent",
        &to,
        "sig_b",
        ConnectionType::Direct,
    )
    .unwrap_err();
    assert_eq!(
        err,
        SignalConnectionError::UnknownFromSignal("nonexistent".into())
    );
}

#[test]
fn unknown_to_signal_returns_error() {
    let mut sender = Sender::new();
    let to = Arc::new(Mutex::new(Receiver::new())) as Arc<Mutex<dyn Object>>;
    let err = connect_signal_to_signal(
        &mut sender,
        "sig_a",
        &to,
        "nonexistent",
        ConnectionType::Direct,
    )
    .unwrap_err();
    assert_eq!(
        err,
        SignalConnectionError::UnknownToSignal("nonexistent".into())
    );
}

#[test]
fn arity_mismatch_returns_error() {
    // New rule: from_arity < to_arity is the only rejection.
    // 0-arg source (NullRecv) → 1-arg target (Receiver): ArityMismatch { from: 0, to: 1 }.
    let mut source = NullRecv {
        base: ObjectBase::new(),
        sig: Signal::new(),
    };
    let to = Arc::new(Mutex::new(Receiver::new())) as Arc<Mutex<dyn Object>>;
    let err = connect_signal_to_signal(&mut source, "sig", &to, "sig_b", ConnectionType::Direct)
        .unwrap_err();
    assert_eq!(err, SignalConnectionError::ArityMismatch { from: 0, to: 1 });
}

#[test]
fn arity_relaxation_truncates_extras() {
    // 1-arg source (Sender / sig_a) → 0-arg target (RecordingNullRecv / sig).
    // from_arity (1) >= to_arity (0): connect succeeds and the target fires on emit.
    let fired = Arc::new(AtomicBool::new(false));
    let mut sender = Sender::new();
    let rnr = RecordingNullRecv {
        base: ObjectBase::new(),
        sig: Signal::new(),
        fired: Arc::clone(&fired),
    };
    let to = Arc::new(Mutex::new(rnr)) as Arc<Mutex<dyn Object>>;
    connect_signal_to_signal(&mut sender, "sig_a", &to, "sig", ConnectionType::Direct)
        .expect("connection must succeed: from_arity (1) >= to_arity (0)");
    sender.sig_a.emit_unconditionally(&(42,));
    assert!(
        fired.load(Ordering::Relaxed),
        "target emit_signal must have been called after truncation"
    );
}

#[test]
fn type_mismatch_returns_error() {
    struct BoolRecv {
        base: ObjectBase,
        pub _sig: Signal<(bool,)>,
    }
    static BOOL_PARAMS: [ParamMeta; 1] = [ParamMeta::new("arg0", "bool")];
    static BOOL_SIGS: [SignalMeta; 1] = [SignalMeta::new("sig", &BOOL_PARAMS)];
    static BOOL_META: MetaObject = MetaObject::new(
        "BoolRecv",
        &[],
        &BOOL_SIGS,
        &[],
        &[],
        noop_lookup_property,
        |name| {
            if name == "sig" {
                Some(BOOL_SIGS[0])
            } else {
                None
            }
        },
        noop_lookup_method,
        noop_lookup_enum,
    );
    impl AsObject for BoolRecv {
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
    impl Object for BoolRecv {
        fn meta_object(&self) -> &'static MetaObject {
            &BOOL_META
        }
        fn read_property(&self, _: &str) -> Option<Value> {
            None
        }
        fn write_property(&mut self, _: &str, _: Value) -> bool {
            false
        }
        fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
            None
        }
        fn connect_signal(
            &mut self,
            _: &str,
            _: SignalCallback,
            _: ConnectionType,
        ) -> Option<ConnectionId> {
            None
        }
        fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
            None
        }
    }
    let mut sender = Sender::new();
    let to = Arc::new(Mutex::new(BoolRecv {
        base: ObjectBase::new(),
        _sig: Signal::new(),
    })) as Arc<Mutex<dyn Object>>;
    let err = connect_signal_to_signal(&mut sender, "sig_a", &to, "sig", ConnectionType::Direct)
        .unwrap_err();
    assert_eq!(
        err,
        SignalConnectionError::TypeMismatch {
            index: 0,
            from: "i32".into(),
            to: "bool".into()
        }
    );
}

#[test]
fn direct_connection_forwards_signal() {
    let mut sender = Sender::new();
    let receiver = Arc::new(Mutex::new(Receiver::new()));
    let counter = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&counter);
        receiver.lock().sig_b.connect(move |args: &(i32,)| {
            c.store(args.0, Ordering::Relaxed);
        });
    }
    let to: Arc<Mutex<dyn Object>> = Arc::clone(&receiver) as Arc<Mutex<dyn Object>>;
    let id = connect_signal_to_signal(&mut sender, "sig_a", &to, "sig_b", ConnectionType::Direct)
        .expect("connection must succeed");
    sender.sig_a.emit_unconditionally(&(42,));
    assert_eq!(
        counter.load(Ordering::Relaxed),
        42,
        "value must be forwarded"
    );
    sender.sig_a.disconnect(id);
    sender.sig_a.emit_unconditionally(&(99,));
    assert_eq!(
        counter.load(Ordering::Relaxed),
        42,
        "value must not change after disconnect"
    );
}

#[test]
fn single_shot_fires_once_and_slot_is_removed() {
    let mut sender = Sender::new();
    let receiver = Arc::new(Mutex::new(Receiver::new()));
    let counter = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&counter);
        receiver
            .lock()
            .sig_b
            .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
    }
    let to: Arc<Mutex<dyn Object>> = Arc::clone(&receiver) as Arc<Mutex<dyn Object>>;
    connect_signal_to_signal(
        &mut sender,
        "sig_a",
        &to,
        "sig_b",
        ConnectionType::SingleShot,
    )
    .expect("connection must succeed");
    sender.sig_a.emit_unconditionally(&(1,));
    assert_eq!(
        counter.load(Ordering::Relaxed),
        1,
        "must fire on first emit"
    );
    sender.sig_a.emit_unconditionally(&(2,));
    assert_eq!(
        counter.load(Ordering::Relaxed),
        1,
        "slot must not fire on second emit"
    );
}

#[test]
fn liveness_silently_drops_when_to_arc_released() {
    let mut sender = Sender::new();
    let receiver = Arc::new(Mutex::new(Receiver::new()));
    let counter = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&counter);
        receiver.lock().sig_b.connect(move |args: &(i32,)| {
            c.store(args.0, Ordering::Relaxed);
        });
    }
    let to: Arc<Mutex<dyn Object>> = Arc::clone(&receiver) as Arc<Mutex<dyn Object>>;
    connect_signal_to_signal(&mut sender, "sig_a", &to, "sig_b", ConnectionType::Direct).unwrap();
    sender.sig_a.emit_unconditionally(&(1,));
    assert_eq!(counter.load(Ordering::Relaxed), 1);
    drop(to);
    drop(receiver);
    sender.sig_a.emit_unconditionally(&(2,));
    assert_eq!(
        counter.load(Ordering::Relaxed),
        1,
        "no forwarding after to is dropped"
    );
}

#[test]
fn auto_same_thread_fires_synchronously() {
    let mut sender = Sender::new();
    let receiver = Arc::new(Mutex::new(Receiver::new()));
    let counter = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&counter);
        receiver
            .lock()
            .sig_b
            .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
    }
    let to: Arc<Mutex<dyn Object>> = Arc::clone(&receiver) as Arc<Mutex<dyn Object>>;
    connect_signal_to_signal(&mut sender, "sig_a", &to, "sig_b", ConnectionType::Auto).unwrap();
    sender.sig_a.emit_unconditionally(&(55,));
    assert_eq!(
        counter.load(Ordering::Relaxed),
        55,
        "AC6 same-thread Auto must fire synchronously"
    );
}

// ── Tests: connect_signals (typed) ───────────────────────────────────────────

#[test]
fn connect_signals_typed_direct_forwards() {
    let mut sender = Sender::new();
    let receiver = Arc::new(Mutex::new(Receiver::new()));
    let counter = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&counter);
        receiver.lock().sig_b.connect(move |args: &(i32,)| {
            c.store(args.0, Ordering::Relaxed);
        });
    }
    let id = connect_signals(
        &mut sender,
        "sig_a",
        |obj: &mut Sender| &mut obj.sig_a,
        &receiver,
        "sig_b",
        ConnectionType::Direct,
    )
    .expect("typed connection must succeed");
    sender.sig_a.emit_unconditionally(&(7,));
    assert_eq!(counter.load(Ordering::Relaxed), 7);
    sender.sig_a.disconnect(id);
    sender.sig_a.emit_unconditionally(&(8,));
    assert_eq!(
        counter.load(Ordering::Relaxed),
        7,
        "no forwarding after disconnect"
    );
}

#[test]
fn connect_signals_typed_liveness() {
    let mut sender = Sender::new();
    let receiver = Arc::new(Mutex::new(Receiver::new()));
    let counter = Arc::new(AtomicI32::new(0));
    {
        let c = Arc::clone(&counter);
        receiver.lock().sig_b.connect(move |args: &(i32,)| {
            c.store(args.0, Ordering::Relaxed);
        });
    }
    connect_signals(
        &mut sender,
        "sig_a",
        |obj: &mut Sender| &mut obj.sig_a,
        &receiver,
        "sig_b",
        ConnectionType::Direct,
    )
    .unwrap();
    sender.sig_a.emit_unconditionally(&(3,));
    assert_eq!(counter.load(Ordering::Relaxed), 3);
    drop(receiver);
    sender.sig_a.emit_unconditionally(&(4,));
    assert_eq!(
        counter.load(Ordering::Relaxed),
        3,
        "no forwarding after drop"
    );
}

#[test]
fn connect_signals_typed_arity_mismatch_returns_error() {
    // New rule: from_arity < to_arity is the only rejection.
    // Sender (1-arg sig_a) → BigReceiver (2-arg sig): ArityMismatch { from: 1, to: 2 }.
    let mut sender = Sender::new();
    let big_recv = Arc::new(Mutex::new(BigReceiver {
        base: ObjectBase::new(),
        _sig: Signal::new(),
    }));
    let err = connect_signals::<_, _, (i32,)>(
        &mut sender,
        "sig_a",
        |obj: &mut Sender| &mut obj.sig_a,
        &big_recv,
        "sig",
        ConnectionType::Direct,
    )
    .unwrap_err();
    assert_eq!(err, SignalConnectionError::ArityMismatch { from: 1, to: 2 });
}

#[test]
fn connect_signals_typed_truncates_extras() {
    // 2-arg source (Sender2 / sig) → 1-arg target (Receiver / sig_b).
    // from_arity (2) >= to_arity (1): connect succeeds; only the first arg is forwarded.
    let mut s2 = Sender2::new();
    let receiver = Arc::new(Mutex::new(Receiver::new()));
    let captured = Arc::new(AtomicI32::new(-1));
    {
        let c = Arc::clone(&captured);
        receiver
            .lock()
            .sig_b
            .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
    }
    connect_signals::<_, _, (i32, i32)>(
        &mut s2,
        "sig",
        |obj: &mut Sender2| &mut obj.sig,
        &receiver,
        "sig_b",
        ConnectionType::Direct,
    )
    .expect("connection must succeed: from_arity (2) >= to_arity (1)");
    s2.sig.emit_unconditionally(&(11, 99));
    assert_eq!(
        captured.load(Ordering::Relaxed),
        11,
        "only the first arg must be forwarded; the second is dropped"
    );
}

#[test]
fn connect_signals_typed_type_mismatch_returns_error() {
    struct BoolRecv2 {
        base: ObjectBase,
        _sig: Signal<(bool,)>,
    }
    static BR2_PARAMS: [ParamMeta; 1] = [ParamMeta::new("arg0", "bool")];
    static BR2_SIGS: [SignalMeta; 1] = [SignalMeta::new("sig", &BR2_PARAMS)];
    static BR2_META: MetaObject = MetaObject::new(
        "BoolRecv2",
        &[],
        &BR2_SIGS,
        &[],
        &[],
        noop_lookup_property,
        |name| {
            if name == "sig" {
                Some(BR2_SIGS[0])
            } else {
                None
            }
        },
        noop_lookup_method,
        noop_lookup_enum,
    );
    impl AsObject for BoolRecv2 {
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
    impl Object for BoolRecv2 {
        fn meta_object(&self) -> &'static MetaObject {
            &BR2_META
        }
        fn read_property(&self, _: &str) -> Option<Value> {
            None
        }
        fn write_property(&mut self, _: &str, _: Value) -> bool {
            false
        }
        fn invoke_method(&mut self, _: &str, _: &[Value]) -> Option<Value> {
            None
        }
        fn connect_signal(
            &mut self,
            _: &str,
            _: SignalCallback,
            _: ConnectionType,
        ) -> Option<ConnectionId> {
            None
        }
        fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
            None
        }
    }
    let mut sender = Sender::new();
    let br2 = Arc::new(Mutex::new(BoolRecv2 {
        base: ObjectBase::new(),
        _sig: Signal::new(),
    }));
    let err = connect_signals::<_, _, (i32,)>(
        &mut sender,
        "sig_a",
        |obj: &mut Sender| &mut obj.sig_a,
        &br2,
        "sig",
        ConnectionType::Direct,
    )
    .unwrap_err();
    assert_eq!(
        err,
        SignalConnectionError::TypeMismatch {
            index: 0,
            from: "i32".into(),
            to: "bool".into()
        }
    );
}

// ── Tests: connect_signal_to_slot ─────────────────────────────────────────────

#[test]
fn slot_arity_validation_truncates_args() {
    // 2-arg source (Sender2 / sig) → RECV_META_VALIDATED_1ARG slot "on_click".
    // from_arity (2) >= slot_arity (1): connect succeeds; slot is invoked with first arg only.
    let recorded = Arc::new(Mutex::new(Vec::<Value>::new()));
    let target = Arc::new(Mutex::new(RecordingSlotRecv::new(
        &RECV_META_VALIDATED_1ARG,
        Arc::clone(&recorded),
    ))) as Arc<Mutex<dyn Object>>;
    let mut s2 = Sender2::new();
    connect_signal_to_slot(&mut s2, "sig", &target, "on_click")
        .expect("connection must succeed: from_arity (2) >= slot_arity (1)");
    s2.sig.emit_unconditionally(&(7, 8));
    assert_eq!(
        *recorded.lock(),
        vec![Value::Int(7)],
        "slot must be invoked with first arg only"
    );
}

#[test]
fn slot_arity_mismatch_returns_error() {
    // 0-arg source (NullRecv / sig) → RECV_META_VALIDATED_1ARG slot "on_click".
    // from_arity (0) < slot_arity (1): ArityMismatch { from: 0, to: 1 }.
    let recorded = Arc::new(Mutex::new(Vec::<Value>::new()));
    let target = Arc::new(Mutex::new(RecordingSlotRecv::new(
        &RECV_META_VALIDATED_1ARG,
        Arc::clone(&recorded),
    ))) as Arc<Mutex<dyn Object>>;
    let mut source = NullRecv {
        base: ObjectBase::new(),
        sig: Signal::new(),
    };
    let err = connect_signal_to_slot(&mut source, "sig", &target, "on_click").unwrap_err();
    assert_eq!(err, SignalConnectionError::ArityMismatch { from: 0, to: 1 });
}

#[test]
fn slot_type_mismatch_returns_error() {
    // 1-arg i32 source (Sender / sig_a) → RECV_META_VALIDATED_BOOL slot "on_click".
    // type_name "i32" != "bool": TypeMismatch { index: 0, from: "i32", to: "bool" }.
    let recorded = Arc::new(Mutex::new(Vec::<Value>::new()));
    let target = Arc::new(Mutex::new(RecordingSlotRecv::new(
        &RECV_META_VALIDATED_BOOL,
        Arc::clone(&recorded),
    ))) as Arc<Mutex<dyn Object>>;
    let mut sender = Sender::new();
    let err = connect_signal_to_slot(&mut sender, "sig_a", &target, "on_click").unwrap_err();
    assert_eq!(
        err,
        SignalConnectionError::TypeMismatch {
            index: 0,
            from: "i32".into(),
            to: "bool".into()
        }
    );
}

#[test]
fn slot_meta_absent_falls_back_to_empty_args() {
    // 1-arg source (Sender / sig_a) → RECV_META_FALLBACK slot "on_click".
    // Slot not in meta-system: connection succeeds, slot invoked with &[] at emit time.
    let recorded = Arc::new(Mutex::new(Vec::<Value>::new()));
    let target = Arc::new(Mutex::new(RecordingSlotRecv::new(
        &RECV_META_FALLBACK,
        Arc::clone(&recorded),
    ))) as Arc<Mutex<dyn Object>>;
    let mut sender = Sender::new();
    connect_signal_to_slot(&mut sender, "sig_a", &target, "on_click")
        .expect("connection must succeed: fallback path (slot not in meta)");
    sender.sig_a.emit_unconditionally(&(42,));
    assert_eq!(
        *recorded.lock(),
        vec![],
        "fallback path must invoke slot with empty args"
    );
}
