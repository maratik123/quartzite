//! [`WindowedApplication`] — winit-backed singleton entry point.

use quartzite_runtime::Application;
use winit::application::ApplicationHandler;
use winit::event_loop::EventLoop;

use crate::RendererError;

/// A windowed application that owns both a quartzite [`Application`] and a
/// winit [`EventLoop`].
///
/// # Examples
///
/// ```no_run
/// use quartzite_renderer::WindowedApplication;
///
/// let app = WindowedApplication::new().expect("failed to create application");
/// ```
pub struct WindowedApplication {
    _app: Application,
    event_loop: EventLoop<()>,
}

impl WindowedApplication {
    /// Creates a new windowed application.
    ///
    /// # Errors
    ///
    /// Returns [`RendererError::Application`] if a [`quartzite_runtime::Application`]
    /// is already live in this process (singleton guard).
    ///
    /// Returns [`RendererError::EventLoop`] if the winit event loop cannot be
    /// created (e.g. no display server available).
    #[inline]
    pub fn new() -> Result<Self, RendererError> {
        fn inner() -> Result<WindowedApplication, RendererError> {
            let app = Application::new()?;
            let event_loop = EventLoop::new()?;
            Ok(WindowedApplication {
                _app: app,
                event_loop,
            })
        }
        inner()
    }

    /// Runs the winit event loop, handing control to `handler`.
    ///
    /// # Parameters
    ///
    /// - `handler`: winit [`ApplicationHandler`] that receives window and lifecycle events.
    ///
    /// # Errors
    ///
    /// Returns [`RendererError`] if the winit event loop exits with an error.
    ///
    /// # Panics
    ///
    /// Panics on some platforms (notably macOS) if not called from the main
    /// thread. Do not call from inside an async runtime — use a sync main
    /// entry point instead.
    pub fn run(self, mut handler: impl ApplicationHandler) -> Result<(), RendererError> {
        self.event_loop
            .run_app(&mut handler)
            .map_err(RendererError::from)
    }
}

#[cfg(test)]
mod tests {
    use quartzite_runtime::{Application, ApplicationError};

    use crate::RendererError;

    /// Verifies that `Application::new()` (the quartzite-runtime singleton) succeeds
    /// on first call — without constructing a winit `EventLoop`, which requires a
    /// display server and would fail in headless CI.
    #[test]
    fn quartzite_application_new_succeeds() {
        // This test is in a unit-test binary; OnceLock may already be taken by
        // a parallel test in the same binary, so only assert the two expected outcomes.
        let result = Application::new();
        assert!(
            result.is_ok() || matches!(result, Err(ApplicationError::AlreadyExists)),
            "Application::new() must return Ok or AlreadyExists"
        );
    }

    /// Verifies that `RendererError::Application` wraps `ApplicationError`.
    #[test]
    fn renderer_error_wraps_application_error() {
        let e = RendererError::Application(ApplicationError::AlreadyExists);
        assert_eq!(e.to_string(), "Application already exists");
    }
}
