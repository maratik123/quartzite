//! Typed signals with multiple connection modes (`Direct`, `SingleShot`, `Queued`, `Auto`).
// With `std` feature: IndexMap<K, V> uses RandomState (two-param form available).
// Without `std`: no default hasher type param; use hashbrown's DefaultHashBuilder explicitly.
#[cfg(feature = "std")]
use indexmap::IndexMap;
#[cfg(not(feature = "std"))]
type IndexMap<K, V> = indexmap::IndexMap<K, V, hashbrown::DefaultHashBuilder>;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use crate::id::ConnectionId;
#[cfg(feature = "std")]
use crate::receiver_guard::ReceiverGuard;
#[cfg(feature = "std")]
use std::sync::Arc;
use tracing::trace;

/// Determines how a slot is invoked when a signal is emitted.
///
/// `Queued` and `Auto` (cross-thread delivery) require the `std` feature and an
/// active `QueuedDispatcher` registered via `set_queued_dispatcher`.
///
/// # Examples
///
/// ```
/// use quartzite_core::signal::{ConnectionType, Signal};
///
/// let mut sig: Signal<()> = Signal::new();
/// sig.connect_typed(|_| {}, ConnectionType::Direct);
/// sig.emit_unconditionally(&());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnectionType {
    /// Invoke the slot immediately in the emitting call stack.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::signal::{ConnectionType, Signal};
    ///
    /// let mut sig: Signal<(i32,)> = Signal::new();
    /// sig.connect_typed(|args| println!("direct: {}", args.0), ConnectionType::Direct);
    /// sig.emit_unconditionally(&(42,));
    /// ```
    Direct,
    /// Invoke the slot exactly once, then automatically disconnect.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::signal::{ConnectionType, Signal};
    ///
    /// let mut sig: Signal<(i32,)> = Signal::new();
    /// sig.connect_typed(|_| {}, ConnectionType::SingleShot);
    /// sig.emit_unconditionally(&(1,)); // fires once
    /// sig.emit_unconditionally(&(2,)); // slot already disconnected; no-op
    /// ```
    SingleShot,
    /// Post the slot to the event loop; requires `std` and an active dispatcher.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    Queued,
    /// Same-thread → `Direct`; cross-thread → `Queued`. Requires `std`.
    ///
    /// The receiver's thread identity is captured at connect time via
    /// [`Signal::connect_auto`]. Changing the receiver's thread affinity after
    /// connecting does not update the stored `ThreadId` (see AC5).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    Auto,
}

/// Internal slot record stored inside a `Signal`.
/// Callbacks require `Send` so that `Signal<Args>` can be wrapped in
/// `Arc<Mutex<Signal<Args>>>` for cross-thread posting (e.g. `Timer`).
struct SlotEntry<Args: 'static> {
    callback: Box<dyn Fn(&Args) + Send>,
    conn_type: ConnectionType,
}

/// Receives closures posted by queued signal connections and executes them on
/// the event-loop thread. Implemented by `ConnectionTable` in `quartzite-runtime`.
///
/// # Examples
///
/// ```
/// use quartzite_core::signal::QueuedDispatcher;
///
/// struct ImmediateDispatcher;
/// impl QueuedDispatcher for ImmediateDispatcher {
///     fn post(&self, f: Box<dyn FnOnce() + Send + 'static>) { f(); }
/// }
/// ```
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub trait QueuedDispatcher: Send + Sync {
    /// Posts a closure to be executed on the event-loop thread.
    ///
    /// # Parameters
    ///
    /// - `f`: boxed closure to enqueue; ownership transfers to the dispatcher.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::signal::QueuedDispatcher;
    ///
    /// struct ImmediateDispatcher;
    /// impl QueuedDispatcher for ImmediateDispatcher {
    ///     fn post(&self, f: Box<dyn FnOnce() + Send + 'static>) { f(); }
    /// }
    ///
    /// let d = ImmediateDispatcher;
    /// d.post(Box::new(|| { let _ = 1 + 1; }));
    /// ```
    fn post(&self, f: Box<dyn FnOnce() + Send + 'static>);
}

#[cfg(feature = "std")]
static QUEUED_DISPATCHER: std::sync::OnceLock<Arc<dyn QueuedDispatcher>> =
    std::sync::OnceLock::new();

/// Error returned by `set_queued_dispatcher` when a dispatcher is already registered.
///
/// # Examples
///
/// ```
/// use quartzite_core::signal::DispatcherAlreadySet;
///
/// let err = DispatcherAlreadySet;
/// assert_eq!(format!("{err}"), "queued dispatcher is already installed");
/// ```
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("queued dispatcher is already installed")]
pub struct DispatcherAlreadySet;

/// Registers the process-wide queued dispatcher.
///
/// Normally called once by `Application::new()`. Returns `Ok(())` on the first
/// call; `Err(DispatcherAlreadySet)` on subsequent calls (the existing
/// dispatcher remains installed).
///
/// # Parameters
///
/// - `d`: dispatcher to install; must be `Send + Sync` so it can be invoked
///   from any signal-emitting thread.
///
/// # Errors
///
/// Returns `DispatcherAlreadySet` if a dispatcher has already been installed
/// in the current process. The slot can be set exactly once for the lifetime
/// of the process (backed by [`std::sync::OnceLock`]).
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use quartzite_core::signal::{QueuedDispatcher, set_queued_dispatcher};
///
/// // Normally called by Application::new(); shown here for illustration.
/// // set_queued_dispatcher(Arc::new(my_dispatcher));
/// ```
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub fn set_queued_dispatcher(d: Arc<dyn QueuedDispatcher>) -> Result<(), DispatcherAlreadySet> {
    QUEUED_DISPATCHER.set(d).map_err(|_| DispatcherAlreadySet)
}

/// Returns a reference to the registered dispatcher, or `None` before
/// `Application` has been created.
///
/// # Examples
///
/// ```no_run
/// use quartzite_core::signal::queued_dispatcher;
///
/// if let Some(d) = queued_dispatcher() {
///     d.post(Box::new(|| println!("posted to event loop")));
/// }
/// ```
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[inline]
pub fn queued_dispatcher() -> Option<&'static Arc<dyn QueuedDispatcher>> {
    QUEUED_DISPATCHER.get()
}

/// Object-safe interface for a queued slot stored inside a `Signal<Args>`.
#[cfg(feature = "std")]
trait DynQueuedSlot<Args: 'static>: Send + Sync {
    fn post_if_alive(&self, args: &Args);
}

/// Concrete queued-slot entry. `Args: Clone + Send` is required to package
/// the arguments into a cross-thread closure.
#[cfg(feature = "std")]
struct QueuedSlotInner<Args: Clone + Send + 'static> {
    callback: Arc<dyn Fn(Args) + Send + Sync>,
    guard: std::sync::Weak<ReceiverGuard>,
}

#[cfg(feature = "std")]
impl<Args: Clone + Send + 'static> DynQueuedSlot<Args> for QueuedSlotInner<Args> {
    fn post_if_alive(&self, args: &Args) {
        if self.guard.upgrade().is_none() {
            return;
        }
        let args_owned = args.clone();
        let cb = Arc::clone(&self.callback);
        if let Some(dispatcher) = queued_dispatcher() {
            dispatcher.post(Box::new(move || cb(args_owned)));
        }
    }
}

/// Object-safe interface for an `Auto`-connection slot stored inside a `Signal<Args>`.
#[cfg(feature = "std")]
trait DynAutoSlot<Args: 'static>: Send + Sync {
    /// Dispatches the slot: same-thread → direct call; cross-thread → post to dispatcher.
    fn dispatch(&self, emit_thread_id: std::thread::ThreadId, args: &Args);
}

/// Concrete `Auto`-connection slot entry.
///
/// `Args: Clone + Send + 'static` is required: the cross-thread path clones args
/// and moves them into a `'static` closure posted to the dispatcher.
#[cfg(feature = "std")]
struct AutoSlotInner<Args: Clone + Send + 'static> {
    receiver_thread_id: std::thread::ThreadId,
    guard: std::sync::Weak<ReceiverGuard>,
    callback: Arc<dyn Fn(Args) + Send + Sync>,
}

#[cfg(feature = "std")]
impl<Args: Clone + Send + 'static> DynAutoSlot<Args> for AutoSlotInner<Args> {
    fn dispatch(&self, emit_thread_id: std::thread::ThreadId, args: &Args) {
        if self.guard.upgrade().is_none() {
            return;
        }
        if emit_thread_id == self.receiver_thread_id {
            (self.callback)(args.clone());
        } else if let Some(dispatcher) = queued_dispatcher() {
            let args_owned = args.clone();
            let cb = Arc::clone(&self.callback);
            dispatcher.post(Box::new(move || cb(args_owned)));
        }
        // No dispatcher installed → silent drop (AC3).
    }
}

/// A typed signal that can have multiple slots connected.
///
/// `Args` is typically a tuple (e.g. `Signal<(i32, bool)>`). Slots receive a
/// shared reference to the args tuple.
///
/// `emit` takes `&mut self` to allow `SingleShot` cleanup without interior
/// mutability. This means a slot cannot call `emit` on the *same* signal
/// instance during emission (the borrow checker prevents it). Re-entrant
/// emission across *different* signals is fine.
///
/// Slot callbacks require `Send` so that `Signal<Args>` can be safely wrapped
/// in `Arc<Mutex<Signal<Args>>>` for cross-thread emission (e.g. `Timer`).
/// The signal itself is not `Sync` — concurrent emission is not supported.
///
/// Slots are stored in an `IndexMap` keyed by `ConnectionId`, preserving
/// insertion order for deterministic emission while providing O(1) disconnect.
///
/// # Examples
///
/// ```
/// use quartzite_core::signal::Signal;
///
/// let mut sig: Signal<(i32,)> = Signal::new();
/// let id = sig.connect(|args| println!("got {}", args.0));
/// sig.emit_unconditionally(&(7,));
/// sig.disconnect(id);
/// ```
pub struct Signal<Args: 'static> {
    slots: IndexMap<ConnectionId, SlotEntry<Args>>,
    #[cfg(feature = "std")]
    queued_slots: IndexMap<ConnectionId, Box<dyn DynQueuedSlot<Args>>>,
    #[cfg(feature = "std")]
    auto_slots: IndexMap<ConnectionId, Box<dyn DynAutoSlot<Args>>>,
}

impl<Args: 'static> core::fmt::Debug for Signal<Args> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Signal")
            .field("connections", &self.slots.len())
            .finish_non_exhaustive()
    }
}

impl<Args: 'static> Default for Signal<Args> {
    fn default() -> Self {
        Signal {
            #[cfg(feature = "std")]
            slots: IndexMap::new(),
            #[cfg(not(feature = "std"))]
            slots: indexmap::IndexMap::with_hasher(hashbrown::DefaultHashBuilder::default()),
            #[cfg(feature = "std")]
            queued_slots: IndexMap::new(),
            #[cfg(feature = "std")]
            auto_slots: IndexMap::new(),
        }
    }
}

impl<Args: 'static> Signal<Args> {
    /// Creates a new signal with no slots connected.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::signal::Signal;
    ///
    /// let mut sig: Signal<(i32,)> = Signal::new();
    /// sig.connect(|args| println!("received {}", args.0));
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Connects a `Direct` slot and returns its `ConnectionId` for later disconnect.
    ///
    /// # Parameters
    ///
    /// - `f`: slot callback invoked with a shared reference to the args tuple
    ///   each time the signal is emitted; must be `Send` and `'static`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::signal::Signal;
    ///
    /// let mut sig: Signal<(i32,)> = Signal::new();
    /// let id = sig.connect(|args| println!("value = {}", args.0));
    /// sig.disconnect(id);
    /// ```
    #[inline]
    pub fn connect<F: Fn(&Args) + Send + 'static>(&mut self, f: F) -> ConnectionId {
        self.connect_typed(f, ConnectionType::Direct)
    }

    /// Connects a slot with an explicit `ConnectionType`.
    ///
    /// Only `Direct` and `SingleShot` are valid here. Use [`connect_queued`](Self::connect_queued)
    /// for `Queued` and [`connect_auto`](Self::connect_auto) for `Auto` — both require additional
    /// parameters (`ReceiverGuard` / `ThreadId`) that this method cannot accept.
    ///
    /// # Parameters
    ///
    /// - `f`: slot callback; invoked with a shared reference to the args tuple
    ///   each time the signal is emitted.
    /// - `ct`: which connection type to install; must be `Direct` or `SingleShot`.
    ///
    /// # Panics
    ///
    /// In `debug` builds, panics via `debug_assert!` when `ct` is `Queued` or
    /// `Auto`. Release builds silently install the slot but it will never fire
    /// because the queued/auto delivery paths read from different storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::signal::{ConnectionType, Signal};
    ///
    /// let mut sig: Signal<(i32,)> = Signal::new();
    /// let id = sig.connect_typed(|args| println!("{}", args.0), ConnectionType::SingleShot);
    /// sig.emit_unconditionally(&(1,)); // fires once, then disconnects
    /// sig.disconnect(id); // safe to call even after auto-disconnect
    /// ```
    pub fn connect_typed<F: Fn(&Args) + Send + 'static>(
        &mut self,
        f: F,
        ct: ConnectionType,
    ) -> ConnectionId {
        #[cfg(feature = "std")]
        debug_assert!(
            ct != ConnectionType::Queued && ct != ConnectionType::Auto,
            "connect_typed does not support Queued or Auto; use connect_queued / connect_auto"
        );
        let id = ConnectionId::new();
        self.slots.insert(
            id,
            SlotEntry {
                callback: Box::new(f),
                conn_type: ct,
            },
        );
        trace!(conn_id = ?id, ?ct, "signal connected");
        id
    }

    /// Connects a `Queued` slot that is invoked on the event-loop thread.
    ///
    /// The receiver guard is checked before posting: if the guard has expired
    /// (receiver destroyed), the closure is silently discarded. Requires
    /// `Args: Clone + Send` so the arguments can be moved across threads.
    ///
    /// # Parameters
    ///
    /// - `f`: slot callback; invoked on the dispatcher thread with an owned
    ///   clone of the emit-time args.
    /// - `guard`: weak handle to the receiver's [`ReceiverGuard`]; used to
    ///   short-circuit posting once the receiver has been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::{receiver_guard::ReceiverGuard, signal::Signal};
    ///
    /// let (guard_arc, guard_weak) = ReceiverGuard::new_pair();
    /// let mut sig: Signal<(i32,)> = Signal::new();
    /// let _id = sig.connect_queued(|args: (i32,)| println!("queued: {}", args.0), guard_weak);
    /// drop(guard_arc); // receiver destroyed; subsequent emits silently skip this slot
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn connect_queued<F>(&mut self, f: F, guard: std::sync::Weak<ReceiverGuard>) -> ConnectionId
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: Clone + Send,
    {
        let id = ConnectionId::new();
        self.queued_slots.insert(
            id,
            Box::new(QueuedSlotInner {
                callback: Arc::new(f),
                guard,
            }),
        );
        trace!(conn_id = ?id, "signal connected (queued)");
        id
    }

    /// Connects an `Auto` slot whose dispatch route is decided at emit time.
    ///
    /// At emit time the emitting thread's id is compared against
    /// `receiver_thread_id` (captured here). If they match the slot is called
    /// synchronously (`Direct` semantics). Otherwise the invocation is posted to
    /// the active `QueuedDispatcher`; if none is installed the invocation is
    /// silently dropped.
    ///
    /// The `guard` is checked before every invocation. If the `Weak` cannot be
    /// upgraded (receiver has been destroyed), the slot is silently skipped on
    /// both the same-thread and cross-thread paths. Pass
    /// `Arc::downgrade(base.receiver_guard())` from the receiver's `ObjectBase`.
    ///
    /// `Args` must implement `Clone + Send + 'static` because the cross-thread
    /// path moves a clone of the args into a `'static` closure. Use
    /// [`connect`](Self::connect) for zero-copy same-thread-only dispatch.
    ///
    /// The `receiver_thread_id` is captured at connect time and is not updated
    /// if the receiver migrates to a different thread later.
    ///
    /// # Parameters
    ///
    /// - `receiver_thread_id`: the receiver's owning thread; captured once and
    ///   not refreshed.
    /// - `guard`: weak handle to the receiver's [`ReceiverGuard`]; expired
    ///   guards short-circuit dispatch on both the same- and cross-thread paths.
    /// - `f`: slot callback; receives an owned clone of the args tuple.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Weak, thread};
    /// use quartzite_core::signal::Signal;
    ///
    /// let mut sig: Signal<(i32,)> = Signal::new();
    /// // Weak::new() stands in for a real receiver guard here.
    /// let _id = sig.connect_auto(thread::current().id(), Weak::new(), |args: (i32,)| println!("auto: {}", args.0));
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn connect_auto<F>(
        &mut self,
        receiver_thread_id: std::thread::ThreadId,
        guard: std::sync::Weak<ReceiverGuard>,
        f: F,
    ) -> ConnectionId
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: Clone + Send,
    {
        let id = ConnectionId::new();
        self.auto_slots.insert(
            id,
            Box::new(AutoSlotInner {
                receiver_thread_id,
                guard,
                callback: Arc::new(f),
            }),
        );
        trace!(conn_id = ?id, "signal connected (auto)");
        id
    }

    /// Invokes all connected slots with `args` unconditionally, bypassing the `signals_blocked` guard.
    ///
    /// Prefer the [`crate::emit!`] macro for object-owned signals — it reads
    /// `signals_blocked()` and skips emission automatically. Generated
    /// `emit_<signal>` wrappers (from `#[derive(Object)]`) call the macro internally.
    /// Use this method directly only when the caller has already checked the blocked
    /// state, or the signal has no owning `AsObject` (e.g. `Arc<Mutex<Signal>>`).
    ///
    /// `SingleShot` slots are called once and then removed in-place.
    /// `Queued` slots are posted to the event-loop thread via the registered
    /// `QueuedDispatcher` (if any).
    /// `Auto` slots inspect the emitting thread: same-thread → direct call;
    /// cross-thread → posted to the dispatcher (silently dropped if none installed).
    ///
    /// Because `emit_unconditionally` takes `&mut self`, no slot can call `connect`,
    /// `disconnect`, or `emit_unconditionally` on the *same* signal instance during
    /// emission — the borrow checker prevents it. Cross-signal mutation from within
    /// a slot is fine.
    ///
    /// # Parameters
    ///
    /// - `args`: arguments forwarded to every slot as a shared reference;
    ///   cloned per slot only on the queued/auto-cross-thread paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::signal::Signal;
    ///
    /// let mut sig: Signal<(i32,)> = Signal::new();
    /// sig.connect(|_| {});
    /// sig.emit_unconditionally(&(42,));
    /// ```
    pub fn emit_unconditionally(&mut self, args: &Args) {
        trace!(direct_slots = self.slots.len(), "signal emit");
        for entry in self.slots.values() {
            (entry.callback)(args);
        }
        self.slots
            .retain(|_, e| e.conn_type != ConnectionType::SingleShot);
        #[cfg(feature = "std")]
        {
            for slot in self.queued_slots.values() {
                slot.post_if_alive(args);
            }
            if !self.auto_slots.is_empty() {
                let emit_thread_id = std::thread::current().id();
                for slot in self.auto_slots.values() {
                    slot.dispatch(emit_thread_id, args);
                }
            }
        }
    }

    /// Removes the slot identified by `id`; a no-op if no matching slot exists.
    ///
    /// Runs in O(1) via `IndexMap::shift_remove`, preserving insertion order
    /// for all remaining slots.
    ///
    /// # Parameters
    ///
    /// - `id`: the connection identifier returned by a previous `connect*` call.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::signal::Signal;
    ///
    /// let mut sig: Signal<()> = Signal::new();
    /// let id = sig.connect(|_| {});
    /// sig.disconnect(id);
    /// sig.emit_unconditionally(&()); // slot no longer called
    /// ```
    pub fn disconnect(&mut self, id: ConnectionId) {
        trace!(conn_id = ?id, "signal disconnected");
        self.slots.shift_remove(&id);
        #[cfg(feature = "std")]
        self.queued_slots.shift_remove(&id);
        #[cfg(feature = "std")]
        self.auto_slots.shift_remove(&id);
    }
}

/// Emits `$field` on `$receiver` unless signals are blocked.
///
/// Extracts `signals_blocked()` from `$receiver` via [`crate::AsObject::object_base`],
/// then calls [`Signal::emit_unconditionally`] only when not blocked. The blocked flag is bound
/// to a local before the signal borrow begins, satisfying the borrow checker.
///
/// `$receiver` must implement [`crate::AsObject`]. `$field` must be a [`Signal`] field
/// directly on `$receiver` (single-level path only).
///
/// For signals behind `Arc<Mutex<Signal<…>>>` (e.g. `Timer::tick`), use an
/// explicit `if !blocked { sig.emit_unconditionally(…) }` guard instead.
///
/// # Parameters
///
/// - `$receiver`: an expression of a type implementing [`crate::AsObject`].
/// - `$field`: the name of a [`Signal`] field on `$receiver`.
/// - `$args`: the args tuple reference forwarded to [`Signal::emit_unconditionally`].
///
/// # Examples
///
/// ```
/// use quartzite_core::{
///     AsObject, ObjectBase,
///     emit,
///     signal::Signal,
/// };
///
/// #[derive(Default)]
/// struct Counter {
///     base: ObjectBase,
///     ticked: Signal<(u32,)>,
/// }
/// impl AsObject for Counter {
///     fn object_base(&self) -> &ObjectBase { &self.base }
///     fn object_base_mut(&mut self) -> &mut ObjectBase { &mut self.base }
///     fn as_any(&self) -> &dyn core::any::Any { self }
///     fn as_any_mut(&mut self) -> &mut dyn core::any::Any { self }
/// }
///
/// let mut c = Counter::default();
/// c.ticked.connect(|args| println!("tick {}", args.0));
/// emit!(c.ticked, &(1,)); // fires — not blocked
/// c.base.block_signals();
/// emit!(c.ticked, &(2,)); // suppressed — blocked
/// ```
#[macro_export]
macro_rules! emit {
    ($receiver:ident . $field:ident, $args:expr) => {{
        // Brings AsObject into scope for method call; caller may already have it imported.
        #[allow(unused_imports)]
        use $crate::AsObject as _;
        let __blocked = $receiver.object_base().signals_blocked();
        if !__blocked {
            $receiver.$field.emit_unconditionally($args);
        }
    }};
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use serial_test::serial;
    #[cfg(feature = "std")]
    use std::sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering},
    };

    // ---------------------------------------------------------------------------
    // Shared test infrastructure for QUEUED_DISPATCHER
    // ---------------------------------------------------------------------------

    /// A `QueuedDispatcher` stub that stores posted closures for inspection.
    /// All auto/queued tests that need a dispatcher call `install_test_dispatcher()`.
    #[cfg(feature = "std")]
    pub(crate) struct TestDispatcher {
        pub(crate) posted: Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>,
    }

    #[cfg(feature = "std")]
    impl QueuedDispatcher for TestDispatcher {
        fn post(&self, f: Box<dyn FnOnce() + Send + 'static>) {
            self.posted.lock().unwrap().push(f);
        }
    }

    /// Process-wide singleton. Set exactly once; reused by all tests in this binary.
    #[cfg(feature = "std")]
    pub(crate) static TEST_DISPATCHER: OnceLock<Arc<TestDispatcher>> = OnceLock::new();

    /// Install the shared `TestDispatcher` as the process-wide `QueuedDispatcher`.
    /// Safe to call from multiple tests in the same binary: only the first call
    /// registers the dispatcher; subsequent calls are no-ops.
    /// Returns the shared `Arc<TestDispatcher>` so callers can drain `posted`.
    #[cfg(feature = "std")]
    pub(crate) fn install_test_dispatcher() -> Arc<TestDispatcher> {
        Arc::clone(TEST_DISPATCHER.get_or_init(|| {
            let d = Arc::new(TestDispatcher {
                posted: Mutex::new(vec![]),
            });
            let _ = set_queued_dispatcher(Arc::clone(&d) as Arc<dyn QueuedDispatcher>);
            d
        }))
    }

    /// Spawn a helper thread, capture its `ThreadId`, join, and return it.
    /// Guaranteed to differ from `thread::current().id()`.
    #[cfg(feature = "std")]
    fn other_thread_id() -> std::thread::ThreadId {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            tx.send(std::thread::current().id()).unwrap();
        })
        .join()
        .unwrap();
        rx.recv().unwrap()
    }

    // ---------------------------------------------------------------------------
    // AC4 (core-types): emit calls all connected Direct slots
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    fn emit_calls_connected_direct_slots() {
        let mut sig: Signal<(i32,)> = Signal::new();
        let a = Arc::new(AtomicI32::new(0));
        let b = Arc::new(AtomicI32::new(0));

        let a2 = Arc::clone(&a);
        let b2 = Arc::clone(&b);

        sig.connect(move |args| a2.store(args.0, Ordering::Relaxed));
        sig.connect(move |args| b2.store(args.0, Ordering::Relaxed));

        sig.emit_unconditionally(&(42,));

        assert_eq!(a.load(Ordering::Relaxed), 42);
        assert_eq!(b.load(Ordering::Relaxed), 42);
    }

    // ---------------------------------------------------------------------------
    // AC5 (core-types): disconnect removes slot
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    fn disconnect_removes_slot() {
        let mut sig: Signal<(i32,)> = Signal::new();
        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);

        let id = sig.connect(move |_args| called2.store(true, Ordering::Relaxed));
        sig.disconnect(id);
        sig.emit_unconditionally(&(1,));

        assert!(
            !called.load(Ordering::Relaxed),
            "disconnected slot must not be called"
        );
    }

    // ---------------------------------------------------------------------------
    // AC1 (lookup-perf): disconnect is O(1) and preserves emission order
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    fn disconnect_is_o1_and_preserves_order() {
        let mut sig: Signal<()> = Signal::new();
        let log: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

        let l1 = Arc::clone(&log);
        let l2 = Arc::clone(&log);
        let l3 = Arc::clone(&log);

        sig.connect(move |_| l1.lock().unwrap().push(1)); // slot A
        let id_b = sig.connect(move |_| l2.lock().unwrap().push(2)); // slot B
        sig.connect(move |_| l3.lock().unwrap().push(3)); // slot C

        sig.disconnect(id_b);
        sig.emit_unconditionally(&());

        assert_eq!(
            *log.lock().unwrap(),
            vec![1, 3],
            "A and C must fire in insertion order; B must be absent"
        );
    }

    // ---------------------------------------------------------------------------
    // AC2 (lookup-perf): SingleShot two-pass retain
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    fn single_shot_removed_by_retain() {
        // Augments single_shot_called_once — specifically verifies the two-pass retain path:
        // connects one SingleShot and one Direct; emits twice; SingleShot fires once, Direct twice.
        let mut sig: Signal<()> = Signal::new();
        let ss_count = Arc::new(AtomicU32::new(0));
        let direct_count = Arc::new(AtomicU32::new(0));

        let ss2 = Arc::clone(&ss_count);
        let d2 = Arc::clone(&direct_count);

        sig.connect_typed(
            move |_| {
                ss2.fetch_add(1, Ordering::Relaxed);
            },
            ConnectionType::SingleShot,
        );
        sig.connect(move |_| {
            d2.fetch_add(1, Ordering::Relaxed);
        });

        sig.emit_unconditionally(&());
        sig.emit_unconditionally(&());

        assert_eq!(
            ss_count.load(Ordering::Relaxed),
            1,
            "SingleShot must fire exactly once"
        );
        assert_eq!(
            direct_count.load(Ordering::Relaxed),
            2,
            "Direct must fire on both emits"
        );
    }

    // ---------------------------------------------------------------------------
    // AC6 (core-types): SingleShot called exactly once
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    fn single_shot_called_once() {
        let mut sig: Signal<()> = Signal::new();
        let count = Arc::new(AtomicU32::new(0));
        let count2 = Arc::clone(&count);

        sig.connect_typed(
            move |_| {
                count2.fetch_add(1, Ordering::Relaxed);
            },
            ConnectionType::SingleShot,
        );

        sig.emit_unconditionally(&());
        sig.emit_unconditionally(&());
        sig.emit_unconditionally(&());

        assert_eq!(
            count.load(Ordering::Relaxed),
            1,
            "SingleShot slot must fire exactly once"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn emit_single_shot_fires_once() {
        let count = Arc::new(AtomicU32::new(0));
        let count2 = Arc::clone(&count);
        let mut sig: Signal<(i32,)> = Signal::new();
        sig.connect_typed(
            move |_| {
                count2.fetch_add(1, Ordering::Relaxed);
            },
            ConnectionType::SingleShot,
        );
        sig.emit_unconditionally(&(1,));
        sig.emit_unconditionally(&(2,)); // slot already removed — must not fire again
        assert_eq!(
            count.load(Ordering::Relaxed),
            1,
            "SingleShot must fire exactly once"
        );
    }

    // --- emit on empty signal must not panic ---

    #[test]
    fn emit_with_no_slots_does_not_panic() {
        let mut sig: Signal<(bool,)> = Signal::new();
        sig.emit_unconditionally(&(true,)); // must not panic
    }

    #[test]
    #[cfg(feature = "std")]
    fn multiple_disconnects_do_not_panic() {
        let mut sig: Signal<()> = Signal::new();
        let id = sig.connect(|_| {});
        sig.disconnect(id);
        sig.disconnect(id); // second disconnect must be a no-op
        sig.emit_unconditionally(&());
    }

    #[test]
    #[cfg(feature = "std")]
    fn all_pre_connected_direct_slots_fire() {
        let mut sig: Signal<()> = Signal::new();
        let count = Arc::new(AtomicU32::new(0));
        let c1 = Arc::clone(&count);
        let c2 = Arc::clone(&count);

        sig.connect(move |_| {
            c1.fetch_add(1, Ordering::Relaxed);
        });
        sig.connect(move |_| {
            c2.fetch_add(1, Ordering::Relaxed);
        });
        sig.emit_unconditionally(&());

        assert_eq!(count.load(Ordering::Relaxed), 2);
    }

    #[test]
    #[cfg(feature = "std")]
    fn disconnect_nonexistent_id_is_noop() {
        let mut sig: Signal<()> = Signal::new();
        let fake_id = ConnectionId::new();
        sig.disconnect(fake_id); // must not panic
        sig.emit_unconditionally(&()); // must not panic
    }

    // ---------------------------------------------------------------------------
    // Queued slot: guard check (uses shared TestDispatcher)
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    #[serial]
    fn queued_slot_not_posted_after_receiver_destroyed() {
        use crate::receiver_guard::ReceiverGuard;

        let dispatcher = install_test_dispatcher();
        let pre_len = dispatcher.posted.lock().unwrap().len();

        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();

        let mut sig: Signal<i32> = Signal::new();
        let posted_values: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
        let pv = Arc::clone(&posted_values);
        sig.connect_queued(move |v| pv.lock().unwrap().push(v), guard_weak.clone());

        // Drop receiver guard to invalidate.
        drop(guard_arc);

        // Emit after receiver is destroyed — must NOT post.
        sig.emit_unconditionally(&99);
        assert_eq!(
            dispatcher.posted.lock().unwrap().len(),
            pre_len,
            "no post must occur after receiver is destroyed"
        );
    }

    // ---------------------------------------------------------------------------
    // AC1: Auto same-thread → slot called synchronously
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    #[serial]
    fn auto_same_thread_calls_slot_synchronously() {
        use crate::receiver_guard::ReceiverGuard;

        let dispatcher = install_test_dispatcher();
        let pre_len = dispatcher.posted.lock().unwrap().len();

        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);

        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();
        let mut sig: Signal<(i32,)> = Signal::new();
        sig.connect_auto(std::thread::current().id(), guard_weak, move |_| {
            called2.store(true, Ordering::SeqCst);
        });
        let _guard = guard_arc;

        sig.emit_unconditionally(&(1,));

        assert!(
            called.load(Ordering::SeqCst),
            "Auto same-thread slot must be called synchronously before emit returns"
        );
        assert_eq!(
            dispatcher.posted.lock().unwrap().len(),
            pre_len,
            "Auto same-thread must not post to dispatcher"
        );
    }

    // ---------------------------------------------------------------------------
    // AC2: Auto cross-thread → posts to dispatcher, not called directly
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    #[serial]
    fn auto_cross_thread_posts_to_dispatcher() {
        use crate::receiver_guard::ReceiverGuard;

        let dispatcher = install_test_dispatcher();
        let pre_len = dispatcher.posted.lock().unwrap().len();

        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);

        let foreign_id = other_thread_id();

        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();
        let mut sig: Signal<(i32,)> = Signal::new();
        sig.connect_auto(foreign_id, guard_weak, move |_| {
            called2.store(true, Ordering::SeqCst);
        });
        let _guard = guard_arc;

        sig.emit_unconditionally(&(42,));

        // Slot must NOT have been called directly during emit.
        assert!(
            !called.load(Ordering::SeqCst),
            "Auto cross-thread slot must NOT be called directly during emit"
        );

        // Drain only entries added by our emit; leave any foreign entries untouched.
        let posted: Vec<_> = dispatcher.posted.lock().unwrap().drain(pre_len..).collect();
        assert_eq!(posted.len(), 1, "exactly one closure must be posted");
        posted.into_iter().for_each(|f| f());

        assert!(
            called.load(Ordering::SeqCst),
            "slot must run when the posted closure is executed"
        );
    }

    // ---------------------------------------------------------------------------
    // AC5 Scenario A: thread_id captured at connect; same-thread path calls directly
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    #[serial]
    fn auto_thread_id_same_thread_calls_directly() {
        use crate::receiver_guard::ReceiverGuard;

        let dispatcher = install_test_dispatcher();
        let pre_len = dispatcher.posted.lock().unwrap().len();

        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);

        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();
        let mut sig: Signal<()> = Signal::new();
        sig.connect_auto(std::thread::current().id(), guard_weak, move |_| {
            called2.store(true, Ordering::SeqCst);
        });
        let _guard = guard_arc;

        sig.emit_unconditionally(&());

        assert!(
            called.load(Ordering::SeqCst),
            "same-thread Auto slot must be called directly"
        );
        assert_eq!(
            dispatcher.posted.lock().unwrap().len(),
            pre_len,
            "no closure must be posted for same-thread Auto"
        );
    }

    // ---------------------------------------------------------------------------
    // AC5 Scenario B: foreign thread_id → always posts, even from the connect thread
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    #[serial]
    fn auto_thread_id_foreign_thread_posts_to_dispatcher() {
        use crate::receiver_guard::ReceiverGuard;

        let dispatcher = install_test_dispatcher();
        let pre_len = dispatcher.posted.lock().unwrap().len();

        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);

        let foreign_id = other_thread_id();

        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();
        let mut sig: Signal<()> = Signal::new();
        sig.connect_auto(foreign_id, guard_weak, move |_| {
            called2.store(true, Ordering::SeqCst);
        });
        let _guard = guard_arc;

        sig.emit_unconditionally(&());

        // Dispatch is governed by the foreign receiver_thread_id, not the emitting thread.
        assert!(
            !called.load(Ordering::SeqCst),
            "foreign-thread-id Auto slot must NOT be called directly"
        );

        // Drain only entries added by our emit; leave any foreign entries untouched.
        let posted: Vec<_> = dispatcher.posted.lock().unwrap().drain(pre_len..).collect();
        assert_eq!(posted.len(), 1, "exactly one closure must be posted");
        posted.into_iter().for_each(|f| f());

        assert!(
            called.load(Ordering::SeqCst),
            "slot must run when the posted closure is executed"
        );
    }

    // ---------------------------------------------------------------------------
    // Auto slot disconnect
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    #[serial]
    fn auto_disconnect_removes_slot() {
        let dispatcher = install_test_dispatcher();
        let pre_len = dispatcher.posted.lock().unwrap().len();

        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);

        let mut sig: Signal<()> = Signal::new();
        let id = sig.connect_auto(
            std::thread::current().id(),
            std::sync::Weak::new(),
            move |_| {
                called2.store(true, Ordering::SeqCst);
            },
        );

        sig.disconnect(id);
        sig.emit_unconditionally(&());

        assert!(
            !called.load(Ordering::SeqCst),
            "disconnected Auto slot must not be called"
        );
        assert_eq!(
            dispatcher.posted.lock().unwrap().len(),
            pre_len,
            "disconnected Auto slot must not post to dispatcher"
        );
    }

    // ---------------------------------------------------------------------------
    // Regression: is_empty check is fragile under concurrent test posts
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    #[serial]
    fn auto_same_thread_does_not_grow_dispatcher_queue_when_foreign_entry_exists() {
        let dispatcher = install_test_dispatcher();

        // Simulate a concurrent test having already posted to the shared dispatcher.
        dispatcher.posted.lock().unwrap().push(Box::new(|| {}));
        let pre_len = dispatcher.posted.lock().unwrap().len();

        let mut sig: Signal<(i32,)> = Signal::new();
        sig.connect_auto(
            std::thread::current().id(),
            std::sync::Weak::new(),
            move |_| {},
        );
        sig.emit_unconditionally(&(1,));

        // Queue must not grow — the foreign entry is still there, but our emit
        // must not have added anything.
        assert_eq!(
            dispatcher.posted.lock().unwrap().len(),
            pre_len,
            "same-thread auto emit must not grow the dispatcher queue"
        );
    }

    // ---------------------------------------------------------------------------
    // AC1: same-thread Auto slot not called after receiver destroyed
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    #[serial]
    fn auto_same_thread_slot_not_called_after_receiver_destroyed() {
        use crate::receiver_guard::ReceiverGuard;

        let dispatcher = install_test_dispatcher();
        let pre_len = dispatcher.posted.lock().unwrap().len();

        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);

        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();
        let mut sig: Signal<(i32,)> = Signal::new();
        sig.connect_auto(std::thread::current().id(), guard_weak, move |_| {
            called2.store(true, Ordering::SeqCst);
        });

        drop(guard_arc); // receiver destroyed

        sig.emit_unconditionally(&(1,));

        assert!(
            !called.load(Ordering::SeqCst),
            "same-thread Auto slot must not be called after receiver is destroyed"
        );
        assert_eq!(
            dispatcher.posted.lock().unwrap().len(),
            pre_len,
            "no post must occur after receiver is destroyed"
        );
    }

    // ---------------------------------------------------------------------------
    // AC2: cross-thread Auto slot not posted after receiver destroyed
    // ---------------------------------------------------------------------------

    #[test]
    #[cfg(feature = "std")]
    #[serial]
    fn auto_cross_thread_slot_not_posted_after_receiver_destroyed() {
        use crate::receiver_guard::ReceiverGuard;

        let dispatcher = install_test_dispatcher();
        let pre_len = dispatcher.posted.lock().unwrap().len();

        let foreign_id = other_thread_id();

        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();
        let mut sig: Signal<(i32,)> = Signal::new();
        sig.connect_auto(foreign_id, guard_weak, |_| {
            panic!("slot must not be invoked after receiver is destroyed");
        });

        drop(guard_arc); // receiver destroyed

        sig.emit_unconditionally(&(99,));

        assert_eq!(
            dispatcher.posted.lock().unwrap().len(),
            pre_len,
            "no closure must be posted after receiver is destroyed"
        );
    }

    // ---------------------------------------------------------------------------
    // emit! macro: AC2, AC3, borrow-split compile check
    // ---------------------------------------------------------------------------

    /// Minimal AsObject implementor for macro tests.
    #[cfg(feature = "std")]
    struct SigHolder {
        base: crate::ObjectBase,
        sig: Signal<(i32,)>,
    }

    #[cfg(feature = "std")]
    impl crate::AsObject for SigHolder {
        fn object_base(&self) -> &crate::ObjectBase {
            &self.base
        }
        fn object_base_mut(&mut self) -> &mut crate::ObjectBase {
            &mut self.base
        }
        fn as_any(&self) -> &dyn core::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
            self
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn emit_macro_suppressed_when_signals_blocked() {
        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);
        let mut obj = SigHolder {
            base: crate::ObjectBase::new(),
            sig: Signal::new(),
        };
        obj.sig
            .connect(move |_| called2.store(true, Ordering::Relaxed));
        obj.base.block_signals();
        emit!(obj.sig, &(1,));
        assert!(
            !called.load(Ordering::Relaxed),
            "emit! must not invoke any slot when signals are blocked"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn emit_macro_fires_when_not_blocked() {
        let value = Arc::new(AtomicI32::new(0));
        let value2 = Arc::clone(&value);
        let mut obj = SigHolder {
            base: crate::ObjectBase::new(),
            sig: Signal::new(),
        };
        obj.sig
            .connect(move |args| value2.store(args.0, Ordering::Relaxed));
        emit!(obj.sig, &(42,));
        assert_eq!(
            value.load(Ordering::Relaxed),
            42,
            "emit! must invoke all slots when not blocked"
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn emit_macro_releases_borrow() {
        // Compile-time check: verifies the let-binding in emit! ends the immutable
        // borrow of obj before the mutable borrow of obj.sig is taken.
        let mut obj = SigHolder {
            base: crate::ObjectBase::new(),
            sig: Signal::new(),
        };
        emit!(obj.sig, &(0,)); // must compile without borrow-checker error
    }

    // ---------------------------------------------------------------------------
    // WeakObjectRef tests (unrelated to Auto — kept for regression)
    // ---------------------------------------------------------------------------

    #[test]
    fn weak_object_ref_wraps_u64() {
        use crate::value::WeakObjectRef;
        let r = WeakObjectRef(42);
        assert_eq!(r.0, 42);
        let r2 = r; // Copy
        assert_eq!(r2, r); // PartialEq
    }

    #[test]
    fn value_object_variant_round_trips() {
        use crate::value::{Value, WeakObjectRef};
        let v = Value::Object(WeakObjectRef(7));
        assert_eq!(v.type_name(), "Object");
        let v2 = v.clone();
        assert_eq!(v, v2);
    }
}
