//! AC19(b) — signal-to-quit wiring test.
//!
//! Verifies that any `Signal<()>` can be wired to `Application::quit` via the
//! `&self` shim `request_quit`, with no mutable handle required in the closure.
//! Separated from `tests/application.rs` so both binaries get a fresh
//! `OnceLock` and there are no singleton races.

use std::{sync::mpsc, thread, time::Duration};

use quartzite_core::signal::Signal;
use quartzite_runtime::Application;

/// AC19(b): connect a `Signal<()>` to `Application::request_quit` and verify
/// that firing the signal stops a running event loop.
///
/// This is the representative pattern for "Quit-button clicked → app exits":
/// ```ignore
/// let mut signal = Signal::<()>::new();
/// signal.connect(|_| Application::global().unwrap().request_quit());
/// // … fire signal from wherever the button lives …
/// ```
#[test]
fn ac19b_signal_connects_to_application_quit() {
    let app = Application::builder().build().unwrap();
    let mut signal: Signal<()> = Signal::new();

    // Connect without capturing any Application handle — `global()` returns a
    // fresh reference on each call, making `&self` access straightforward.
    signal.connect(|()| {
        Application::global().unwrap().request_quit();
    });

    // Run the event loop on a background thread so we can fire the signal.
    let (tx, rx) = mpsc::channel::<()>();
    let handle = thread::spawn(move || {
        Application::global().unwrap().exec();
        let _ = tx.send(());
    });

    // Give exec() time to start blocking.
    thread::sleep(Duration::from_millis(5));

    // Fire the signal — must stop the loop.
    signal.emit_unconditionally(&());

    rx.recv_timeout(Duration::from_millis(200))
        .expect("signal must stop the Application event loop within 200 ms");
    handle.join().expect("exec() thread must exit cleanly");

    drop(app);
}
