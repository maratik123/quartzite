//! [`WindowedAppHandler`] — user-facing lifecycle trait for windowed apps.

use crate::window_registry::WindowRegistry;

/// Lifecycle callbacks for a windowed application.
///
/// Pass an implementation to [`WindowedApplication::run`] instead of a raw
/// `winit::ApplicationHandler`. The framework manages window registry fan-out
/// and calls these hooks at the right moments.
///
/// Both methods receive `&mut WindowRegistry` so the implementation can call
/// [`WindowRegistry::try_create_window`] and inspect live windows.
///
/// [`WindowedApplication::run`]: crate::application::WindowedApplication::run
///
/// # Examples
///
/// ```no_run
/// use quartzite_renderer::{WindowedAppHandler, WindowRegistry};
///
/// struct MyApp;
///
/// impl WindowedAppHandler for MyApp {
///     fn on_start(&mut self, _registry: &mut WindowRegistry) {
///         // create initial windows here
///     }
/// }
/// ```
pub trait WindowedAppHandler {
    /// Called once the winit event loop is ready to create windows.
    ///
    /// Equivalent to `winit::ApplicationHandler::resumed`. Most callers create
    /// their initial windows here via [`WindowRegistry::try_create_window`].
    ///
    /// # Parameters
    ///
    /// - `registry`: the live window registry; use it to create and inspect
    ///   windows for this session.
    fn on_start(&mut self, registry: &mut WindowRegistry);

    /// Called whenever the window registry becomes empty (all windows closed).
    ///
    /// Default implementation is a no-op. Useful when
    /// `quit_on_last_window_closed` is `false` — the caller can react to the
    /// last-window-closed event (e.g. show a re-open dialog or explicitly exit
    /// via [`Application::quit`]).
    ///
    /// # Parameters
    ///
    /// - `registry`: the live window registry; at the time of this call it is
    ///   empty (`registry.windows().next().is_none()`).
    ///
    /// [`Application::quit`]: quartzite_runtime::Application::quit
    #[inline]
    fn on_last_window_closed(&mut self, _registry: &mut WindowRegistry) {}
}
