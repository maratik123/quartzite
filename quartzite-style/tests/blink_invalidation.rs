//! Integration tests for [`DefaultStyle::start_blink_timer`].
//!
//! Exercises the caret-blink invalidation seam end-to-end using a synchronous
//! [`MockTimerDriver`] so that tick delivery is deterministic (no wall-clock
//! dependency, no thread spawning).
//!
//! `MockTimerDriver` is defined locally in this file: the production library
//! only exposes the `start_blink_timer` API; the driver fixture is a test
//! concern that does not belong in the production `quartzite-style` crate.
#![cfg(feature = "runtime-blink")]

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use parking_lot::Mutex;
use quartzite_runtime::{TimerConfig, TimerDriver};
use quartzite_style::DefaultStyle;

// ────────────────────────────────────────────────────────────────────────────
// MockTimerDriver — synchronous tick-firing test fixture
// ────────────────────────────────────────────────────────────────────────────

/// Synchronous in-process [`TimerDriver`] for testing.
///
/// Stores the `start` callback supplied by the [`Timer`] and fires it
/// synchronously when [`tick_now`] is called.  Dropping the [`Timer`] (which
/// calls `stop`) clears the stored callback — subsequent `tick_now` calls
/// become no-ops.
struct MockTimerDriver {
    callback: Mutex<Option<Box<dyn Fn() + Send + Sync + 'static>>>,
    last_interval: Mutex<Option<Duration>>,
}

impl MockTimerDriver {
    fn new() -> Self {
        Self {
            callback: Mutex::new(None),
            last_interval: Mutex::new(None),
        }
    }

    /// Synchronously fires the registered tick callback once.
    ///
    /// No-op if the [`Timer`] has already been dropped.
    fn tick_now(&self) {
        if let Some(cb) = self.callback.lock().as_ref() {
            cb();
        }
    }
}

impl TimerDriver for MockTimerDriver {
    fn start(&self, config: TimerConfig, callback: Box<dyn Fn() + Send + Sync + 'static>) {
        *self.last_interval.lock() = Some(config.interval);
        *self.callback.lock() = Some(callback);
    }

    fn stop(&self, _id: quartzite_core::ObjectId) {
        *self.callback.lock() = None;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn start_blink_timer_fires_callback_on_each_mock_tick() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = Arc::clone(&counter);
    let driver = Arc::new(MockTimerDriver::new());
    let _timer = DefaultStyle::new().start_blink_timer(
        Arc::clone(&driver) as _,
        Arc::new(move || {
            c.fetch_add(1, Ordering::SeqCst);
        }),
    );

    for _ in 0..5 {
        driver.tick_now();
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        5,
        "callback must fire once per tick_now call"
    );
}

#[test]
fn start_blink_timer_returns_timer_that_drops_cleanly() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = Arc::clone(&counter);
    let driver = Arc::new(MockTimerDriver::new());

    {
        let _timer = DefaultStyle::new().start_blink_timer(
            Arc::clone(&driver) as _,
            Arc::new(move || {
                c.fetch_add(1, Ordering::SeqCst);
            }),
        );
        // Timer is alive: tick should fire.
        driver.tick_now();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
    // Timer dropped: subsequent ticks must be no-ops.
    driver.tick_now();
    driver.tick_now();
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "dropped timer must not fire the callback"
    );
}

#[test]
fn start_blink_timer_interval_is_530ms() {
    let driver = Arc::new(MockTimerDriver::new());
    let _timer = DefaultStyle::new().start_blink_timer(Arc::clone(&driver) as _, Arc::new(|| {}));

    let interval = driver
        .last_interval
        .lock()
        .expect("driver must have been started");
    assert_eq!(
        interval,
        Duration::from_millis(530),
        "blink timer interval must be 530 ms"
    );
}
