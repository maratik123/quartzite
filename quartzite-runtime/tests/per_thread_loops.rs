//! Integration tests for per-thread event loops (AC12, AC13).
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use parking_lot::Mutex;
use quartzite_core::signal::QueuedDispatcher;
use quartzite_runtime::{ConnectionTable, EventLoop};

// AC12 — closure posted via QueuedDispatcher to a worker thread executes on that thread's loop.
#[test]
fn queued_dispatch_executes_on_worker_thread() {
    // Spawn a worker loop; capture its ThreadId via a channel inside `f`.
    let (tid_tx, tid_rx) = mpsc::sync_channel::<thread::ThreadId>(1);
    let (el, handle) = EventLoop::spawn(move || {
        tid_tx.send(thread::current().id()).unwrap();
    })
    .unwrap();

    let worker_tid = tid_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_ne!(
        worker_tid,
        thread::current().id(),
        "worker thread must differ from test thread"
    );

    // Post a closure that records which thread executes it.
    let executed_on: Arc<Mutex<Option<thread::ThreadId>>> = Arc::new(Mutex::new(None));
    let executed_on2 = Arc::clone(&executed_on);
    let table = ConnectionTable::new();
    table.post(
        worker_tid,
        Box::new(move || {
            *executed_on2.lock() = Some(thread::current().id());
        }),
    );

    // Give the worker loop time to process the closure, then stop it.
    thread::sleep(Duration::from_millis(50));
    el.stop();
    handle.join().expect("worker thread must exit cleanly");

    let recorded = *executed_on.lock();
    assert_eq!(
        recorded,
        Some(worker_tid),
        "closure must have executed on the worker thread's loop"
    );
}

// AC13 — posting to a deregistered thread logs a warning and drops the closure.
#[test]
fn queued_dispatch_to_deregistered_thread_drops_closure() {
    // Spawn a worker loop and record its ThreadId.
    let (tid_tx, tid_rx) = mpsc::sync_channel::<thread::ThreadId>(1);
    let (el, handle) = EventLoop::spawn(move || {
        tid_tx.send(thread::current().id()).unwrap();
    })
    .unwrap();

    let worker_tid = tid_rx.recv_timeout(Duration::from_secs(2)).unwrap();

    // Stop the loop and join so the thread is gone and the registry entry is removed.
    el.stop();
    handle.join().expect("worker thread must exit cleanly");

    // Now post to the deregistered ThreadId — closure must NOT execute.
    let executed = Arc::new(AtomicBool::new(false));
    let executed2 = Arc::clone(&executed);
    let table = ConnectionTable::new();
    table.post(
        worker_tid,
        Box::new(move || {
            executed2.store(true, Ordering::Relaxed);
        }),
    );

    // Give a moment to confirm no execution occurs.
    thread::sleep(Duration::from_millis(20));
    assert!(
        !executed.load(Ordering::Relaxed),
        "closure must not execute when target thread has no registered loop"
    );
}
