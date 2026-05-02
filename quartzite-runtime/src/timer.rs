//! Interval timer that fires a signal via the event loop.
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use quartzite_core::{ConnectionId, object_base::ObjectBase, signal::Signal};

/// Fires its `timeout` signal at a given interval via the event loop.
///
/// The background thread posts a closure to the event loop each interval.
/// `Signal` is not `Sync`, so the signal is wrapped in `Arc<Mutex<>>` to allow
/// the background thread to capture and emit it on the event-loop thread.
pub struct Timer {
    /// Base object state (id, name, thread affinity).
    pub base: ObjectBase,
    /// Duration between `timeout` signal emissions.
    pub interval: Duration,
    /// When `true` the timer fires once and then stops automatically.
    pub single_shot: bool,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    /// Shared with the background thread so emission can be posted to the
    /// event loop. Slot callbacks always execute on the event-loop thread.
    timeout: Arc<Mutex<Signal<()>>>,
}

impl Timer {
    /// Create a new repeating timer that fires every `interval`.
    ///
    /// The timer is not started; call [`start`](Self::start) to begin firing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_runtime::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(500));
    /// assert!(!timer.is_running());
    /// ```
    pub fn new(interval: Duration) -> Self {
        Self {
            base: ObjectBase::new(),
            interval,
            single_shot: false,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            timeout: Arc::new(Mutex::new(Signal::new())),
        }
    }

    /// Connect a slot to the `timeout` signal. The closure must be `Send`
    /// because it may be called on the event-loop thread (not the caller's thread).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_runtime::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(500));
    /// let id = timer.connect_timeout(|_| println!("tick"));
    /// timer.disconnect_timeout(id);
    /// ```
    pub fn connect_timeout<F: Fn(&()) + Send + 'static>(&self, f: F) -> ConnectionId {
        self.timeout.lock().unwrap().connect(f)
    }

    /// Disconnect a previously connected timeout slot.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_runtime::Timer;
    ///
    /// let timer = Timer::new(Duration::from_millis(500));
    /// let id = timer.connect_timeout(|_| {});
    /// timer.disconnect_timeout(id);
    /// ```
    pub fn disconnect_timeout(&self, id: ConnectionId) {
        self.timeout.lock().unwrap().disconnect(id);
    }

    /// Start the timer. `post` must be a `Sender` cloned from the active `EventLoop`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_runtime::{EventLoop, Timer};
    ///
    /// let el = EventLoop::new();
    /// let mut timer = Timer::new(Duration::from_millis(100));
    /// timer.start(el.sender());
    /// assert!(timer.is_running());
    /// timer.stop();
    /// ```
    pub fn start(&mut self, post: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);
        let interval = self.interval;
        let single_shot = self.single_shot;
        let timeout = Arc::clone(&self.timeout);

        let handle = thread::spawn(move || {
            loop {
                thread::sleep(interval);
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                let sig = Arc::clone(&timeout);
                // Post emission to the event-loop thread.
                let _ = post.send(Box::new(move || {
                    sig.lock().unwrap().emit(&());
                }));
                if single_shot {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
            }
        });
        self.handle = Some(handle);
    }

    /// Stop the timer and join the background thread.
    ///
    /// No-op if the timer is not running. Blocks until the background thread exits.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_runtime::{EventLoop, Timer};
    ///
    /// let el = EventLoop::new();
    /// let mut timer = Timer::new(Duration::from_millis(100));
    /// timer.start(el.sender());
    /// timer.stop();
    /// assert!(!timer.is_running());
    /// ```
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    /// Returns `true` while the background thread is active.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_runtime::Timer;
    ///
    /// let mut timer = Timer::new(Duration::from_millis(100));
    /// assert!(!timer.is_running());
    /// ```
    #[inline]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_new_not_running() {
        let t = Timer::new(Duration::from_millis(50));
        assert!(!t.is_running());
    }

    #[test]
    fn connect_and_disconnect_timeout() {
        let timer = Timer::new(Duration::from_millis(50));
        let id = timer.connect_timeout(|_| {});
        timer.disconnect_timeout(id);
        // No panic — sufficient.
    }
}
