//! Single-threaded event loop for posting and executing closures.
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::Duration,
};

const TICK_MS: u64 = 1;

/// Single-threaded event loop. Run `run()` on the main thread; post work from
/// any thread via `post()`.
pub struct EventLoop {
    sender: Sender<Box<dyn FnOnce() + Send>>,
    receiver: std::sync::Mutex<Receiver<Box<dyn FnOnce() + Send>>>,
    running: Arc<AtomicBool>,
}

impl EventLoop {
    /// Create a new, idle event loop. Call [`run`](Self::run) on the intended
    /// loop thread to start processing events.
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

    /// Post a closure to be executed on the event-loop thread. Callable from
    /// any thread.
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
        let _ = self.sender.send(f);
    }

    /// Clone the sender so callers can post without holding a reference to the
    /// loop itself.
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

    /// Run the event loop on the calling thread. Blocks until `stop()` is called.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_runtime::EventLoop;
    ///
    /// let el = Arc::new(EventLoop::new());
    /// let el2 = Arc::clone(&el);
    /// std::thread::spawn(move || { std::thread::sleep(std::time::Duration::from_millis(10)); el2.stop(); });
    /// el.run(); // blocks until stop() is called above
    /// ```
    pub fn run(&self) {
        self.running.store(true, Ordering::SeqCst);
        let receiver = self.receiver.lock().unwrap();
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

    /// Signal the event loop to stop. May be called from any thread.
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

        let result = handle.join();
        assert!(result.is_ok());
    }
}
