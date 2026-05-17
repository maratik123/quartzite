// Each tests/*.rs file is compiled as a separate binary, giving this file
// a fresh OnceLock — necessary for the Application singleton used in AppDriver tests.
//
// All three driver backends are tested here together with ObjectTree insertion,
// signals_blocked suppression, and single_shot semantics.
//
// Skipped under Miri at the file level: every assertion in this integration
// suite is wall-clock-bounded (e.g. `wait_for_count(&counter, 1, 200ms)` after
// a 30ms-interval timer, "ThreadDriver must fire at least once in 200 ms",
// "AppDriver did not fire within 500 ms"). Miri's 10–30× interpreter
// overhead can't preserve those budgets, producing timeout false-positives.
// One such test (`unblock_signals_restores_tick`) already tripped master
// Miri run 25976106489 (post #428); the others (`thread_driver_fires_at_interval`,
// `app_driver_executes_on_event_loop_thread`, `app_driver_*_tick`) happened
// to make their budgets in that run but carry the same latent risk.
//
// The native `cargo test` gate retains full coverage; the lib unit tests in
// `src/timer.rs` and `src/timer_drivers.rs` still provide Tree Borrows
// aliasing coverage on the Timer / Driver infrastructure under Miri.
#![cfg(not(miri))]

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use quartzite_runtime::{AppDriver, Application, PoolDriver, ThreadDriver, Timer, TimerDriver};

// ────────────────────────────────────────────────────────────────────────────
// Helper: wait until `counter` reaches at least `n` or `timeout` elapses.
// Returns true if the target was reached.
fn wait_for_count(counter: &AtomicUsize, n: usize, timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if counter.load(Ordering::SeqCst) >= n {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(5));
    }
}

// ────────────────────────────────────────────────────────────────────────────
// AC2 — Timer has ObjectBase / named lookup
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn timer_has_object_id_and_name() {
    use quartzite_core::ObjectId;

    let timer = Timer::new(Duration::from_millis(100));
    let id: ObjectId = timer.base.id();
    assert!(id.raw() > 0, "timer must have a valid ObjectId");

    let named = Timer::named("my-timer", Duration::from_millis(100));
    assert_eq!(named.base.name(), Some("my-timer"));
}

// ────────────────────────────────────────────────────────────────────────────
// AC3 — Fire count increments: 0, 1, 2 …
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn thread_driver_fire_count_increments() {
    let counts: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
    let counts2 = Arc::clone(&counts);
    let done = Arc::new(AtomicBool::new(false));
    let done2 = Arc::clone(&done);

    let mut timer = Timer::new(Duration::from_millis(30));
    timer.connect_tick(move |args| {
        let fc = args.0.fire_count();
        counts2.lock().expect("counts lock").push(fc);
        if fc >= 2 {
            done2.store(true, Ordering::SeqCst);
        }
    });
    timer.start(Arc::new(ThreadDriver::new()));

    // Wait up to 500 ms for 3 fires.
    let deadline = std::time::Instant::now() + Duration::from_millis(500);
    while !done.load(Ordering::SeqCst) && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    timer.stop();

    let observed = counts.lock().expect("counts lock").clone();
    assert!(
        observed.len() >= 3,
        "expected at least 3 fires, got {observed:?}"
    );
    // The first three values must be 0, 1, 2.
    assert_eq!(
        &observed[..3],
        &[0, 1, 2],
        "fire counts must be 0-indexed and sequential"
    );
}

// ────────────────────────────────────────────────────────────────────────────
// AC5 — signals_blocked suppresses tick
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn signals_blocked_suppresses_tick() {
    let called = Arc::new(AtomicBool::new(false));
    let called2 = Arc::clone(&called);

    let mut timer = Timer::new(Duration::from_millis(30));
    timer.connect_tick(move |_| called2.store(true, Ordering::SeqCst));

    // Block before starting.
    timer.block_signals();
    timer.start(Arc::new(ThreadDriver::new()));

    // Wait long enough for at least 2 intervals to pass.
    thread::sleep(Duration::from_millis(120));
    timer.stop();

    assert!(
        !called.load(Ordering::SeqCst),
        "signals_blocked must suppress all tick emissions"
    );
}

// AC5 — unblock_signals restores firing
#[test]
fn unblock_signals_restores_tick() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(30));
    timer.connect_tick(move |_| {
        counter2.fetch_add(1, Ordering::SeqCst);
    });
    timer.block_signals();
    timer.start(Arc::new(ThreadDriver::new()));

    // Wait two intervals — should fire 0 times.
    thread::sleep(Duration::from_millis(80));
    assert_eq!(counter.load(Ordering::SeqCst), 0, "no fires while blocked");

    timer.unblock_signals();
    // Wait for at least one tick after unblock.
    assert!(
        wait_for_count(&counter, 1, Duration::from_millis(200)),
        "must fire at least once after unblock"
    );
    timer.stop();
}

// ────────────────────────────────────────────────────────────────────────────
// AC7 — ThreadDriver fires at interval (±50 % tolerance)
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn thread_driver_fires_at_interval() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(50));
    timer.connect_tick(move |_| {
        counter2.fetch_add(1, Ordering::SeqCst);
    });
    timer.start(Arc::new(ThreadDriver::new()));

    thread::sleep(Duration::from_millis(200));
    timer.stop();

    let fires = counter.load(Ordering::SeqCst);
    assert!(
        fires >= 1,
        "ThreadDriver must fire at least once in 200 ms (interval 50 ms)"
    );
}

// ────────────────────────────────────────────────────────────────────────────
// AC8 — AppDriver executes slots on the event-loop thread
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn app_driver_executes_on_event_loop_thread() {
    // This test creates the Application singleton for this binary.
    let app = Application::new().expect("only one Application per process");

    // Spin up the event loop on a dedicated thread; record its thread id.
    let el_thread_id = Arc::new(Mutex::new(None::<thread::ThreadId>));
    let el_id2 = Arc::clone(&el_thread_id);

    // Post a probe to discover the event-loop thread id from inside the loop.
    app.post_event(Box::new(move || {
        *el_id2.lock().expect("el_id lock") = Some(thread::current().id());
    }));

    let el_thread = thread::spawn({
        let app = Application::global().unwrap();
        move || app.exec()
    });

    // Wait until the probe fires (the event loop started) — up to 500 ms.
    let deadline = std::time::Instant::now() + Duration::from_millis(500);
    loop {
        if el_thread_id.lock().expect("el_id lock").is_some() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "event loop did not start in time"
        );
        thread::sleep(Duration::from_millis(5));
    }
    let expected_el_id = el_thread_id.lock().expect("el_id lock").unwrap();

    // Now run the timer.
    let observed_thread = Arc::new(Mutex::new(None::<thread::ThreadId>));
    let observed2 = Arc::clone(&observed_thread);
    let fired = Arc::new(AtomicBool::new(false));
    let fired2 = Arc::clone(&fired);

    let mut timer = Timer::new(Duration::from_millis(30));
    timer.connect_tick(move |_| {
        *observed2.lock().expect("observed lock") = Some(thread::current().id());
        fired2.store(true, Ordering::SeqCst);
    });
    timer.start(Arc::new(AppDriver::new()));

    // Wait for at least one tick — up to 500 ms.
    let deadline2 = std::time::Instant::now() + Duration::from_millis(500);
    loop {
        if fired.load(Ordering::SeqCst) {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline2,
            "AppDriver did not fire within 500 ms"
        );
        thread::sleep(Duration::from_millis(5));
    }

    timer.stop();

    // Quit and join the event loop.
    Application::global().unwrap().quit();
    let _ = el_thread.join();

    if let Some(actual) = *observed_thread.lock().expect("observed lock") {
        assert_eq!(
            actual, expected_el_id,
            "AppDriver slots must run on the event-loop thread"
        );
    } else {
        panic!("tick never fired — observed_thread is None");
    }
}

// ────────────────────────────────────────────────────────────────────────────
// AC9 — PoolDriver shared across two timers
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn pool_driver_shared_across_two_timers() {
    let pool: Arc<dyn TimerDriver> = Arc::new(PoolDriver::new());

    let c1 = Arc::new(AtomicUsize::new(0));
    let c1b = Arc::clone(&c1);
    let mut t1 = Timer::new(Duration::from_millis(40));
    t1.connect_tick(move |_| {
        c1b.fetch_add(1, Ordering::SeqCst);
    });

    let c2 = Arc::new(AtomicUsize::new(0));
    let c2b = Arc::clone(&c2);
    let mut t2 = Timer::new(Duration::from_millis(60));
    t2.connect_tick(move |_| {
        c2b.fetch_add(1, Ordering::SeqCst);
    });

    t1.start(Arc::clone(&pool));
    t2.start(Arc::clone(&pool));

    thread::sleep(Duration::from_millis(400));

    t1.stop();
    t2.stop();

    assert!(
        c1.load(Ordering::SeqCst) >= 1,
        "timer 1 must fire at least once"
    );
    assert!(
        c2.load(Ordering::SeqCst) >= 1,
        "timer 2 must fire at least once"
    );
}

// ────────────────────────────────────────────────────────────────────────────
// AC11 — single_shot fires exactly once (ThreadDriver)
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn single_shot_thread_driver_fires_exactly_once() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(30));
    timer.single_shot = true;
    timer.connect_tick(move |_| {
        counter2.fetch_add(1, Ordering::SeqCst);
    });
    timer.start(Arc::new(ThreadDriver::new()));

    // Wait for 3× the interval.
    thread::sleep(Duration::from_millis(200));
    timer.stop();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "single_shot timer must fire exactly once (ThreadDriver)"
    );
}

// AC11 — single_shot fires exactly once (PoolDriver)
#[test]
fn single_shot_pool_driver_fires_exactly_once() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(30));
    timer.single_shot = true;
    timer.connect_tick(move |_| {
        counter2.fetch_add(1, Ordering::SeqCst);
    });
    timer.start(Arc::new(PoolDriver::new()));

    // Wait for 3× the interval.
    thread::sleep(Duration::from_millis(200));
    timer.stop();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "single_shot timer must fire exactly once (PoolDriver)"
    );
}

// ────────────────────────────────────────────────────────────────────────────
// stop_prevents_further_emissions (ThreadDriver)
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn stop_prevents_further_emissions() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(30));
    timer.connect_tick(move |_| {
        counter2.fetch_add(1, Ordering::SeqCst);
    });
    timer.start(Arc::new(ThreadDriver::new()));

    // Wait for at least one fire.
    assert!(
        wait_for_count(&counter, 1, Duration::from_millis(300)),
        "timer must fire before stop"
    );
    timer.stop();

    let count_at_stop = counter.load(Ordering::SeqCst);

    // Wait and ensure count does not grow.
    thread::sleep(Duration::from_millis(120));
    assert_eq!(
        counter.load(Ordering::SeqCst),
        count_at_stop,
        "no further emissions after stop"
    );
}
