use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use quartzite_runtime::{EventLoop, Timer};

fn start_loop(el: Arc<EventLoop>) -> thread::JoinHandle<()> {
    thread::spawn(move || el.run())
}

// AC7 — 50ms timer fires at least once within 200ms.
#[test]
fn timer_fires_within_deadline() {
    let el = Arc::new(EventLoop::new());
    let handle = start_loop(Arc::clone(&el));

    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(50));
    timer.connect_timeout(move |_| {
        counter2.fetch_add(1, Ordering::SeqCst);
    });
    timer.start(el.sender());

    thread::sleep(Duration::from_millis(200));

    el.stop();
    handle.join().unwrap();
    drop(timer);

    assert!(
        counter.load(Ordering::SeqCst) >= 1,
        "timer must fire at least once within 200ms"
    );
}

// Stop prevents further emissions.
#[test]
fn timer_stop_prevents_further_emissions() {
    let el = Arc::new(EventLoop::new());
    let handle = start_loop(Arc::clone(&el));

    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(40));
    timer.connect_timeout(move |_| {
        counter2.fetch_add(1, Ordering::SeqCst);
    });
    timer.start(el.sender());

    // Wait for at least one fire.
    thread::sleep(Duration::from_millis(120));
    timer.stop();

    // Drain the event loop: any emission already queued before stop() will be
    // processed before we read the counter, eliminating a race between the
    // background thread posting and the event loop executing the post.
    let (drain_tx, drain_rx) = std::sync::mpsc::channel::<()>();
    el.post(Box::new(move || {
        let _ = drain_tx.send(());
    }));
    drain_rx.recv().unwrap();

    let count_at_stop = counter.load(Ordering::SeqCst);
    assert!(count_at_stop >= 1, "must have fired before stop");

    // Wait more — count must not increase.
    thread::sleep(Duration::from_millis(150));
    assert_eq!(
        counter.load(Ordering::SeqCst),
        count_at_stop,
        "no further emissions after stop"
    );

    el.stop();
    handle.join().unwrap();
}

// Single-shot fires exactly once.
#[test]
fn single_shot_fires_exactly_once() {
    let el = Arc::new(EventLoop::new());
    let handle = start_loop(Arc::clone(&el));

    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(50));
    timer.single_shot = true;
    timer.connect_timeout(move |_| {
        counter2.fetch_add(1, Ordering::SeqCst);
    });
    timer.start(el.sender());

    thread::sleep(Duration::from_millis(300));

    el.stop();
    handle.join().unwrap();
    drop(timer);

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "single-shot timer must fire exactly once"
    );
}
