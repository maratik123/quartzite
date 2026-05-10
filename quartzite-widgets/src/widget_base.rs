//! [`WidgetBase`] — the hierarchy root for all widgets.

use std::sync::Arc;

use quartzite_core::{ObjectBase, ObjectId};
use quartzite_geometry::{Rect, Size};
use quartzite_macros::Extend;

use crate::{CursorShape, FocusPolicy, Font, Palette, SizePolicy};

/// Hierarchy root for all quartzite widgets.
///
/// Every concrete widget holds a `#[base] widget_base: WidgetBase` field and
/// receives the [`AsWidget`] and [`quartzite_core::AsObject`] implementations for free via
/// `#[derive(Extend)]`.
///
/// Fields are `pub` so [`crate::WidgetExt`] can read and write them through the
/// blanket impl.
///
/// # Examples
///
/// ```
/// use quartzite_widgets::{WidgetBase, WidgetExt};
///
/// let mut w = WidgetBase::new();
/// assert!(!w.is_visible());
/// w.show();
/// assert!(w.is_visible());
/// ```
#[derive(Extend)]
#[root]
pub struct WidgetBase {
    #[base]
    object: ObjectBase,
    /// Bounding rectangle of the widget in parent coordinates.
    pub geometry: Rect,
    /// Whether the widget is currently visible.
    pub visible: bool,
    /// Whether the widget is currently enabled (accepts input).
    pub enabled: bool,
    /// Keyboard focus policy for this widget.
    pub focus_policy: FocusPolicy,
    /// Size policy controlling how the widget participates in layout.
    pub size_policy: SizePolicy,
    /// Mouse cursor displayed over this widget.
    pub cursor: CursorShape,
    /// Shared font; multiple widgets may reference the same `Arc`.
    pub font: Arc<Font>,
    /// Shared palette; multiple widgets may reference the same `Arc`.
    pub palette: Arc<Palette>,
    /// [`ObjectId`] of the layout manager attached to this widget, if any.
    pub layout: Option<ObjectId>,
    /// [`ObjectId`]s of installed event filters (dispatch deferred to plan #47).
    pub event_filters: Vec<ObjectId>,
    /// Set to `true` by [`crate::WidgetExt::update`]; consumed by the renderer.
    pub pending_update: bool,
    /// Minimum size hint returned by [`crate::WidgetExt::minimum_size`].
    pub min_size: Size,
    /// Maximum size hint returned by [`crate::WidgetExt::maximum_size`].
    pub max_size: Size,
}

impl WidgetBase {
    /// Creates a new anonymous [`WidgetBase`] with default values.
    ///
    /// The widget starts hidden (`visible = false`), enabled, with zero geometry and
    /// default font/palette shared instances.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::WidgetBase;
    ///
    /// let w = WidgetBase::new();
    /// assert!(!w.visible);
    /// assert!(w.enabled);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            object: ObjectBase::new(),
            geometry: Rect::default(),
            visible: false,
            enabled: true,
            focus_policy: FocusPolicy::default(),
            size_policy: SizePolicy::default(),
            cursor: CursorShape::default(),
            font: Arc::new(Font::default()),
            palette: Arc::new(Palette::default()),
            layout: None,
            event_filters: Vec::new(),
            pending_update: false,
            min_size: Size::default(),
            max_size: Size::default(),
        }
    }
}

impl Default for WidgetBase {
    /// Returns a new `WidgetBase` with defaults — equivalent to [`WidgetBase::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::WidgetBase;
    ///
    /// let w = WidgetBase::default();
    /// assert!(!w.visible);
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget_ext::WidgetExt;
    use quartzite_core::AsObject;

    #[test]
    fn new_widget_base_defaults() {
        let w = WidgetBase::new();
        assert!(!w.visible);
        assert!(w.enabled);
        assert_eq!(w.geometry, Rect::default());
        assert!(w.layout.is_none());
        assert!(w.event_filters.is_empty());
        assert!(!w.pending_update);
    }

    #[test]
    fn as_widget_self_ref() {
        let w = WidgetBase::new();
        // AsWidget::widget_base() returns &self
        let r: &WidgetBase = w.widget_base();
        assert!(!r.visible);
    }

    #[test]
    fn as_object_delegation() {
        let w = WidgetBase::new();
        // AsObject::object_base() delegates to the inner ObjectBase
        let _ = w.object_base();
    }

    #[test]
    fn show_hide() {
        let mut w = WidgetBase::new();
        w.show();
        assert!(w.visible);
        w.hide();
        assert!(!w.visible);
    }

    #[test]
    fn update_sets_pending() {
        let mut w = WidgetBase::new();
        assert!(!w.pending_update);
        w.update();
        assert!(w.pending_update);
    }

    #[test]
    fn minimum_size_returns_min_size() {
        use quartzite_geometry::Size;
        let mut w = WidgetBase::new();
        w.min_size = Size::new(100, 50);
        assert_eq!(w.minimum_size(), Size::new(100, 50));
    }
}
