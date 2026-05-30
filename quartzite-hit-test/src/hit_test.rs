//! The [`hit_test`] free function and its recursive [`find_hit`] helper.

use quartzite_core::ObjectId;
use quartzite_geometry::{Point, Rect};
use quartzite_widgets::{WidgetChildren, WidgetState};

use crate::WidgetResolver;

/// Finds the topmost visible widget under `point` in the subtree rooted at `root`.
///
/// `hit_test` is the structural inverse of `dispatch_paint`: it walks the same
/// [`WidgetResolver`] + [`quartzite_widgets::AsWidget::children`] tree, applies the same
/// visibility-skips-subtree and `children_clip_rect()` rules, and emits the same
/// resolver-miss `warn!` — but iterates children in **reverse** order and tests
/// child-before-parent, so the visually-topmost (last-painted) widget wins.
///
/// `point` is in the root widget's local coordinate space (`(0,0)` = root's
/// top-left). The returned [`Point`] is the **accumulated origin offset**: the
/// sum of the parent-relative origins traversed from `root` down to the hit
/// widget. A caller maps the original `point` into the hit widget's local space
/// by subtracting this offset (`point - offset`) without re-walking the tree.
///
/// Returns [`None`] when the point lands outside the visible tree, the root is
/// hidden, or the root id does not resolve (the latter also emits a `warn!`).
///
/// # Parameters
///
/// - `root`: the [`ObjectId`] of the root widget to start the search from.
/// - `point`: the query point in `root`'s local coordinate space.
/// - `resolver`: maps [`ObjectId`] to `&dyn AsWidget`; see [`WidgetResolver`].
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use quartzite_core::ObjectId;
/// use quartzite_geometry::{Point, Rect, Size};
/// use quartzite_widgets::{AsWidget, Container, Label, WidgetExt};
/// use quartzite_hit_test::{WidgetResolver, hit_test};
///
/// struct MapResolver(HashMap<ObjectId, Box<dyn AsWidget>>);
///
/// impl WidgetResolver for MapResolver {
///     fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> {
///         self.0.get(&id).map(|b| b.as_ref() as &dyn AsWidget)
///     }
/// }
///
/// // Container at (0,0) sized 100×100, with a Label child at (10,20) sized 50×20.
/// let root_id = ObjectId::new();
/// let label_id = ObjectId::new();
///
/// let mut root = Container::new();
/// root.show();
/// root.set_geometry(Rect::new(Point::new(0, 0), Size::new(100, 100)));
/// root.add_child(label_id);
///
/// let mut label = Label::new("Hello".into());
/// label.show();
/// label.set_geometry(Rect::new(Point::new(10, 20), Size::new(50, 20)));
///
/// let mut map: HashMap<ObjectId, Box<dyn AsWidget>> = HashMap::new();
/// map.insert(root_id, Box::new(root));
/// map.insert(label_id, Box::new(label));
/// let resolver = MapResolver(map);
///
/// // A point inside the label hits the label, with the label's origin as offset.
/// assert_eq!(
///     hit_test(root_id, Point::new(15, 25), &resolver),
///     Some((label_id, Point::new(10, 20))),
/// );
///
/// // A point on the container chrome (outside the label) hits the container.
/// assert_eq!(
///     hit_test(root_id, Point::new(5, 5), &resolver),
///     Some((root_id, Point::default())),
/// );
///
/// // A point outside the root misses entirely.
/// assert_eq!(hit_test(root_id, Point::new(200, 200), &resolver), None);
/// ```
pub fn hit_test(
    root: ObjectId,
    point: Point,
    resolver: &dyn WidgetResolver,
) -> Option<(ObjectId, Point)> {
    let _span = tracing::debug_span!("hit_test", root = ?root, point = ?point).entered();
    find_hit(root, point, resolver)
}

/// Recursive helper: finds the topmost hit in the subtree rooted at `id`.
///
/// `point` arrives in `id`'s **own local space**; the returned offset is relative
/// to that node (root call: offset starts at [`Point::default`], satisfying the
/// zero-offset root hit). Each level subtracts the child's parent-relative origin
/// before recursing and adds it back on the way up, accumulating the offset.
fn find_hit(
    id: ObjectId,
    point: Point,
    resolver: &dyn WidgetResolver,
) -> Option<(ObjectId, Point)> {
    let Some(widget) = resolver.resolve(id) else {
        tracing::warn!(?id, "hit_test: resolver miss");
        return None;
    };
    // An invisible widget is not a candidate and hides its whole subtree.
    if !widget.widget_base().state.contains(WidgetState::Visible) {
        return None;
    }

    // Children are only candidates when the point lies inside the clip rect
    // (in this node's local space). `None` clip exposes children unconditionally.
    let clip = widget.children_clip_rect();
    if clip.is_none_or(|c| c.contains(point)) {
        // Iterate children in reverse so the visually-topmost sibling wins, and
        // test child-before-parent: the first reverse-child hit is the answer.
        for child_id in reversed_children(&widget.children()) {
            let Some(child) = resolver.resolve(child_id) else {
                tracing::warn!(id = ?child_id, "hit_test: resolver miss");
                continue;
            };
            let child_origin = child.widget_base().geometry.origin();
            if let Some((hit, offset)) = find_hit(child_id, point - child_origin, resolver) {
                return Some((hit, child_origin + offset));
            }
        }
    }

    // No child claimed the hit: test this node's own (local-space) rectangle.
    // `geometry()` is parent-relative, so membership uses an origin-zero rect of
    // the node's own size — not `geometry().contains(point)`.
    if Rect::new(Point::default(), widget.widget_base().geometry.size()).contains(point) {
        Some((id, Point::default()))
    } else {
        None
    }
}

/// Collects a widget's children into reverse iteration order.
///
/// Only [`WidgetChildren::Slice`] holds more than one element, so it is the only
/// variant that needs reversing; `Optional`/`Empty` reverse trivially without
/// allocation.
fn reversed_children<'a>(children: &WidgetChildren<'a>) -> ReversedChildren<'a> {
    match *children {
        WidgetChildren::Slice(s) => ReversedChildren::Slice(s.iter().rev()),
        WidgetChildren::Optional(o) => ReversedChildren::Optional(o.into_iter()),
        WidgetChildren::Empty => ReversedChildren::Empty,
    }
}

/// Reverse iterator over a widget's children, yielding [`ObjectId`] values.
enum ReversedChildren<'a> {
    Slice(core::iter::Rev<core::slice::Iter<'a, ObjectId>>),
    Optional(core::option::IntoIter<ObjectId>),
    Empty,
}

impl Iterator for ReversedChildren<'_> {
    type Item = ObjectId;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Slice(it) => it.next().copied(),
            Self::Optional(it) => it.next(),
            Self::Empty => None,
        }
    }
}
