//! Integration tests for `EventLoop`: cross-thread posting, loop-thread execution, and stop semantics.

use std::{sync::Arc, thread, time::Duration};

use parking_lot::Mutex;
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
    el.stop();
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
    el.stop();
    handle.join().unwrap();

    assert_eq!(*log.lock(), vec![1, 2, 3]);
}

// stop() causes run() to return within a reasonable timeout.
#[test]
fn stop_terminates_run() {
    let el = Arc::new(EventLoop::new());
    let el2 = Arc::clone(&el);
    let handle = thread::spawn(move || el2.run());

    thread::sleep(Duration::from_millis(5));
    el.stop();

    handle
        .join()
        .expect("event loop thread must exit cleanly after stop()");
}
