//! [`StyleRegistry`] — process-global slot for the active [`Style`].
//!
//! Backed by `OnceLock<Mutex<Option<&'static dyn Style>>>`. [`StyleRegistry::set_style`]
//! calls [`Box::leak`] on the supplied `Box<dyn Style>` to obtain the
//! `'static` reference; replacing the style leaks the prior box (acceptable
//! for a process-lifetime registry — typical applications swap styles zero
//! or one times).

use parking_lot::Mutex;
use std::sync::OnceLock;

use crate::Style;

/// Process-wide slot holding the active style (if any).
///
/// Initialised lazily on first access; the [`Mutex`] is constructed empty
/// (`None`) and is filled only by [`StyleRegistry::set_style`].
static REGISTRY: OnceLock<Mutex<Option<&'static dyn Style>>> = OnceLock::new();

/// Lazily initialises and returns a reference to the registry mutex.
#[inline]
fn slot() -> &'static Mutex<Option<&'static dyn Style>> {
    REGISTRY.get_or_init(|| Mutex::new(None))
}

/// Namespace for the global style registry.
///
/// `StyleRegistry` is a unit struct used purely as a method namespace —
/// callers always go through the static [`set_style`](Self::set_style) and
/// [`try_style`](Self::try_style) entry points.
///
/// # Examples
///
/// ```no_run
/// use quartzite_paint_api::Painter;
/// use quartzite_style::{Palette, Style, StyleRegistry};
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
/// StyleRegistry::set_style(Box::new(NoopStyle));
/// let style: &'static dyn Style = StyleRegistry::try_style().expect("just set");
/// # let _ = style;
/// ```
pub struct StyleRegistry;

impl StyleRegistry {
    /// Installs `style` as the active style.
    ///
    /// The supplied `Box<dyn Style>` is leaked via [`Box::leak`] to obtain a
    /// `'static` reference (the registry hands out `&'static dyn Style`).
    /// If a style was already installed, its box stays leaked — this is
    /// acceptable for a process-lifetime registry; typical applications swap
    /// styles zero or one times. Repeated calls retain each previous box's
    /// allocation for the rest of the process lifetime.
    ///
    /// # Parameters
    ///
    /// - `style`: the new style. Ownership is transferred to the registry
    ///   (leaked to `'static`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_paint_api::Painter;
    /// use quartzite_style::{Palette, Style, StyleRegistry};
    /// use quartzite_widgets::AsWidget;
    ///
    /// struct NoopStyle;
    /// impl Style for NoopStyle {
    ///     fn draw_widget(
    ///         &self,
    ///         _w: &dyn AsWidget,
    ///         _p: &mut dyn Painter,
    ///         _pal: &Palette,
    ///     ) {}
    /// }
    ///
    /// StyleRegistry::set_style(Box::new(NoopStyle));
    /// assert!(StyleRegistry::try_style().is_some());
    /// ```
    pub fn set_style(style: Box<dyn Style>) {
        let leaked: &'static dyn Style = Box::leak(style);
        let mut guard = slot().lock();
        *guard = Some(leaked);
    }

    /// Returns the active style, or [`None`] if no style is installed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_style::StyleRegistry;
    ///
    /// // Before any set_style: returns None.
    /// let _maybe_style = StyleRegistry::try_style();
    /// ```
    #[must_use]
    pub fn try_style() -> Option<&'static dyn Style> {
        let guard = slot().lock();
        *guard
    }
}

impl StyleRegistry {
    /// Resets the active style to `None`.
    ///
    /// The leaked box from a previous [`set_style`][Self::set_style] call is
    /// **not** reclaimed. Intended for use in integration tests and test-helper
    /// crates that are outside `quartzite-style`; available only when the
    /// `test-support` feature is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_style::StyleRegistry;
    ///
    /// StyleRegistry::clear_for_test();
    /// assert!(StyleRegistry::try_style().is_none());
    /// ```
    #[cfg(any(test, feature = "test-support"))]
    #[inline]
    pub fn clear_for_test() {
        let mut guard = slot().lock();
        *guard = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use quartzite_paint_api::{
        Brush, Font, Image, Painter, Path, Pen, TextCaretCursor, TextVisualLine,
        TextVisualLineCursor,
    };
    use quartzite_style_types::Palette;
    use quartzite_widgets::AsWidget;

    /// Records how many times each fixture's `draw_widget` body executed.
    static A_CALLS: AtomicUsize = AtomicUsize::new(0);
    static B_CALLS: AtomicUsize = AtomicUsize::new(0);

    /// Marker fixture A.
    struct StyleA;

    impl Style for StyleA {
        fn draw_widget(
            &self,
            _widget: &dyn AsWidget,
            _painter: &mut dyn Painter,
            _palette: &Palette,
        ) {
            A_CALLS.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Marker fixture B (separate type so address-equality tests compile).
    struct StyleB;

    impl Style for StyleB {
        fn draw_widget(
            &self,
            _widget: &dyn AsWidget,
            _painter: &mut dyn Painter,
            _palette: &Palette,
        ) {
            B_CALLS.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// No-op `Painter` used solely to satisfy the trait-method signature.
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
        fn draw_rect(&mut self, _rect: quartzite_geometry::Rect, _pen: &Pen, _brush: &Brush) {}
        fn fill_rect(&mut self, _rect: quartzite_geometry::Rect, _brush: &Brush) {}
        fn draw_line(
            &mut self,
            _from: quartzite_geometry::Point,
            _to: quartzite_geometry::Point,
            _pen: &Pen,
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
            _font: &Font,
            _brush: &Brush,
        ) {
        }
        fn draw_text_in(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _text: &str,
            _font: &Font,
            _brush: &Brush,
            _alignment: quartzite_geometry::Alignment,
        ) {
        }
        fn draw_image(&mut self, _rect: quartzite_geometry::Rect, _image: &Image) {}
        fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {}
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

    #[test]
    fn try_style_returns_none_before_set() {
        let _lock = quartzite_test_helpers::test_lock();
        StyleRegistry::clear_for_test();
        assert!(StyleRegistry::try_style().is_none());
    }

    #[test]
    fn try_style_returns_some_after_set() {
        let _lock = quartzite_test_helpers::test_lock();
        StyleRegistry::clear_for_test();
        StyleRegistry::set_style(Box::new(StyleA));
        assert!(StyleRegistry::try_style().is_some());
    }

    #[test]
    fn set_style_replaces_previous() {
        let _lock = quartzite_test_helpers::test_lock();
        StyleRegistry::clear_for_test();
        StyleRegistry::set_style(Box::new(StyleA));
        let first = StyleRegistry::try_style().expect("first set");
        StyleRegistry::set_style(Box::new(StyleB));
        let second = StyleRegistry::try_style().expect("second set");
        // Compare the full fat pointer (data + vtable). Distinct concrete
        // `Style` impls have distinct vtables even when both are ZSTs, so
        // `std::ptr::eq` over the wide-pointer form distinguishes them.
        assert!(
            !std::ptr::eq(
                std::ptr::from_ref::<dyn Style>(first),
                std::ptr::from_ref::<dyn Style>(second)
            ),
            "second set_style did not replace the first",
        );
    }

    #[test]
    #[allow(
        clippy::items_after_statements,
        reason = "nested helper placed after local setup is more readable here"
    )]
    fn registered_style_dispatches_draw_widget() {
        let _lock = quartzite_test_helpers::test_lock();
        StyleRegistry::clear_for_test();
        let before_a = A_CALLS.load(Ordering::SeqCst);
        let before_b = B_CALLS.load(Ordering::SeqCst);

        StyleRegistry::set_style(Box::new(StyleA));
        let style = StyleRegistry::try_style().expect("style was just installed");
        let widget = quartzite_widgets::WidgetBase::new();
        let mut painter = NullPainter::new();
        let palette = Palette::default();
        style.draw_widget(&widget, &mut painter, &palette);

        StyleRegistry::set_style(Box::new(StyleB));
        let style = StyleRegistry::try_style().expect("style was just replaced");
        style.draw_widget(&widget, &mut painter, &palette);

        assert_eq!(A_CALLS.load(Ordering::SeqCst), before_a + 1);
        assert_eq!(B_CALLS.load(Ordering::SeqCst), before_b + 1);

        // Exercise the NullPainter trait-impl bodies so coverage doesn't
        // regress on the no-op stubs.
        use quartzite_geometry::{Point, Rect, Size};
        let p: &mut dyn Painter = &mut painter;
        let pen = Pen::new(quartzite_paint_api::Color::BLACK, 1.0);
        let brush = Brush::solid(quartzite_paint_api::Color::WHITE);
        let rect = Rect::new(Point::new(0, 0), Size::new(1, 1));
        let pt = Point::new(0, 0);
        let font = Font::new("a", 1.0);
        let image = Image::try_new(1, 1, vec![0u8, 0, 0, 0]).unwrap();
        let path = Path::new();
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
