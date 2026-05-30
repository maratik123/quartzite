//! [`Container`] — generic widget container.

use quartzite_core::ObjectId;
use quartzite_macros::{Extend, Object, object_impl};

use crate::{WidgetBase, widget_base::AsWidget};

/// A generic container widget that holds child widget ids.
///
/// [`Container`] acts as a simple parent for grouping widgets. Children are
/// tracked by id; layout is delegated to the widget's associated [`Layout`](crate::Layout).
///
/// # Examples
///
/// ```
/// use quartzite_core::Object;
/// use quartzite_widgets::Container;
///
/// let c = Container::new();
/// assert_eq!(c.meta_object().class_name, "Container");
/// assert!(c.children().is_empty());
/// ```
#[derive(Debug, Extend, Object)]
#[widget_view(variant = "Container")]
pub struct Container {
    /// Base widget — delegates geometry, state, focus policy, and object core.
    #[base]
    pub widget_base: WidgetBase,
    /// Ordered list of child widget ids.
    #[widget_children(slice)]
    pub children: Vec<ObjectId>,
}

impl Container {
    /// Creates a new empty [`Container`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Container;
    ///
    /// let c = Container::new();
    /// assert!(c.children().is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            widget_base: WidgetBase::new(),
            children: Vec::new(),
        }
    }

    /// Appends `child` to this container's child list.
    ///
    /// # Parameters
    ///
    /// - `child`: id of the widget to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_widgets::Container;
    ///
    /// let mut c = Container::new();
    /// let id = ObjectId::new();
    /// c.add_child(id);
    /// assert_eq!(c.children(), &[id]);
    /// ```
    pub fn add_child(&mut self, child: ObjectId) {
        self.children.push(child);
    }

    /// Removes the first occurrence of `child` from the child list.
    ///
    /// Does nothing if `child` is not present.
    ///
    /// # Parameters
    ///
    /// - `child`: id of the widget to remove.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_widgets::Container;
    ///
    /// let mut c = Container::new();
    /// let id = ObjectId::new();
    /// c.add_child(id);
    /// c.remove_child(id);
    /// assert!(c.children().is_empty());
    /// ```
    pub fn remove_child(&mut self, child: ObjectId) {
        if let Some(pos) = self.children.iter().position(|&id| id == child) {
            self.children.remove(pos);
        }
    }

    /// Returns a slice of the child widget ids in insertion order.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Container;
    ///
    /// let c = Container::new();
    /// assert!(c.children().is_empty());
    /// ```
    #[inline]
    pub fn children(&self) -> &[ObjectId] {
        &self.children
    }
}

impl Default for Container {
    /// Returns a new empty `Container`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Container;
    ///
    /// let c = Container::default();
    /// assert!(c.children().is_empty());
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[object_impl]
impl Container {}

#[cfg(test)]
mod tests {
    use quartzite_core::{Object, ObjectId};

    use super::*;

    #[test]
    fn class_name_is_container() {
        let c = Container::new();
        assert_eq!(c.meta_object().class_name, "Container");
    }

    #[test]
    fn add_child_appends_in_order() {
        let mut c = Container::new();
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        c.add_child(id1);
        c.add_child(id2);
        assert_eq!(c.children(), &[id1, id2]);
    }

    #[test]
    fn remove_child_removes_first_occurrence() {
        let mut c = Container::new();
        let id = ObjectId::new();
        c.add_child(id);
        c.remove_child(id);
        assert!(c.children().is_empty());
    }

    #[test]
    fn remove_child_noop_when_absent() {
        let mut c = Container::new();
        let id = ObjectId::new();
        c.remove_child(id);
        assert!(c.children().is_empty());
    }

    #[test]
    fn as_widget_children_returns_all_ids() {
        use crate::widget_base::AsWidget;
        let mut c = Container::new();
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        c.add_child(id1);
        c.add_child(id2);
        // Use trait-qualified call to reach AsWidget::children, not the inherent method.
        let ids: Vec<ObjectId> = <Container as AsWidget>::children(&c).into_iter().collect();
        assert_eq!(ids, [id1, id2]);
    }

    #[test]
    fn as_widget_children_empty_when_no_children() {
        use crate::widget_base::AsWidget;
        let c = Container::new();
        assert_eq!(<Container as AsWidget>::children(&c).into_iter().count(), 0);
    }

    #[test]
    fn widget_view_returns_container_variant() {
        let c = Container::new();
        assert!(matches!(c.widget_view(), crate::WidgetView::Container(_)));
    }
}
