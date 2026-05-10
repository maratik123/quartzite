//! [`Style`] — trait describing how to paint a widget.
//!
//! Concrete [`Style`] implementations route on widget type via downcast or a
//! visitor; per-widget primitive methods (`draw_button`, `draw_label`, …) are
//! intentionally **not** part of the trait surface. The [`Style`] trait carries
//! a `Send + Sync` bound because [`StyleRegistry`](crate::StyleRegistry) hands
//! out `&'static dyn Style` references reachable from any thread.

use quartzite_paint_api::Painter;
use quartzite_style_types::Palette;
use quartzite_widgets::AsWidget;

/// Painting strategy for the widget tree.
///
/// A [`Style`] is the single hook a renderer uses to draw any concrete widget:
/// [`draw_widget`](Self::draw_widget) takes the widget as `&dyn AsWidget`,
/// inspects the runtime type (via downcast or a visitor pattern, depending on
/// the concrete impl), and dispatches to the appropriate drawing routine.
/// All routing is the implementor's responsibility — the trait surface is
/// deliberately a single method.
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
/// }
///
/// // Trait is object-safe — boxing through `dyn Style` compiles.
/// let _boxed: Box<dyn Style> = Box::new(NoopStyle);
/// ```
pub trait Style: Send + Sync {
    /// Paints `widget` using `painter`, resolving any colour references through `palette`.
    ///
    /// The implementor is responsible for routing on the concrete widget type
    /// (typically via a downcast against the `AsWidget` upcast path or a custom
    /// visitor). The trait deliberately exposes a single method so adding new
    /// widget variants does not require trait churn.
    ///
    /// # Parameters
    ///
    /// - `widget`: the widget to paint, accessed via the [`AsWidget`] upcast.
    /// - `painter`: the active backend painter; methods on `&mut dyn Painter`
    ///   accumulate draw operations.
    /// - `palette`: colour-role lookup used by the implementor when resolving
    ///   widget colours.
    fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette);
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};

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
    }

    /// Recording painter — accepts every method as a no-op so we can dispatch
    /// `draw_widget` against a `&mut dyn Painter` without bringing in renderer.
    struct NullPainter;

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
            _alignment: quartzite_geometry::Alignment,
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
    fn draw_widget_dispatches_through_trait_object() {
        let style: Box<dyn Style> = Box::new(OnlyDraw);
        let mut painter = NullPainter;
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
        );
        p.draw_image(rect, &image);
        p.draw_path(&path, &pen, &brush);
    }
}
