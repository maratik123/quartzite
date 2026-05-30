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

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_geometry::Size;
    use quartzite_widgets::{AsWidget, Container, Label, ScrollArea, WidgetBase, WidgetExt};
    use std::assert_matches;
    use std::collections::HashMap;

    // ── fixtures ──────────────────────────────────────────────────────────────

    /// Paint-free HashMap-backed resolver, ported from `dispatch.rs` without any
    /// `RecordingPainter` / `MarkStyle` / `StyleRegistry` machinery (hit-test is
    /// paint-free, so no `test_lock()` style-registry serialization is needed).
    struct StubResolver(HashMap<ObjectId, Box<dyn AsWidget>>);

    impl StubResolver {
        fn new() -> Self {
            Self(HashMap::new())
        }

        fn insert<W: AsWidget + 'static>(&mut self, id: ObjectId, widget: W) {
            self.0.insert(id, Box::new(widget));
        }
    }

    impl WidgetResolver for StubResolver {
        fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> {
            self.0.get(&id).map(|b| b.as_ref() as &dyn AsWidget)
        }
    }

    /// Builds a visible `Label` leaf with the given geometry.
    fn leaf(geom: Rect) -> Label {
        let mut label = Label::new("leaf".into());
        label.show();
        label.set_geometry(geom);
        label
    }

    /// Builds a visible `Container` with the given geometry and children (in
    /// paint / child-iteration order — earlier entries paint first / below).
    fn container(children: &[ObjectId], geom: Rect) -> Container {
        let mut c = Container::new();
        c.show();
        c.set_geometry(geom);
        for &child in children {
            c.add_child(child);
        }
        c
    }

    fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(w, h))
    }

    /// Test-only widget that clips its single child to a fixed rect (à la the
    /// `ClippingWidget` in `dispatch.rs`). `children_clip_rect()` is in the
    /// widget's own local space.
    #[derive(Debug)]
    struct ClippingWidget {
        base: WidgetBase,
        child: ObjectId,
        clip: Rect,
    }

    impl quartzite_core::AsObject for ClippingWidget {
        fn object_base(&self) -> &quartzite_core::ObjectBase {
            self.base.object_base()
        }
        fn object_base_mut(&mut self) -> &mut quartzite_core::ObjectBase {
            self.base.object_base_mut()
        }
        fn as_any(&self) -> &dyn ::core::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any {
            self
        }
    }

    impl AsWidget for ClippingWidget {
        fn widget_base(&self) -> &WidgetBase {
            &self.base
        }
        fn widget_base_mut(&mut self) -> &mut WidgetBase {
            &mut self.base
        }
        fn widget_view(&self) -> quartzite_widgets::WidgetView<'_> {
            quartzite_widgets::WidgetView::Other(self)
        }
        fn children(&self) -> WidgetChildren<'_> {
            WidgetChildren::Optional(Some(self.child))
        }
        fn children_clip_rect(&self) -> Option<Rect> {
            Some(self.clip)
        }
    }

    // ── AC1: single visible root ──────────────────────────────────────────────

    #[test]
    fn visible_root_containing_point_is_hit_with_zero_offset() {
        let root_id = ObjectId::new();
        let mut resolver = StubResolver::new();
        resolver.insert(root_id, leaf(rect(0, 0, 100, 100)));

        assert_eq!(
            hit_test(root_id, Point::new(50, 50), &resolver),
            Some((root_id, Point::default())),
        );
    }

    #[test]
    fn point_outside_root_geometry_misses() {
        let root_id = ObjectId::new();
        let mut resolver = StubResolver::new();
        resolver.insert(root_id, leaf(rect(0, 0, 100, 100)));

        assert_eq!(hit_test(root_id, Point::new(150, 150), &resolver), None);
    }

    // ── AC1b: accumulated offset over nested origins ──────────────────────────

    #[test]
    fn nested_hit_returns_summed_parent_relative_origins() {
        // Container(0,0) { Inner(10,20) { Leaf(5,5) } }
        let root_id = ObjectId::new();
        let inner_id = ObjectId::new();
        let leaf_id = ObjectId::new();

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, container(&[inner_id], rect(0, 0, 100, 100)));
        resolver.insert(inner_id, container(&[leaf_id], rect(10, 20, 80, 70)));
        resolver.insert(leaf_id, leaf(rect(5, 5, 30, 30)));

        // Point (20,30) root-local: inside Inner (>=10,20) and inside Leaf
        // (>=15,25 after Inner+Leaf origins). Offset = (10,20)+(5,5) = (15,25).
        let point = Point::new(20, 30);
        let hit = hit_test(root_id, point, &resolver);
        assert_eq!(hit, Some((leaf_id, Point::new(15, 25))));

        // Subtracting the offset maps the root-local point into Leaf-local space,
        // which must land inside Leaf's own size-rect (origin-zero, 30×30).
        let (_, offset) = hit.expect("hit present");
        let leaf_local = point - offset;
        assert_eq!(leaf_local, Point::new(5, 5));
        assert!(rect(0, 0, 30, 30).contains(leaf_local));
    }

    // ── AC2: reverse z-order — later-iterated sibling wins ────────────────────

    #[test]
    fn two_siblings_same_geometry_topmost_iterated_last_wins() {
        let root_id = ObjectId::new();
        let first_id = ObjectId::new();
        let second_id = ObjectId::new();

        let mut resolver = StubResolver::new();
        // first added before second → first paints below, second paints on top.
        resolver.insert(
            root_id,
            container(&[first_id, second_id], rect(0, 0, 100, 100)),
        );
        resolver.insert(first_id, leaf(rect(10, 10, 40, 40)));
        resolver.insert(second_id, leaf(rect(10, 10, 40, 40)));

        // Point inside both siblings → the visually-topmost (second) wins.
        assert_eq!(
            hit_test(root_id, Point::new(20, 20), &resolver),
            Some((second_id, Point::new(10, 10))),
        );
    }

    // ── AC3: child-before-parent + boundary semantics ─────────────────────────

    #[test]
    fn point_in_child_hits_child_not_parent() {
        let root_id = ObjectId::new();
        let child_id = ObjectId::new();

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, container(&[child_id], rect(0, 0, 100, 100)));
        resolver.insert(child_id, leaf(rect(10, 10, 30, 30)));

        assert_eq!(
            hit_test(root_id, Point::new(15, 15), &resolver),
            Some((child_id, Point::new(10, 10))),
        );
    }

    #[test]
    fn point_on_parent_chrome_outside_children_hits_parent() {
        let root_id = ObjectId::new();
        let child_id = ObjectId::new();

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, container(&[child_id], rect(0, 0, 100, 100)));
        resolver.insert(child_id, leaf(rect(10, 10, 30, 30)));

        // (5,5) is inside the parent but outside the child.
        assert_eq!(
            hit_test(root_id, Point::new(5, 5), &resolver),
            Some((root_id, Point::default())),
        );
    }

    #[test]
    fn child_boundary_is_inclusive_left_top_exclusive_right_bottom() {
        let root_id = ObjectId::new();
        let child_id = ObjectId::new();

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, container(&[child_id], rect(0, 0, 100, 100)));
        // Child occupies [10,40) × [10,40) in root-local space.
        resolver.insert(child_id, leaf(rect(10, 10, 30, 30)));

        // Inclusive top-left corner → child.
        assert_eq!(
            hit_test(root_id, Point::new(10, 10), &resolver),
            Some((child_id, Point::new(10, 10))),
        );
        // Exclusive bottom-right corner (40,40) → falls back to parent.
        assert_eq!(
            hit_test(root_id, Point::new(40, 40), &resolver),
            Some((root_id, Point::default())),
        );
    }

    // ── AC4: coordinate transform (local-space vs parent-relative guard) ──────

    #[test]
    fn coordinate_transform_routes_point_into_label_local_space() {
        // Container(0,0) { Label(10,20, 50×20) }
        let root_id = ObjectId::new();
        let label_id = ObjectId::new();

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, container(&[label_id], rect(0, 0, 100, 100)));
        resolver.insert(label_id, leaf(rect(10, 20, 50, 20)));

        // (15,25) is inside the label (label-local (5,5)).
        assert_eq!(
            hit_test(root_id, Point::new(15, 25), &resolver),
            Some((label_id, Point::new(10, 20))),
        );
        // (5,5) is inside the container but outside the label.
        assert_eq!(
            hit_test(root_id, Point::new(5, 5), &resolver),
            Some((root_id, Point::default())),
        );
    }

    // ── AC5: visibility skips widget + whole subtree ──────────────────────────

    #[test]
    fn hidden_root_misses() {
        let root_id = ObjectId::new();
        let mut root = Container::new();
        // intentionally NOT shown
        root.set_geometry(rect(0, 0, 100, 100));

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, root);

        assert_eq!(hit_test(root_id, Point::new(50, 50), &resolver), None);
    }

    #[test]
    fn hidden_child_subtree_skipped_parent_is_hit() {
        let root_id = ObjectId::new();
        let child_id = ObjectId::new();

        let mut hidden_child = Container::new();
        // hidden: NOT shown
        hidden_child.set_geometry(rect(10, 10, 30, 30));

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, container(&[child_id], rect(0, 0, 100, 100)));
        resolver.insert(child_id, hidden_child);

        // Point inside the hidden child's geometry → child skipped, parent hit.
        assert_eq!(
            hit_test(root_id, Point::new(15, 15), &resolver),
            Some((root_id, Point::default())),
        );
    }

    #[test]
    fn hidden_child_with_point_outside_parent_misses() {
        let root_id = ObjectId::new();
        let child_id = ObjectId::new();

        let mut hidden_child = Container::new();
        hidden_child.set_geometry(rect(10, 10, 30, 30));

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, container(&[child_id], rect(0, 0, 50, 50)));
        resolver.insert(child_id, hidden_child);

        // (200,200) is outside both → None even though it would be in the child's
        // local subtree had the child been visible.
        assert_eq!(hit_test(root_id, Point::new(200, 200), &resolver), None);
    }

    // ── AC6: clip gates children ──────────────────────────────────────────────

    #[test]
    fn point_outside_clip_does_not_hit_child() {
        let root_id = ObjectId::new();
        let child_id = ObjectId::new();

        let mut root = ClippingWidget {
            base: WidgetBase::new(),
            child: child_id,
            // clip = [5,5)..(80,60) in root-local space.
            clip: rect(5, 5, 75, 55),
        };
        root.show();
        root.set_geometry(rect(0, 0, 100, 100));

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, root);
        // Child geometry extends below the clip's bottom edge (y up to 90).
        resolver.insert(child_id, leaf(rect(10, 10, 60, 80)));

        // (40,70): inside child geometry but BELOW the clip (clip bottom = 60) →
        // child not exposed; the clipping widget itself is the hit.
        assert_eq!(
            hit_test(root_id, Point::new(40, 70), &resolver),
            Some((root_id, Point::default())),
        );
    }

    #[test]
    fn point_inside_clip_and_child_hits_child() {
        let root_id = ObjectId::new();
        let child_id = ObjectId::new();

        let mut root = ClippingWidget {
            base: WidgetBase::new(),
            child: child_id,
            clip: rect(5, 5, 75, 55),
        };
        root.show();
        root.set_geometry(rect(0, 0, 100, 100));

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, root);
        resolver.insert(child_id, leaf(rect(10, 10, 60, 80)));

        // (40,40): inside both clip and child geometry → child wins.
        assert_eq!(
            hit_test(root_id, Point::new(40, 40), &resolver),
            Some((child_id, Point::new(10, 10))),
        );
    }

    #[test]
    fn scroll_area_clip_gates_content() {
        // ScrollArea exercises the real `children_clip_rect()` path.
        let area_id = ObjectId::new();
        let label_id = ObjectId::new();

        let mut area = ScrollArea::new();
        area.show();
        area.set_geometry(rect(0, 0, 100, 80));
        area.content_widget = Some(label_id);

        let label = leaf(rect(0, 0, 100, 80));

        let mut resolver = StubResolver::new();
        resolver.insert(area_id, area);
        resolver.insert(label_id, label);

        // A point well inside the scroll area + content hits the content label.
        assert_matches!(
            hit_test(area_id, Point::new(20, 20), &resolver),
            Some((id, _)) if id == label_id
        );
    }

    // ── AC7: resolver miss → warn + skip subtree, siblings still tested ───────

    #[test]
    #[tracing_test::traced_test]
    fn resolver_miss_mid_tree_skips_subtree_warns_siblings_still_hit() {
        let root_id = ObjectId::new();
        let missing_id = ObjectId::new();
        let sibling_id = ObjectId::new();

        let mut resolver = StubResolver::new();
        // missing_id added LAST so it is iterated FIRST (reverse order); its
        // subtree must be skipped, then the sibling is still tested.
        resolver.insert(
            root_id,
            container(&[sibling_id, missing_id], rect(0, 0, 100, 100)),
        );
        // missing_id intentionally NOT inserted.
        resolver.insert(sibling_id, leaf(rect(10, 10, 30, 30)));

        // (15,15) is inside the sibling → sibling is the hit despite the miss.
        assert_eq!(
            hit_test(root_id, Point::new(15, 15), &resolver),
            Some((sibling_id, Point::new(10, 10))),
        );
        assert!(logs_contain("hit_test: resolver miss"));
    }

    #[test]
    #[tracing_test::traced_test]
    fn resolver_miss_on_root_misses_and_warns() {
        let root_id = ObjectId::new();
        // root_id NOT inserted.
        let resolver = StubResolver::new();

        assert_eq!(hit_test(root_id, Point::new(0, 0), &resolver), None);
        assert!(logs_contain("hit_test: resolver miss"));
    }

    // ── AC8: no containing widget anywhere → None, no panic ───────────────────

    #[test]
    fn no_containing_widget_in_visible_tree_misses() {
        let root_id = ObjectId::new();
        let child_id = ObjectId::new();

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, container(&[child_id], rect(0, 0, 40, 40)));
        resolver.insert(child_id, leaf(rect(5, 5, 10, 10)));

        // (200,200) is outside both root and child → None, no panic.
        assert_eq!(hit_test(root_id, Point::new(200, 200), &resolver), None);
    }
}
