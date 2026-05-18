//! Shared test helpers for quartzite-renderer integration tests.
//!
//! Each integration test file that uses these helpers must declare
//! `mod support;` at the top (a bare `tests/support.rs` would be compiled
//! as its own test binary with no test functions, which is wasteful and
//! confusing — the subdirectory form avoids that).

// Items in this module are only used by `#[cfg(target_os = "linux")]`-gated
// tests in the sibling integration-test files (`multi_window.rs`,
// `xvfb_smoke.rs`). Suppress dead_code / unused_imports on non-Linux rather
// than gating every item individually.
#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

use std::cell::Cell;
use std::sync::Arc;

use parking_lot::Mutex;
use quartzite_events::{KeyEvent, MouseEvent};
use quartzite_geometry::Size;
use quartzite_paint_api::Painter;
use quartzite_renderer::{AppEvent, WindowedApplication};
use winit::event_loop::EventLoopProxy;

/// Records which lifecycle events a window root received.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootEvent {
    /// `paint` was called.
    Paint,
    /// `on_resize` was called with the given size.
    Resize(Size),
    /// `on_mouse_press` was called.
    MousePress,
    /// `on_mouse_release` was called.
    MouseRelease,
    /// `on_key_press` was called.
    KeyPress,
    /// `on_key_release` was called.
    KeyRelease,
}

/// A [`WidgetRoot`] implementation that records every event it receives.
///
/// [`WidgetRoot`]: quartzite_renderer::WidgetRoot
pub struct RecordingRoot {
    pub records: Arc<Mutex<Vec<RootEvent>>>,
    // `paint` takes `&self`; use the shared records mutex (Mutex is Sync+Send)
    // to record from the shared-reference receiver.
    _painted: Cell<bool>,
}

impl RecordingRoot {
    /// Creates a new recording root sharing the given event log.
    pub fn new(records: Arc<Mutex<Vec<RootEvent>>>) -> Self {
        Self {
            records,
            _painted: Cell::new(false),
        }
    }
}

impl quartzite_renderer::WidgetRoot for RecordingRoot {
    fn paint(&self, _painter: &mut dyn Painter) {
        self.records.lock().push(RootEvent::Paint);
    }

    fn on_resize(&mut self, size: Size) {
        self.records.lock().push(RootEvent::Resize(size));
    }

    fn on_mouse_press(&mut self, _event: &MouseEvent) {
        self.records.lock().push(RootEvent::MousePress);
    }

    fn on_mouse_release(&mut self, _event: &MouseEvent) {
        self.records.lock().push(RootEvent::MouseRelease);
    }

    fn on_key_press(&mut self, _event: &KeyEvent) {
        self.records.lock().push(RootEvent::KeyPress);
    }

    fn on_key_release(&mut self, _event: &KeyEvent) {
        self.records.lock().push(RootEvent::KeyRelease);
    }
}

/// Sends an [`AppEvent::Exit`] through a proxy; ignores send errors (loop
/// may have already exited).
pub fn proxy_send_exit(proxy: &EventLoopProxy<AppEvent>) {
    proxy.send_event(AppEvent::Exit).ok();
}

/// Builds a [`WindowedApplication`] using the `with_any_thread` X11/Wayland
/// extensions so the call succeeds from a `cargo test` worker thread.
///
/// Returns `None` when:
/// - `SKIP_RENDER_SNAPSHOT` is set (CI bailout without display).
/// - The `Application` singleton is already held by another test.
/// - The event loop fails to build (no display server available).
///
/// Non-Linux platforms always return `None` — the xvfb pattern is
/// Linux-only.
#[allow(dead_code)]
pub fn build_test_app(quit_on_last_window_closed: bool) -> Option<WindowedApplication> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = quit_on_last_window_closed;
        None
    }
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("SKIP_RENDER_SNAPSHOT").is_some_and(|v| !v.is_empty()) {
            eprintln!("multi_window: SKIP_RENDER_SNAPSHOT set; skipping");
            return None;
        }
        WindowedApplication::builder()
            .quit_on_last_window_closed(quit_on_last_window_closed)
            .with_any_thread(true)
            .build()
            .ok()
    }
}
