//! Single-threaded event loop for posting and executing closures.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
};
use std::time::Duration;
use tracing::debug_span;

use crate::loop_registry::{LoopAlreadyInstalled, LoopRegistry, RegistryGuard};

const TICK_MS: u64 = 1;

/// Single-threaded event loop.
///
/// Call [`run`](Self::run) on the main thread; post work from any thread via
/// [`post`](Self::post).
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::EventLoop;
///
/// let el = EventLoop::new();
/// el.post(Box::new(|| println!("hello")));
/// el.stop();
/// ```
pub struct EventLoop {
    sender: Sender<Box<dyn FnOnce() + Send>>,
    receiver: parking_lot::Mutex<Receiver<Box<dyn FnOnce() + Send>>>,
    running: Arc<AtomicBool>,
}

impl EventLoop {
    /// Creates a new, idle event loop.
    ///
    /// Call [`run`](Self::run) on the intended loop thread to start processing events.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = EventLoop::new();
    /// assert!(!el.is_running());
    /// ```
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: parking_lot::Mutex::new(receiver),
            running: Arc::new(AtomicBool::new(false)),
        }
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

    /// Runs the event loop on the calling thread. Blocks until [`stop`](Self::stop) is called.
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
    ///     el2.stop();
    /// });
    /// el.run(); // blocks until stop() is called above
    /// ```
    pub fn run(&self) {
        // `RegistryGuard` deregisters the current thread from `LoopRegistry` on drop,
        // ensuring cleanup even when a posted closure panics and unwinds through `run`.
        let _guard = RegistryGuard;
        let receiver = self.receiver.lock();
        self.running.store(true, Ordering::SeqCst);
        while self.running.load(Ordering::SeqCst) {
            while let Ok(f) = receiver.try_recv() {
                f();
            }
            match receiver.recv_timeout(Duration::from_millis(TICK_MS)) {
                Ok(f) => f(),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        // Drain any remaining messages before exiting.
        while let Ok(f) = receiver.try_recv() {
            f();
        }
    }

    /// Signals the event loop to stop. May be called from any thread.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = EventLoop::new();
    /// el.stop();
    /// ```
    pub fn stop(&self) {
        let _span = debug_span!("event_loop::stop").entered();
        self.running.store(false, Ordering::SeqCst);
        // Wake the loop by posting a no-op.
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

    /// Spawns a new thread with an installed, running [`EventLoop`].
    ///
    /// The thread installs a fresh loop, calls `f`, then runs the loop until
    /// [`stop`](Self::stop) is called. Returns the `Arc<EventLoop>` for the new thread
    /// (usable to post closures or stop the loop) and the [`JoinHandle`](std::thread::JoinHandle).
    ///
    /// # Parameters
    ///
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
    /// let (el, handle) = EventLoop::spawn(|| {}).unwrap();
    /// el.post(Box::new(|| println!("on worker thread")));
    /// el.stop();
    /// handle.join().unwrap();
    /// ```
    pub fn spawn(
        f: impl FnOnce() + Send + 'static,
    ) -> Result<(Arc<Self>, std::thread::JoinHandle<()>), LoopAlreadyInstalled> {
        let el = Arc::new(EventLoop::new());
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
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    fn start_loop(el: Arc<EventLoop>) -> thread::JoinHandle<()> {
        thread::spawn(move || el.run())
    }

    #[test]
    fn post_from_other_thread_executes_on_loop_thread() {
        let el = Arc::new(EventLoop::new());
        let loop_thread_id: Arc<Mutex<Option<thread::ThreadId>>> = Arc::new(Mutex::new(None));

        let tid = Arc::clone(&loop_thread_id);
        let el2 = Arc::clone(&el);
        let handle = start_loop(Arc::clone(&el));

        thread::sleep(Duration::from_millis(5));

        el2.post(Box::new(move || {
            *tid.lock().unwrap() = Some(thread::current().id());
        }));

        thread::sleep(Duration::from_millis(20));
        el.stop();
        handle.join().unwrap();

        let recorded = loop_thread_id.lock().unwrap();
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
            el.post(Box::new(move || log2.lock().unwrap().push(i)));
        }

        thread::sleep(Duration::from_millis(20));
        el.stop();
        handle.join().unwrap();

        assert_eq!(*log.lock().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn stop_terminates_run() {
        let el = Arc::new(EventLoop::new());
        let el2 = Arc::clone(&el);
        let handle = thread::spawn(move || el2.run());

        thread::sleep(Duration::from_millis(5));
        el.stop();

        assert!(handle.join().is_ok());
    }
}
