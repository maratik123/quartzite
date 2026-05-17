//! [`WidgetBase`] — the hierarchy root for all widgets.

use std::sync::Arc;

use quartzite_core::{ObjectBase, ObjectId};
use quartzite_geometry::{Rect, Size};
use quartzite_macros::Extend;

use crate::widgets::{Button, Container, Label, LineEdit, ScrollArea, TextEdit};
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
    /// `true` while the mouse cursor is over the widget's [`Self::geometry`].
    ///
    /// Updated by the input-plumbing pass via [`crate::WidgetExt::set_hovered`].
    pub hovered: bool,
    /// `true` while a mouse button is held with press-initiated state on this widget.
    ///
    /// Set to `true` by the [`crate::WidgetExt::on_mouse_press`] default and cleared
    /// by [`crate::WidgetExt::on_mouse_release`].
    pub pressed: bool,
    /// `true` while this widget owns keyboard focus.
    ///
    /// Set to `true` by the [`crate::WidgetExt::on_focus_in`] default and cleared
    /// by [`crate::WidgetExt::on_focus_out`].
    pub focused: bool,
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
            hovered: false,
            pressed: false,
            focused: false,
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

/// A typed view of a concrete widget, used for pattern-matching dispatch in
/// `quartzite_style::Style::draw_widget`.
///
/// Built-in widgets return their own variant; third-party widgets default to
/// [`WidgetView::Other`]. The `#[non_exhaustive]` attribute means match arms must include
/// a catch-all, keeping new built-in variants non-breaking.
#[non_exhaustive]
pub enum WidgetView<'a> {
    /// A [`Button`] widget.
    Button(&'a Button),
    /// A [`Label`] widget.
    Label(&'a Label),
    /// A [`TextEdit`] widget.
    TextEdit(&'a TextEdit),
    /// A [`ScrollArea`] widget.
    ScrollArea(&'a ScrollArea),
    /// A [`Container`] widget.
    Container(&'a Container),
    /// A [`LineEdit`] widget.
    LineEdit(&'a LineEdit),
    /// A widget not in the built-in set — the open-set escape hatch.
    ///
    /// Third-party or unknown widget types surface here. A custom
    /// `quartzite_style::Style` that wants to handle a specific type overrides
    /// `Style::draw_widget` and pattern-matches:
    ///
    /// ```text
    /// WidgetView::Other(other) => {
    ///     if let Some(w) = other.as_any().downcast_ref::<MyWidget>() {
    ///         self.paint(w, painter, palette);
    ///     }
    /// }
    /// ```
    ///
    /// The default behaviour is a **silent no-op** (no panic, no warning). This is
    /// intentional: `Other` is a valid extension point; per-frame warnings would spam logs.
    Other(&'a dyn AsWidget),
}

/// Children of a widget, as returned by [`AsWidget::children`].
///
/// A borrowed view over a widget's child [`ObjectId`]s, allowing iteration without
/// allocation. The common shapes are a contiguous slice ([`WidgetChildren::Slice`]),
/// a single optional child ([`WidgetChildren::Optional`]), and no children
/// ([`WidgetChildren::Empty`]).
///
/// # Examples
///
/// ```
/// use quartzite_widgets::WidgetChildren;
///
/// let children = WidgetChildren::Empty;
/// assert_eq!(children.into_iter().count(), 0);
/// ```
pub enum WidgetChildren<'a> {
    /// Slice of child [`ObjectId`]s — the common case for container widgets.
    Slice(&'a [ObjectId]),
    /// At most one child — used by [`ScrollArea`].
    Optional(Option<ObjectId>),
    /// No children — the default for leaf widgets.
    Empty,
}

impl<'a> IntoIterator for WidgetChildren<'a> {
    type Item = ObjectId;
    type IntoIter = WidgetChildrenIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Slice(s) => WidgetChildrenIter::Slice(s.iter()),
            Self::Optional(o) => WidgetChildrenIter::Optional(o.into_iter()),
            Self::Empty => WidgetChildrenIter::Empty,
        }
    }
}

/// Iterator over [`WidgetChildren`], yielding [`ObjectId`] values.
///
/// Obtained by calling `.into_iter()` on a [`WidgetChildren`] value.
pub enum WidgetChildrenIter<'a> {
    /// Iterator over a slice of [`ObjectId`]s.
    Slice(std::slice::Iter<'a, ObjectId>),
    /// Iterator over an optional single [`ObjectId`].
    Optional(std::option::IntoIter<ObjectId>),
    /// An iterator that yields nothing.
    Empty,
}

impl Iterator for WidgetChildrenIter<'_> {
    type Item = ObjectId;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Slice(it) => it.next().copied(),
            Self::Optional(it) => it.next(),
            Self::Empty => None,
        }
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
        assert!(!w.hovered);
        assert!(!w.pressed);
        assert!(!w.focused);
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

    #[test]
    fn widget_children_empty_yields_zero() {
        assert_eq!(WidgetChildren::Empty.into_iter().count(), 0);
    }

    #[test]
    fn widget_children_slice_yields_all_ids() {
        let ids = [ObjectId::new(), ObjectId::new(), ObjectId::new()];
        let v: Vec<ObjectId> = WidgetChildren::Slice(&ids).into_iter().collect();
        assert_eq!(v, ids);
    }

    #[test]
    fn widget_children_optional_none_yields_zero() {
        assert_eq!(WidgetChildren::Optional(None).into_iter().count(), 0);
    }

    #[test]
    fn widget_children_optional_some_yields_one() {
        let id = ObjectId::new();
        let v: Vec<ObjectId> = WidgetChildren::Optional(Some(id)).into_iter().collect();
        assert_eq!(v, [id]);
    }

    #[test]
    fn widget_base_widget_view_returns_other() {
        let w = WidgetBase::new();
        assert!(matches!(w.widget_view(), WidgetView::Other(_)));
    }

    #[test]
    fn widget_base_children_returns_empty() {
        let w = WidgetBase::new();
        assert_eq!(w.children().into_iter().count(), 0);
    }
}
