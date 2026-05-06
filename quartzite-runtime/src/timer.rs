//! Interval timer backed by pluggable driver implementations.
//!
//! The primary types are [`Timer`] (a quartzite object that can live in an
//! [`ObjectTree`](crate::ObjectTree)), [`TimerConfig`] (snapshot of timer parameters),
//! [`TimerDriver`] (the pluggable backend trait), and the three built-in drivers —
//! [`ThreadDriver`], [`AppDriver`], and [`PoolDriver`].

use tracing::debug;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use parking_lot::Mutex;

use quartzite_core::{
    ConnectionId, ObjectBase, ObjectId, receiver_guard::ReceiverGuard, signal::Signal,
};
use quartzite_event_types::TimerEvent;
use quartzite_macros::{Extend, Object, object_impl};

pub use crate::timer_drivers::{AppDriver, PoolDriver, ThreadDriver};

// ────────────────────────────────────────────────────────────────────────────
// TimerConfig
// ────────────────────────────────────────────────────────────────────────────

/// Snapshot of a timer's configuration, passed to [`TimerDriver::start`].
///
/// The driver uses these values for the lifetime of one timer run. Live property
/// changes on the owning [`Timer`] only take effect on the next [`Timer::start`] call.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use quartzite_core::ObjectId;
/// use quartzite_runtime::timer::TimerConfig;
///
/// let cfg = TimerConfig {
///     timer_id: ObjectId::new(),
///     interval: Duration::from_millis(100),
///     single_shot: false,
/// };
/// assert_eq!(cfg.interval, Duration::from_millis(100));
/// ```
#[derive(Clone, Debug)]
pub struct TimerConfig {
    /// The unique identifier of the owning [`Timer`] object.
    pub timer_id: ObjectId,
    /// Duration between successive tick emissions.
    pub interval: Duration,
    /// When `true` the timer fires once and then stops automatically.
    pub single_shot: bool,
}

// ────────────────────────────────────────────────────────────────────────────
// TimerState
// ────────────────────────────────────────────────────────────────────────────

/// Shared state between a [`Timer`] and its active driver callback.
///
/// `Timer` owns an `Arc<TimerState>` and the driver callback captures a clone of
/// that same `Arc`. The atomic flags let both sides coordinate without holding
/// the full timer lock. The `signal` field shares the same `Arc<Mutex<Signal<(TimerEvent,)>>>`
/// as `Timer::tick` so both sides emit through the same `Signal` instance.
pub(crate) struct TimerState {
    /// Shared tick signal — the same `Arc` as `Timer::tick`.
    pub(crate) signal: Arc<Mutex<Signal<(TimerEvent,)>>>,
    /// Monotonically increasing count; 0 on the first fire, 1 on the second, etc.
    pub(crate) fire_count: AtomicUsize,
    /// Set to `true` while the driver is running; cleared by [`TimerDriver::stop`] / single-shot.
    pub(crate) running: AtomicBool,
    /// Mirrors `ObjectBase::signals_blocked()`; kept in sync by [`Timer::block_signals`] /
    /// [`Timer::unblock_signals`]. Driver callbacks read this instead of the object base
    /// so they do not need `&mut Timer`.
    pub(crate) signals_blocked: AtomicBool,
}

impl TimerState {
    /// Creates a `TimerState` sharing the given `signal` Arc.
    pub(crate) fn new(signal: Arc<Mutex<Signal<(TimerEvent,)>>>) -> Arc<Self> {
        Arc::new(Self {
            signal,
            fire_count: AtomicUsize::new(0),
            running: AtomicBool::new(false),
            signals_blocked: AtomicBool::new(false),
        })
    }
}

// ────────────────────────────────────────────────────────────────────────────
// TimerDriver trait
// ────────────────────────────────────────────────────────────────────────────

/// Pluggable timer backend trait.
///
/// Implementors schedule periodic (or one-shot) callbacks as described by [`TimerConfig`].
/// The three built-in implementations are [`ThreadDriver`], [`AppDriver`], and [`PoolDriver`].
///
/// `TimerDriver` requires `Send + Sync + 'static` so it can be wrapped in `Arc` and shared
/// across threads (required by [`PoolDriver`]).
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use quartzite_runtime::timer::{TimerDriver, TimerConfig, ThreadDriver};
///
/// let driver: Arc<dyn TimerDriver> = Arc::new(ThreadDriver::new());
/// ```
pub trait TimerDriver: Send + Sync + 'static {
    /// Starts timing with `config`, calling `callback` at each interval.
    ///
    /// The driver takes ownership of `callback` and invokes it on its own schedule.
    ///
    /// # Parameters
    ///
    /// - `config`: snapshot of the timer's parameters for this run.
    /// - `callback`: closure to invoke at each tick; must be `Send + Sync + 'static`.
    fn start(&self, config: TimerConfig, callback: Box<dyn Fn() + Send + Sync + 'static>);

    /// Stops the timer identified by `id`.
    ///
    /// For single-timer drivers ([`ThreadDriver`], [`AppDriver`]) the `id` is not used.
    /// For [`PoolDriver`] the `id` is the key into its per-timer maps.
    ///
    /// # Parameters
    ///
    /// - `id`: the `timer_id` from the [`TimerConfig`] passed to [`start`](Self::start).
    fn stop(&self, id: ObjectId);
}

// ────────────────────────────────────────────────────────────────────────────
// Timer struct
// ────────────────────────────────────────────────────────────────────────────

/// Interval timer that can be inserted into an [`ObjectTree`](crate::ObjectTree).
///
/// The `tick` signal carries a `usize` fire count starting at `0`. Connect slots via
/// [`connect_tick`](Self::connect_tick), [`connect_tick_queued`](Self::connect_tick_queued),
/// or [`connect_tick_auto`](Self::connect_tick_auto).
///
/// Call [`Timer::block_signals`] / [`Timer::unblock_signals`] (not the raw
/// `base.block_signals()`) so that the driver-side `TimerState` is also updated and
/// background-thread emissions are correctly gated.
///
/// # Examples
///
/// ```no_run
/// use std::{sync::Arc, time::Duration};
/// use quartzite_runtime::timer::{Timer, ThreadDriver};
///
/// let mut timer = Timer::new(Duration::from_millis(100));
/// timer.connect_tick(|args| println!("tick {}", args.0.fire_count()));
/// timer.start(Arc::new(ThreadDriver::new()));
/// // … later …
/// timer.stop();
/// ```
#[derive(Extend, Object)]
#[root]
pub struct Timer {
    /// Core object data (id, name, thread affinity, signal-block flag).
    ///
    /// Use [`Timer::block_signals`] / [`Timer::unblock_signals`] instead of
    /// `base.block_signals()` directly — the wrappers also update the driver-side
    /// `TimerState` so driver-initiated emissions are correctly gated.
    #[base]
    pub base: ObjectBase,
    /// Duration between successive `tick` emissions.
    ///
    /// Changes take effect on the next [`start`](Self::start) call.
    #[prop]
    pub interval: Duration,
    /// When `true`, the timer fires exactly once and then stops.
    ///
    /// Changes take effect on the next [`start`](Self::start) call.
    #[prop]
    pub single_shot: bool,
    /// Shared tick signal — the same `Arc` as `TimerState::signal`.
    tick: Arc<Mutex<Signal<(TimerEvent,)>>>,
    /// Shared state accessed by the driver callback.
    state: Arc<TimerState>,
    /// Active driver handle — `None` when stopped.
    driver: Option<Arc<dyn TimerDriver>>,
}

impl Timer {
    /// Creates a new anonymous, stopped timer with the given `interval`.
    ///
    /// # Parameters
    ///
    /// - `interval`: duration between successive `tick` emissions.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(500));
    /// assert!(!timer.is_running());
    /// ```
    pub fn new(interval: Duration) -> Self {
        let tick = Arc::new(Mutex::new(Signal::new()));
        let state = TimerState::new(Arc::clone(&tick));
        Self {
            base: ObjectBase::new(),
            interval,
            single_shot: false,
            tick,
            state,
            driver: None,
        }
    }

    /// Creates a new named, stopped timer with the given `interval`.
    ///
    /// The name is visible in the [`ObjectTree`](crate::ObjectTree) for lookup by name.
    ///
    /// # Parameters
    ///
    /// - `name`: human-readable identifier registered with the object base.
    /// - `interval`: duration between successive `tick` emissions.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let timer = Timer::named("heartbeat", Duration::from_secs(1));
    /// assert_eq!(timer.base.name(), Some("heartbeat"));
    /// ```
    pub fn named(name: impl Into<std::string::String>, interval: Duration) -> Self {
        let tick = Arc::new(Mutex::new(Signal::new()));
        let state = TimerState::new(Arc::clone(&tick));
        Self {
            base: ObjectBase::named(name),
            interval,
            single_shot: false,
            tick,
            state,
            driver: None,
        }
    }

    // ── signal wrappers ──────────────────────────────────────────────────────

    /// Connects a `Direct` slot to the `tick` signal.
    ///
    /// The slot receives a reference to `(TimerEvent,)` where [`TimerEvent`] carries
    /// the timer id and the 0-indexed fire count.
    ///
    /// # Parameters
    ///
    /// - `f`: callback invoked on the emitting thread each time the signal fires.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let mut timer = Timer::new(Duration::from_millis(100));
    /// let id = timer.connect_tick(|args| println!("fire #{}", args.0.fire_count()));
    /// timer.disconnect_tick(id);
    /// ```
    pub fn connect_tick<F: Fn(&(TimerEvent,)) + Send + 'static>(&self, f: F) -> ConnectionId {
        self.tick.lock().connect(f)
    }

    /// Connects a `Queued` slot to the `tick` signal.
    ///
    /// The slot is posted to the active queued dispatcher and invoked on the dispatcher thread.
    /// The `guard` is checked before posting; if the receiver has been dropped the slot is
    /// silently skipped.
    ///
    /// # Parameters
    ///
    /// - `f`: callback invoked on the dispatcher thread with an owned clone of the args.
    /// - `guard`: weak handle to the receiver's [`ReceiverGuard`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_core::receiver_guard::ReceiverGuard;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(100));
    /// let (guard_arc, guard_weak) = ReceiverGuard::new_pair();
    /// let id = timer.connect_tick_queued(
    ///     |args: (quartzite_event_types::TimerEvent,)| println!("queued #{}", args.0.fire_count()),
    ///     guard_weak,
    /// );
    /// drop(guard_arc);
    /// timer.disconnect_tick(id);
    /// ```
    pub fn connect_tick_queued<F>(
        &self,
        f: F,
        guard: std::sync::Weak<ReceiverGuard>,
    ) -> ConnectionId
    where
        F: Fn((TimerEvent,)) + Send + Sync + 'static,
    {
        self.tick.lock().connect_queued(f, guard)
    }

    /// Connects an `Auto` slot to the `tick` signal.
    ///
    /// Same-thread at emit time → `Direct` delivery; cross-thread → `Queued` delivery.
    /// The receiver thread id is captured at connect time and is not refreshed later.
    ///
    /// # Parameters
    ///
    /// - `receiver_thread_id`: the receiver's owning thread, captured once at connect time.
    /// - `guard`: weak handle to the receiver's [`ReceiverGuard`].
    /// - `f`: callback invoked with an owned clone of the args tuple.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::{thread, time::Duration};
    /// use quartzite_core::receiver_guard::ReceiverGuard;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(100));
    /// let (guard_arc, guard_weak) = ReceiverGuard::new_pair();
    /// let id = timer.connect_tick_auto(
    ///     thread::current().id(),
    ///     guard_weak,
    ///     |args: (quartzite_event_types::TimerEvent,)| println!("auto #{}", args.0.fire_count()),
    /// );
    /// drop(guard_arc);
    /// timer.disconnect_tick(id);
    /// ```
    pub fn connect_tick_auto<F>(
        &self,
        receiver_thread_id: std::thread::ThreadId,
        guard: std::sync::Weak<ReceiverGuard>,
        f: F,
    ) -> ConnectionId
    where
        F: Fn((TimerEvent,)) + Send + Sync + 'static,
    {
        self.tick.lock().connect_auto(receiver_thread_id, guard, f)
    }

    /// Disconnects a previously connected `tick` slot.
    ///
    /// # Parameters
    ///
    /// - `id`: connection identifier returned by a previous `connect_tick*` call.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(100));
    /// let id = timer.connect_tick(|_| {});
    /// timer.disconnect_tick(id);
    /// ```
    pub fn disconnect_tick(&self, id: ConnectionId) {
        self.tick.lock().disconnect(id);
    }

    /// Emits the `tick` signal with the given [`TimerEvent`] unless signals are blocked.
    ///
    /// # Parameters
    ///
    /// - `event`: the timer event to deliver to connected slots.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use quartzite_core::ObjectId;
    /// use quartzite_event_types::TimerEvent;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(100));
    /// timer.emit_tick(TimerEvent::new(timer.base.id(), 0));
    /// ```
    pub fn emit_tick(&self, event: TimerEvent) {
        let blocked = self.base.signals_blocked();
        if !blocked {
            self.tick.lock().emit_unconditionally(&(event,));
        }
    }

    // ── signal-block wrappers ────────────────────────────────────────────────

    /// Suppresses all `tick` emissions until [`unblock_signals`](Self::unblock_signals) is called.
    ///
    /// Updates both `self.base` and the driver-side `TimerState` so that even
    /// emissions initiated from the background thread are suppressed.
    ///
    /// Prefer this over calling `self.base.block_signals()` directly — the direct call
    /// does not update the driver-side flag and may allow up to one spurious tick.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let mut timer = Timer::new(Duration::from_millis(100));
    /// timer.block_signals();
    /// assert!(timer.signals_blocked());
    /// ```
    #[inline]
    pub fn block_signals(&mut self) {
        self.base.block_signals();
        self.state.signals_blocked.store(true, Ordering::Relaxed);
    }

    /// Re-enables `tick` emissions after a previous [`block_signals`](Self::block_signals) call.
    ///
    /// Updates both `self.base` and the driver-side `TimerState`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let mut timer = Timer::new(Duration::from_millis(100));
    /// timer.block_signals();
    /// timer.unblock_signals();
    /// assert!(!timer.signals_blocked());
    /// ```
    #[inline]
    pub fn unblock_signals(&mut self) {
        self.base.unblock_signals();
        self.state.signals_blocked.store(false, Ordering::Relaxed);
    }

    /// Returns `true` if signal emissions are currently blocked.
    ///
    /// Delegates to `self.base.signals_blocked()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(100));
    /// assert!(!timer.signals_blocked());
    /// ```
    #[inline]
    pub fn signals_blocked(&self) -> bool {
        self.base.signals_blocked()
    }

    // ── lifecycle ────────────────────────────────────────────────────────────

    /// Starts the timer with the given `driver`. No-op if already running.
    ///
    /// Snapshots `self.interval` and `self.single_shot` into a [`TimerConfig`] and passes
    /// it to [`TimerDriver::start`] together with a callback that emits the `tick` signal.
    ///
    /// # Parameters
    ///
    /// - `driver`: the backend implementation to use for scheduling.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::{sync::Arc, time::Duration};
    /// use quartzite_runtime::timer::{Timer, ThreadDriver};
    ///
    /// let mut timer = Timer::new(Duration::from_millis(100));
    /// timer.start(Arc::new(ThreadDriver::new()));
    /// assert!(timer.is_running());
    /// timer.stop();
    /// ```
    pub fn start(&mut self, driver: Arc<dyn TimerDriver>) {
        if self.state.running.load(Ordering::SeqCst) {
            return;
        }
        debug!(timer_id = ?self.base.id(), "timer: start");
        self.state.running.store(true, Ordering::SeqCst);
        self.state.fire_count.store(0, Ordering::SeqCst);

        let single_shot = self.single_shot;
        let config = TimerConfig {
            timer_id: self.base.id(),
            interval: self.interval,
            single_shot,
        };

        let timer_id = self.base.id();
        let state = Arc::clone(&self.state);
        let callback: Box<dyn Fn() + Send + Sync + 'static> = Box::new(move || {
            // Exit if stop() was called or single_shot already fired.
            if !state.running.load(Ordering::SeqCst) {
                return;
            }
            let count = state.fire_count.fetch_add(1, Ordering::SeqCst);
            if !state.signals_blocked.load(Ordering::Relaxed) {
                state
                    .signal
                    .lock()
                    .emit_unconditionally(&(TimerEvent::new(timer_id, count),));
            }
            if single_shot {
                state.running.store(false, Ordering::SeqCst);
            }
        });

        driver.start(config, callback);
        self.driver = Some(driver);
    }

    /// Stops the timer. No-op if already stopped.
    ///
    /// Delegates to [`TimerDriver::stop`] and clears the running flag.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::{sync::Arc, time::Duration};
    /// use quartzite_runtime::timer::{Timer, ThreadDriver};
    ///
    /// let mut timer = Timer::new(Duration::from_millis(100));
    /// timer.start(Arc::new(ThreadDriver::new()));
    /// timer.stop();
    /// assert!(!timer.is_running());
    /// ```
    pub fn stop(&mut self) {
        if !self.state.running.swap(false, Ordering::SeqCst) {
            return;
        }
        debug!(timer_id = ?self.base.id(), "timer: stop");
        if let Some(driver) = self.driver.take() {
            driver.stop(self.base.id());
        }
    }

    /// Returns `true` while the driver is actively scheduling ticks.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use quartzite_runtime::timer::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(100));
    /// assert!(!timer.is_running());
    /// ```
    #[inline]
    pub fn is_running(&self) -> bool {
        self.state.running.load(Ordering::SeqCst)
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.stop();
    }
}

// Generates impl Object for Timer (property dispatch for interval + single_shot)
// and impl AsObject for Timer (via the #[base] ObjectBase delegation from Extend).
#[object_impl]
impl Timer {}

// ────────────────────────────────────────────────────────────────────────────
// Unit tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;

    use super::*;

    #[test]
    fn timer_new_not_running() {
        let t = Timer::new(Duration::from_millis(50));
        assert!(!t.is_running());
    }

    #[test]
    fn connect_and_disconnect_tick() {
        let timer = Timer::new(Duration::from_millis(50));
        let id = timer.connect_tick(|_| {});
        timer.disconnect_tick(id);
        // No panic — sufficient.
    }

    #[test]
    fn emit_tick_suppressed_when_blocked() {
        let mut timer = Timer::new(Duration::from_millis(50));
        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);
        timer.connect_tick(move |_| called2.store(true, Ordering::SeqCst));
        timer.block_signals();
        timer.emit_tick(TimerEvent::new(timer.base.id(), 0));
        assert!(!called.load(Ordering::SeqCst));
    }

    #[test]
    fn emit_tick_fires_when_unblocked() {
        let timer = Timer::new(Duration::from_millis(50));
        let count = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::clone(&count);
        timer.connect_tick(move |args| count2.store(args.0.fire_count(), Ordering::SeqCst));
        timer.emit_tick(TimerEvent::new(timer.base.id(), 7));
        assert_eq!(count.load(Ordering::SeqCst), 7);
    }

    #[test]
    fn block_unblock_restores_emission() {
        let mut timer = Timer::new(Duration::from_millis(50));
        let called = Arc::new(AtomicBool::new(false));
        let called2 = Arc::clone(&called);
        timer.connect_tick(move |_| called2.store(true, Ordering::SeqCst));

        timer.block_signals();
        timer.emit_tick(TimerEvent::new(timer.base.id(), 0));
        assert!(!called.load(Ordering::SeqCst));

        timer.unblock_signals();
        timer.emit_tick(TimerEvent::new(timer.base.id(), 1));
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn timer_named_sets_name() {
        let timer = Timer::named("my-timer", Duration::from_millis(100));
        assert_eq!(timer.base.name(), Some("my-timer"));
    }

    #[test]
    fn thread_driver_default() {
        let _d = ThreadDriver::default();
    }

    #[test]
    fn app_driver_default() {
        let _d = AppDriver::default();
    }

    #[test]
    fn pool_driver_default() {
        let _d = PoolDriver::default();
    }

    #[test]
    fn timer_stop_when_not_running_is_noop() {
        let mut timer = Timer::new(Duration::from_millis(50));
        timer.stop(); // must not panic
        assert!(!timer.is_running());
    }

    #[test]
    fn timer_config_fields() {
        let id = ObjectId::new();
        let cfg = TimerConfig {
            timer_id: id,
            interval: Duration::from_millis(200),
            single_shot: true,
        };
        assert_eq!(cfg.timer_id, id);
        assert_eq!(cfg.interval, Duration::from_millis(200));
        assert!(cfg.single_shot);
    }

    #[test]
    fn timer_state_signal_shared_with_tick() {
        // Connecting through Timer.tick and emitting through TimerState.signal must
        // reach the same slot because both point at the same Arc<Mutex<Signal>>.
        let timer = Timer::new(Duration::from_millis(100));
        let id = timer.base.id();
        let count = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::clone(&count);
        timer.connect_tick(move |args| count2.store(args.0.fire_count() + 1, Ordering::SeqCst));

        // Emit via TimerState's signal Arc (same underlying Mutex).
        timer
            .state
            .signal
            .lock()
            .emit_unconditionally(&(TimerEvent::new(id, 41),));
        assert_eq!(count.load(Ordering::SeqCst), 42);
    }

    // ── AC10 — connect_tick_auto same-thread delivery ─────────────────────────
    // Auto mode: same thread → Direct (no dispatcher needed). This exercises
    // the connect_tick_auto API and confirms slots are called synchronously when
    // the receiver thread matches the emitting thread.

    #[test]
    fn connect_tick_auto_same_thread_direct_delivery() {
        use quartzite_core::receiver_guard::ReceiverGuard;

        let timer = Timer::new(Duration::from_millis(100));
        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();

        let count = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::clone(&count);

        // Connect from the current thread — same-thread emit will use Direct delivery.
        let receiver_thread = std::thread::current().id();
        timer.connect_tick_auto(receiver_thread, guard_weak, move |args: (TimerEvent,)| {
            count2.fetch_add(args.0.fire_count() + 1, Ordering::SeqCst);
        });

        // Emit on the same thread — Direct path, no queued dispatcher required.
        timer.emit_tick(TimerEvent::new(timer.base.id(), 5));
        assert_eq!(
            count.load(Ordering::SeqCst),
            6,
            "same-thread auto slot must fire immediately (Direct delivery)"
        );

        drop(guard_arc); // guard lifetime outlives the assert above — correct
    }

    #[test]
    fn connect_tick_auto_guard_dropped_skips_slot() {
        use quartzite_core::receiver_guard::ReceiverGuard;

        let timer = Timer::new(Duration::from_millis(100));
        let (guard_arc, guard_weak) = ReceiverGuard::new_pair();

        let count = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::clone(&count);

        let receiver_thread = std::thread::current().id();
        timer.connect_tick_auto(receiver_thread, guard_weak, move |_: (TimerEvent,)| {
            count2.fetch_add(1, Ordering::SeqCst);
        });

        // Drop the guard — slot must be silently skipped.
        drop(guard_arc);
        timer.emit_tick(TimerEvent::new(timer.base.id(), 0));
        assert_eq!(
            count.load(Ordering::SeqCst),
            0,
            "slot must be skipped after guard is dropped"
        );
    }

    // ── AC2 — Timer in ObjectTree ─────────────────────────────────────────────

    #[test]
    fn timer_object_tree_insert_and_retrieve() {
        use crate::ObjectTree;

        let mut tree = ObjectTree::new();
        let timer = Timer::named("heartbeat", Duration::from_millis(100));
        let id = timer.base.id();

        let inserted_id = tree.insert(Box::new(timer), None);
        assert_eq!(inserted_id, id);
        assert!(tree.contains(id));

        // Named lookup.
        let found_ids = tree.find_by_name("heartbeat");
        assert_eq!(found_ids, vec![id]);
    }

    #[test]
    fn timer_object_tree_downcast() {
        use crate::ObjectTree;

        let mut tree = ObjectTree::new();
        let timer = Timer::named("t", Duration::from_millis(50));
        let id = tree.insert(Box::new(timer), None);

        let name = tree
            .with(id, |obj| {
                obj.as_any()
                    .downcast_ref::<Timer>()
                    .and_then(|t| t.base.name())
                    .map(str::to_owned)
            })
            .flatten();
        assert_eq!(name, Some("t".to_owned()));
    }

    // ── AC4 — property system read/write ─────────────────────────────────────

    #[test]
    fn timer_read_property_interval() {
        use quartzite_core::Value;
        use quartzite_core::traits::Object;

        let timer = Timer::new(Duration::from_millis(200));
        assert_eq!(
            timer.read_property("interval"),
            Some(Value::Duration(Duration::from_millis(200)))
        );
    }

    #[test]
    fn timer_write_property_interval() {
        use quartzite_core::Value;
        use quartzite_core::traits::Object;

        let mut timer = Timer::new(Duration::from_millis(100));
        let ok = timer.write_property("interval", Value::Duration(Duration::from_secs(1)));
        assert!(ok);
        assert_eq!(timer.interval, Duration::from_secs(1));
    }

    #[test]
    fn timer_read_property_single_shot() {
        use quartzite_core::Value;
        use quartzite_core::traits::Object;

        let mut timer = Timer::new(Duration::from_millis(100));
        assert_eq!(timer.read_property("single_shot"), Some(Value::Bool(false)));
        timer.single_shot = true;
        assert_eq!(timer.read_property("single_shot"), Some(Value::Bool(true)));
    }

    #[test]
    fn timer_write_property_single_shot() {
        use quartzite_core::Value;
        use quartzite_core::traits::Object;

        let mut timer = Timer::new(Duration::from_millis(100));
        let ok = timer.write_property("single_shot", Value::Bool(true));
        assert!(ok);
        assert!(timer.single_shot);
    }

    #[test]
    fn timer_read_property_unknown_returns_none() {
        use quartzite_core::traits::Object;

        let timer = Timer::new(Duration::from_millis(100));
        assert_eq!(timer.read_property("nonexistent"), None);
    }

    #[test]
    fn timer_write_property_wrong_type_returns_false() {
        use quartzite_core::Value;
        use quartzite_core::traits::Object;

        let mut timer = Timer::new(Duration::from_millis(100));
        // interval expects Duration, not Bool
        assert!(!timer.write_property("interval", Value::Bool(true)));
        assert_eq!(timer.interval, Duration::from_millis(100)); // unchanged
    }

    #[test]
    fn timer_meta_object_class_name() {
        use quartzite_core::traits::Object;

        let timer = Timer::new(Duration::from_millis(100));
        assert_eq!(timer.meta_object().class_name, "Timer");
    }

    #[test]
    fn single_shot_clears_running_flag() {
        // Verifies finding 1 fix: TimerState::running must be false after a single_shot fires.
        let counter = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::clone(&counter);

        let mut timer = Timer::new(Duration::from_millis(20));
        timer.single_shot = true;
        timer.connect_tick(move |_| {
            counter2.fetch_add(1, Ordering::SeqCst);
        });
        timer.start(Arc::new(ThreadDriver::new()));

        // Wait for the single fire.
        let deadline = std::time::Instant::now() + Duration::from_millis(300);
        while counter.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        // Give the callback a moment to also clear the running flag.
        std::thread::sleep(Duration::from_millis(20));

        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "single_shot must fire exactly once"
        );
        assert!(
            !timer.is_running(),
            "is_running() must be false after single_shot fires"
        );
    }

    #[test]
    fn timer_meta_object_property_lookup() {
        use quartzite_core::traits::Object;

        let timer = Timer::new(Duration::from_millis(100));
        let meta = timer.meta_object();
        assert!(meta.property("interval").is_some());
        assert!(meta.property("single_shot").is_some());
        assert!(meta.property("nonexistent").is_none());
    }
}
