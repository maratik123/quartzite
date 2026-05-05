//! Built-in [`TimerDriver`] implementations.
//!
//! This module provides the three standard backends:
//! - [`ThreadDriver`]: one dedicated background thread per timer.
//! - [`AppDriver`]: ticks posted to the application event-loop thread.
//! - [`PoolDriver`]: single shared thread driving multiple timers via a min-heap.

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle, Thread},
    time::{Duration, Instant},
};

use quartzite_core::ObjectId;

use crate::timer::{TimerConfig, TimerDriver};

// ────────────────────────────────────────────────────────────────────────────
// ThreadDriver
// ────────────────────────────────────────────────────────────────────────────

/// One dedicated background thread per timer.
///
/// Uses `thread::park_timeout` for sleeping so [`stop`](TimerDriver::stop) can wake the
/// thread immediately via `unpark`.
///
/// # Examples
///
/// ```no_run
/// use std::{sync::Arc, time::Duration};
/// use quartzite_runtime::timer::{Timer, ThreadDriver};
///
/// let mut timer = Timer::new(Duration::from_millis(50));
/// timer.connect_tick(|args| println!("tick #{}", args.0));
/// timer.start(Arc::new(ThreadDriver::new()));
/// std::thread::sleep(Duration::from_millis(200));
/// timer.stop();
/// ```
pub struct ThreadDriver {
    running: Arc<AtomicBool>,
    /// `(Thread handle for unpark, JoinHandle for join)` stored together so `stop()` is atomic.
    handle: Mutex<Option<(Thread, JoinHandle<()>)>>,
}

impl ThreadDriver {
    /// Creates a new idle `ThreadDriver`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_runtime::timer::ThreadDriver;
    ///
    /// let driver = ThreadDriver::new();
    /// ```
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            handle: Mutex::new(None),
        }
    }
}

impl Default for ThreadDriver {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TimerDriver for ThreadDriver {
    fn start(&self, config: TimerConfig, callback: Box<dyn Fn() + Send + Sync + 'static>) {
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);
        let interval = config.interval;
        let single_shot = config.single_shot;

        let join = thread::spawn(move || {
            loop {
                thread::park_timeout(interval);
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                callback();
                if single_shot {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
            }
        });

        let thread_handle = join.thread().clone();
        *self.handle.lock().unwrap_or_else(|e| e.into_inner()) = Some((thread_handle, join));
    }

    fn stop(&self, _id: ObjectId) {
        self.running.store(false, Ordering::SeqCst);
        if let Some((thread_handle, join)) =
            self.handle.lock().unwrap_or_else(|e| e.into_inner()).take()
        {
            thread_handle.unpark();
            let _ = join.join();
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// AppDriver
// ────────────────────────────────────────────────────────────────────────────

/// Timer driver that executes tick callbacks on the application event-loop thread.
///
/// Each tick is posted via [`Application::post_event`](crate::Application::post_event).
/// If [`Application::global()`](crate::Application::global) returns `None` (application
/// already dropped), the tick is silently skipped.
///
/// # Examples
///
/// ```no_run
/// use std::{sync::Arc, time::Duration};
/// use quartzite_runtime::{Application, timer::{Timer, AppDriver}};
///
/// let app = Application::new().unwrap();
/// let mut timer = Timer::new(Duration::from_millis(50));
/// timer.connect_tick(|args| println!("app-thread tick #{}", args.0));
/// timer.start(Arc::new(AppDriver::new()));
/// std::thread::sleep(Duration::from_millis(200));
/// timer.stop();
/// ```
pub struct AppDriver {
    running: Arc<AtomicBool>,
    handle: Mutex<Option<(Thread, JoinHandle<()>)>>,
}

impl AppDriver {
    /// Creates a new idle `AppDriver`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_runtime::timer::AppDriver;
    ///
    /// let driver = AppDriver::new();
    /// ```
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            handle: Mutex::new(None),
        }
    }
}

impl Default for AppDriver {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TimerDriver for AppDriver {
    fn start(&self, config: TimerConfig, callback: Box<dyn Fn() + Send + Sync + 'static>) {
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);
        let interval = config.interval;
        let single_shot = config.single_shot;
        // Wrap callback in Arc so we can clone it per tick for the FnOnce post.
        let cb: Arc<dyn Fn() + Send + Sync + 'static> = Arc::from(callback);

        let join = thread::spawn(move || {
            loop {
                thread::park_timeout(interval);
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                let cb_clone = Arc::clone(&cb);
                if let Some(app) = crate::application::Application::global() {
                    app.post_event(Box::new(move || cb_clone()));
                }
                if single_shot {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
            }
        });

        let thread_handle = join.thread().clone();
        *self.handle.lock().unwrap_or_else(|e| e.into_inner()) = Some((thread_handle, join));
    }

    fn stop(&self, _id: ObjectId) {
        self.running.store(false, Ordering::SeqCst);
        if let Some((thread_handle, join)) =
            self.handle.lock().unwrap_or_else(|e| e.into_inner()).take()
        {
            thread_handle.unpark();
            let _ = join.join();
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// PoolDriver
// ────────────────────────────────────────────────────────────────────────────

/// All mutable pool state, protected by a single `Mutex` to avoid lock-ordering questions.
struct PoolState {
    /// Min-heap of `(deadline, timer_id)` pairs (`Reverse` makes `BinaryHeap` a min-heap).
    heap: BinaryHeap<Reverse<(Instant, ObjectId)>>,
    /// Maps `timer_id` → callback.
    callbacks: HashMap<ObjectId, Arc<dyn Fn() + Send + Sync>>,
    /// Maps `timer_id` → interval for re-scheduling repeating timers.
    intervals: HashMap<ObjectId, Duration>,
    /// Maps `timer_id` → `single_shot` flag.
    single_shots: HashMap<ObjectId, bool>,
}

impl PoolState {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            callbacks: HashMap::new(),
            intervals: HashMap::new(),
            single_shots: HashMap::new(),
        }
    }
}

struct PoolInner {
    state: Mutex<PoolState>,
    condvar: Condvar,
    running: AtomicBool,
}

/// Single shared background thread + min-heap, driving multiple timers.
///
/// Multiple [`Timer`](crate::timer::Timer)s can share one `Arc<PoolDriver>`. The pool
/// dispatches ticks using a `BinaryHeap` of `(Instant, ObjectId)` deadlines and a single
/// background thread.
///
/// # Examples
///
/// ```no_run
/// use std::{sync::Arc, time::Duration};
/// use quartzite_runtime::timer::{Timer, PoolDriver, TimerDriver};
///
/// let pool: Arc<dyn TimerDriver> = Arc::new(PoolDriver::new());
///
/// let mut t1 = Timer::new(Duration::from_millis(50));
/// t1.connect_tick(|args| println!("t1 tick #{}", args.0));
/// t1.start(Arc::clone(&pool));
///
/// let mut t2 = Timer::new(Duration::from_millis(80));
/// t2.connect_tick(|args| println!("t2 tick #{}", args.0));
/// t2.start(Arc::clone(&pool));
///
/// std::thread::sleep(Duration::from_millis(400));
/// t1.stop();
/// t2.stop();
/// ```
pub struct PoolDriver {
    inner: Arc<PoolInner>,
}

impl PoolDriver {
    /// Creates a new `PoolDriver` and spawns its background scheduling thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_runtime::timer::PoolDriver;
    ///
    /// let _pool = PoolDriver::new();
    /// ```
    pub fn new() -> Self {
        let inner = Arc::new(PoolInner {
            state: Mutex::new(PoolState::new()),
            condvar: Condvar::new(),
            running: AtomicBool::new(true),
        });

        let inner_clone = Arc::clone(&inner);
        thread::spawn(move || Self::pool_loop(&inner_clone));

        Self { inner }
    }

    fn pool_loop(inner: &PoolInner) {
        loop {
            let mut guard = inner.state.lock().unwrap_or_else(|e| e.into_inner());

            // Wait while the heap is empty.
            while guard.heap.is_empty() {
                if !inner.running.load(Ordering::SeqCst) {
                    return;
                }
                guard = inner.condvar.wait(guard).unwrap_or_else(|e| e.into_inner());
            }

            if !inner.running.load(Ordering::SeqCst) {
                return;
            }

            // Peek at the earliest deadline.
            let now = Instant::now();
            let Some(&Reverse((deadline, _))) = guard.heap.peek() else {
                continue;
            };

            if deadline > now {
                let wait = deadline - now;
                let (new_guard, _) = inner
                    .condvar
                    .wait_timeout(guard, wait)
                    .unwrap_or_else(|e| e.into_inner());
                guard = new_guard;
                // Re-check emptiness and deadline from the top.
                continue;
            }

            // Pop the earliest entry.
            let Some(Reverse((_, id))) = guard.heap.pop() else {
                continue;
            };

            // Skip entries for cancelled timers (id absent from callbacks map).
            let callback = match guard.callbacks.get(&id) {
                Some(cb) => Arc::clone(cb),
                None => continue,
            };
            let interval = guard.intervals.get(&id).copied();
            let single_shot = guard.single_shots.get(&id).copied().unwrap_or(false);

            if single_shot {
                // Remove all state so it won't fire again.
                guard.callbacks.remove(&id);
                guard.intervals.remove(&id);
                guard.single_shots.remove(&id);
            } else if let Some(iv) = interval {
                // Re-push with new deadline before dropping the lock.
                guard.heap.push(Reverse((Instant::now() + iv, id)));
            }

            // Drop the lock before invoking the callback.
            drop(guard);

            callback();
        }
    }
}

impl Default for PoolDriver {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TimerDriver for PoolDriver {
    fn start(&self, config: TimerConfig, callback: Box<dyn Fn() + Send + Sync + 'static>) {
        let deadline = Instant::now() + config.interval;
        let id = config.timer_id;
        let cb: Arc<dyn Fn() + Send + Sync> = Arc::from(callback);

        let mut guard = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());

        guard.heap.push(Reverse((deadline, id)));
        guard.callbacks.insert(id, cb);
        guard.intervals.insert(id, config.interval);
        guard.single_shots.insert(id, config.single_shot);

        drop(guard);
        self.inner.condvar.notify_one();
    }

    fn stop(&self, id: ObjectId) {
        let mut guard = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());

        // Remove all per-timer state; stale heap entries will be discarded at pop time
        // because the id is no longer present in the callbacks map.
        guard.callbacks.remove(&id);
        guard.intervals.remove(&id);
        guard.single_shots.remove(&id);

        drop(guard);
        // Wake the pool thread so it skips the stale heap entry promptly.
        self.inner.condvar.notify_one();
    }
}

impl Drop for PoolDriver {
    fn drop(&mut self) {
        self.inner.running.store(false, Ordering::SeqCst);
        self.inner.condvar.notify_all();
        // Background thread exits on next condvar wake.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_driver_new_is_idle() {
        let d = ThreadDriver::new();
        assert!(!d.running.load(Ordering::Relaxed));
    }

    #[test]
    fn pool_driver_new_starts_background_thread() {
        let d = PoolDriver::new();
        assert!(d.inner.running.load(Ordering::Relaxed));
    }
}
