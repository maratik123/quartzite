//! [`WidgetRoot`] — per-window widget dispatch target.

use quartzite_events::{KeyEvent, MouseEvent};
use quartzite_geometry::Size;
use quartzite_paint_api::Painter;

/// Dispatch target for events routed to a single window.
///
/// Implement this trait and pass your implementation to
/// [`WindowRegistry::try_create_window`] to receive events for that window.
///
/// The `paint` receiver is `&self` to match `WidgetExt::paint` in
/// `quartzite-widgets`. State mutation belongs to the lifecycle hooks
/// (`on_resize`, `on_mouse_press`, etc.); `paint` is a read-only projection
/// of the widget's current state onto the [`Painter`]. If a widget needs
/// interior mutability during paint (e.g. caching tessellated paths), use
/// `Cell` / `RefCell` inside the implementation — do not escalate the trait
/// receiver to `&mut self`.
///
/// [`WindowRegistry::try_create_window`]: crate::window_registry::WindowRegistry::try_create_window
///
/// # Examples
///
/// ```
/// use quartzite_renderer::WidgetRoot;
/// use quartzite_events::{KeyEvent, MouseEvent};
/// use quartzite_geometry::Size;
/// use quartzite_paint_api::Painter;
///
/// struct NoOpRoot;
///
/// impl WidgetRoot for NoOpRoot {
///     fn paint(&self, _painter: &mut dyn Painter) {}
///     fn on_resize(&mut self, _size: Size) {}
///     fn on_mouse_press(&mut self, _event: &MouseEvent) {}
///     fn on_mouse_release(&mut self, _event: &MouseEvent) {}
///     fn on_key_press(&mut self, _event: &KeyEvent) {}
///     fn on_key_release(&mut self, _event: &KeyEvent) {}
/// }
/// ```
pub trait WidgetRoot: 'static {
    /// Paints the window content using the provided [`Painter`].
    ///
    /// Called in response to [`winit::event::WindowEvent::RedrawRequested`].
    /// The receiver is `&self` because painting is a read-only projection of
    /// widget state; use `Cell`/`RefCell` for any interior caching.
    ///
    /// # Parameters
    ///
    /// - `painter`: the backend painter for this frame; valid only for the
    ///   duration of this call.
    fn paint(&self, painter: &mut dyn Painter);

    /// Called when the window is resized.
    ///
    /// # Parameters
    ///
    /// - `size`: new client-area size in physical pixels.
    fn on_resize(&mut self, size: Size);

    /// Called when a mouse button is pressed inside the window.
    ///
    /// # Parameters
    ///
    /// - `event`: describes the button, position, and modifier state.
    fn on_mouse_press(&mut self, event: &MouseEvent);

    /// Called when a mouse button is released inside the window.
    ///
    /// # Parameters
    ///
    /// - `event`: describes the button, position, and modifier state.
    fn on_mouse_release(&mut self, event: &MouseEvent);

    /// Called when a key is pressed while the window has focus.
    ///
    /// # Parameters
    ///
    /// - `event`: describes the key and modifier state.
    fn on_key_press(&mut self, event: &KeyEvent);

    /// Called when a key is released while the window has focus.
    ///
    /// # Parameters
    ///
    /// - `event`: describes the key and modifier state.
    fn on_key_release(&mut self, event: &KeyEvent);
}
