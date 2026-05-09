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
    use quartzite_paint_api::Painter;
    use quartzite_style_types::Palette;
    use quartzite_widgets::AsWidget;
    use serial_test::serial;

    /// Marker fixture A.
    struct StyleA;

    impl Style for StyleA {
        fn draw_widget(
            &self,
            _widget: &dyn AsWidget,
            _painter: &mut dyn Painter,
            _palette: &Palette,
        ) {
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
        }
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
}
