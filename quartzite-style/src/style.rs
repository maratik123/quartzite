//! [`Style`] — trait describing how to paint a widget.
//!
//! Concrete [`Style`] implementations route on widget type via the **Hybrid
//! `Paint<W>`** mechanism: a widget-side dispatch hook
//! ([`AsWidget::widget_view`](quartzite_widgets::AsWidget::widget_view)) returns a
//! [`WidgetView`](quartzite_widgets::WidgetView) that the style pattern-matches,
//! routing each built-in variant to the matching typed
//! [`Paint<W>`](crate::Paint) impl. Third-party widgets surface as
//! [`WidgetView::Other`](quartzite_widgets::WidgetView::Other) — the open-set
//! escape hatch. The [`Style`] trait carries a `Send + Sync` bound because
//! [`StyleRegistry`](crate::StyleRegistry) hands out `&'static dyn Style`
//! references reachable from any thread.
//!
//! ## Implementing `Paint<W>` for a third-party widget
//!
//! A crate that defines a custom widget can integrate with the dispatch system
//! without modifying `quartzite-widgets` or `quartzite-style`:
//!
//! 1. Define the widget using `#[derive(Extend)]` (no `#[widget_view]` attribute
//!    → `widget_view()` returns `WidgetView::Other(self)` automatically).
//! 2. Implement [`Paint<MyWidget>`](crate::Paint) for your style type.
//! 3. Override [`Style::draw_widget`] to pattern-match `WidgetView::Other` and
//!    downcast the payload via `as_any()` (from `quartzite_core::AsObject`).
//!
//! ```ignore
//! use quartzite_core::AsObject;
//! use quartzite_macros::Extend;
//! use quartzite_paint_api::Painter;
//! use quartzite_style::{Paint, Palette, Style};
//! use quartzite_widgets::{AsWidget, WidgetBase, WidgetView};
//!
//! #[derive(Extend)]
//! struct MyWidget {
//!     #[base]
//!     widget_base: WidgetBase,
//! }
//!
//! struct MyStyle;
//!
//! impl Paint<MyWidget> for MyStyle {
//!     fn paint(&self, _widget: &MyWidget, _painter: &mut dyn Painter, _palette: &Palette) {
//!         // draw MyWidget
//!     }
//! }
//!
//! impl Style for MyStyle {
//!     fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
//!         if let WidgetView::Other(other) = widget.widget_view() {
//!             if let Some(w) = other.as_any().downcast_ref::<MyWidget>() {
//!                 self.paint(w, painter, palette);
//!             }
//!         }
//!     }
//! }
//! ```

use quartzite_paint_api::Painter;
use quartzite_style_types::Palette;
use quartzite_widgets::AsWidget;

/// Painting strategy for the widget tree.
///
/// A [`Style`] is the single hook a renderer uses to draw any concrete widget.
/// [`draw_widget`](Self::draw_widget) takes the widget as `&dyn AsWidget`, calls
/// [`widget.widget_view()`](quartzite_widgets::AsWidget::widget_view) to obtain a
/// [`WidgetView`](quartzite_widgets::WidgetView), and pattern-matches the result to
/// route each built-in variant to the matching typed
/// [`Paint<W>`](crate::Paint) impl. This is the **Hybrid `Paint<W>`** dispatch
/// mechanism — widget→style flow with statically-typed per-widget paint code.
///
/// ## Open-set contract
///
/// Third-party widgets (widgets not in `quartzite-widgets`) return
/// [`WidgetView::Other`](quartzite_widgets::WidgetView::Other) from
/// `widget_view()`. A custom [`Style`] that wants to handle a third-party widget
/// type overrides `draw_widget`, pattern-matches the `Other` payload, and
/// dispatches through its own [`Paint<W>`](crate::Paint) impl.
///
/// `Style: Send + Sync` is required so the global [`StyleRegistry`](crate::StyleRegistry)
/// can hand out `&'static dyn Style` references across threads.
///
/// # Examples
///
/// A minimal no-op style; sufficient to satisfy the trait, suitable as a
/// placeholder before a real renderer is wired up.
///
/// ```
/// use quartzite_paint_api::Painter;
/// use quartzite_style::{Palette, Style};
/// use quartzite_widgets::AsWidget;
///
/// struct NoopStyle;
///
/// impl Style for NoopStyle {
///     fn draw_widget(
///         &self,
///         _widget: &dyn AsWidget,
///         _painter: &mut dyn Painter,
///         _palette: &Palette,
///     ) {
///     }
///
///     fn caret_visible_now(&self) -> bool {
///         false
///     }
/// }
///
/// // Trait is object-safe — boxing through `dyn Style` compiles.
/// let _boxed: Box<dyn Style> = Box::new(NoopStyle);
/// ```
pub trait Style: Send + Sync {
    /// Paints `widget` using `painter`, resolving any colour references through `palette`.
    ///
    /// Implementors typically call
    /// [`widget.widget_view()`](quartzite_widgets::AsWidget::widget_view) and
    /// `match` the result, routing each
    /// [`WidgetView`](quartzite_widgets::WidgetView) variant to the corresponding
    /// [`Paint<W>`](crate::Paint) impl:
    ///
    /// ```text
    /// fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
    ///     match widget.widget_view() {
    ///         WidgetView::Button(w)  => self.paint(w, painter, palette),
    ///         WidgetView::Label(w)   => self.paint(w, painter, palette),
    ///         // … other built-ins …
    ///         WidgetView::Other(_)   => {} // documented no-op
    ///         _ => {}                      // #[non_exhaustive] catch-all
    ///     }
    /// }
    /// ```
    ///
    /// The [`WidgetView::Other`](quartzite_widgets::WidgetView::Other) arm is a
    /// documented silent no-op — unknown widget types do not panic. A custom
    /// style may pattern-match `Other`'s `&dyn AsWidget` payload and downcast
    /// via `as_any()` (from `quartzite_core::AsObject`) to handle third-party widgets
    /// through its own [`Paint<W>`](crate::Paint) impl.
    ///
    /// # Parameters
    ///
    /// - `widget`: the widget to paint, accessed via the [`AsWidget`] upcast.
    /// - `painter`: the active backend painter; methods on `&mut dyn Painter`
    ///   accumulate draw operations.
    /// - `palette`: colour-role lookup used by the implementor when resolving
    ///   widget colours.
    fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette);

    /// Returns `true` when the caret should be drawn in the current paint pass.
    ///
    /// The default implementation returns `false`, keeping the caret invisible.
    /// Concrete styles that own a [`crate::StyleClock`] should delegate to
    /// [`StyleClock::caret_visible_now`](crate::StyleClock::caret_visible_now).
    ///
    /// Paint paths call this method through `&self` — no `&mut` needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Painter;
    /// use quartzite_style::{Palette, Style, StyleClock};
    /// use quartzite_widgets::AsWidget;
    ///
    /// struct PinnedStyle {
    ///     clock: StyleClock,
    /// }
    ///
    /// impl Style for PinnedStyle {
    ///     fn draw_widget(
    ///         &self,
    ///         _widget: &dyn AsWidget,
    ///         _painter: &mut dyn Painter,
    ///         _palette: &Palette,
    ///     ) {
    ///     }
    ///
    ///     fn caret_visible_now(&self) -> bool {
    ///         self.clock.caret_visible_now()
    ///     }
    /// }
    ///
    /// let style = PinnedStyle { clock: StyleClock::pinned(true) };
    /// assert!(style.caret_visible_now());
    /// ```
    fn caret_visible_now(&self) -> bool;

    /// Returns `true` when the platform reports a reduced-motion accessibility
    /// preference, indicating that blinking animations should be suppressed.
    ///
    /// The default implementation returns `false`.  Concrete styles that query
    /// the platform for this preference should override the default.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Painter;
    /// use quartzite_style::{Palette, Style};
    /// use quartzite_widgets::AsWidget;
    ///
    /// struct NoopStyle;
    ///
    /// impl Style for NoopStyle {
    ///     fn draw_widget(
    ///         &self,
    ///         _widget: &dyn AsWidget,
    ///         _painter: &mut dyn Painter,
    ///         _palette: &Palette,
    ///     ) {
    ///     }
    ///
    ///     fn caret_visible_now(&self) -> bool {
    ///         false
    ///     }
    /// }
    ///
    /// // Default impl returns false — no reduced-motion signal.
    /// assert!(!NoopStyle.prefers_reduced_motion());
    /// ```
    #[inline]
    fn prefers_reduced_motion(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use quartzite_paint_api::{TextCaretCursor, TextVisualLine, TextVisualLineCursor};

    /// Counts every `draw_widget` call so tests can prove the body executed.
    static DRAW_CALLS: AtomicUsize = AtomicUsize::new(0);

    /// Zero-sized fixture proving the trait can be satisfied by a single method impl.
    struct OnlyDraw;

    impl Style for OnlyDraw {
        fn draw_widget(
            &self,
            _widget: &dyn AsWidget,
            _painter: &mut dyn Painter,
            _palette: &Palette,
        ) {
            DRAW_CALLS.fetch_add(1, Ordering::SeqCst);
        }

        fn caret_visible_now(&self) -> bool {
            false
        }
    }

    /// Recording painter — accepts every method as a no-op so we can dispatch
    /// `draw_widget` against a `&mut dyn Painter` without bringing in renderer.
    struct NullPainter {
        null_caret: NullCaretCursor,
        null_lines: NullLineCursor,
    }

    impl NullPainter {
        fn new() -> Self {
            Self {
                null_caret: NullCaretCursor,
                null_lines: NullLineCursor,
            }
        }
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

    impl Painter for NullPainter {
        fn draw_rect(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _pen: &quartzite_paint_api::Pen,
            _brush: &quartzite_paint_api::Brush,
        ) {
        }
        fn fill_rect(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _brush: &quartzite_paint_api::Brush,
        ) {
        }
        fn draw_line(
            &mut self,
            _from: quartzite_geometry::Point,
            _to: quartzite_geometry::Point,
            _pen: &quartzite_paint_api::Pen,
        ) {
        }
        fn clip_rect(&mut self, _rect: quartzite_geometry::Rect) {}
        fn translate(&mut self, _delta: quartzite_geometry::Point) {}
        fn save(&mut self) {}
        fn restore(&mut self) {}
        fn draw_text(
            &mut self,
            _pos: quartzite_geometry::Point,
            _text: &str,
            _font: &quartzite_paint_api::Font,
            _brush: &quartzite_paint_api::Brush,
        ) {
        }
        fn draw_text_in(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _text: &str,
            _font: &quartzite_paint_api::Font,
            _brush: &quartzite_paint_api::Brush,
            _h_align: quartzite_geometry::Alignment,
            _v_align: quartzite_geometry::Alignment,
        ) {
        }
        fn draw_image(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _image: &quartzite_paint_api::Image,
        ) {
        }
        fn draw_path(
            &mut self,
            _path: &quartzite_paint_api::Path,
            _pen: &quartzite_paint_api::Pen,
            _brush: &quartzite_paint_api::Brush,
        ) {
        }
        fn text_carets(
            &mut self,
            _text: &str,
            _font: &quartzite_paint_api::Font,
        ) -> &mut dyn TextCaretCursor {
            &mut self.null_caret
        }
        fn text_visual_lines(
            &mut self,
            _text: &str,
            _font: &quartzite_paint_api::Font,
            _wrap_width: i32,
        ) -> &mut dyn TextVisualLineCursor {
            &mut self.null_lines
        }
    }

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn concrete_style_with_only_draw_widget_satisfies_trait() {
        // Constructing the impl above and storing it as a trait object is
        // sufficient evidence that no other required methods exist; if the
        // trait grew an additional required method this would stop compiling.
        let _boxed: Box<dyn Style> = Box::new(OnlyDraw);
    }

    #[test]
    fn style_trait_object_is_send_sync() {
        // Box<dyn Style> being Send + Sync proves the trait carries the
        // bound and that the registry's &'static dyn Style is well-formed.
        assert_send_sync::<Box<dyn Style>>();
        assert_send_sync::<&'static dyn Style>();
    }

    #[test]
    fn caret_visible_now_dispatches_through_trait_object() {
        // Both new methods must be callable through &dyn Style.
        let style: Box<dyn Style> = Box::new(OnlyDraw);
        let s: &dyn Style = style.as_ref();
        // OnlyDraw::caret_visible_now returns false.
        assert!(!s.caret_visible_now(), "caret_visible_now must dispatch");
        // prefers_reduced_motion uses the default-impl returning false.
        assert!(
            !s.prefers_reduced_motion(),
            "prefers_reduced_motion must dispatch"
        );
    }

    #[test]
    #[allow(
        clippy::items_after_statements,
        reason = "nested helper placed after local setup is more readable here"
    )]
    fn draw_widget_dispatches_through_trait_object() {
        let style: Box<dyn Style> = Box::new(OnlyDraw);
        let mut painter = NullPainter::new();
        let palette = Palette::default();
        let widget = quartzite_widgets::WidgetBase::new();
        let before = DRAW_CALLS.load(Ordering::SeqCst);
        style.draw_widget(&widget, &mut painter, &palette);
        let after = DRAW_CALLS.load(Ordering::SeqCst);
        assert_eq!(after, before + 1, "draw_widget body did not run");

        // Exercise the NullPainter trait-impl bodies so coverage doesn't
        // regress on the no-op stubs.
        use quartzite_geometry::{Point, Rect, Size};
        use quartzite_paint_api::{Brush, Color, Font, Image, Pen};
        let p: &mut dyn Painter = &mut painter;
        let pen = Pen::new(Color::BLACK, 1.0);
        let brush = Brush::solid(Color::WHITE);
        let rect = Rect::new(Point::new(0, 0), Size::new(1, 1));
        let pt = Point::new(0, 0);
        let font = Font::new("a", 1.0);
        let image = Image::try_new(1, 1, vec![0u8, 0, 0, 0]).unwrap();
        let path = quartzite_paint_api::Path::new();
        p.draw_rect(rect, &pen, &brush);
        p.fill_rect(rect, &brush);
        p.draw_line(pt, pt, &pen);
        p.clip_rect(rect);
        p.translate(pt);
        p.save();
        p.restore();
        p.draw_text(pt, "x", &font, &brush);
        p.draw_text_in(
            rect,
            "x",
            &font,
            &brush,
            quartzite_geometry::Alignment::Left,
            quartzite_geometry::Alignment::Left,
        );
        p.draw_image(rect, &image);
        p.draw_path(&path, &pen, &brush);
    }
}
