//! AC3: cross-thread Auto with no dispatcher installed → silent drop, no panic.
//!
//! This binary deliberately never calls `set_queued_dispatcher`, so the
//! `QUEUED_DISPATCHER` `OnceLock` is empty for the entire process lifetime.
//! That guarantees isolation from the unit-test binary, which may install a
//! `TestDispatcher`.

#[cfg(feature = "std")]
use quartzite_core::signal::Signal;

#[test]
#[cfg(feature = "std")]
fn auto_cross_thread_no_dispatcher_silent_drop() {
    // Obtain a ThreadId that is guaranteed to differ from the current thread.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        tx.send(std::thread::current().id()).unwrap();
    })
    .join()
    .unwrap();
    let foreign_id = rx.recv().unwrap();

    // Callback panics if called — verifies it is never invoked.
    let mut sig: Signal<(i32,)> = Signal::new();
    sig.connect_auto(foreign_id, |_| {
        panic!("slot must not be called when no dispatcher is installed");
    });

    // Must not panic, must not error.
    sig.emit(&(99,));
}
