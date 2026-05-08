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
    /// _Simple._
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert_eq!(w.geometry(), Default::default());
    /// ```
    fn geometry(&self) -> Rect {
        self.widget_base().geometry
    }

    /// Sets the bounding rectangle.
    ///
    /// _Simple._
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
    fn set_geometry(&mut self, rect: Rect) {
        self.widget_base_mut().geometry = rect;
    }

    /// Returns the widget's top-left position in parent coordinates.
    ///
    /// _Simple._
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
    fn pos(&self) -> Point {
        self.widget_base().geometry.origin()
    }

    /// Returns the widget's current size.
    ///
    /// _Simple._
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert_eq!(w.size(), Default::default());
    /// ```
    fn size(&self) -> Size {
        self.widget_base().geometry.size()
    }

    /// Resizes the widget, keeping its current position.
    ///
    /// _Simple._
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
    fn resize(&mut self, size: Size) {
        let origin = self.widget_base().geometry.origin();
        self.widget_base_mut().geometry = Rect::new(origin, size);
    }

    /// Moves the widget to `point`, keeping its current size.
    ///
    /// _Simple._
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
    fn move_to(&mut self, point: Point) {
        let size = self.widget_base().geometry.size();
        self.widget_base_mut().geometry = Rect::new(point, size);
    }

    // ── visibility ────────────────────────────────────────────────────────────

    /// Shows the widget by setting `visible = true`.
    ///
    /// _Simple._
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
    fn show(&mut self) {
        self.widget_base_mut().visible = true;
    }

    /// Hides the widget by setting `visible = false`.
    ///
    /// _Simple._
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
    fn hide(&mut self) {
        self.widget_base_mut().visible = false;
    }

    /// Returns `true` when the widget is visible.
    ///
    /// _Simple._
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert!(!w.is_visible());
    /// ```
    fn is_visible(&self) -> bool {
        self.widget_base().visible
    }

    /// Sets the visibility flag directly.
    ///
    /// _Simple._
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
    fn set_visible(&mut self, visible: bool) {
        self.widget_base_mut().visible = visible;
    }

    // ── enabled ───────────────────────────────────────────────────────────────

    /// Returns `true` when the widget accepts user input.
    ///
    /// _Simple._
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert!(w.is_enabled());
    /// ```
    fn is_enabled(&self) -> bool {
        self.widget_base().enabled
    }

    /// Enables or disables the widget.
    ///
    /// _Simple._
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
    fn set_enabled(&mut self, enabled: bool) {
        self.widget_base_mut().enabled = enabled;
    }

    // ── repaint ───────────────────────────────────────────────────────────────

    /// Marks the widget as needing a repaint on the next render pass.
    ///
    /// _Simple._
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
    fn update(&mut self) {
        self.widget_base_mut().pending_update = true;
    }

    // ── size hints ────────────────────────────────────────────────────────────

    /// Returns the preferred size hint; defaults to `Size::default()` (zero).
    ///
    /// _Simple._
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
    fn size_hint(&self) -> Size {
        Size::default()
    }

    /// Returns the minimum size from `WidgetBase::min_size`.
    ///
    /// _Simple._
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
    fn minimum_size(&self) -> Size {
        self.widget_base().min_size
    }

    /// Returns the maximum size from `WidgetBase::max_size`.
    ///
    /// _Simple._
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{WidgetBase, WidgetExt};
    ///
    /// let w = WidgetBase::new();
    /// assert_eq!(w.maximum_size(), Default::default());
    /// ```
    fn maximum_size(&self) -> Size {
        self.widget_base().max_size
    }

    // ── lifecycle hooks (default no-op) ───────────────────────────────────────

    /// Paints the widget using `painter`. Default implementation is a no-op.
    ///
    /// _Simple._
    ///
    /// Override in concrete widgets to render content.
    ///
    /// # Parameters
    ///
    /// - `painter`: mutable reference to the abstract paint interface.
    fn paint(&self, _painter: &mut dyn Painter) {}

    /// Called when the widget is resized to `size`.
    ///
    /// _Simple._
    ///
    /// # Parameters
    ///
    /// - `size`: the new widget size.
    fn on_resize(&mut self, _size: Size) {}

    /// Called when the widget becomes visible.
    ///
    /// _Simple._
    fn on_show(&mut self) {}

    /// Called when the widget is hidden.
    ///
    /// _Simple._
    fn on_hide(&mut self) {}

    /// Called when a mouse button is pressed over the widget.
    ///
    /// _Simple._
    ///
    /// # Parameters
    ///
    /// - `event`: the mouse press event.
    fn on_mouse_press(&mut self, _event: &MouseEvent) {}

    /// Called when a mouse button is released over the widget.
    ///
    /// _Simple._
    ///
    /// # Parameters
    ///
    /// - `event`: the mouse release event.
    fn on_mouse_release(&mut self, _event: &MouseEvent) {}

    /// Called when a key is pressed while the widget has focus.
    ///
    /// _Simple._
    ///
    /// # Parameters
    ///
    /// - `event`: the key press event.
    fn on_key_press(&mut self, _event: &KeyEvent) {}

    /// Called when a key is released while the widget has focus.
    ///
    /// _Simple._
    ///
    /// # Parameters
    ///
    /// - `event`: the key release event.
    fn on_key_release(&mut self, _event: &KeyEvent) {}

    /// Called when the widget gains keyboard focus.
    ///
    /// _Simple._
    fn on_focus_in(&mut self) {}

    /// Called when the widget loses keyboard focus.
    ///
    /// _Simple._
    fn on_focus_out(&mut self) {}
}

/// Blanket implementation — every `AsWidget` automatically gets `WidgetExt`.
impl<T: AsWidget> WidgetExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetBase;
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
}
