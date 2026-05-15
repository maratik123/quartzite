//! Signal-to-signal connection utilities.
use std::sync::{Arc, Weak};

use parking_lot::Mutex;

/// Re-exported so callers can name the bound: `Args: connect::ArgsToValues`.
#[doc(hidden)]
pub use crate::args_to_values::ArgsToValues;

use crate::{
    ConnectionId,
    receiver_guard::ReceiverGuard,
    signal::{ConnectionType, Signal, queued_dispatcher},
    traits::{Object, SignalCallback},
    value::Value,
};

/// Error returned when a signal-to-signal connection cannot be established.
///
/// # Examples
///
/// ```
/// use quartzite_core::connect::SignalConnectionError;
///
/// let err = SignalConnectionError::UnknownFromSignal("clicked".into());
/// assert!(err.to_string().contains("clicked"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SignalConnectionError {
    /// The named signal was not found on the source object.
    #[error("unknown signal `{0}` on source object")]
    UnknownFromSignal(String),
    /// The named signal was not found on the target object.
    #[error("unknown signal `{0}` on target object")]
    UnknownToSignal(String),
    /// The source and target signals have different parameter counts.
    #[error("arity mismatch: source signal has {from} parameters, target has {to}")]
    ArityMismatch {
        /// Parameter count of the source signal.
        from: usize,
        /// Parameter count of the target signal.
        to: usize,
    },
    /// A parameter's type name differs between source and target.
    #[error("type mismatch at parameter {index}: source `{from}`, target `{to}`")]
    TypeMismatch {
        /// Zero-based parameter index where the mismatch occurred.
        index: usize,
        /// Type name from the source signal.
        from: String,
        /// Type name from the target signal.
        to: String,
    },
    /// `connect_signal` rejected a signal name that `meta_object().signal()` accepted.
    ///
    /// This always indicates an inconsistency in the [`Object`] implementation.
    #[error("internal error: `connect_signal` rejected validated signal `{0}`")]
    InternalError(String),
}

/// Connects `from_signal` on `from` to `to_signal` on `to` using the dynamic meta-system.
///
/// When `from_signal` is emitted, the forwarding callback invokes `to_signal` on `to`
/// with the same argument values. The connection silently breaks when all strong [`Arc`]
/// holders of `to` are released.
///
/// Type compatibility is validated at connection time by comparing
/// [`SignalMeta::params`](crate::meta::SignalMeta) arity and `type_name` strings.
///
/// # Cycles
///
/// Cycle detection is **not** performed. A `Direct` cycle (A emits → B emits → A)
/// causes unbounded recursion and a stack overflow. A `Queued` cycle posts tasks
/// indefinitely. It is the caller's responsibility to avoid cyclic topologies.
///
/// # Parameters
///
/// - `from`: source object whose signal will be forwarded.
/// - `from_signal`: name of the signal on `from` to forward from.
/// - `to`: shared target object; wrapped in `Arc<Mutex<dyn Object>>` so the
///   forwarding callback can invoke it after the connection is established.
/// - `to_signal`: name of the signal on `to` to emit on each forward.
/// - `conn_type`: delivery mode for the forwarding callback; `Auto` detects
///   whether the emit occurs on `to`'s owner thread at runtime.
///
/// # Errors
///
/// Returns [`SignalConnectionError`] when either signal name is unknown, arities
/// differ, any `type_name` pair mismatches, or the validated signal name is
/// unexpectedly rejected by [`Object::connect_signal`]
/// ([`SignalConnectionError::InternalError`]).
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use parking_lot::Mutex;
/// use quartzite_core::{Object, connect::{connect_signal_to_signal, SignalConnectionError}};
/// use quartzite_core::signal::ConnectionType;
/// # fn example(from: &mut impl Object, to: Arc<Mutex<impl Object + Send + 'static>>) {
/// let id = connect_signal_to_signal(from, "clicked", to, "clicked", ConnectionType::Direct);
/// # }
/// ```
pub fn connect_signal_to_signal(
    from: &mut dyn Object,
    from_signal: &str,
    to: Arc<Mutex<dyn Object>>,
    to_signal: &str,
    conn_type: ConnectionType,
) -> Result<ConnectionId, SignalConnectionError> {
    let from_meta = from
        .meta_object()
        .signal(from_signal)
        .ok_or_else(|| SignalConnectionError::UnknownFromSignal(from_signal.into()))?;

    let to_signal_name = to_signal.to_owned();
    let to_meta = {
        let guard = to.lock();
        guard
            .meta_object()
            .signal(to_signal)
            .ok_or_else(|| SignalConnectionError::UnknownToSignal(to_signal.into()))?
    };

    // Validate arity.
    let from_arity = from_meta.params.len();
    let to_arity = to_meta.params.len();
    if from_arity != to_arity {
        return Err(SignalConnectionError::ArityMismatch {
            from: from_arity,
            to: to_arity,
        });
    }

    // Validate type names.
    for (i, (fp, tp)) in from_meta
        .params
        .iter()
        .zip(to_meta.params.iter())
        .enumerate()
    {
        if fp.type_name != tp.type_name {
            return Err(SignalConnectionError::TypeMismatch {
                index: i,
                from: fp.type_name.into(),
                to: tp.type_name.into(),
            });
        }
    }

    let to_thread_id = to.lock().object_base().thread_id;
    let to_weak: Weak<Mutex<dyn Object>> = Arc::downgrade(&to);

    let callback: SignalCallback = match conn_type {
        ConnectionType::Direct => Box::new(move |args: &[Value]| {
            if let Some(arc) = to_weak.upgrade() {
                let _ = arc.lock().emit_signal(&to_signal_name, args);
            }
        }),
        ConnectionType::SingleShot => Box::new(move |args: &[Value]| {
            if let Some(arc) = to_weak.upgrade() {
                let _ = arc.lock().emit_signal(&to_signal_name, args);
            }
        }),
        ConnectionType::Queued => Box::new(move |args: &[Value]| {
            let Some(arc) = to_weak.upgrade() else {
                return;
            };
            let args_owned: Vec<Value> = args.to_vec();
            let sig_name = to_signal_name.clone();
            if let Some(d) = queued_dispatcher() {
                d.post(
                    to_thread_id,
                    Box::new(move || {
                        let _ = arc.lock().emit_signal(&sig_name, &args_owned);
                    }),
                );
            }
        }),
        ConnectionType::Auto => Box::new(move |args: &[Value]| {
            let Some(arc) = to_weak.upgrade() else {
                return;
            };
            if std::thread::current().id() == to_thread_id {
                let _ = arc.lock().emit_signal(&to_signal_name, args);
            } else {
                let args_owned: Vec<Value> = args.to_vec();
                let sig_name = to_signal_name.clone();
                if let Some(d) = queued_dispatcher() {
                    d.post(
                        to_thread_id,
                        Box::new(move || {
                            let _ = arc.lock().emit_signal(&sig_name, &args_owned);
                        }),
                    );
                }
            }
        }),
    };

    // Queued and Auto encode their delivery logic in the closure; at the signal level they
    // register as Direct so the slot persists. SingleShot propagates so Signal::emit_unconditionally
    // removes the slot after first delivery via its `retain` pass.
    let slot_type = if matches!(conn_type, ConnectionType::SingleShot) {
        ConnectionType::SingleShot
    } else {
        ConnectionType::Direct
    };
    from.connect_signal(from_signal, callback, slot_type)
        .ok_or_else(|| SignalConnectionError::InternalError(from_signal.into()))
}

/// Connects a typed signal field on `from` to a named signal on `to`.
///
/// Unlike [`connect_signal_to_signal`], this function uses the actual typed [`Signal<Args>`]
/// field for the source, avoiding the `&[Value]` round-trip on the hot path for `Auto`
/// and `Queued` connections.
///
/// Type compatibility is still validated at connection time via the meta-system.
///
/// # Parameters
///
/// - `from_obj`: source object.
/// - `from_signal_name`: meta-system name of the source signal, used for validation only.
/// - `from_signal_field`: function that extracts the `&mut Signal<Args>` from `from_obj`.
/// - `to`: shared target object.
/// - `to_signal_name`: name of the signal to emit on `to`.
/// - `conn_type`: delivery mode.
///
/// # Errors
///
/// - [`SignalConnectionError::UnknownFromSignal`] when `from_signal_name` is not
///   declared on `from_obj`.
/// - [`SignalConnectionError::UnknownToSignal`] when `to_signal_name` is not
///   declared on `to`.
/// - [`SignalConnectionError::ArityMismatch`] when the two signals have different
///   parameter counts.
/// - [`SignalConnectionError::TypeMismatch`] when any `type_name` string differs
///   between corresponding parameters.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use parking_lot::Mutex;
/// use quartzite_core::signal::{ConnectionType, Signal};
/// use quartzite_core::connect::{connect_signals, SignalConnectionError};
/// # use quartzite_core::Object;
/// # fn example<From: Object, To: Object + Send + 'static>(
/// #     from: &mut From,
/// #     to: Arc<Mutex<To>>,
/// # ) where
/// #     (i32,): quartzite_core::connect::ArgsToValues + Clone + Send + 'static,
/// # {
/// let _id = connect_signals::<_, _, (i32,)>(
///     from, "value_changed",
///     |_obj: &mut From| todo!(),
///     to, "value_changed",
///     ConnectionType::Direct,
/// );
/// # }
/// ```
pub fn connect_signals<From, To, Args>(
    from_obj: &mut From,
    from_signal_name: &str,
    from_signal_field: fn(&mut From) -> &mut Signal<Args>,
    to: Arc<Mutex<To>>,
    to_signal_name: &str,
    conn_type: ConnectionType,
) -> Result<ConnectionId, SignalConnectionError>
where
    From: Object,
    To: Object + Send + 'static,
    Args: ArgsToValues + Clone + Send + 'static,
{
    // Validate from_signal.
    let from_meta = from_obj
        .meta_object()
        .signal(from_signal_name)
        .ok_or_else(|| SignalConnectionError::UnknownFromSignal(from_signal_name.into()))?;

    // Validate to_signal.
    let to_signal_str = to_signal_name.to_owned();
    let to_meta = {
        let guard = to.lock();
        guard
            .meta_object()
            .signal(to_signal_name)
            .ok_or_else(|| SignalConnectionError::UnknownToSignal(to_signal_name.into()))?
    };

    // Arity check.
    let from_arity = from_meta.params.len();
    let to_arity = to_meta.params.len();
    if from_arity != to_arity {
        return Err(SignalConnectionError::ArityMismatch {
            from: from_arity,
            to: to_arity,
        });
    }

    // Type-name check.
    for (i, (fp, tp)) in from_meta
        .params
        .iter()
        .zip(to_meta.params.iter())
        .enumerate()
    {
        if fp.type_name != tp.type_name {
            return Err(SignalConnectionError::TypeMismatch {
                index: i,
                from: fp.type_name.into(),
                to: tp.type_name.into(),
            });
        }
    }

    let sig = from_signal_field(from_obj);

    let id = match conn_type {
        ConnectionType::Direct | ConnectionType::SingleShot => {
            let to_weak = Arc::downgrade(&to);
            let ct = conn_type;
            sig.connect_typed(
                move |args: &Args| {
                    let Some(arc) = to_weak.upgrade() else {
                        return;
                    };
                    let values = args.to_values();
                    let _ = arc.lock().emit_signal(&to_signal_str, &values);
                },
                ct,
            )
        }
        ConnectionType::Queued => {
            let (to_weak, guard_weak, thread_id) = {
                let guard = to.lock();
                let w: Weak<ReceiverGuard> = Arc::downgrade(guard.object_base().receiver_guard());
                let tid = guard.object_base().thread_id;
                (Arc::downgrade(&to), w, tid)
            };
            sig.connect_queued(
                thread_id,
                move |args: Args| {
                    let Some(arc) = to_weak.upgrade() else {
                        return;
                    };
                    let values = args.to_values();
                    let _ = arc.lock().emit_signal(&to_signal_str, &values);
                },
                guard_weak,
            )
        }
        ConnectionType::Auto => {
            let (to_weak, guard_weak, thread_id) = {
                let guard = to.lock();
                let w: Weak<ReceiverGuard> = Arc::downgrade(guard.object_base().receiver_guard());
                let tid = guard.object_base().thread_id;
                (Arc::downgrade(&to), w, tid)
            };
            sig.connect_auto(thread_id, guard_weak, move |args: Args| {
                let Some(arc) = to_weak.upgrade() else {
                    return;
                };
                let values = args.to_values();
                let _ = arc.lock().emit_signal(&to_signal_str, &values);
            })
        }
    };
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    };

    use parking_lot::Mutex;

    use crate::{
        id::ConnectionId,
        meta::MetaObject,
        object_base::ObjectBase,
        signal::{ConnectionType, Signal},
        value::Value,
    };

    // Minimal hand-written object with two signals for testing.
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

    impl crate::AsObject for Sender {
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

    static SENDER_PARAMS: [crate::meta::ParamMeta; 1] =
        [crate::meta::ParamMeta::new("arg0", "i32")];
    static SENDER_SIGNALS: [crate::meta::SignalMeta; 1] =
        [crate::meta::SignalMeta::new("sig_a", &SENDER_PARAMS)];
    static SENDER_META: MetaObject = MetaObject::new(
        "Sender",
        &[],
        &SENDER_SIGNALS,
        &[],
        &[],
        crate::meta::noop_lookup_property,
        |name| {
            if name == "sig_a" {
                Some(SENDER_SIGNALS[0])
            } else {
                None
            }
        },
        crate::meta::noop_lookup_method,
        crate::meta::noop_lookup_enum,
    );

    impl crate::Object for Sender {
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
            callback: crate::traits::SignalCallback,
            conn_type: ConnectionType,
        ) -> Option<ConnectionId> {
            match signal {
                "sig_a" => {
                    let cb = Arc::new(callback);
                    Some(self.sig_a.connect_typed(
                        move |args: &(i32,)| (*cb)(&[crate::IntoValue::into_value(args.0)]),
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
                    let v: i32 = crate::FromValue::from_value(args[0].clone()).ok()?;
                    crate::emit!(self.sig_a, &(v,));
                    Some(())
                }
                _ => None,
            }
        }
    }

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

    impl crate::AsObject for Receiver {
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

    static RECV_PARAMS: [crate::meta::ParamMeta; 1] = [crate::meta::ParamMeta::new("arg0", "i32")];
    static RECV_SIGNALS: [crate::meta::SignalMeta; 1] =
        [crate::meta::SignalMeta::new("sig_b", &RECV_PARAMS)];
    static RECV_META: MetaObject = MetaObject::new(
        "Receiver",
        &[],
        &RECV_SIGNALS,
        &[],
        &[],
        crate::meta::noop_lookup_property,
        |name| {
            if name == "sig_b" {
                Some(RECV_SIGNALS[0])
            } else {
                None
            }
        },
        crate::meta::noop_lookup_method,
        crate::meta::noop_lookup_enum,
    );

    impl crate::Object for Receiver {
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
            callback: crate::traits::SignalCallback,
            conn_type: ConnectionType,
        ) -> Option<ConnectionId> {
            match signal {
                "sig_b" => {
                    let cb = Arc::new(callback);
                    Some(self.sig_b.connect_typed(
                        move |args: &(i32,)| (*cb)(&[crate::IntoValue::into_value(args.0)]),
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
                    let v: i32 = crate::FromValue::from_value(args[0].clone()).ok()?;
                    crate::emit!(self.sig_b, &(v,));
                    Some(())
                }
                _ => None,
            }
        }
    }

    #[test]
    fn unknown_from_signal_returns_error() {
        let mut sender = Sender::new();
        let to = Arc::new(Mutex::new(Receiver::new())) as Arc<Mutex<dyn Object>>;
        let err = connect_signal_to_signal(
            &mut sender,
            "nonexistent",
            to,
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
            to,
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
        // Sender has sig_a: Signal<(i32,)> — 1 param.
        // Create a receiver with zero-param signal.
        struct NullRecv {
            base: ObjectBase,
            pub _sig: Signal<()>,
        }
        static NULL_SIGS: [crate::meta::SignalMeta; 1] = [crate::meta::SignalMeta::new("sig", &[])];
        static NULL_META: MetaObject = MetaObject::new(
            "NullRecv",
            &[],
            &NULL_SIGS,
            &[],
            &[],
            crate::meta::noop_lookup_property,
            |name| {
                if name == "sig" {
                    Some(NULL_SIGS[0])
                } else {
                    None
                }
            },
            crate::meta::noop_lookup_method,
            crate::meta::noop_lookup_enum,
        );
        impl crate::AsObject for NullRecv {
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
        impl crate::Object for NullRecv {
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
                _signal: &str,
                _callback: crate::traits::SignalCallback,
                _conn_type: ConnectionType,
            ) -> Option<ConnectionId> {
                None
            }
            fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
                None
            }
        }
        let mut sender = Sender::new();
        let nr = NullRecv {
            base: ObjectBase::new(),
            _sig: Signal::new(),
        };
        let to = Arc::new(Mutex::new(nr)) as Arc<Mutex<dyn Object>>;
        let err = connect_signal_to_signal(&mut sender, "sig_a", to, "sig", ConnectionType::Direct)
            .unwrap_err();
        assert_eq!(err, SignalConnectionError::ArityMismatch { from: 1, to: 0 });
    }

    #[test]
    fn type_mismatch_returns_error() {
        struct BoolRecv {
            base: ObjectBase,
            pub _sig: Signal<(bool,)>,
        }
        static BOOL_PARAMS: [crate::meta::ParamMeta; 1] =
            [crate::meta::ParamMeta::new("arg0", "bool")];
        static BOOL_SIGS: [crate::meta::SignalMeta; 1] =
            [crate::meta::SignalMeta::new("sig", &BOOL_PARAMS)];
        static BOOL_META: MetaObject = MetaObject::new(
            "BoolRecv",
            &[],
            &BOOL_SIGS,
            &[],
            &[],
            crate::meta::noop_lookup_property,
            |name| {
                if name == "sig" {
                    Some(BOOL_SIGS[0])
                } else {
                    None
                }
            },
            crate::meta::noop_lookup_method,
            crate::meta::noop_lookup_enum,
        );
        impl crate::AsObject for BoolRecv {
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
        impl crate::Object for BoolRecv {
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
                _signal: &str,
                _callback: crate::traits::SignalCallback,
                _conn_type: ConnectionType,
            ) -> Option<ConnectionId> {
                None
            }
            fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
                None
            }
        }
        let mut sender = Sender::new();
        let br = BoolRecv {
            base: ObjectBase::new(),
            _sig: Signal::new(),
        };
        let to = Arc::new(Mutex::new(br)) as Arc<Mutex<dyn Object>>;
        let err = connect_signal_to_signal(&mut sender, "sig_a", to, "sig", ConnectionType::Direct)
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
        let id =
            connect_signal_to_signal(&mut sender, "sig_a", to, "sig_b", ConnectionType::Direct)
                .expect("connection must succeed");

        sender.sig_a.emit_unconditionally(&(42,));
        assert_eq!(
            counter.load(Ordering::Relaxed),
            42,
            "value must be forwarded"
        );

        // AC8: disconnect stops forwarding.
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
            to,
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
            "slot must not fire on second emit — auto-disconnected after first delivery"
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
        connect_signal_to_signal(&mut sender, "sig_a", to, "sig_b", ConnectionType::Direct)
            .unwrap();
        // Verify it fires before drop.
        sender.sig_a.emit_unconditionally(&(1,));
        assert_eq!(counter.load(Ordering::Relaxed), 1);
        // Drop the strong Arc — Weak inside the callback now returns None.
        drop(receiver);
        // AC7: no panic, connection silently skipped.
        sender.sig_a.emit_unconditionally(&(2,));
        assert_eq!(
            counter.load(Ordering::Relaxed),
            1,
            "no forwarding after to is dropped"
        );
    }

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
            Arc::clone(&receiver),
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
            Arc::clone(&receiver),
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

    // AC6 — Auto same-thread: fires synchronously (no dispatcher needed).
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
        connect_signal_to_signal(&mut sender, "sig_a", to, "sig_b", ConnectionType::Auto).unwrap();

        sender.sig_a.emit_unconditionally(&(55,));
        assert_eq!(
            counter.load(Ordering::Relaxed),
            55,
            "AC6 same-thread Auto must fire synchronously"
        );
    }

    // R2 partial — connect_signals arity / type mismatch error arms
    // (connect_signal_to_signal already has arity/type tests; these cover
    // the same checks inside the typed connect_signals wrapper at lines 293-315.)

    #[test]
    fn connect_signals_typed_arity_mismatch_returns_error() {
        // Sender has sig_a: Signal<(i32,)> — 1 param.
        // Receiver has sig_b: Signal<(i32,)> — 1 param.
        // Pass `(i32, i32)` as Args to force a mismatch with Receiver's 1-param signal.
        // Note: from_arity in this path is inferred from Args length (2), but the
        // meta says sig_a has 1 param — so from_arity (1) != to_arity (1 for sig_b)
        // doesn't trigger, but connect_signals delegates to connect_signal_to_signal
        // which already does the check. To force arity mismatch within connect_signals
        // itself, use a 2-arg sender meta pointing at a 1-arg receiver.
        // Easiest approach: use connect_signal_to_signal's arity check result by
        // building a 2-arg sender. Since connect_signals calls connect_signal_to_signal
        // internally, validate via the top-level error result.
        struct Sender2 {
            base: ObjectBase,
            pub sig: Signal<(i32, i32)>,
        }
        static S2_PARAMS: [crate::meta::ParamMeta; 2] = [
            crate::meta::ParamMeta::new("a", "i32"),
            crate::meta::ParamMeta::new("b", "i32"),
        ];
        static S2_SIGS: [crate::meta::SignalMeta; 1] =
            [crate::meta::SignalMeta::new("sig", &S2_PARAMS)];
        static S2_META: MetaObject = MetaObject::new(
            "Sender2",
            &[],
            &S2_SIGS,
            &[],
            &[],
            crate::meta::noop_lookup_property,
            |name| {
                if name == "sig" {
                    Some(S2_SIGS[0])
                } else {
                    None
                }
            },
            crate::meta::noop_lookup_method,
            crate::meta::noop_lookup_enum,
        );
        impl crate::AsObject for Sender2 {
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
        impl crate::Object for Sender2 {
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
                _s: &str,
                _cb: crate::traits::SignalCallback,
                _ct: ConnectionType,
            ) -> Option<ConnectionId> {
                None
            }
            fn emit_signal(&mut self, _: &str, _: &[Value]) -> Option<()> {
                None
            }
        }
        let mut s2 = Sender2 {
            base: ObjectBase::new(),
            sig: Signal::new(),
        };
        let receiver = Arc::new(Mutex::new(Receiver::new()));
        let err = connect_signals::<_, _, (i32, i32)>(
            &mut s2,
            "sig",
            |obj: &mut Sender2| &mut obj.sig,
            Arc::clone(&receiver),
            "sig_b",
            ConnectionType::Direct,
        )
        .unwrap_err();
        assert_eq!(err, SignalConnectionError::ArityMismatch { from: 2, to: 1 });
    }

    #[test]
    fn connect_signals_typed_type_mismatch_returns_error() {
        // Sender has sig_a: Signal<(i32,)> — type "i32".
        // Build a receiver with sig carrying type "bool" to trigger TypeMismatch.
        struct BoolRecv2 {
            base: ObjectBase,
            pub sig: Signal<(bool,)>,
        }
        static BR2_PARAMS: [crate::meta::ParamMeta; 1] =
            [crate::meta::ParamMeta::new("arg0", "bool")];
        static BR2_SIGS: [crate::meta::SignalMeta; 1] =
            [crate::meta::SignalMeta::new("sig", &BR2_PARAMS)];
        static BR2_META: MetaObject = MetaObject::new(
            "BoolRecv2",
            &[],
            &BR2_SIGS,
            &[],
            &[],
            crate::meta::noop_lookup_property,
            |name| {
                if name == "sig" {
                    Some(BR2_SIGS[0])
                } else {
                    None
                }
            },
            crate::meta::noop_lookup_method,
            crate::meta::noop_lookup_enum,
        );
        impl crate::AsObject for BoolRecv2 {
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
        impl crate::Object for BoolRecv2 {
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
                _s: &str,
                _cb: crate::traits::SignalCallback,
                _ct: ConnectionType,
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
            sig: Signal::new(),
        }));
        let err = connect_signals::<_, _, (i32,)>(
            &mut sender,
            "sig_a",
            |obj: &mut Sender| &mut obj.sig_a,
            Arc::clone(&br2),
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

    // AC6 — Auto cross-thread: callback is posted to the queued dispatcher.
    #[test]
    #[cfg(feature = "std")]
    #[serial_test::serial]
    fn auto_cross_thread_posts_to_dispatcher() {
        use crate::signal::tests::install_test_dispatcher;

        let dispatcher = install_test_dispatcher();
        // Drain any leftovers from earlier tests.
        dispatcher.posted.lock().clear();

        let mut sender = Sender::new();
        // Build receiver on a different thread so its thread_id != current().
        let (tx, rx) = std::sync::mpsc::channel::<Receiver>();
        std::thread::spawn(move || {
            tx.send(Receiver::new()).unwrap();
        })
        .join()
        .unwrap();
        let receiver_from_other_thread = rx.recv().unwrap();

        let receiver = Arc::new(Mutex::new(receiver_from_other_thread));
        let counter = Arc::new(AtomicI32::new(0));
        {
            let c = Arc::clone(&counter);
            receiver
                .lock()
                .sig_b
                .connect(move |args: &(i32,)| c.store(args.0, Ordering::Relaxed));
        }
        let to: Arc<Mutex<dyn Object>> = Arc::clone(&receiver) as Arc<Mutex<dyn Object>>;
        connect_signal_to_signal(&mut sender, "sig_a", to, "sig_b", ConnectionType::Auto).unwrap();

        // Emit from THIS thread (which differs from the receiver's thread_id).
        sender.sig_a.emit_unconditionally(&(77,));
        // The callback should NOT have fired yet (it was queued).
        assert_eq!(
            counter.load(Ordering::Relaxed),
            0,
            "AC6 cross-thread Auto must not fire synchronously"
        );
        // Drain dispatcher and verify the closure was posted.
        let tasks: Vec<_> = dispatcher.posted.lock().drain(..).collect();
        assert!(!tasks.is_empty(), "AC6: at least one task must be posted");
        for task in tasks {
            task();
        }
        assert_eq!(
            counter.load(Ordering::Relaxed),
            77,
            "AC6 cross-thread Auto: value must be forwarded after drain"
        );
    }
}
