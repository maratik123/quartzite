//! Integration test (AC11) verifying `Timer::single_shot` fires exactly once on the `AppDriver` backend, isolated in its own binary so the `Application` singleton is fresh.

// Separate binary so the Application singleton is fresh (no conflict with timer.rs).
// Tests AC11: single_shot fires exactly once when using AppDriver.
//
// Skipped under Miri at the file level: interpreter budget. The whole-file
// shape is required (not per-test `cfg_attr`) because every assertion is
// wall-clock-bounded (500 ms timeout after a 30 ms-interval timer); Miri's
// 10–30× interpreter overhead cannot preserve those budgets, producing
// timeout false-positives. Alternative coverage: native `cargo test` exercises
// this file on the standard test job.
#![cfg(not(miri))]

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use quartzite_runtime::{AppDriver, Application, Timer};

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

// AC11 — single_shot fires exactly once (AppDriver)
#[test]
fn single_shot_app_driver_fires_exactly_once() {
    let app = Application::builder()
        .build()
        .expect("only one Application per process");

    let el_thread = thread::spawn({
        let app = Application::global().unwrap();
        move || app.exec()
    });

    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(30));
    timer.single_shot = true;
    timer.connect_tick(move |_| {
        counter2.fetch_add(1, Ordering::SeqCst);
    });
    timer.start(Arc::new(AppDriver::new()));

    // Wait for the single fire — up to 500 ms.
    assert!(
        wait_for_count(&counter, 1, Duration::from_millis(500)),
        "AppDriver single_shot must fire at least once"
    );

    // Wait an extra 3× interval to confirm no second fire.
    thread::sleep(Duration::from_millis(120));

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "AppDriver single_shot must fire exactly once"
    );

    timer.stop();
    Application::global().unwrap().request_quit();
    let _ = el_thread.join();
    drop(app);
}
