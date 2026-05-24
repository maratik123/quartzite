//! Single-threaded event loop for posting and executing closures.
//!
//! By default every [`EventLoop`] is **tickless**: [`run`](EventLoop::run) blocks on
//! [`Receiver::recv`] and only wakes when a closure is posted or
//! [`stop`](EventLoop::stop) / [`request_stop`](EventLoop::request_stop) is called.
//! This eliminates the 1 ms polling overhead that was present in earlier versions.
//!
//! A tick-based loop (useful for animation drivers or polled I/O) can be obtained via
//! [`EventLoop::with_tick`]: the loop then wakes at most once per tick even when no
//! closure is posted.
//!
//! # Tickless vs. tick-based
//!
//! | Constructor | Behaviour |
//! |---|---|
//! | [`EventLoop::new()`] | Tickless — `recv()` blocks until a closure arrives |
//! | [`EventLoop::with_tick(Some(d))`](EventLoop::with_tick) | Tick-based — `recv_timeout(d)` wakes at most every `d` |
//! | [`EventLoop::with_tick(None)`](EventLoop::with_tick) | Same as `new()` |
//!
//! Passing `Some(Duration::ZERO)` to `with_tick` is silently normalised to `None`
//! (tickless) because a zero-duration timeout would busy-loop without doing useful work.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
};
use std::time::Duration;
use tracing::debug_span;

use quartzite_core::ObjectBase;
use quartzite_macros::{Extend, Object, object_impl};

use crate::loop_registry::{LoopAlreadyInstalled, LoopRegistry, RegistryGuard};

/// Single-threaded event loop.
///
/// Call [`run`](Self::run) on the main thread; post work from any thread via
/// [`post`](Self::post).
///
/// The loop is **tickless by default** — it blocks on [`Receiver::recv`] and only
/// wakes when a closure is posted or [`request_stop`](Self::request_stop) is called.
/// Use [`EventLoop::with_tick`] to construct a tick-based loop.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::EventLoop;
///
/// let el = EventLoop::new();
/// el.post(Box::new(|| println!("hello")));
/// el.request_stop();
/// ```
#[derive(Extend, Object)]
#[root]
pub struct EventLoop {
    /// Core object data (id, name, thread affinity, signal-block flag).
    #[base]
    pub base: ObjectBase,
    sender: Sender<Box<dyn FnOnce() + Send>>,
    receiver: parking_lot::Mutex<Receiver<Box<dyn FnOnce() + Send>>>,
    /// Set to `true` while `run` is executing; `false` before and after.
    running: Arc<AtomicBool>,
    /// Set to `true` by `stop`; checked by `run` to decide when to exit.
    /// Decoupled from `running` so that `stop` called before `run` is visible.
    stop_requested: AtomicBool,
    /// Tick duration for the event loop.
    ///
    /// `None` → tickless (block on `recv()`).
    /// `Some(d)` → tick-based (wake at most every `d` via `recv_timeout(d)`).
    tick: Option<Duration>,
}

#[object_impl]
impl EventLoop {
    /// Creates a new, tickless, idle event loop.
    ///
    /// The loop blocks on [`Receiver::recv`] and only wakes when a closure is posted
    /// or [`request_stop`](Self::request_stop) is called. For a tick-based loop use
    /// [`EventLoop::with_tick`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = EventLoop::new();
    /// assert!(!el.is_running());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::with_tick(None)
    }

    /// Creates a new, idle event loop with the given tick policy.
    ///
    /// - `None` or `Some(Duration::ZERO)` → tickless: blocks on [`Receiver::recv`] until
    ///   a closure arrives or the channel disconnects.
    /// - `Some(d)` where `d > 0` → tick-based: uses [`Receiver::recv_timeout(d)`] so
    ///   the loop wakes at most every `d` even when the channel is idle.
    ///
    /// Passing `Some(Duration::ZERO)` is silently normalised to `None` (tickless) because
    /// a zero-duration timeout would busy-loop without doing useful work.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_runtime::EventLoop;
    ///
    /// let tickless = EventLoop::with_tick(None);
    /// let ticked = EventLoop::with_tick(Some(Duration::from_millis(50)));
    /// assert_eq!(ticked.tick(), Some(Duration::from_millis(50)));
    /// ```
    #[inline]
    pub fn with_tick(tick: Option<Duration>) -> Self {
        let tick = tick.filter(|d| !d.is_zero());
        let (sender, receiver) = mpsc::channel();
        Self {
            base: ObjectBase::new(),
            sender,
            receiver: parking_lot::Mutex::new(receiver),
            running: Arc::new(AtomicBool::new(false)),
            stop_requested: AtomicBool::new(false),
            tick,
        }
    }

    /// Returns the configured tick duration, or `None` if the loop is tickless.
    ///
    /// This method is `#[doc(hidden)]` because it is intended for use by integration
    /// tests that need to verify tick propagation; it is not part of the stable public API.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = EventLoop::with_tick(Some(Duration::from_millis(50)));
    /// assert_eq!(el.tick(), Some(Duration::from_millis(50)));
    /// ```
    #[doc(hidden)]
    #[inline]
    pub const fn tick(&self) -> Option<Duration> {
        self.tick
    }

    /// Posts a closure to be executed on the event-loop thread. Callable from any thread.
    ///
    /// # Parameters
    ///
    /// - `f`: closure to run on the event-loop thread; runs in FIFO order with other
    ///   posted closures.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = EventLoop::new();
    /// el.post(Box::new(|| println!("on loop thread")));
    /// ```
    pub fn post(&self, f: Box<dyn FnOnce() + Send>) {
        #[cfg(feature = "verbose-tracing")]
        let _span = tracing::trace_span!("event_loop::post").entered();
        let _ = self.sender.send(f);
    }

    /// Returns a clone of the sender so callers can post without holding a reference to
    /// the loop itself.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = EventLoop::new();
    /// let tx = el.sender();
    /// tx.send(Box::new(|| println!("posted"))).ok();
    /// ```
    #[inline]
    pub fn sender(&self) -> Sender<Box<dyn FnOnce() + Send>> {
        self.sender.clone()
    }

    /// Runs the event loop on the calling thread. Blocks until [`request_stop`](Self::request_stop) is called.
    ///
    /// The loop behaviour depends on the tick policy:
    /// - Tickless (the default): blocks on [`Receiver::recv`] until a closure arrives or the
    ///   channel disconnects. CPU usage is zero while idle.
    /// - Tick-based: uses [`Receiver::recv_timeout`] so the loop wakes at most every tick
    ///   even when no closure is posted.
    ///
    /// # Panics
    ///
    /// If a posted closure panics, the panic propagates through `run` to its caller.
    /// In normal use `run` is called once on the main thread.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = Arc::new(EventLoop::new());
    /// let el2 = Arc::clone(&el);
    /// std::thread::spawn(move || {
    ///     std::thread::sleep(std::time::Duration::from_millis(10));
    ///     el2.request_stop();
    /// });
    /// el.run(); // blocks until request_stop() is called above
    /// ```
    pub fn run(&self) {
        // `RegistryGuard` deregisters the current thread from `LoopRegistry` on drop,
        // ensuring cleanup even when a posted closure panics and unwinds through `run`.
        let _guard = RegistryGuard;
        let receiver = self.receiver.lock();
        self.running.store(true, Ordering::SeqCst);
        // Check `stop_requested` (set by `request_stop()`) rather than `running` (set by us):
        // this makes a `request_stop()` call that arrived before `run()` visible on the first
        // iteration, preventing the loop from running forever when the caller calls
        // `request_stop()` before the worker thread enters `run()`.
        while !self.stop_requested.load(Ordering::SeqCst) {
            while let Ok(f) = receiver.try_recv() {
                f();
            }
            match self.tick {
                None => {
                    // Tickless path: block until a closure arrives or the channel disconnects.
                    match receiver.recv() {
                        Ok(f) => f(),
                        Err(mpsc::RecvError) => break,
                    }
                }
                Some(d) => {
                    // Tick-based path: wake at most every `d`.
                    match receiver.recv_timeout(d) {
                        Ok(f) => f(),
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
            }
        }
        self.running.store(false, Ordering::SeqCst);
        // Drain any remaining messages before exiting.
        while let Ok(f) = receiver.try_recv() {
            f();
        }
    }

    /// Signals the event loop to stop. Takes `&mut self` for slot-dispatch compatibility.
    ///
    /// For cross-thread and ergonomic use without a mutable borrow, prefer
    /// [`request_stop`](Self::request_stop).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::EventLoop;
    ///
    /// let mut el = EventLoop::new();
    /// el.stop();
    /// ```
    #[slot]
    pub fn stop(&mut self) {
        let _span = debug_span!("event_loop::stop").entered();
        self.stop_requested.store(true, Ordering::SeqCst);
        // Wake the loop by posting a no-op so recv() / recv_timeout() returns immediately.
        let _ = self.sender.send(Box::new(|| {}));
    }

    /// Signals the event loop to stop. Callable from any thread via a shared reference.
    ///
    /// This is the preferred API for cross-thread shutdown (e.g. from a thread holding
    /// `Arc<EventLoop>`). For reflection-based invocation via `invoke_method`, use
    /// [`stop`](Self::stop) (which requires `&mut self`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = Arc::new(EventLoop::new());
    /// let el2 = Arc::clone(&el);
    /// std::thread::spawn(move || el2.request_stop());
    /// el.run();
    /// ```
    #[inline]
    pub fn request_stop(&self) {
        let _span = debug_span!("event_loop::request_stop").entered();
        self.stop_requested.store(true, Ordering::SeqCst);
        // Wake the loop by posting a no-op so recv() / recv_timeout() returns immediately.
        let _ = self.sender.send(Box::new(|| {}));
    }

    /// Returns `true` while the loop is running.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = EventLoop::new();
    /// assert!(!el.is_running());
    /// ```
    #[inline]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Registers this loop in the process-wide `LoopRegistry`
    /// for the calling thread.
    ///
    /// After installation, queued signal invocations targeting the calling thread will be routed
    /// to this loop. Call [`run`](Self::run) on the same thread afterward to start processing.
    ///
    /// # Parameters
    ///
    /// - `self`: `Arc`-wrapped loop to register; the registry holds a clone of this `Arc`.
    ///
    /// # Errors
    ///
    /// Returns [`LoopAlreadyInstalled`] if a loop is already registered for the calling thread.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = Arc::new(EventLoop::new());
    /// el.clone().install_for_current_thread().expect("no loop installed yet");
    /// ```
    pub fn install_for_current_thread(self: Arc<Self>) -> Result<(), LoopAlreadyInstalled> {
        let _span = debug_span!("event_loop::install_for_current_thread").entered();
        LoopRegistry::install(std::thread::current().id(), self)
    }

    /// Removes the current thread's `LoopRegistry` entry.
    ///
    /// No-op if the calling thread has no registered loop. Typically called automatically by
    /// [`run`](Self::run) via `RegistryGuard` on exit.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = Arc::new(EventLoop::new());
    /// el.clone().install_for_current_thread().unwrap();
    /// EventLoop::uninstall_for_current_thread();
    /// ```
    pub fn uninstall_for_current_thread() {
        let _span = debug_span!("event_loop::uninstall_for_current_thread").entered();
        LoopRegistry::uninstall(std::thread::current().id());
    }

    /// Spawns a new thread with an installed, running [`EventLoop`] using the given tick policy.
    ///
    /// The thread installs a fresh loop, calls `f`, then runs the loop until
    /// [`request_stop`](Self::request_stop) is called. Returns the `Arc<EventLoop>` for the new
    /// thread (usable to post closures or stop the loop) and the
    /// [`JoinHandle`](std::thread::JoinHandle).
    ///
    /// # Parameters
    ///
    /// - `tick`: tick policy for the spawned loop. Pass `None` for tickless (recommended).
    /// - `f`: callback invoked on the new thread before the loop starts; use it to post initial
    ///   work or pass the `Arc<EventLoop>` to other parts of the program.
    ///
    /// # Errors
    ///
    /// Returns [`LoopAlreadyInstalled`] if the spawned thread's [`ThreadId`](std::thread::ThreadId)
    /// is already registered in the loop registry. In practice this cannot happen because
    /// `ThreadId` values are guaranteed never to be reused within a process lifetime.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::EventLoop;
    ///
    /// let (el, handle) = EventLoop::spawn(None, || {}).unwrap();
    /// el.post(Box::new(|| println!("on worker thread")));
    /// el.request_stop();
    /// handle.join().unwrap();
    /// ```
    pub fn spawn(
        tick: Option<Duration>,
        f: impl FnOnce() + Send + 'static,
    ) -> Result<(Arc<Self>, std::thread::JoinHandle<()>), LoopAlreadyInstalled> {
        let el = Arc::new(Self::with_tick(tick));
        let el_thread = Arc::clone(&el);
        let (tx, rx) = mpsc::channel::<Result<(), LoopAlreadyInstalled>>();
        let handle = std::thread::spawn(move || {
            let result = Arc::clone(&el_thread).install_for_current_thread();
            let ok = result.is_ok();
            let _ = tx.send(result);
            if ok {
                f();
                el_thread.run();
            }
        });
        rx.recv().unwrap_or(Ok(()))?;
        Ok((el, handle))
    }
}

impl Default for EventLoop {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::Mutex;
    use std::{sync::Arc, thread, time::Duration};

    fn start_loop(el: Arc<EventLoop>) -> thread::JoinHandle<()> {
        thread::spawn(move || el.run())
    }

    #[test]
    #[allow(
        clippy::significant_drop_tightening,
        reason = "MutexGuard held intentionally to keep critical section atomic"
    )]
    fn post_from_other_thread_executes_on_loop_thread() {
        let el = Arc::new(EventLoop::new());
        let loop_thread_id: Arc<Mutex<Option<thread::ThreadId>>> = Arc::new(Mutex::new(None));

        let tid = Arc::clone(&loop_thread_id);
        let el2 = Arc::clone(&el);
        let handle = start_loop(Arc::clone(&el));

        thread::sleep(Duration::from_millis(5));

        el2.post(Box::new(move || {
            *tid.lock() = Some(thread::current().id());
        }));

        thread::sleep(Duration::from_millis(20));
        el.request_stop();
        handle.join().unwrap();

        let recorded = loop_thread_id.lock();
        assert!(recorded.is_some());
        assert_ne!(*recorded, Some(thread::current().id()));
    }

    #[test]
    fn post_multiple_preserves_order() {
        let el = Arc::new(EventLoop::new());
        let log: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

        let handle = start_loop(Arc::clone(&el));
        thread::sleep(Duration::from_millis(5));

        for i in 1u32..=3 {
            let log2 = Arc::clone(&log);
            el.post(Box::new(move || log2.lock().push(i)));
        }

        thread::sleep(Duration::from_millis(20));
        el.request_stop();
        handle.join().unwrap();

        assert_eq!(*log.lock(), vec![1, 2, 3]);
    }

    #[test]
    fn stop_terminates_run() {
        let el = Arc::new(EventLoop::new());
        let el2 = Arc::clone(&el);
        let handle = thread::spawn(move || el2.run());

        thread::sleep(Duration::from_millis(5));
        el.request_stop();

        assert!(handle.join().is_ok());
    }

    #[test]
    fn stop_before_run_exits_immediately() {
        use std::sync::mpsc;
        let el = Arc::new(EventLoop::new());
        // request_stop() fires before run() is entered; the loop must not start.
        el.request_stop();
        let el2 = Arc::clone(&el);
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            el2.run();
            let _ = tx.send(());
        });
        rx.recv_timeout(Duration::from_millis(500))
            .expect("run() must exit within 500 ms when request_stop() was called before run()");
    }

    #[test]
    fn new_is_tickless() {
        let el = EventLoop::new();
        assert_eq!(el.tick(), None);
    }

    #[test]
    fn with_tick_some_stores_duration() {
        let el = EventLoop::with_tick(Some(Duration::from_millis(50)));
        assert_eq!(el.tick(), Some(Duration::from_millis(50)));
    }

    #[test]
    fn with_tick_zero_normalises_to_none() {
        let el = EventLoop::with_tick(Some(Duration::ZERO));
        assert_eq!(el.tick(), None);
    }

    #[test]
    fn default_is_tickless() {
        let el = EventLoop::default();
        assert_eq!(el.tick(), None);
    }
}
