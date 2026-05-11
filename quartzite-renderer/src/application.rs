//! [`WindowedApplication`] — winit-backed singleton entry point.

use quartzite_runtime::Application;
use winit::event_loop::{EventLoop, EventLoopProxy};

use crate::RendererError;
use crate::application_builder::{AppEvent, WindowedApplicationBuilder};
use crate::window_registry::WindowRegistry;
use crate::windowed_app_handler::WindowedAppHandler;
use crate::wrapped_handler::WrappedHandler;

/// A windowed application that owns a quartzite [`Application`] singleton and
/// a winit [`EventLoop`].
///
/// Construct via [`WindowedApplication::builder`].
///
/// # Examples
///
/// ```no_run
/// use quartzite_renderer::{WindowedApplication, WindowedAppHandler, WindowRegistry};
///
/// struct MyApp;
/// impl WindowedAppHandler for MyApp {
///     fn on_start(&mut self, _registry: &mut WindowRegistry) {}
/// }
///
/// WindowedApplication::builder()
///     .build()
///     .unwrap()
///     .run(MyApp)
///     .unwrap();
/// ```
pub struct WindowedApplication {
    _app: Application,
    event_loop: EventLoop<AppEvent>,
    instance: wgpu::Instance,
    quit_on_last_window_closed: bool,
}

impl WindowedApplication {
    /// Returns a builder for constructing a [`WindowedApplication`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::WindowedApplication;
    ///
    /// let app = WindowedApplication::builder().build().unwrap();
    /// ```
    #[inline]
    pub fn builder() -> WindowedApplicationBuilder {
        WindowedApplicationBuilder::new()
    }

    /// Constructs from already-initialised parts. Used only by
    /// [`WindowedApplicationBuilder::build`].
    #[inline]
    pub(crate) fn from_parts(
        app: Application,
        event_loop: EventLoop<AppEvent>,
        instance: wgpu::Instance,
        quit_on_last_window_closed: bool,
    ) -> Self {
        Self {
            _app: app,
            event_loop,
            instance,
            quit_on_last_window_closed,
        }
    }

    /// Returns an [`EventLoopProxy`] that can send [`AppEvent`]s into the
    /// running event loop from any thread.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::{WindowedApplication, AppEvent};
    ///
    /// let app = WindowedApplication::builder().build().unwrap();
    /// let proxy = app.event_proxy();
    /// // From another thread:
    /// proxy.send_event(AppEvent::Exit).ok();
    /// ```
    #[inline]
    pub fn event_proxy(&self) -> EventLoopProxy<AppEvent> {
        self.event_loop.create_proxy()
    }

    /// Runs the winit event loop, handing control to `handler`.
    ///
    /// Blocks until the event loop exits. The loop exits when:
    ///
    /// - All windows are closed **and** `quit_on_last_window_closed` is `true`
    ///   (the default).
    /// - [`AppEvent::Exit`] is sent via [`WindowedApplication::event_proxy`].
    /// - The platform sends a quit request.
    ///
    /// # Parameters
    ///
    /// - `handler`: the application lifecycle handler; receives `on_start` once
    ///   and `on_last_window_closed` whenever the window registry becomes empty.
    ///
    /// # Errors
    ///
    /// Returns [`RendererError::EventLoop`] if the winit event loop exits with
    /// an error.
    ///
    /// # Panics
    ///
    /// Panics on some platforms (notably macOS) if not called from the main
    /// thread.
    pub fn run(self, handler: impl WindowedAppHandler) -> Result<(), RendererError> {
        let registry = WindowRegistry::new(self.quit_on_last_window_closed, self.instance);
        let mut wrapped = WrappedHandler::new(registry, handler);
        self.event_loop
            .run_app(&mut wrapped)
            .map_err(RendererError::from)
    }
}

#[cfg(test)]
mod tests {
    use quartzite_runtime::ApplicationError;

    use crate::RendererError;

    use super::*;

    #[test]
    fn quartzite_application_new_succeeds() {
        let result = Application::new();
        assert!(
            result.is_ok() || matches!(result, Err(ApplicationError::AlreadyExists)),
            "Application::new() must return Ok or AlreadyExists"
        );
    }

    #[test]
    fn renderer_error_wraps_application_error() {
        let e = RendererError::Application(ApplicationError::AlreadyExists);
        assert_eq!(e.to_string(), "Application already exists");
    }
}
