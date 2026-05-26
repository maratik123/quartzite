//! [`ScrollArea`] — scrollable viewport widget.

use quartzite_core::ObjectId;
use quartzite_macros::{Extend, MetaEnum, Object, object_impl};

use crate::{WidgetBase, widget_base::AsWidget};

/// Controls when scrollbars appear in a [`ScrollArea`].
///
/// # Examples
///
/// ```
/// use quartzite_widgets::ScrollPolicy;
///
/// assert_eq!(ScrollPolicy::default(), ScrollPolicy::AsNeeded);
/// ```
#[derive(MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ScrollPolicy {
    /// Show scrollbar only when content overflows.
    #[default]
    AsNeeded = 0,
    /// Always show the scrollbar.
    AlwaysOn = 1,
    /// Never show the scrollbar.
    AlwaysOff = 2,
}

/// A scrollable container that displays a single child widget.
///
/// # Examples
///
/// ```
/// use quartzite_core::{Object, Value};
/// use quartzite_widgets::ScrollArea;
///
/// let area = ScrollArea::new();
/// assert_eq!(area.meta_object().class_name, "ScrollArea");
/// ```
#[derive(Extend, Object)]
#[widget_view(variant = "ScrollArea")]
pub struct ScrollArea {
    #[base]
    widget_base: WidgetBase,
    /// The widget shown inside the scroll area (if any).
    #[widget_children(optional)]
    pub content_widget: Option<ObjectId>,
    /// Horizontal scrollbar policy.
    #[property]
    pub horizontal_policy: ScrollPolicy,
    /// Vertical scrollbar policy.
    #[property]
    pub vertical_policy: ScrollPolicy,
}

impl ScrollArea {
    /// Creates a new empty [`ScrollArea`] with [`AsNeeded`](ScrollPolicy::AsNeeded) scroll policies.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{ScrollArea, ScrollPolicy};
    ///
    /// let area = ScrollArea::new();
    /// assert!(area.content_widget.is_none());
    /// assert_eq!(area.horizontal_policy, ScrollPolicy::AsNeeded);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            widget_base: WidgetBase::new(),
            content_widget: None,
            horizontal_policy: ScrollPolicy::AsNeeded,
            vertical_policy: ScrollPolicy::AsNeeded,
        }
    }
}

impl Default for ScrollArea {
    /// Returns a new empty `ScrollArea`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::ScrollArea;
    ///
    /// let area = ScrollArea::default();
    /// assert!(area.content_widget.is_none());
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[object_impl]
impl ScrollArea {}

#[cfg(test)]
mod tests {
    use quartzite_core::{Object, ObjectId};

    use super::*;

    #[test]
    fn class_name_is_scroll_area() {
        let area = ScrollArea::new();
        assert_eq!(area.meta_object().class_name, "ScrollArea");
    }

    #[test]
    fn default_policies_are_as_needed() {
        let area = ScrollArea::new();
        assert_eq!(area.horizontal_policy, ScrollPolicy::AsNeeded);
        assert_eq!(area.vertical_policy, ScrollPolicy::AsNeeded);
    }

    #[test]
    fn content_widget_initially_none() {
        let area = ScrollArea::new();
        assert!(area.content_widget.is_none());
    }

    #[test]
    fn content_widget_assigned_directly() {
        let mut area = ScrollArea::new();
        let id = ObjectId::new();
        area.content_widget = Some(id);
        assert_eq!(area.content_widget, Some(id));
    }

    #[test]
    fn as_widget_children_with_content_yields_one_id() {
        use crate::widget_base::AsWidget;
        let mut area = ScrollArea::new();
        let id = ObjectId::new();
        area.content_widget = Some(id);
        let ids: Vec<ObjectId> = area.children().into_iter().collect();
        assert_eq!(ids, [id]);
    }

    #[test]
    fn as_widget_children_without_content_yields_zero() {
        use crate::widget_base::AsWidget;
        let area = ScrollArea::new();
        assert_eq!(area.children().into_iter().count(), 0);
    }

    #[test]
    fn widget_view_returns_scroll_area_variant() {
        let area = ScrollArea::new();
        assert!(matches!(
            area.widget_view(),
            crate::WidgetView::ScrollArea(_)
        ));
    }
}
