//! [`WidgetExt`] — ergonomic blanket extension for all `AsWidget` types.

use quartzite_geometry::{Point, Rect, Size};
use quartzite_paint_api::Painter;

use quartzite_events::{KeyEvent, MouseEvent};

use crate::widget_base::AsWidget;

/// Convenience extension trait blanket-implemented for every [`AsWidget`] type.
///
/// Provides geometry accessors, visibility/enabled helpers, lifecycle hooks with
/// default no-op implementations, and `update()` to mark a pending repaint.
///
/// Override lifecycle hooks in your widget's `#[object_impl]` block (or a plain
/// `impl` block) — the defaults here are intentional no-ops.
pub trait WidgetExt: AsWidget {
    // ── geometry ──────────────────────────────────────────────────────────────

    /// Returns the bounding rectangle in parent coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert_eq!(w.geometry(), Default::default());
    /// ```
    #[inline]
    fn geometry(&self) -> Rect {
        self.widget_base().geometry
    }

    /// Sets the bounding rectangle.
    ///
    /// # Parameters
    ///
    /// - `rect`: new bounding rectangle in parent coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    /// use quartzite_geometry::{Point, Rect, Size};
    ///
    /// let mut w = WidgetBase::new();
    /// w.set_geometry(Rect::new(Point::new(10, 20), Size::new(100, 50)));
    /// assert_eq!(w.geometry().left(), 10);
    /// ```
    #[inline]
    fn set_geometry(&mut self, rect: Rect) {
        self.widget_base_mut().geometry = rect;
    }

    /// Returns the widget's top-left position in parent coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    /// use quartzite_geometry::Point;
    ///
    /// let w = WidgetBase::new();
    /// assert_eq!(w.pos(), Point::default());
    /// ```
    #[inline]
    fn pos(&self) -> Point {
        self.widget_base().geometry.origin()
    }

    /// Returns the widget's current size.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert_eq!(w.size(), Default::default());
    /// ```
    #[inline]
    fn size(&self) -> Size {
        self.widget_base().geometry.size()
    }

    /// Resizes the widget, keeping its current position.
    ///
    /// # Parameters
    ///
    /// - `size`: new width and height.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    /// use quartzite_geometry::Size;
    ///
    /// let mut w = WidgetBase::new();
    /// w.resize(Size::new(200, 100));
    /// assert_eq!(w.size(), Size::new(200, 100));
    /// ```
    #[inline]
    fn resize(&mut self, size: Size) {
        let origin = self.widget_base().geometry.origin();
        self.widget_base_mut().geometry = Rect::new(origin, size);
    }

    /// Moves the widget to `point`, keeping its current size.
    ///
    /// # Parameters
    ///
    /// - `point`: new top-left position in parent coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    /// use quartzite_geometry::Point;
    ///
    /// let mut w = WidgetBase::new();
    /// w.move_to(Point::new(50, 50));
    /// assert_eq!(w.pos(), Point::new(50, 50));
    /// ```
    #[inline]
    fn move_to(&mut self, point: Point) {
        let size = self.widget_base().geometry.size();
        self.widget_base_mut().geometry = Rect::new(point, size);
    }

    // ── visibility ────────────────────────────────────────────────────────────

    /// Shows the widget by setting `visible = true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.show();
    /// assert!(w.is_visible());
    /// ```
    #[inline]
    fn show(&mut self) {
        self.widget_base_mut().visible = true;
    }

    /// Hides the widget by setting `visible = false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.show();
    /// w.hide();
    /// assert!(!w.is_visible());
    /// ```
    #[inline]
    fn hide(&mut self) {
        self.widget_base_mut().visible = false;
    }

    /// Returns `true` when the widget is visible.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert!(!w.is_visible());
    /// ```
    #[inline]
    fn is_visible(&self) -> bool {
        self.widget_base().visible
    }

    /// Sets the visibility flag directly.
    ///
    /// # Parameters
    ///
    /// - `visible`: `true` to show, `false` to hide.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.set_visible(true);
    /// assert!(w.is_visible());
    /// ```
    #[inline]
    fn set_visible(&mut self, visible: bool) {
        self.widget_base_mut().visible = visible;
    }

    // ── enabled ───────────────────────────────────────────────────────────────

    /// Returns `true` when the widget accepts user input.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert!(w.is_enabled());
    /// ```
    #[inline]
    fn is_enabled(&self) -> bool {
        self.widget_base().enabled
    }

    /// Enables or disables the widget.
    ///
    /// # Parameters
    ///
    /// - `enabled`: `true` to enable, `false` to disable.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.set_enabled(false);
    /// assert!(!w.is_enabled());
    /// ```
    #[inline]
    fn set_enabled(&mut self, enabled: bool) {
        self.widget_base_mut().enabled = enabled;
    }

    // ── hovered / pressed / focused ───────────────────────────────────────────

    /// Returns `true` while the mouse cursor is over the widget's bounding rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// assert!(!w.is_hovered());
    /// w.set_hovered(true);
    /// assert!(w.is_hovered());
    /// ```
    #[inline]
    fn is_hovered(&self) -> bool {
        self.widget_base().hovered
    }

    /// Sets the hovered state flag.
    ///
    /// # Parameters
    ///
    /// - `value`: `true` when the cursor is over the widget, `false` when it leaves.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.set_hovered(true);
    /// assert!(w.is_hovered());
    /// ```
    #[inline]
    fn set_hovered(&mut self, value: bool) {
        self.widget_base_mut().hovered = value;
    }

    /// Returns `true` while a mouse button is held with press-initiated state on this widget.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// assert!(!w.is_pressed());
    /// w.set_pressed(true);
    /// assert!(w.is_pressed());
    /// ```
    #[inline]
    fn is_pressed(&self) -> bool {
        self.widget_base().pressed
    }

    /// Sets the pressed state flag.
    ///
    /// # Parameters
    ///
    /// - `value`: `true` when a press is initiated, `false` on release.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.set_pressed(true);
    /// assert!(w.is_pressed());
    /// ```
    #[inline]
    fn set_pressed(&mut self, value: bool) {
        self.widget_base_mut().pressed = value;
    }

    /// Returns `true` while this widget owns keyboard focus.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// assert!(!w.is_focused());
    /// w.set_focused(true);
    /// assert!(w.is_focused());
    /// ```
    #[inline]
    fn is_focused(&self) -> bool {
        self.widget_base().focused
    }

    /// Sets the focused state flag.
    ///
    /// # Parameters
    ///
    /// - `value`: `true` when focus is gained, `false` when lost.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.set_focused(true);
    /// assert!(w.is_focused());
    /// ```
    #[inline]
    fn set_focused(&mut self, value: bool) {
        self.widget_base_mut().focused = value;
    }

    // ── repaint ───────────────────────────────────────────────────────────────

    /// Marks the widget as needing a repaint on the next render pass.
    ///
    /// Sets `WidgetBase::pending_update` to `true`; the renderer consumes this flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{AsWidget, WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.update();
    /// assert!(w.widget_base().pending_update);
    /// ```
    #[inline]
    fn update(&mut self) {
        self.widget_base_mut().pending_update = true;
    }

    // ── size hints ────────────────────────────────────────────────────────────

    /// Returns the preferred size hint; defaults to `Size::default()` (zero).
    ///
    /// Override in concrete widgets to return a meaningful hint.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert_eq!(w.size_hint(), Default::default());
    /// ```
    #[inline]
    fn size_hint(&self) -> Size {
        Size::default()
    }

    /// Returns the minimum size from `WidgetBase::min_size`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{AsWidget, WidgetBase, WidgetExt};
    /// use quartzite_geometry::Size;
    ///
    /// let mut w = WidgetBase::new();
    /// w.widget_base_mut().min_size = Size::new(10, 5);
    /// assert_eq!(w.minimum_size(), Size::new(10, 5));
    /// ```
    #[inline]
    fn minimum_size(&self) -> Size {
        self.widget_base().min_size
    }

    /// Returns the maximum size from `WidgetBase::max_size`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert_eq!(w.maximum_size(), Default::default());
    /// ```
    #[inline]
    fn maximum_size(&self) -> Size {
        self.widget_base().max_size
    }

    // ── lifecycle hooks (default no-op) ───────────────────────────────────────

    /// Paints the widget using `painter`. Default implementation is a no-op.
    ///
    /// Override in concrete widgets to render content.
    ///
    /// # Parameters
    ///
    /// - `painter`: mutable reference to the abstract paint interface.
    #[inline]
    fn paint(&self, _painter: &mut dyn Painter) {}

    /// Called when the widget is resized to `size`.
    ///
    /// # Parameters
    ///
    /// - `size`: the new widget size.
    #[inline]
    fn on_resize(&mut self, _size: Size) {}

    /// Called when the widget becomes visible.
    #[inline]
    fn on_show(&mut self) {}

    /// Called when the widget is hidden.
    #[inline]
    fn on_hide(&mut self) {}

    /// Called when a mouse button is pressed over the widget.
    ///
    /// Default impl sets `WidgetBase::pressed = true`. Override to add widget-specific
    /// behaviour; call `self.set_pressed(true)` from the override body to keep the
    /// default flag-mutation.
    ///
    /// # Parameters
    ///
    /// - `event`: the mouse press event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{AsWidget, WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// assert!(!w.is_pressed());
    /// // The default impl sets pressed=true; the input-plumbing pass calls this.
    /// // Directly verify the accessor:
    /// w.set_pressed(true);
    /// assert!(w.is_pressed());
    /// ```
    #[inline]
    fn on_mouse_press(&mut self, _event: &MouseEvent) {
        self.set_pressed(true);
    }

    /// Called when a mouse button is released over the widget.
    ///
    /// Default impl sets `WidgetBase::pressed = false`. Override to add widget-specific
    /// behaviour; call `self.set_pressed(false)` from the override body to keep the
    /// default flag-mutation.
    ///
    /// # Parameters
    ///
    /// - `event`: the mouse release event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{AsWidget, WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.set_pressed(true);
    /// assert!(w.is_pressed());
    /// w.set_pressed(false);
    /// assert!(!w.is_pressed());
    /// ```
    #[inline]
    fn on_mouse_release(&mut self, _event: &MouseEvent) {
        self.set_pressed(false);
    }

    /// Called when a key is pressed while the widget has focus.
    ///
    /// # Parameters
    ///
    /// - `event`: the key press event.
    #[inline]
    fn on_key_press(&mut self, _event: &KeyEvent) {}

    /// Called when a key is released while the widget has focus.
    ///
    /// # Parameters
    ///
    /// - `event`: the key release event.
    #[inline]
    fn on_key_release(&mut self, _event: &KeyEvent) {}

    /// Called when the widget gains keyboard focus.
    ///
    /// Default impl sets `WidgetBase::focused = true`. Override to add widget-specific
    /// behaviour; call `self.set_focused(true)` from the override body to keep the
    /// default flag-mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{AsWidget, WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// assert!(!w.is_focused());
    /// w.set_focused(true);
    /// assert!(w.is_focused());
    /// ```
    #[inline]
    fn on_focus_in(&mut self) {
        self.set_focused(true);
    }

    /// Called when the widget loses keyboard focus.
    ///
    /// Default impl sets `WidgetBase::focused = false`. Override to add widget-specific
    /// behaviour; call `self.set_focused(false)` from the override body to keep the
    /// default flag-mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{AsWidget, WidgetBase, WidgetExt};
    ///
    /// let mut w = WidgetBase::new();
    /// w.set_focused(true);
    /// assert!(w.is_focused());
    /// w.set_focused(false);
    /// assert!(!w.is_focused());
    /// ```
    #[inline]
    fn on_focus_out(&mut self) {
        self.set_focused(false);
    }
}

/// Blanket implementation — every `AsWidget` automatically gets `WidgetExt`.
impl<T: AsWidget> WidgetExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetBase;
    use quartzite_events::{KeyModifiers, MouseButtons, MouseEventKind};
    use quartzite_geometry::{Point, Size};

    #[test]
    fn show_sets_visible() {
        let mut w = WidgetBase::new();
        w.show();
        assert!(w.is_visible());
    }

    #[test]
    fn hide_clears_visible() {
        let mut w = WidgetBase::new();
        w.show();
        w.hide();
        assert!(!w.is_visible());
    }

    #[test]
    fn set_visible_true_and_false() {
        let mut w = WidgetBase::new();
        w.set_visible(true);
        assert!(w.is_visible());
        w.set_visible(false);
        assert!(!w.is_visible());
    }

    #[test]
    fn update_sets_pending_update() {
        let mut w = WidgetBase::new();
        w.update();
        assert!(w.widget_base().pending_update);
    }

    #[test]
    fn minimum_size_returns_min_size_field() {
        let mut w = WidgetBase::new();
        w.widget_base_mut().min_size = Size::new(100, 50);
        assert_eq!(w.minimum_size(), Size::new(100, 50));
    }

    #[test]
    fn resize_keeps_position() {
        let mut w = WidgetBase::new();
        w.move_to(Point::new(10, 20));
        w.resize(Size::new(200, 100));
        assert_eq!(w.pos(), Point::new(10, 20));
        assert_eq!(w.size(), Size::new(200, 100));
    }

    #[test]
    fn move_to_keeps_size() {
        let mut w = WidgetBase::new();
        w.resize(Size::new(50, 30));
        w.move_to(Point::new(5, 7));
        assert_eq!(w.size(), Size::new(50, 30));
        assert_eq!(w.pos(), Point::new(5, 7));
    }

    #[test]
    fn size_hint_default_is_zero() {
        let w = WidgetBase::new();
        assert_eq!(w.size_hint(), Size::default());
    }

    #[test]
    fn is_enabled_default_true() {
        let w = WidgetBase::new();
        assert!(w.is_enabled());
    }

    #[test]
    fn set_enabled_false() {
        let mut w = WidgetBase::new();
        w.set_enabled(false);
        assert!(!w.is_enabled());
    }

    // ── AC2: hovered / pressed / focused accessors ────────────────────────────

    #[test]
    fn is_hovered_default_false() {
        let w = WidgetBase::new();
        assert!(!w.is_hovered());
    }

    #[test]
    fn set_hovered_flips() {
        let mut w = WidgetBase::new();
        w.set_hovered(true);
        assert!(w.is_hovered());
        w.set_hovered(false);
        assert!(!w.is_hovered());
    }

    #[test]
    fn is_pressed_default_false() {
        let w = WidgetBase::new();
        assert!(!w.is_pressed());
    }

    #[test]
    fn set_pressed_flips() {
        let mut w = WidgetBase::new();
        w.set_pressed(true);
        assert!(w.is_pressed());
        w.set_pressed(false);
        assert!(!w.is_pressed());
    }

    #[test]
    fn is_focused_default_false() {
        let w = WidgetBase::new();
        assert!(!w.is_focused());
    }

    #[test]
    fn set_focused_flips() {
        let mut w = WidgetBase::new();
        w.set_focused(true);
        assert!(w.is_focused());
        w.set_focused(false);
        assert!(!w.is_focused());
    }

    // ── AC7: event-handler default bodies ─────────────────────────────────────

    fn fake_mouse_event(kind: MouseEventKind) -> MouseEvent {
        MouseEvent::new(
            Point::default(),
            Point::default(),
            MouseButtons::empty(),
            MouseButtons::empty(),
            KeyModifiers::default(),
            kind,
        )
    }

    #[test]
    fn on_mouse_press_default_sets_pressed() {
        let mut w = WidgetBase::new();
        w.on_mouse_press(&fake_mouse_event(MouseEventKind::Press));
        assert!(w.widget_base().pressed);
    }

    #[test]
    fn on_mouse_release_default_clears_pressed() {
        let mut w = WidgetBase::new();
        w.set_pressed(true);
        w.on_mouse_release(&fake_mouse_event(MouseEventKind::Release));
        assert!(!w.widget_base().pressed);
    }

    #[test]
    fn on_focus_in_default_sets_focused() {
        let mut w = WidgetBase::new();
        w.on_focus_in();
        assert!(w.widget_base().focused);
    }

    #[test]
    fn on_focus_out_default_clears_focused() {
        let mut w = WidgetBase::new();
        w.set_focused(true);
        w.on_focus_out();
        assert!(!w.widget_base().focused);
    }
}
