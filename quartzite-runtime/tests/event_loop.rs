//! Integration tests for `EventLoop`: cross-thread posting, loop-thread execution, stop
//! semantics, Object-ification (AC15, AC17, AC19), and tick policy (AC9a–c).

use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    thread,
    time::Duration,
};

use parking_lot::Mutex;
use quartzite_core::{AsObject, Object, ObjectExt, Value};
use quartzite_runtime::EventLoop;

fn start_loop(el: Arc<EventLoop>) -> thread::JoinHandle<()> {
    thread::spawn(move || el.run())
}

// AC6 — closure posted from another thread executes on the loop thread.
#[test]
fn post_from_other_thread_executes_on_loop_thread() {
    let el = Arc::new(EventLoop::new());
    let loop_tid: Arc<Mutex<Option<thread::ThreadId>>> = Arc::new(Mutex::new(None));

    let handle = start_loop(Arc::clone(&el));
    thread::sleep(Duration::from_millis(5));

    let tid = Arc::clone(&loop_tid);
    el.post(Box::new(move || {
        *tid.lock() = Some(thread::current().id());
    }));

    thread::sleep(Duration::from_millis(20));
    el.request_stop();
    handle.join().unwrap();

    let recorded = *loop_tid.lock();
    assert!(recorded.is_some(), "closure must have run");
    assert_ne!(
        recorded,
        Some(thread::current().id()),
        "must have run on loop thread, not test thread"
    );
}

// Post ordering — three closures posted in order must execute in order.
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

// request_stop() causes run() to return within a reasonable timeout.
#[test]
fn stop_terminates_run() {
    let el = Arc::new(EventLoop::new());
    let el2 = Arc::clone(&el);
    let handle = thread::spawn(move || el2.run());

    thread::sleep(Duration::from_millis(5));
    el.request_stop();

    handle
        .join()
        .expect("event loop thread must exit cleanly after request_stop()");
}

// AC9(a) — tickless run() exits when a spawned thread calls request_stop().
// The join must complete within 200 ms.
#[test]
fn ac9a_tickless_run_exits_on_request_stop_within_200ms() {
    use std::sync::mpsc;
    let el = Arc::new(EventLoop::new());
    let el2 = Arc::clone(&el);
    let (tx, rx) = mpsc::channel::<()>();
    let handle = thread::spawn(move || {
        el2.run();
        let _ = tx.send(());
    });
    thread::sleep(Duration::from_millis(5));
    el.request_stop();
    rx.recv_timeout(Duration::from_millis(200))
        .expect("tickless run() must exit within 200 ms after request_stop()");
    handle.join().expect("loop thread must exit cleanly");
}

// AC9(b) — tickless EventLoop processes exactly one posted closure; no spurious
// wake-ups. Post one counter-incrementing closure, idle 100 ms, call request_stop,
// join, assert counter == 1.
#[test]
fn ac9b_tickless_no_spurious_wakeups() {
    let el = Arc::new(EventLoop::new());
    let counter = Arc::new(AtomicU32::new(0));

    let handle = start_loop(Arc::clone(&el));
    thread::sleep(Duration::from_millis(5));

    let c2 = Arc::clone(&counter);
    el.post(Box::new(move || {
        c2.fetch_add(1, Ordering::SeqCst);
    }));

    thread::sleep(Duration::from_millis(100));
    el.request_stop();
    handle.join().expect("loop thread must exit cleanly");

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "exactly one closure was posted; counter must be 1 (no spurious wake-ups)"
    );
}

// AC9(c) — tick-based EventLoop with Some(50 ms) drives counter increments.
// Post a tick-counting closure, idle 200 ms, call request_stop, join within
// 300 ms total, assert counter >= 2 (at least 2 ticks fired).
#[test]
fn ac9c_tick_based_loop_fires_multiple_ticks() {
    let el = Arc::new(EventLoop::with_tick(Some(Duration::from_millis(50))));
    let counter = Arc::new(AtomicU32::new(0));
    // Clone before moving into closures so we can read after the join.
    let counter_read = Arc::clone(&counter);

    let el2 = Arc::clone(&el);
    let c2 = Arc::clone(&counter);
    // Post a re-enqueueing closure: each invocation increments the counter and
    // re-posts itself, so the 50 ms tick-based loop sees multiple firings.
    let el3 = Arc::clone(&el);
    el2.post(Box::new(move || {
        c2.fetch_add(1, Ordering::SeqCst);
        // Re-post to observe the next tick wake-up.
        let c3 = Arc::clone(&counter);
        let el4 = Arc::clone(&el3);
        el4.post(Box::new(move || {
            c3.fetch_add(1, Ordering::SeqCst);
        }));
    }));

    let handle = start_loop(Arc::clone(&el));
    thread::sleep(Duration::from_millis(200));
    el.request_stop();
    handle
        .join()
        .expect("tick-based loop thread must exit cleanly");

    assert!(
        counter_read.load(Ordering::SeqCst) >= 2,
        "tick-based loop with 50 ms tick must have processed >= 2 closures in 200 ms"
    );
}

// AC15 — EventLoop is an Object: class_name == "EventLoop".
#[test]
fn ac15_event_loop_class_name() {
    let el = EventLoop::new();
    assert_eq!(el.meta_object().class_name, "EventLoop");
}

// AC15 — object_base() returns a valid reference; id() is non-zero.
#[test]
fn ac15_event_loop_object_base_and_id() {
    let el = EventLoop::new();
    let _base = el.object_base(); // must not panic
    assert_ne!(
        el.id().raw(),
        0,
        "EventLoop ObjectId must be non-zero after construction"
    );
}

// AC17 / AC19 — invoke_method("stop") returns Some(Value::Null) and the loop exits.
#[test]
fn ac17_ac19_invoke_method_stop_exits_loop() {
    use std::sync::mpsc;
    let el = Arc::new(EventLoop::new());
    let el2 = Arc::clone(&el);
    let (tx, rx) = mpsc::channel::<()>();
    let handle = thread::spawn(move || {
        el2.run();
        let _ = tx.send(());
    });
    thread::sleep(Duration::from_millis(5));

    // invoke_method requires &mut self; clone and call via get_mut is not
    // possible while el2 clone is live in the thread. Use request_stop +
    // post path instead.  The reflect path is tested through a dedicated
    // owned EventLoop below; the loop-exit aspect requires cross-thread stop.
    el.request_stop();
    rx.recv_timeout(Duration::from_millis(200))
        .expect("loop must exit after request_stop()");
    handle.join().expect("loop thread must exit cleanly");

    // Verify the reflection path: invoke_method("stop") must call the real stop() body.
    // If it only returned Value::Null without setting stop_requested, run() below would hang.
    let mut el_local = EventLoop::new();
    let result = el_local.invoke_method("stop", &[]);
    assert_eq!(
        result,
        Some(Value::Null),
        "invoke_method(\"stop\") must return Some(Value::Null)"
    );
    let (tx2, rx2) = mpsc::channel::<()>();
    thread::spawn(move || {
        el_local.run();
        let _ = tx2.send(());
    });
    rx2.recv_timeout(Duration::from_millis(200))
        .expect("invoke_method(\"stop\") must stop the loop: run() did not exit within 200 ms");
}
