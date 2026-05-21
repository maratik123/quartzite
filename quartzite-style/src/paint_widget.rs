//! [`Paint<W>`] — typed per-widget paint trait for style implementors.

use quartzite_paint_api::Painter;
use quartzite_style_types::Palette;
use quartzite_widgets::AsWidget;

/// Typed paint hook for a single widget kind.
///
/// Where [`Style::draw_widget`](crate::Style::draw_widget) accepts `&dyn AsWidget`
/// and routes on the runtime type, `Paint<W>` lets a style author write typed
/// paint code for a specific widget kind `W`.
///
/// # How dispatch works
///
/// Inside [`Style::draw_widget`](crate::Style::draw_widget) the concrete
/// [`WidgetView`](quartzite_widgets::WidgetView) match arm unwraps a typed `&W`
/// reference and calls `self.paint(w, painter, palette)`.
///
/// ```text
/// // In a Style::draw_widget body:
/// match widget.widget_view() {
///     WidgetView::Button(w) => self.paint(w, painter, palette),
///     WidgetView::Other(_)  => {}  // silent no-op — documented contract
///     _ => {}
/// }
/// ```
///
/// # Implementing `Paint<W>` for a built-in widget
///
/// ```
/// use quartzite_paint_api::Painter;
/// use quartzite_style::{Paint, Palette};
/// use quartzite_widgets::Button;
///
/// struct MyStyle;
///
/// impl Paint<Button> for MyStyle {
///     fn paint(&self, _widget: &Button, _painter: &mut dyn Painter, _palette: &Palette) {
///         // draw Button here
///     }
/// }
///
/// // Paint<W> is object-safe — &dyn Paint<Button> compiles.
/// let style = MyStyle;
/// let _: &dyn Paint<Button> = &style;
/// ```
///
/// # Implementing `Paint<W>` for a third-party widget
///
/// Third-party styles can handle custom widget types by implementing
/// `Paint<MyWidget> for MyStyle` and routing `WidgetView::Other` in their
/// [`Style::draw_widget`](crate::Style::draw_widget) override:
///
/// ```ignore
/// use quartzite_paint_api::Painter;
/// use quartzite_style::{Paint, Palette, Style};
/// use quartzite_widgets::{AsWidget, WidgetBase, WidgetView};
///
/// // A user-defined widget without a registered WidgetView variant defaults to WidgetView::Other(self).
/// struct MyWidget { widget_base: WidgetBase }
/// # impl MyWidget { fn new() -> Self { Self { widget_base: WidgetBase::new() } } }
/// // (AsWidget impl elided for example brevity.)
///
/// struct MyStyle;
///
/// impl Paint<MyWidget> for MyStyle {
///     fn paint(&self, _widget: &MyWidget, _painter: &mut dyn Painter, _palette: &Palette) {
///         // draw MyWidget here
///     }
/// }
///
/// impl Style for MyStyle {
///     fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
///         if let WidgetView::Other(other) = widget.widget_view() {
///             if let Some(w) = other.as_any().downcast_ref::<MyWidget>() {
///                 self.paint(w, painter, palette);
///             }
///         }
///     }
/// }
/// ```
///
/// # `WidgetView::Other` — documented no-op
///
/// A [`Style`](crate::Style) that has no `Paint<W>` impl for a given `W` will reach
/// the `WidgetView::Other` arm in its `draw_widget` body and do nothing. This is
/// intentional — an unrecognised widget type silently produces no paint output rather
/// than panicking.
pub trait Paint<W: AsWidget + ?Sized> {
    /// Paints `widget` onto `painter` using the given `palette`.
    ///
    /// # Parameters
    ///
    /// - `widget` — the widget to paint; typed as `&W` for zero-cost dispatch.
    /// - `painter` — the drawing target; geometry and text calls go through this.
    /// - `palette` — colour roles used by the style implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Painter;
    /// use quartzite_style::{Paint, Palette};
    /// use quartzite_widgets::Button;
    ///
    /// struct MyStyle;
    ///
    /// impl Paint<Button> for MyStyle {
    ///     fn paint(&self, _widget: &Button, _painter: &mut dyn Painter, _palette: &Palette) {}
    /// }
    /// ```
    fn paint(&self, widget: &W, painter: &mut dyn Painter, palette: &Palette);
}

#[cfg(test)]
mod tests {
    use quartzite_geometry::{Alignment, Point, Rect};
    use quartzite_paint_api::{Brush, Font, Image, Painter, Path, Pen};
    use quartzite_style_types::Palette;
    use quartzite_widgets::Button;

    use super::Paint;

    struct FakeStyle;

    impl Paint<Button> for FakeStyle {
        fn paint(&self, _widget: &Button, _painter: &mut dyn Painter, _palette: &Palette) {}
    }

    struct NullPainter;

    impl Painter for NullPainter {
        fn draw_rect(&mut self, _rect: Rect, _pen: &Pen, _brush: &Brush) {}
        fn fill_rect(&mut self, _rect: Rect, _brush: &Brush) {}
        fn draw_line(&mut self, _from: Point, _to: Point, _pen: &Pen) {}
        fn clip_rect(&mut self, _rect: Rect) {}
        fn translate(&mut self, _delta: Point) {}
        fn save(&mut self) {}
        fn restore(&mut self) {}
        fn draw_text(&mut self, _pos: Point, _text: &str, _font: &Font, _brush: &Brush) {}
        fn draw_text_in(
            &mut self,
            _rect: Rect,
            _text: &str,
            _font: &Font,
            _brush: &Brush,
            _alignment: Alignment,
        ) {
        }
        fn draw_image(&mut self, _rect: Rect, _image: &Image) {}
        fn draw_path(&mut self, _path: &Path, _pen: &Pen, _brush: &Brush) {}
    }

    fn assert_send_sync<T: Send + Sync>() {}

    // Paint<Button> is object-safe — &dyn Paint<Button> compiles.
    #[test]
    fn paint_button_is_object_safe() {
        let style = FakeStyle;
        let _: &dyn Paint<Button> = &style;
    }

    // Box<dyn Paint<Button>> is constructable; FakeStyle is Send + Sync.
    #[test]
    fn boxed_paint_button_constructs() {
        assert_send_sync::<FakeStyle>();
        let _boxed: Box<dyn Paint<Button>> = Box::new(FakeStyle);
    }

    // paint() is callable through the trait — exercises the method body.
    #[test]
    fn paint_method_is_callable() {
        let style = FakeStyle;
        let widget = Button::new("ok".into());
        let mut painter = NullPainter;
        let palette = Palette::default();
        style.paint(&widget, &mut painter, &palette);
    }
}
