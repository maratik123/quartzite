//! Single-threaded event loop for posting and executing closures.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
};
use std::time::Duration;
use tracing::{debug, trace};

/// Error returned by [`EventLoop::run`].
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::{EventLoop, RunError};
///
/// let el = EventLoop::new();
/// match el.run() {
///     Ok(()) => {}
///     Err(RunError::Poisoned) => eprintln!("event loop poisoned — previous run() panicked"),
/// }
/// ```
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RunError {
    /// The receiver mutex is poisoned: a previous [`EventLoop::run`] call panicked while
    /// dispatching a closure.
    #[error("event loop receiver mutex is poisoned — a previous run() panicked mid-loop")]
    Poisoned,
}

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
    receiver: std::sync::Mutex<Receiver<Box<dyn FnOnce() + Send>>>,
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
            receiver: std::sync::Mutex::new(receiver),
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
        trace!("event loop: posting closure");
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
    /// # Errors
    ///
    /// Returns [`RunError::Poisoned`] if the receiver mutex is poisoned because a previous
    /// `run()` call panicked while dispatching a closure.
    ///
    /// # Panics
    ///
    /// If a posted closure panics, the panic propagates through `run()` to its caller.
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
    /// el.run().expect("event loop poisoned"); // blocks until stop() is called above
    /// ```
    pub fn run(&self) -> Result<(), RunError> {
        let receiver = self.receiver.lock().map_err(|_| RunError::Poisoned)?;
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
        Ok(())
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
        debug!("event loop: stop requested");
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

    fn start_loop(el: Arc<EventLoop>) -> thread::JoinHandle<Result<(), RunError>> {
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
        handle.join().unwrap().unwrap();

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
        handle.join().unwrap().unwrap();

        assert_eq!(*log.lock().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn stop_terminates_run() {
        let el = Arc::new(EventLoop::new());
        let el2 = Arc::clone(&el);
        let handle = thread::spawn(move || el2.run());

        thread::sleep(Duration::from_millis(5));
        el.stop();

        let result = handle.join();
        assert!(result.unwrap().is_ok());
    }
}
