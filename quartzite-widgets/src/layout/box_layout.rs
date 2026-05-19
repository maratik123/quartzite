//! [`BoxLayout`] — horizontal or vertical stacking layout.

use quartzite_core::{ObjectBase, ObjectId};
use quartzite_geometry::{Point, Rect, Size};
use quartzite_macros::{Extend, Object, object_impl};

use crate::layout::{Layout, WidgetResolver};

/// Stacking direction for a [`BoxLayout`].
///
/// # Examples
///
/// ```
/// use quartzite_widgets::Direction;
///
/// let d = Direction::Horizontal;
/// assert_eq!(d, Direction::Horizontal);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    /// Children are stacked left-to-right.
    #[default]
    Horizontal,
    /// Children are stacked top-to-bottom.
    Vertical,
}

/// Stacks child widgets horizontally or vertically, distributing space by stretch factor.
///
/// Children are added via [`BoxLayout::add_child`]. When [`Layout::set_geometry`] is
/// called, space is divided proportionally to each child's stretch factor (equal stretch
/// → equal share).
///
/// # Examples
///
/// ```
/// use quartzite_widgets::{BoxLayout, Direction};
///
/// let layout = BoxLayout::new(Direction::Horizontal);
/// assert_eq!(layout.direction, Direction::Horizontal);
/// ```
#[derive(Extend, Object)]
#[root]
pub struct BoxLayout {
    #[base]
    object: ObjectBase,
    /// Stacking direction.
    pub direction: Direction,
    children: Vec<(ObjectId, i32)>,
}

impl BoxLayout {
    /// Creates a new empty [`BoxLayout`] with the given `direction`.
    ///
    /// # Parameters
    ///
    /// - `direction`: whether children stack horizontally or vertically.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{BoxLayout, Direction};
    ///
    /// let layout = BoxLayout::new(Direction::Vertical);
    /// assert_eq!(layout.direction, Direction::Vertical);
    /// ```
    #[inline]
    pub fn new(direction: Direction) -> Self {
        Self {
            object: ObjectBase::new(),
            direction,
            children: Vec::new(),
        }
    }

    /// Appends `widget` to this layout with the given `stretch` factor.
    ///
    /// A larger stretch factor causes the child to receive proportionally more space.
    ///
    /// # Parameters
    ///
    /// - `widget`: id of the widget to add.
    /// - `stretch`: non-negative stretch factor; `0` is treated as `1` for distribution.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_widgets::{BoxLayout, Direction};
    ///
    /// let mut layout = BoxLayout::new(Direction::Horizontal);
    /// let id = ObjectId::new();
    /// layout.add_child(id, 1);
    /// assert_eq!(layout.child_count(), 1);
    /// ```
    pub fn add_child(&mut self, widget: ObjectId, stretch: i32) {
        self.children.push((widget, stretch));
    }

    /// Returns the number of children in this layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{BoxLayout, Direction};
    ///
    /// let layout = BoxLayout::new(Direction::Horizontal);
    /// assert_eq!(layout.child_count(), 0);
    /// ```
    #[inline]
    pub const fn child_count(&self) -> usize {
        self.children.len()
    }
}

impl Layout for BoxLayout {
    fn set_geometry(&mut self, resolver: &mut dyn WidgetResolver, rect: Rect) {
        if self.children.is_empty() {
            return;
        }
        let total_stretch: i32 = self.children.iter().map(|(_, s)| (*s).max(1)).sum();

        let (total, cross_origin, cross_len) = match self.direction {
            Direction::Horizontal => (rect.size().width(), rect.top(), rect.size().height()),
            Direction::Vertical => (rect.size().height(), rect.left(), rect.size().width()),
        };
        let origin_main = match self.direction {
            Direction::Horizontal => rect.left(),
            Direction::Vertical => rect.top(),
        };

        let children: Vec<(ObjectId, i32)> = self.children.clone();
        let n = children.len();
        let mut offset = origin_main;

        for (i, (widget_id, stretch)) in children.iter().enumerate() {
            let stretch = (*stretch).max(1);
            let share = if i + 1 < n {
                total * stretch / total_stretch
            } else {
                // Last child takes the remaining space to absorb rounding error.
                origin_main + total - offset
            };

            let child_rect = match self.direction {
                Direction::Horizontal => Rect::new(
                    Point::new(offset, cross_origin),
                    Size::new(share, cross_len),
                ),
                Direction::Vertical => Rect::new(
                    Point::new(cross_origin, offset),
                    Size::new(cross_len, share),
                ),
            };
            if let Some(wb) = resolver.resolve_widget_mut(*widget_id) {
                wb.geometry = child_rect;
            }
            offset += share;
        }
    }

    #[inline]
    fn size_hint(&self) -> Size {
        Size::default()
    }

    #[inline]
    fn minimum_size(&self) -> Size {
        Size::default()
    }
}

#[object_impl]
impl BoxLayout {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::WidgetBase;

    /// Inline stub resolver backed by a `HashMap<ObjectId, WidgetBase>`.
    struct StubResolver(HashMap<ObjectId, WidgetBase>);

    impl WidgetResolver for StubResolver {
        fn resolve_widget_mut(&mut self, id: ObjectId) -> Option<&mut WidgetBase> {
            self.0.get_mut(&id)
        }
    }

    #[test]
    fn empty_layout_is_noop() {
        let mut layout = BoxLayout::new(Direction::Horizontal);
        let mut resolver = StubResolver(HashMap::new());
        layout.set_geometry(
            &mut resolver,
            Rect::new(Point::new(0, 0), Size::new(200, 100)),
        );
    }

    #[test]
    fn two_equal_stretch_split_horizontal() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        let mut layout = BoxLayout::new(Direction::Horizontal);
        layout.add_child(id1, 1);
        layout.add_child(id2, 1);

        let mut map = HashMap::new();
        map.insert(id1, WidgetBase::new());
        map.insert(id2, WidgetBase::new());
        let mut resolver = StubResolver(map);

        layout.set_geometry(
            &mut resolver,
            Rect::new(Point::new(0, 0), Size::new(200, 100)),
        );

        let g1 = resolver.0[&id1].geometry;
        let g2 = resolver.0[&id2].geometry;
        assert_eq!(g1.size().width(), 100, "first child should get half");
        assert_eq!(g2.size().width(), 100, "second child should get half");
        assert_eq!(g1.left(), 0);
        assert_eq!(g2.left(), 100);
    }

    #[test]
    fn single_child_gets_full_width() {
        let id = ObjectId::new();
        let mut layout = BoxLayout::new(Direction::Horizontal);
        layout.add_child(id, 2);

        let mut map = HashMap::new();
        map.insert(id, WidgetBase::new());
        let mut resolver = StubResolver(map);

        layout.set_geometry(
            &mut resolver,
            Rect::new(Point::new(10, 20), Size::new(300, 50)),
        );
        let g = resolver.0[&id].geometry;
        assert_eq!(g.size().width(), 300);
        assert_eq!(g.left(), 10);
    }

    #[test]
    fn unequal_stretch_distributes_proportionally() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        let mut layout = BoxLayout::new(Direction::Horizontal);
        layout.add_child(id1, 1);
        layout.add_child(id2, 3);

        let mut map = HashMap::new();
        map.insert(id1, WidgetBase::new());
        map.insert(id2, WidgetBase::new());
        let mut resolver = StubResolver(map);

        layout.set_geometry(
            &mut resolver,
            Rect::new(Point::new(0, 0), Size::new(400, 100)),
        );
        let g1 = resolver.0[&id1].geometry;
        let g2 = resolver.0[&id2].geometry;
        assert_eq!(g1.size().width(), 100);
        assert_eq!(g2.size().width(), 300);
    }
}
