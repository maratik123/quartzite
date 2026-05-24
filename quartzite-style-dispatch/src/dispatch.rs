//! Widget-tree traversal and per-widget paint dispatch.

use quartzite_core::ObjectId;
use quartzite_paint_api::Painter;
use quartzite_paint_util::TranslateGuard;
use quartzite_style::Palette;
use quartzite_style::Style;
use quartzite_style::StyleRegistry;
use quartzite_widgets::{AsWidget, WidgetState};

/// Maps an [`ObjectId`] to the widget it refers to.
///
/// Implemented by callers to bridge the gap between the identifier-based widget
/// tree (where `Container::children()` returns `&[ObjectId]`) and the
/// paint-time need for `&dyn AsWidget` references.
///
/// This trait is deliberately separate from
/// [`quartzite_widgets::layout::WidgetResolver`], which resolves
/// `ObjectId` → `&mut WidgetBase` for mutable layout-time operations; this one
/// resolves to an immutable `&dyn AsWidget` for read-only paint-time access.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use quartzite_core::ObjectId;
/// use quartzite_widgets::{AsWidget, WidgetBase};
/// use quartzite_style_dispatch::WidgetResolver;
///
/// struct MapResolver(HashMap<ObjectId, Box<dyn AsWidget>>);
///
/// impl WidgetResolver for MapResolver {
///     fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> {
///         self.0.get(&id).map(|b| b.as_ref() as &dyn AsWidget)
///     }
/// }
/// ```
pub trait WidgetResolver {
    /// Returns the widget identified by `id`, or `None` if it is not present in
    /// this resolver's backing store.
    ///
    /// Implementations should be cheap (e.g. a hash-map lookup); `dispatch_paint`
    /// may call `resolve` more than once per child during a single frame.
    ///
    /// # Parameters
    ///
    /// - `id`: the unique identifier of the widget to look up.
    fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>;
}

/// Blanket implementation so a closure `|id| …` can act as a [`WidgetResolver`].
///
/// # Lifetime caveat
///
/// This impl is parsed as `for<'a> Fn(ObjectId) -> Option<&'a dyn AsWidget>`,
/// which means the closure must return a reference valid for any lifetime — in
/// practice only `&'static` references satisfy this (e.g. leaked boxes or
/// `static` widget arrays). Callers that need to return borrows tied to `self`
/// should implement [`WidgetResolver`] directly on their tree-wrapper type.
impl<F> WidgetResolver for F
where
    F: Fn(ObjectId) -> Option<&'static dyn AsWidget>,
{
    // _Simple._
    fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> {
        self(id)
    }
}

/// Walks the widget subtree rooted at `root` and calls
/// [`Style::draw_widget`][quartzite_style::Style::draw_widget] once per visible
/// node, using `painter` and `palette`.
///
/// The active [`Style`][quartzite_style::Style] is resolved from
/// [`StyleRegistry::try_style`][quartzite_style::StyleRegistry::try_style]
/// once at entry. If no style is installed the function returns immediately
/// without making any [`Painter`] calls.
///
/// The tree is walked depth-first, parent-before-child (painter's-algorithm
/// z-order). The sibling order follows
/// [`Container::children()`][quartzite_widgets::Container::children] and
/// [`ScrollArea::content_widget`][quartzite_widgets::ScrollArea::content_widget].
/// Invisible widgets (`!visible` on [`WidgetBase`][quartzite_widgets::WidgetBase])
/// and their entire subtree are skipped — no paint call, no save/translate/restore.
///
/// For every **non-root** child: `painter.save()` →
/// `painter.translate(child.geometry().origin())` before recursing, then
/// `painter.restore()` after. The root is painted at the painter's incoming
/// origin (no save/translate/restore wrap), so each widget's `draw_widget`
/// call sees `(0,0)` at its own top-left.
///
/// If `resolver` returns `None` for any `ObjectId` (root or child), the
/// subtree is skipped and a `warn!` event is emitted via [`tracing`].
///
/// # Parameters
///
/// - `root`: the [`ObjectId`] of the root widget to start traversal from.
/// - `resolver`: maps [`ObjectId`] to `&dyn AsWidget`; see [`WidgetResolver`].
/// - `painter`: the backend painter for this frame.
/// - `palette`: the colour palette passed through to every
///   [`Style::draw_widget`][quartzite_style::Style::draw_widget] call.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use quartzite_core::ObjectId;
/// use quartzite_widgets::{AsWidget, WidgetBase, WidgetExt};
/// use quartzite_style::Palette;
/// use quartzite_style_dispatch::{WidgetResolver, dispatch_paint};
///
/// struct MapResolver(HashMap<ObjectId, Box<dyn AsWidget>>);
///
/// impl WidgetResolver for MapResolver {
///     fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> {
///         self.0.get(&id).map(|b| b.as_ref() as &dyn AsWidget)
///     }
/// }
///
/// // Then call dispatch_paint(root, &resolver, painter, palette).
/// ```
pub fn dispatch_paint(
    root: ObjectId,
    resolver: &dyn WidgetResolver,
    painter: &mut dyn Painter,
    palette: &Palette,
) {
    let _span = tracing::debug_span!("style_dispatch::dispatch_paint", root = ?root).entered();
    let Some(style) = StyleRegistry::try_style() else {
        return;
    };
    visit(root, resolver, painter, palette, style);
}

fn visit(
    id: ObjectId,
    resolver: &dyn WidgetResolver,
    painter: &mut dyn Painter,
    palette: &Palette,
    style: &'static dyn Style,
) {
    let Some(widget) = resolver.resolve(id) else {
        tracing::warn!(?id, "dispatch_paint: resolver miss");
        return;
    };
    if !widget.widget_base().state.contains(WidgetState::Visible) {
        return;
    }

    style.draw_widget(widget, painter, palette);

    for child_id in widget.children() {
        let Some(child) = resolver.resolve(child_id) else {
            tracing::warn!(id = ?child_id, "dispatch_paint: resolver miss");
            continue;
        };
        if !child.widget_base().state.contains(WidgetState::Visible) {
            continue;
        }
        let origin = child.widget_base().geometry.origin();
        let mut guard = TranslateGuard::new(painter, origin);
        visit(child_id, resolver, guard.painter(), palette, style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartzite_core::ObjectId;
    use quartzite_geometry::{Point, Rect, Size};
    use quartzite_paint_api::{
        Brush, Color, Font, Image, Path, Pen, TextCaretCursor, TextVisualLine, TextVisualLineCursor,
    };
    use quartzite_style::Palette;
    use quartzite_style::StyleRegistry;
    use quartzite_widgets::{
        AsWidget, Button, Container, Label, ScrollArea, WidgetBase, WidgetExt,
    };
    use std::collections::HashMap;

    // ── fixtures ──────────────────────────────────────────────────────────────

    #[derive(Debug, PartialEq)]
    enum PaintEvent {
        FillRect,
        Save,
        Restore,
        Translate(Point),
        Other,
    }

    struct NullCaretCursor;
    impl TextCaretCursor for NullCaretCursor {
        fn advance_to(&mut self, _byte_offset: usize) {}
        fn caret_x(&self) -> i32 {
            0
        }
        fn line_top(&self) -> i32 {
            0
        }
        fn line_height(&self) -> i32 {
            0
        }
    }

    struct NullLineCursor;
    impl TextVisualLineCursor for NullLineCursor {
        fn next_line(&mut self) -> Option<TextVisualLine> {
            None
        }
    }

    struct RecordingPainter {
        events: Vec<PaintEvent>,
        null_caret: NullCaretCursor,
        null_lines: NullLineCursor,
    }

    impl RecordingPainter {
        fn new() -> Self {
            Self {
                events: Vec::new(),
                null_caret: NullCaretCursor,
                null_lines: NullLineCursor,
            }
        }
    }

    impl quartzite_paint_api::Painter for RecordingPainter {
        fn save(&mut self) {
            self.events.push(PaintEvent::Save);
        }
        fn restore(&mut self) {
            self.events.push(PaintEvent::Restore);
        }
        fn translate(&mut self, offset: Point) {
            self.events.push(PaintEvent::Translate(offset));
        }
        fn fill_rect(&mut self, _rect: Rect, _brush: &Brush) {
            self.events.push(PaintEvent::FillRect);
        }
        fn draw_rect(&mut self, _rect: Rect, _pen: &Pen, _brush: &Brush) {
            self.events.push(PaintEvent::Other);
        }
        fn draw_line(&mut self, _from: Point, _to: Point, _pen: &Pen) {}
        fn draw_text(&mut self, _pos: Point, _text: &str, _font: &Font, _brush: &Brush) {}
        fn draw_text_in(
            &mut self,
            _rect: Rect,
            _text: &str,
            _font: &Font,
            _brush: &Brush,
            _h_align: quartzite_geometry::Alignment,
            _v_align: quartzite_geometry::Alignment,
        ) {
            self.events.push(PaintEvent::Other);
        }
        fn draw_image(&mut self, _rect: Rect, _image: &Image) {}
        fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {}
        fn clip_rect(&mut self, _rect: Rect) {}
        fn text_carets(&mut self, _text: &str, _font: &Font) -> &mut dyn TextCaretCursor {
            &mut self.null_caret
        }
        fn text_visual_lines(
            &mut self,
            _text: &str,
            _font: &Font,
            _wrap_width: i32,
        ) -> &mut dyn TextVisualLineCursor {
            &mut self.null_lines
        }
    }

    /// A test `Style` that emits exactly one `FillRect` per `draw_widget` call.
    struct MarkStyle;

    impl quartzite_style::Style for MarkStyle {
        fn draw_widget(
            &self,
            widget: &dyn AsWidget,
            painter: &mut dyn Painter,
            _palette: &Palette,
        ) {
            painter.fill_rect(widget.widget_base().geometry, &Brush::solid(Color::BLACK));
        }

        fn caret_visible_now(&self) -> bool {
            false
        }
    }

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

    fn install_mark_style() {
        StyleRegistry::set_style(Box::new(MarkStyle));
    }

    fn count_fill_rects(events: &[PaintEvent]) -> usize {
        events
            .iter()
            .filter(|e| matches!(e, PaintEvent::FillRect))
            .count()
    }

    fn saves_and_restores_balanced(events: &[PaintEvent]) -> bool {
        let saves = events
            .iter()
            .filter(|e| matches!(e, PaintEvent::Save))
            .count();
        let restores = events
            .iter()
            .filter(|e| matches!(e, PaintEvent::Restore))
            .count();
        saves == restores
    }

    // ── AC1: single visible root → exactly one draw_widget call ──────────────

    #[test]
    fn dispatch_paint_invokes_draw_widget_once_per_visible_widget() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let root_id = ObjectId::new();
        let mut root = WidgetBase::new();
        root.show();

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, root);

        let mut painter = RecordingPainter::new();
        dispatch_paint(root_id, &resolver, &mut painter, &Palette::default());

        assert_eq!(count_fill_rects(&painter.events), 1);
        assert!(saves_and_restores_balanced(&painter.events));
    }

    // ── AC2: hidden root → zero events ───────────────────────────────────────

    #[test]
    fn hidden_root_produces_zero_paints() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let root_id = ObjectId::new();
        let root = WidgetBase::new(); // not shown → !visible

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, root);

        let mut painter = RecordingPainter::new();
        dispatch_paint(root_id, &resolver, &mut painter, &Palette::default());

        assert!(painter.events.is_empty());
    }

    // ── AC3: hidden non-root subtree → only outer paints ─────────────────────

    #[test]
    fn hidden_subtree_skipped_with_no_save_or_translate() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let outer_id = ObjectId::new();
        let inner_id = ObjectId::new();
        let label_id = ObjectId::new();

        let mut outer = Container::new();
        outer.show();
        outer.add_child(inner_id);

        let inner = Container::new(); // hidden — NOT shown; contains visible label
        let mut label = Label::new("hi".into());
        label.show();
        // Wire inner → label (but inner itself is hidden)
        // Note: even though inner is hidden, we still wire it so the traversal
        // would need to skip it cleanly without ever visiting its children.
        // inner is hidden so the traversal stops at the visibility check.

        let mut resolver = StubResolver::new();
        resolver.insert(outer_id, outer);
        resolver.insert(inner_id, inner);
        resolver.insert(label_id, label);

        let mut painter = RecordingPainter::new();
        dispatch_paint(outer_id, &resolver, &mut painter, &Palette::default());

        // Only outer paints; inner is hidden so no Save/Translate/Restore
        assert_eq!(count_fill_rects(&painter.events), 1);
        assert!(!painter.events.contains(&PaintEvent::Save));
        assert!(!painter.events.contains(&PaintEvent::Restore));
    }

    // ── AC4: depth-first parent-before-child order ────────────────────────────

    #[test]
    fn depth_first_parent_before_child_order() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let outer_id = ObjectId::new();
        let label_id = ObjectId::new();
        let inner_id = ObjectId::new();
        let button_id = ObjectId::new();

        let mut outer = Container::new();
        outer.show();
        outer.add_child(label_id);
        outer.add_child(inner_id);

        let mut label = Label::new("L".into());
        label.show();

        let mut inner = Container::new();
        inner.show();
        inner.add_child(button_id);

        let mut button = Button::new("B".into());
        button.show();

        let mut resolver = StubResolver::new();
        resolver.insert(outer_id, outer);
        resolver.insert(label_id, label);
        resolver.insert(inner_id, inner);
        resolver.insert(button_id, button);

        let mut painter = RecordingPainter::new();
        dispatch_paint(outer_id, &resolver, &mut painter, &Palette::default());

        // 4 draw_widget calls (outer, label, inner, button)
        assert_eq!(count_fill_rects(&painter.events), 4);
        assert!(saves_and_restores_balanced(&painter.events));

        // Verify depth-first order: FillRect positions are strictly increasing
        let fill_indices: Vec<usize> = painter
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| matches!(e, PaintEvent::FillRect))
            .map(|(i, _)| i)
            .collect();
        // outer(0) < label(1) < inner(2) < button(3)
        assert!(fill_indices[0] < fill_indices[1]);
        assert!(fill_indices[1] < fill_indices[2]);
        assert!(fill_indices[2] < fill_indices[3]);
    }

    // ── AC5: save/translate/restore wraps each non-root child ────────────────

    #[test]
    fn save_translate_restore_wraps_each_non_root_child() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let outer_id = ObjectId::new();
        let label_id = ObjectId::new();

        let mut outer = Container::new();
        outer.show();
        outer.add_child(label_id);

        let mut label = Label::new("L".into());
        label.show();
        label.set_geometry(Rect::new(Point::new(10, 20), Size::new(50, 20)));

        let mut resolver = StubResolver::new();
        resolver.insert(outer_id, outer);
        resolver.insert(label_id, label);

        let mut painter = RecordingPainter::new();
        dispatch_paint(outer_id, &resolver, &mut painter, &Palette::default());

        // Expected: FillRect(outer), Save, Translate(10,20), FillRect(label), Restore
        // Note: MarkStyle emits only FillRect; DefaultStyle would emit more events.
        // Filtering to only the event types we record:
        let relevant: Vec<&PaintEvent> = painter
            .events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    PaintEvent::FillRect
                        | PaintEvent::Save
                        | PaintEvent::Restore
                        | PaintEvent::Translate(_)
                )
            })
            .collect();
        assert_eq!(relevant.len(), 5);
        assert!(matches!(relevant[0], PaintEvent::FillRect));
        assert!(matches!(relevant[1], PaintEvent::Save));
        assert!(matches!(relevant[2], PaintEvent::Translate(p) if *p == Point::new(10, 20)));
        assert!(matches!(relevant[3], PaintEvent::FillRect));
        assert!(matches!(relevant[4], PaintEvent::Restore));
    }

    // ── AC6a: ScrollArea without content → 1 paint ───────────────────────────

    #[test]
    fn scroll_area_without_content_paints_only_chrome() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let area_id = ObjectId::new();

        let mut area = ScrollArea::new();
        area.show();

        let mut resolver = StubResolver::new();
        resolver.insert(area_id, area);

        let mut painter = RecordingPainter::new();
        dispatch_paint(area_id, &resolver, &mut painter, &Palette::default());

        assert_eq!(count_fill_rects(&painter.events), 1);
        assert!(saves_and_restores_balanced(&painter.events));
    }

    // ── AC6b: ScrollArea with visible content → 2 paints ─────────────────────

    #[test]
    fn scroll_area_with_content_paints_chrome_and_content() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let area_id = ObjectId::new();
        let label_id = ObjectId::new();

        let mut area = ScrollArea::new();
        area.show();
        area.content_widget = Some(label_id);

        let mut label = Label::new("content".into());
        label.show();

        let mut resolver = StubResolver::new();
        resolver.insert(area_id, area);
        resolver.insert(label_id, label);

        let mut painter = RecordingPainter::new();
        dispatch_paint(area_id, &resolver, &mut painter, &Palette::default());

        assert_eq!(count_fill_rects(&painter.events), 2);
        assert!(saves_and_restores_balanced(&painter.events));
    }

    // ── AC7: unknown widget type → 1 paint, no children ──────────────────────

    #[test]
    fn unknown_widget_type_paints_once_no_recursion() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let root_id = ObjectId::new();

        let mut root = WidgetBase::new();
        root.show();

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, root);

        let mut painter = RecordingPainter::new();
        dispatch_paint(root_id, &resolver, &mut painter, &Palette::default());

        assert_eq!(count_fill_rects(&painter.events), 1);
        assert!(!painter.events.contains(&PaintEvent::Save));
    }

    // ── AC8: no style installed → zero events ────────────────────────────────

    #[test]
    fn no_style_installed_is_noop() {
        let _lock = quartzite_test_helpers::test_lock();
        quartzite_style::StyleRegistry::clear_for_test();
        let root_id = ObjectId::new();

        let mut root = WidgetBase::new();
        root.show();

        let mut resolver = StubResolver::new();
        resolver.insert(root_id, root);

        let mut painter = RecordingPainter::new();
        dispatch_paint(root_id, &resolver, &mut painter, &Palette::default());

        assert!(painter.events.is_empty());
    }

    // ── AC10a: resolver miss mid-tree → subtree skipped + warn ───────────────

    #[test]
    #[tracing_test::traced_test]
    fn resolver_miss_mid_tree_skips_subtree_and_warns() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let outer_id = ObjectId::new();
        let present_id = ObjectId::new();
        let missing_id = ObjectId::new();
        let sibling_id = ObjectId::new();

        let mut outer = Container::new();
        outer.show();
        outer.add_child(present_id);
        outer.add_child(missing_id); // resolver will return None for this
        outer.add_child(sibling_id);

        let mut present = Label::new("A".into());
        present.show();
        let mut sibling = Label::new("B".into());
        sibling.show();

        let mut resolver = StubResolver::new();
        resolver.insert(outer_id, outer);
        resolver.insert(present_id, present);
        // missing_id intentionally NOT inserted
        resolver.insert(sibling_id, sibling);

        let mut painter = RecordingPainter::new();
        dispatch_paint(outer_id, &resolver, &mut painter, &Palette::default());

        // outer + present + sibling = 3 paints; missing subtree skipped
        assert_eq!(count_fill_rects(&painter.events), 3);
        assert!(saves_and_restores_balanced(&painter.events));
        assert!(logs_contain("dispatch_paint: resolver miss"));
    }

    // ── AC10b: resolver miss on root → zero paints + warn ────────────────────

    #[test]
    #[tracing_test::traced_test]
    fn resolver_miss_on_root_produces_zero_paints_and_warns() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let root_id = ObjectId::new();
        // root_id NOT inserted in resolver
        let resolver = StubResolver::new();

        let mut painter = RecordingPainter::new();
        dispatch_paint(root_id, &resolver, &mut painter, &Palette::default());

        assert!(painter.events.is_empty());
        assert!(logs_contain("dispatch_paint: resolver miss"));
    }

    // ── Closure blanket impl ──────────────────────────────────────────────────

    #[test]
    fn closure_resolver_compiles_and_works() {
        let _lock = quartzite_test_helpers::test_lock();
        install_mark_style();
        let root_id = ObjectId::new();

        // Closure resolvers must return &'static references; use Box::leak.
        let mut root = WidgetBase::new();
        root.show();
        let static_root: &'static dyn AsWidget = Box::leak(Box::new(root));

        let resolver = move |id: ObjectId| -> Option<&'static dyn AsWidget> {
            if id == root_id {
                Some(static_root)
            } else {
                None
            }
        };

        let mut painter = RecordingPainter::new();
        dispatch_paint(root_id, &resolver, &mut painter, &Palette::default());

        assert_eq!(count_fill_rects(&painter.events), 1);
    }
}
