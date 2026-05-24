//! [`WindowedApplicationBuilder`] — builder for [`WindowedApplication`].

use std::time::Duration;

use quartzite_runtime::Application;
use winit::event_loop::EventLoop;

use crate::RendererError;
use crate::application::WindowedApplication;

/// Custom user events for the winit event loop.
///
/// Allows [`WindowedApplication::event_proxy`] callers to send cross-thread or
/// deferred requests into the event loop. Currently only [`Exit`][AppEvent::Exit]
/// is defined; additional variants can be added as the API evolves.
///
/// # Examples
///
/// ```no_run
/// # use quartzite_renderer::WindowedApplication;
/// # let app = WindowedApplication::builder().build().unwrap();
/// let proxy = app.event_proxy();
/// // In another thread or deferred callback:
/// proxy.send_event(quartzite_renderer::AppEvent::Exit).ok();
/// ```
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Requests that the event loop stop and [`WindowedApplication::run`] return.
    ///
    /// [`WindowedApplication::run`]: crate::application::WindowedApplication::run
    Exit,
}

/// Builder for [`WindowedApplication`].
///
/// Obtain via [`WindowedApplication::builder`].
///
/// By default the inner quartzite-runtime [`EventLoop`](quartzite_runtime::EventLoop) is
/// **tickless** — it blocks on `recv()` until a closure arrives. Use
/// [`tick_duration`](Self::tick_duration) to request a tick-based loop instead.
///
/// # Examples
///
/// ```no_run
/// use quartzite_renderer::WindowedApplication;
///
/// let app = WindowedApplication::builder()
///     .quit_on_last_window_closed(false)
///     .build()
///     .expect("failed to create application");
/// ```
#[must_use = "call `.build()` to construct the WindowedApplication"]
pub struct WindowedApplicationBuilder {
    quit_on_last_window_closed: bool,
    /// Tick duration forwarded to the inner quartzite-runtime [`EventLoop`](quartzite_runtime::EventLoop).
    ///
    /// `None` (the default) produces a tickless loop; `Some(d)` produces a tick-based loop.
    tick: Option<Duration>,
    /// When `true`, passes `with_any_thread(true)` to both the X11 and Wayland
    /// `EventLoop` builders on Linux. Used only in tests (xvfb + worker threads).
    #[cfg(target_os = "linux")]
    any_thread: bool,
}

impl WindowedApplicationBuilder {
    /// Creates a builder with default settings.
    ///
    /// Default: `quit_on_last_window_closed = true`, `tick = None` (tickless).
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            quit_on_last_window_closed: true,
            tick: None,
            #[cfg(target_os = "linux")]
            any_thread: false,
        }
    }

    /// Opts out of winit's main-thread check on Linux (X11 + Wayland).
    ///
    /// Pass `true` when constructing a `WindowedApplication` from a `cargo
    /// test` worker thread under `xvfb-run`. Production code should never
    /// call this — the main-thread check exists for a reason (macOS requires
    /// the event loop on the main thread; the check protects portability).
    ///
    /// Only available on Linux.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(target_os = "linux")]
    /// use quartzite_renderer::WindowedApplication;
    ///
    /// # #[cfg(target_os = "linux")]
    /// let app = WindowedApplication::builder()
    ///     .with_any_thread(true)
    ///     .build()
    ///     .expect("failed to create application");
    /// ```
    #[cfg(target_os = "linux")]
    #[inline]
    pub const fn with_any_thread(mut self, any_thread: bool) -> Self {
        self.any_thread = any_thread;
        self
    }

    /// Sets whether the event loop exits when the last window closes.
    ///
    /// Defaults to `true` (Qt-style). Pass `false` to keep the loop running
    /// after all windows are closed — useful for tray-icon or background apps
    /// that may have zero visible windows transiently.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::WindowedApplication;
    ///
    /// let app = WindowedApplication::builder()
    ///     .quit_on_last_window_closed(false)
    ///     .build()
    ///     .unwrap();
    /// ```
    #[inline]
    pub const fn quit_on_last_window_closed(mut self, quit: bool) -> Self {
        self.quit_on_last_window_closed = quit;
        self
    }

    /// Sets the tick duration for the inner quartzite-runtime
    /// [`EventLoop`](quartzite_runtime::EventLoop).
    ///
    /// - `None` → tickless (default): the event loop blocks on `recv()` until a closure
    ///   arrives — no spurious wake-ups when idle.
    /// - `Some(d)` → tick-based: the event loop wakes at most every `d` via
    ///   `recv_timeout(d)`.
    ///
    /// Passing `Some(Duration::ZERO)` is silently normalised to `None` (tickless) because
    /// a zero-duration timeout would busy-loop without doing useful work.
    ///
    /// This setting is independent of [`quit_on_last_window_closed`](Self::quit_on_last_window_closed)
    /// and the winit event-loop tick policy — it only governs the quartzite-runtime
    /// `EventLoop` that handles posted closures and signals.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_renderer::WindowedApplication;
    ///
    /// let app = WindowedApplication::builder()
    ///     .tick_duration(Some(Duration::from_millis(50)))
    ///     .build()
    ///     .expect("failed to create application");
    /// ```
    pub const fn tick_duration(mut self, tick: Option<Duration>) -> Self {
        self.tick = match tick {
            Some(d) if d.is_zero() => None,
            other => other,
        };
        self
    }

    /// Builds a [`WindowedApplication`].
    ///
    /// Initialises the quartzite [`Application`] singleton and the winit
    /// [`EventLoop`]. Fails if the singleton is already taken or if the
    /// display server is unavailable.
    ///
    /// # Errors
    ///
    /// - [`RendererError::Application`] — singleton already live.
    /// - [`RendererError::EventLoop`] — winit `EventLoop::new()` failed.
    pub fn build(self) -> Result<WindowedApplication, RendererError> {
        let app = Application::builder().tick_duration(self.tick).build()?;
        let mut builder = EventLoop::<AppEvent>::with_user_event();
        #[cfg(target_os = "linux")]
        if self.any_thread {
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            use winit::platform::x11::EventLoopBuilderExtX11;
            EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
            EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);
        }
        let event_loop = builder.build()?;
        let instance = wgpu::Instance::default();
        Ok(WindowedApplication::from_parts(
            app,
            event_loop,
            instance,
            self.quit_on_last_window_closed,
        ))
    }
}

#[cfg(test)]
mod tests {
    // `ApplicationError` and `RendererError` are only referenced by the
    // `#[cfg(target_os = "linux")]`-gated `build_result_is_ok_or_already_exists`
    // test below, so they appear unused on non-Linux targets.
    #![cfg_attr(not(target_os = "linux"), allow(unused_imports))]

    use quartzite_runtime::ApplicationError;

    use super::*;
    use crate::RendererError;

    #[test]
    fn default_builder_has_quit_on_last_window_closed_true() {
        let builder = WindowedApplicationBuilder::new();
        assert!(builder.quit_on_last_window_closed);
    }

    #[test]
    fn builder_opt_out() {
        let builder = WindowedApplicationBuilder::new().quit_on_last_window_closed(false);
        assert!(!builder.quit_on_last_window_closed);
    }

    #[test]
    fn default_tick_is_none() {
        let builder = WindowedApplicationBuilder::new();
        assert!(builder.tick.is_none());
    }

    #[test]
    fn tick_duration_set() {
        let builder =
            WindowedApplicationBuilder::new().tick_duration(Some(Duration::from_millis(50)));
        assert_eq!(builder.tick, Some(Duration::from_millis(50)));
    }

    #[test]
    fn tick_duration_zero_normalises_to_none() {
        let builder = WindowedApplicationBuilder::new().tick_duration(Some(Duration::ZERO));
        assert!(builder.tick.is_none());
    }

    #[test]
    fn tick_duration_none_leaves_tickless() {
        let builder = WindowedApplicationBuilder::new().tick_duration(None);
        assert!(builder.tick.is_none());
    }

    #[test]
    fn tick_and_quit_compose_independently() {
        let builder = WindowedApplicationBuilder::new()
            .tick_duration(Some(Duration::from_millis(100)))
            .quit_on_last_window_closed(false);
        assert_eq!(builder.tick, Some(Duration::from_millis(100)));
        assert!(!builder.quit_on_last_window_closed);
    }

    // `build()` creates a winit `EventLoop`; winit enforces a main-thread check
    // on all platforms. `cargo test` runs tests on worker threads, so the test
    // must set `with_any_thread(true)` on Linux. macOS and Windows have no
    // equivalent API — the build path is exercised there only when the test
    // binary is run single-threaded (or via the integration test on Linux).
    #[cfg(target_os = "linux")]
    #[test]
    fn build_result_is_ok_or_already_exists() {
        // Three outcomes are acceptable; anything else is a real bug:
        //   - Ok                          — happy path.
        //   - Application(AlreadyExists)  — singleton already taken by
        //     another test in this shared process (cargo test parallelism).
        //   - EventLoop(_)                — no X11/Wayland display server
        //     reachable (e.g. headless CI runner). Tripped PR #434 CI run
        //     25978212074 on ubuntu-latest where winit returns
        //     EventLoopError::Os.
        let result = WindowedApplicationBuilder::new()
            .with_any_thread(true)
            .build();
        let is_ok = result.is_ok();
        let is_already_exists = matches!(
            result,
            Err(RendererError::Application(ApplicationError::AlreadyExists))
        );
        let is_event_loop_error = matches!(result, Err(RendererError::EventLoop(_)));
        assert!(
            is_ok || is_already_exists || is_event_loop_error,
            "build() must return Ok, Application(AlreadyExists), or EventLoop(_)"
        );
    }
}
