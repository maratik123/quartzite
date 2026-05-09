//! [`Style`] — trait describing how to paint a widget.
//!
//! Concrete `Style` implementations route on widget type via downcast or a
//! visitor; per-widget primitive methods (`draw_button`, `draw_label`, …) are
//! intentionally **not** part of the trait surface. The [`Style`] trait carries
//! a `Send + Sync` bound because [`StyleRegistry`](crate::StyleRegistry) hands
//! out `&'static dyn Style` references reachable from any thread.

use quartzite_paint_api::Painter;
use quartzite_style_types::Palette;
use quartzite_widgets::AsWidget;

/// Painting strategy for the widget tree.
///
/// A `Style` is the single hook a renderer uses to draw any concrete widget:
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

    /// Zero-sized fixture proving the trait can be satisfied by a single method impl.
    struct OnlyDraw;

    impl Style for OnlyDraw {
        fn draw_widget(
            &self,
            _widget: &dyn AsWidget,
            _painter: &mut dyn Painter,
            _palette: &Palette,
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
}
