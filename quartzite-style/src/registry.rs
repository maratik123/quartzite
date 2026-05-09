//! [`StyleRegistry`] — process-global slot for the active [`Style`].
//!
//! Backed by `OnceLock<Mutex<Option<&'static dyn Style>>>`. [`StyleRegistry::set_style`]
//! calls [`Box::leak`] on the supplied `Box<dyn Style>` to obtain the
//! `'static` reference; replacing the style leaks the prior box (acceptable
//! for a process-lifetime registry — typical applications swap styles zero
//! or one times). Lock-poisoning is recovered via
//! `lock().unwrap_or_else(|e| e.into_inner())` per AGENTS.md library-safety
//! idioms.

use std::sync::{Mutex, OnceLock};

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
        let mut guard = slot().lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(leaked);
    }

    /// Returns the active style, or [`None`] if no style is installed.
    ///
    /// The [`Mutex`] poison flag is intentionally tolerated:
    /// `lock().unwrap_or_else(|e| e.into_inner())` recovers the inner
    /// `Option` on a poisoned mutex per AGENTS.md library-safety idioms.
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
        let guard = slot().lock().unwrap_or_else(|e| e.into_inner());
        *guard
    }
}

/// Resets the registry to `None` for the next test.
///
/// Used only by `#[cfg(test)]` consumers — the leaked box from a previous
/// `set_style` is **not** reclaimed (cannot be — leaks are forever).
#[cfg(test)]
pub(crate) fn clear_for_test() {
    let mut guard = slot().lock().unwrap_or_else(|e| e.into_inner());
    *guard = None;
}

/// Forces the registry mutex into the poisoned state for the next lock.
///
/// Spawns a thread that locks and panics; the join handle's `Err` confirms
/// the panic propagated. Subsequent `lock()` calls return `Err(PoisonError)`,
/// which the registry's `unwrap_or_else(|e| e.into_inner())` recovers from.
#[cfg(test)]
pub(crate) fn poison_for_test() {
    let mutex_ref: &'static Mutex<Option<&'static dyn Style>> = slot();
    let handle = std::thread::spawn(move || {
        let _guard = mutex_ref.lock().unwrap_or_else(|e| e.into_inner());
        panic!("intentional panic to poison the registry mutex for tests");
    });
    // The thread panicked — join returns Err. We discard the error; the
    // poisoned state is the artefact we wanted.
    let _ = handle.join();
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use quartzite_paint_api::{Brush, Font, Image, Painter, Path, Pen};
    use quartzite_style_types::Palette;
    use quartzite_widgets::AsWidget;
    use serial_test::serial;

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
    struct NullPainter;

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
    }

    #[test]
    #[serial]
    fn try_style_returns_none_before_set() {
        clear_for_test();
        assert!(StyleRegistry::try_style().is_none());
    }

    #[test]
    #[serial]
    fn try_style_returns_some_after_set() {
        clear_for_test();
        StyleRegistry::set_style(Box::new(StyleA));
        assert!(StyleRegistry::try_style().is_some());
    }

    #[test]
    #[serial]
    fn set_style_replaces_previous() {
        clear_for_test();
        StyleRegistry::set_style(Box::new(StyleA));
        let first = StyleRegistry::try_style().expect("first set");
        StyleRegistry::set_style(Box::new(StyleB));
        let second = StyleRegistry::try_style().expect("second set");
        // Compare the full fat pointer (data + vtable). Distinct concrete
        // `Style` impls have distinct vtables even when both are ZSTs, so
        // `std::ptr::eq` over the wide-pointer form distinguishes them.
        assert!(
            !std::ptr::eq(first as *const dyn Style, second as *const dyn Style),
            "second set_style did not replace the first",
        );
    }

    #[test]
    #[serial]
    fn try_style_recovers_from_poisoned_mutex() {
        clear_for_test();
        StyleRegistry::set_style(Box::new(StyleA));

        // Force the next lock() to observe a PoisonError.
        poison_for_test();

        // The recovery branch must turn `Err(PoisonError)` into the inner
        // guard — try_style() therefore returns Some(_) without panicking.
        let recovered = StyleRegistry::try_style();
        assert!(recovered.is_some(), "poison-recovery branch returned None");
    }

    #[test]
    #[serial]
    fn registered_style_dispatches_draw_widget() {
        clear_for_test();
        let before_a = A_CALLS.load(Ordering::SeqCst);
        let before_b = B_CALLS.load(Ordering::SeqCst);

        StyleRegistry::set_style(Box::new(StyleA));
        let style = StyleRegistry::try_style().expect("style was just installed");
        let widget = quartzite_widgets::WidgetBase::new();
        let mut painter = NullPainter;
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
