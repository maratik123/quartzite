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
/// with the first `to_arity` argument values; any extra source arguments are dropped.
/// The connection silently breaks when all strong [`Arc`] holders of `to` are released.
///
/// Arity is validated as `from_arity >= to_arity` at connection time. `type_name` strings
/// are compared on the first `to_arity` parameters; extra source arguments are dropped at
/// emit time.
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
/// - [`SignalConnectionError::UnknownFromSignal`] when `from_signal` is not declared on `from`.
/// - [`SignalConnectionError::UnknownToSignal`] when `to_signal` is not declared on `to`.
/// - [`SignalConnectionError::ArityMismatch`] when `from_arity < to_arity` (source signal
///   has fewer parameters than the target requires).
/// - [`SignalConnectionError::TypeMismatch`] when any `type_name` string differs on the
///   first `to_arity` parameters.
/// - [`SignalConnectionError::InternalError`] when the validated signal name is unexpectedly
///   rejected by [`Object::connect_signal`].
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use parking_lot::Mutex;
/// use quartzite_core::{Object, connect::{connect_signal_to_signal, SignalConnectionError}};
/// use quartzite_core::signal::ConnectionType;
/// # fn example(from: &mut impl Object, to: &Arc<Mutex<dyn Object>>) {
/// let id = connect_signal_to_signal(from, "clicked", to, "clicked", ConnectionType::Direct);
/// # }
/// ```
#[allow(
    clippy::significant_drop_tightening,
    reason = "MutexGuard held intentionally to keep critical section atomic"
)]
pub fn connect_signal_to_signal(
    from: &mut dyn Object,
    from_signal: &str,
    to: &Arc<Mutex<dyn Object>>,
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
    if from_arity < to_arity {
        return Err(SignalConnectionError::ArityMismatch {
            from: from_arity,
            to: to_arity,
        });
    }

    // Validate type names on the retained prefix (first to_arity parameters).
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
    let to_weak: Weak<Mutex<dyn Object>> = Arc::downgrade(to);

    let callback: SignalCallback = match conn_type {
        ConnectionType::Direct => Box::new(move |args: &[Value]| {
            if let Some(arc) = to_weak.upgrade() {
                let _ = arc.lock().emit_signal(&to_signal_name, &args[..to_arity]);
            }
        }),
        ConnectionType::SingleShot => Box::new(move |args: &[Value]| {
            if let Some(arc) = to_weak.upgrade() {
                let _ = arc.lock().emit_signal(&to_signal_name, &args[..to_arity]);
            }
        }),
        ConnectionType::Queued => Box::new(move |args: &[Value]| {
            let Some(arc) = to_weak.upgrade() else {
                return;
            };
            let args_owned: Vec<Value> = args[..to_arity].to_vec();
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
                let _ = arc.lock().emit_signal(&to_signal_name, &args[..to_arity]);
            } else {
                let args_owned: Vec<Value> = args[..to_arity].to_vec();
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

/// Connects a named signal on `source` to a named slot on `target`.
///
/// When `signal_name` is emitted on `source`, the registered callback upgrades a
/// `Weak<Mutex<dyn Object>>` to a strong [`Arc`], locks `target`, and calls
/// `target.invoke_method(slot_name, args)`.
///
/// **Meta-validated path** — when `target.meta_object().method(slot_name)` returns
/// `Some(meta)` at connection time: arity (`from_arity >= meta.params.len()`) and
/// `type_name` equality on the first `meta.params.len()` parameters are validated
/// eagerly. At emit time the slot is invoked with `&args[..slot_arity]` (the retained
/// prefix only; extra source arguments are dropped).
///
/// **Fallback path** — when `meta_object().method(slot_name)` returns `None` (slot is
/// not declared in the meta-system, e.g. a hand-rolled `invoke_method` implementation):
/// the connection succeeds with no further validation, and the slot is invoked with
/// `&[]` at emit time. Non-zero-arity hand-rolled slots silently no-op in this branch.
///
/// The callback holds only a `Weak<Mutex<dyn Object>>` downgraded from `target`.
/// If all strong [`Arc`] handles to `target` are released before an emission, the
/// upgrade silently fails and the slot call is skipped — no panic, no log.
///
/// # Parameters
///
/// - `source`: source object whose signal is subscribed to.
/// - `signal_name`: meta-system name of the signal on `source`.
/// - `target`: shared target object; held weakly inside the callback.
/// - `slot_name`: meta-system method name to invoke on `target` on each emission.
///
/// # Errors
///
/// - [`SignalConnectionError::UnknownFromSignal`] when `signal_name` is not declared
///   on `source`. Always validated eagerly at connection time.
/// - [`SignalConnectionError::ArityMismatch`] when `from_arity < slot_arity`. Only
///   returned when `target.meta_object().method(slot_name)` returns `Some(_)`.
/// - [`SignalConnectionError::TypeMismatch`] when any `type_name` string on the
///   retained prefix differs. Only returned when `meta_object().method(slot_name)`
///   returns `Some(_)`.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use parking_lot::Mutex;
/// use quartzite_core::{Object, connect::{connect_signal_to_slot, SignalConnectionError}};
/// # fn example(source: &mut impl Object, target: Arc<Mutex<dyn Object>>) -> Result<(), SignalConnectionError> {
/// connect_signal_to_slot(source, "click", &target, "quit")?;
/// # Ok(())
/// # }
/// ```
pub fn connect_signal_to_slot(
    source: &mut dyn Object,
    signal_name: &str,
    target: &Arc<Mutex<dyn Object>>,
    slot_name: &str,
) -> Result<ConnectionId, SignalConnectionError> {
    // Validate signal_name eagerly — must fire UnknownFromSignal before any slot lookup.
    let from_meta = source
        .meta_object()
        .signal(signal_name)
        .ok_or_else(|| SignalConnectionError::UnknownFromSignal(signal_name.into()))?;

    let slot_name_owned = slot_name.to_owned();
    let target_weak: Weak<Mutex<dyn Object>> = Arc::downgrade(target);

    // Look up slot arity from the target's meta-system (may be absent for hand-rolled objects).
    let slot_meta = target.lock().meta_object().method(slot_name);

    let callback: SignalCallback = match slot_meta {
        Some(meta) => {
            // Validated path: enforce arity and type-name compatibility at connection time.
            let from_arity = from_meta.params.len();
            let slot_arity = meta.params.len();
            if from_arity < slot_arity {
                return Err(SignalConnectionError::ArityMismatch {
                    from: from_arity,
                    to: slot_arity,
                });
            }
            // Validate type names on the retained prefix.
            for (i, (fp, sp)) in from_meta.params.iter().zip(meta.params.iter()).enumerate() {
                if fp.type_name != sp.type_name {
                    return Err(SignalConnectionError::TypeMismatch {
                        index: i,
                        from: fp.type_name.into(),
                        to: sp.type_name.into(),
                    });
                }
            }
            Box::new(move |args: &[Value]| {
                if let Some(arc) = target_weak.upgrade() {
                    let _ = arc
                        .lock()
                        .invoke_method(&slot_name_owned, &args[..slot_arity]);
                }
            })
        }
        None => {
            // Fallback path: slot not in meta-system; invoke with empty args (current behaviour).
            Box::new(move |_args: &[Value]| {
                if let Some(arc) = target_weak.upgrade() {
                    let _ = arc.lock().invoke_method(&slot_name_owned, &[]);
                }
            })
        }
    };

    source
        .connect_signal(signal_name, callback, crate::signal::ConnectionType::Direct)
        .ok_or_else(|| SignalConnectionError::InternalError(signal_name.into()))
}

/// Connects a typed signal field on `from` to a named signal on `to`.
///
/// Unlike [`connect_signal_to_signal`], this function uses the actual typed [`Signal<Args>`]
/// field for the source, avoiding the `&[Value]` round-trip on the hot path for `Auto`
/// and `Queued` connections.
///
/// Arity is validated as `from_arity >= to_arity` at connection time. `type_name` strings
/// are compared on the first `to_arity` parameters; extra source arguments are dropped at
/// emit time via `&values[..to_arity]`.
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
/// - [`SignalConnectionError::ArityMismatch`] when `from_arity < to_arity` (source signal
///   has fewer parameters than the target requires).
/// - [`SignalConnectionError::TypeMismatch`] when any `type_name` string differs on the
///   first `to_arity` parameters.
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
/// #     to: &Arc<Mutex<To>>,
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
#[allow(
    clippy::significant_drop_tightening,
    reason = "MutexGuard held intentionally to keep critical section atomic"
)]
pub fn connect_signals<From, To, Args>(
    from_obj: &mut From,
    from_signal_name: &str,
    from_signal_field: fn(&mut From) -> &mut Signal<Args>,
    to: &Arc<Mutex<To>>,
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
    if from_arity < to_arity {
        return Err(SignalConnectionError::ArityMismatch {
            from: from_arity,
            to: to_arity,
        });
    }

    // Type-name check on the retained prefix (first to_arity parameters).
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
            let to_weak = Arc::downgrade(to);
            let ct = conn_type;
            sig.connect_typed(
                move |args: &Args| {
                    let Some(arc) = to_weak.upgrade() else {
                        return;
                    };
                    let values = args.to_values();
                    let _ = arc.lock().emit_signal(&to_signal_str, &values[..to_arity]);
                },
                ct,
            )
        }
        ConnectionType::Queued => {
            let (to_weak, guard_weak, thread_id) = {
                let guard = to.lock();
                let w: Weak<ReceiverGuard> = Arc::downgrade(guard.object_base().receiver_guard());
                let tid = guard.object_base().thread_id;
                (Arc::downgrade(to), w, tid)
            };
            sig.connect_queued(
                thread_id,
                move |args: Args| {
                    let Some(arc) = to_weak.upgrade() else {
                        return;
                    };
                    let values = args.to_values();
                    let _ = arc.lock().emit_signal(&to_signal_str, &values[..to_arity]);
                },
                guard_weak,
            )
        }
        ConnectionType::Auto => {
            let (to_weak, guard_weak, thread_id) = {
                let guard = to.lock();
                let w: Weak<ReceiverGuard> = Arc::downgrade(guard.object_base().receiver_guard());
                let tid = guard.object_base().thread_id;
                (Arc::downgrade(to), w, tid)
            };
            sig.connect_auto(thread_id, guard_weak, move |args: Args| {
                let Some(arc) = to_weak.upgrade() else {
                    return;
                };
                let values = args.to_values();
                let _ = arc.lock().emit_signal(&to_signal_str, &values[..to_arity]);
            })
        }
    };
    Ok(id)
}

/// Tests that require access to `crate::signal::tests` (a `#[cfg(test)]`-private
/// dispatcher helper) live here. All other connect tests are in
/// `quartzite-core/tests/connect.rs` (integration tests) to keep this file
/// within the 1000-line production / 1500-line total hard limit.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        id::ConnectionId,
        meta::MetaObject,
        object_base::ObjectBase,
        signal::{ConnectionType, Signal},
        value::Value,
    };
    use parking_lot::Mutex;
    use std::sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    };

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

    // AC6 — Auto cross-thread: callback is posted to the queued dispatcher.
    #[test]
    #[cfg(feature = "std")]
    #[allow(
        clippy::items_after_statements,
        reason = "nested helper placed after local setup is more readable here"
    )]
    fn auto_cross_thread_posts_to_dispatcher() {
        let _lock = quartzite_test_helpers::test_lock();
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
        connect_signal_to_signal(&mut sender, "sig_a", &to, "sig_b", ConnectionType::Auto).unwrap();

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
