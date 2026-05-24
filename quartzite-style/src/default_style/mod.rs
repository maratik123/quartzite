//! [`DefaultStyle`] — built-in flat default style for quartzite widgets.

mod text_edit;

use quartzite_paint_api::{Brush, Color, Painter, Pen};
use quartzite_style_types::{ColorGroup, ColorRole, Palette};
use quartzite_widgets::{
    Alignment, AsWidget, Button, Container, Label, LineEdit, ScrollArea, WidgetExt, WidgetView,
};

use crate::clock::StyleClock;
use crate::{Paint, Style};

/// Alpha applied to [`ColorRole::WindowText`] to form the read-only surface overlay.
///
/// Low enough to remain translucent, high enough to be visually distinct on any palette.
pub(super) const READ_ONLY_OVERLAY_ALPHA: f32 = 0.10;

/// Alpha applied to [`ColorRole::Text`] when a widget is in read-only mode.
///
/// Preserves legibility while visually conveying the non-editable state.
pub(super) const READ_ONLY_TEXT_ALPHA: f32 = 0.65;

/// Stroke width in pixels for the focus-ring outline drawn around a focused [`Button`].
pub(super) const FOCUS_RING_WIDTH: f32 = 2.0;

/// Built-in concrete [`Style`] implementation using a flat visual design.
///
/// `DefaultStyle` owns a [`StyleClock`] that drives caret-blink timing.
/// Construct it with [`DefaultStyle::new`] (wall-clock blink) or
/// [`DefaultStyle::with_clock`] (custom / pinned clock for tests).
///
/// Its [`draw_widget`](Style::draw_widget) body routes on the runtime widget
/// type via [`WidgetView`] pattern matching and dispatches to the appropriate
/// [`Paint<W>`](crate::Paint) impl:
///
/// - [`Button`] — flat fill, 1 px outline, centered label; checked/disabled variants.
/// - [`Label`] — background fill + left-aligned (or widget-specified) text.
/// - [`quartzite_widgets::TextEdit`] — base fill, 1 px outline, plain-text content; read-only overlay.
/// - [`ScrollArea`] — chrome only (background fill + 1 px outline); no child traversal.
/// - [`Container`] — Window background fill + 1 px `WindowText` outline; no child traversal.
/// - [`LineEdit`] — Base fill, 1 px outline, single-line text; read-only overlay; placeholder.
///
/// Unknown widget types (the [`WidgetView::Other`] arm) fall through silently — no
/// painter methods are called and no panic is issued. This is intentional: new widget
/// types can be added to the widget tree without breaking `DefaultStyle`.
///
/// `DefaultStyle` is **not** auto-installed. Callers must register it
/// explicitly via [`StyleRegistry::set_style`](crate::StyleRegistry::set_style).
///
/// # Examples
///
/// ```
/// use quartzite_style::{DefaultStyle, StyleRegistry};
///
/// StyleRegistry::set_style(Box::new(DefaultStyle::new()));
/// assert!(StyleRegistry::try_style().is_some());
/// ```
///
/// ```
/// use quartzite_style::{DefaultStyle, Style};
///
/// // DefaultStyle implements Style — it can be boxed as a trait object.
/// let _: Box<dyn Style> = Box::new(DefaultStyle::new());
/// ```
#[derive(Clone, Debug)]
pub struct DefaultStyle {
    clock: StyleClock,
}

impl DefaultStyle {
    /// Constructs a new [`DefaultStyle`] with a wall-clock [`StyleClock`].
    ///
    /// The clock starts at [`std::time::Instant::now`] and alternates the caret
    /// phase every 530 ms.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_style::DefaultStyle;
    ///
    /// let style = DefaultStyle::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            clock: StyleClock::new(),
        }
    }

    /// Constructs a [`DefaultStyle`] with an explicit [`StyleClock`].
    ///
    /// Use this in tests or snapshot harnesses to pin the caret phase to a
    /// deterministic value via [`StyleClock::pinned`].
    ///
    /// # Parameters
    ///
    /// - `clock`: the clock to use for caret-blink phase queries.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_style::{DefaultStyle, Style, StyleClock};
    ///
    /// // Caret always visible — useful for snapshot tests.
    /// let style = DefaultStyle::with_clock(StyleClock::pinned(true));
    /// assert!(style.caret_visible_now());
    /// ```
    #[inline]
    pub const fn with_clock(clock: StyleClock) -> Self {
        Self { clock }
    }

    /// Constructs a 530 ms-interval [`quartzite_runtime::Timer`] that invokes
    /// `on_tick` on each blink tick and starts it immediately with `driver`.
    ///
    /// Only available when the `runtime-blink` cargo feature is enabled
    /// (the default). Consumers who opt out of the runtime layer
    /// (`default-features = false`) can still poll the read-side seam via
    /// [`Style::caret_visible_now`] with a [`StyleClock`] they manage manually.
    ///
    /// The caller must hold the returned [`quartzite_runtime::Timer`] for as
    /// long as blink invalidation is desired; dropping the timer stops the blink.
    ///
    /// Use this to wire the caret-blink invalidation seam: when `on_tick`
    /// fires, schedule a repaint of the focused `TextEdit` widget.
    ///
    /// # Parameters
    ///
    /// - `driver`: timer backend used to schedule the blink interval.
    /// - `on_tick`: callback invoked on every tick (typically schedules a repaint).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use quartzite_runtime::ThreadDriver;
    /// use quartzite_style::DefaultStyle;
    ///
    /// let _timer = DefaultStyle::new()
    ///     .start_blink_timer(Arc::new(ThreadDriver::new()), Arc::new(|| {}));
    /// // Drop `_timer` to stop blinking.
    /// ```
    #[cfg(feature = "runtime-blink")]
    pub fn start_blink_timer(
        &self,
        driver: std::sync::Arc<dyn quartzite_runtime::TimerDriver>,
        on_tick: std::sync::Arc<dyn Fn() + Send + Sync>,
    ) -> quartzite_runtime::Timer {
        use std::time::Duration;

        const BLINK_INTERVAL_MS: u64 = 530;

        let mut timer = quartzite_runtime::Timer::new(Duration::from_millis(BLINK_INTERVAL_MS));
        timer.connect_tick(move |_event| {
            on_tick();
        });
        timer.start(driver);
        timer
    }
}

impl Default for DefaultStyle {
    /// Returns a [`DefaultStyle`] with a fresh wall-clock [`StyleClock`].
    ///
    /// Equivalent to [`DefaultStyle::new`].
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Style for DefaultStyle {
    fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
        match widget.widget_view() {
            WidgetView::Button(w) => self.paint(w, painter, palette),
            WidgetView::Label(w) => self.paint(w, painter, palette),
            WidgetView::TextEdit(w) => self.paint(w, painter, palette),
            WidgetView::ScrollArea(w) => self.paint(w, painter, palette),
            WidgetView::Container(w) => self.paint(w, painter, palette),
            WidgetView::LineEdit(w) => self.paint(w, painter, palette),
            // Unknown widget type — deliberate no-op; does not panic.
            _ => {}
        }
    }

    #[inline]
    fn caret_visible_now(&self) -> bool {
        self.clock.caret_visible_now()
    }

    #[inline]
    fn prefers_reduced_motion(&self) -> bool {
        self.clock.prefers_reduced_motion()
    }
}

impl Paint<Button> for DefaultStyle {
    fn paint(&self, w: &Button, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();
        let enabled = w.is_enabled();
        let hovered = w.is_hovered();
        let pressed = w.is_pressed();
        let focused = w.is_focused();

        // State group: pressed wins over hovered; Normal is the idle default.
        let group = state_group(pressed, hovered);

        // Role selection: pressed or checked maps to Highlight/HighlightedText.
        // `disabled` is an alpha modifier applied after role selection (not a role-selector).
        let (fill_role, text_role) = if pressed || w.checked {
            (ColorRole::Highlight, ColorRole::HighlightedText)
        } else {
            (ColorRole::Button, ColorRole::ButtonText)
        };
        let fill_color = maybe_disabled(palette.color(fill_role, group), enabled);
        let text_color = maybe_disabled(palette.color(text_role, group), enabled);

        // `focused` is an additive outline modifier — always 2 px FocusRing, never alpha-halved.
        let (outline_color, outline_width) = if focused {
            (
                palette.color(ColorRole::FocusRing, ColorGroup::Normal),
                FOCUS_RING_WIDTH,
            )
        } else {
            (text_color, 1.0)
        };

        painter.fill_rect(geom, &Brush::solid(fill_color));
        painter.draw_rect(
            geom,
            &Pen::new(outline_color, outline_width),
            &Brush::solid(Color::TRANSPARENT),
        );
        painter.draw_text_in(
            geom,
            &w.text,
            &font,
            &Brush::solid(text_color),
            Alignment::Center,
        );
    }
}

impl Paint<Label> for DefaultStyle {
    fn paint(&self, w: &Label, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();
        let enabled = w.is_enabled();
        let hovered = w.is_hovered();
        let pressed = w.is_pressed();
        let focused = w.is_focused();

        let group = state_group(pressed, hovered);
        let (fill_role, text_role) = if pressed {
            (ColorRole::Highlight, ColorRole::HighlightedText)
        } else {
            (ColorRole::Window, ColorRole::WindowText)
        };
        let fill_color = maybe_disabled(palette.color(fill_role, group), enabled);
        let text_color = maybe_disabled(palette.color(text_role, group), enabled);

        painter.fill_rect(geom, &Brush::solid(fill_color));
        // `focused` is an additive 2 px FocusRing outline; never alpha-halved.
        if focused {
            painter.draw_rect(
                geom,
                &Pen::new(
                    palette.color(ColorRole::FocusRing, ColorGroup::Normal),
                    FOCUS_RING_WIDTH,
                ),
                &Brush::solid(Color::TRANSPARENT),
            );
        }
        painter.draw_text_in(geom, &w.text, &font, &Brush::solid(text_color), w.alignment);
    }
}

impl Paint<ScrollArea> for DefaultStyle {
    fn paint(&self, w: &ScrollArea, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let enabled = w.is_enabled();
        let hovered = w.is_hovered();
        let pressed = w.is_pressed();
        let focused = w.is_focused();

        let group = state_group(pressed, hovered);
        // Idle/hover keep outline = WindowText; pressed swaps to HighlightedText
        // for legibility under the inverted Highlight fill.
        let (fill_role, outline_role_idle) = if pressed {
            (ColorRole::Highlight, ColorRole::HighlightedText)
        } else {
            (ColorRole::Base, ColorRole::WindowText)
        };
        let fill_color = maybe_disabled(palette.color(fill_role, group), enabled);
        let outline_color_idle = maybe_disabled(palette.color(outline_role_idle, group), enabled);

        painter.fill_rect(geom, &Brush::solid(fill_color));
        // `focused` widens the outline to 2 px FocusRing (full alpha — never alpha-halved).
        let (outline_color, outline_width) = if focused {
            (
                palette.color(ColorRole::FocusRing, ColorGroup::Normal),
                FOCUS_RING_WIDTH,
            )
        } else {
            (outline_color_idle, 1.0)
        };
        painter.draw_rect(
            geom,
            &Pen::new(outline_color, outline_width),
            &Brush::solid(Color::TRANSPARENT),
        );
    }
}

impl Paint<Container> for DefaultStyle {
    fn paint(&self, w: &Container, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        painter.fill_rect(geom, &brush(palette, ColorRole::Window));
        painter.draw_rect(
            geom,
            &Pen::new(
                palette.color(ColorRole::WindowText, ColorGroup::Normal),
                1.0,
            ),
            &Brush::solid(Color::TRANSPARENT),
        );
    }
}

impl Paint<LineEdit> for DefaultStyle {
    fn paint(&self, w: &LineEdit, painter: &mut dyn Painter, palette: &Palette) {
        let geom = w.geometry();
        let font = w.widget_base().font.clone();
        let enabled = w.is_enabled();
        let hovered = w.is_hovered();
        let pressed = w.is_pressed();
        let focused = w.is_focused();

        let group = state_group(pressed, hovered);
        let (fill_role, text_role) = if pressed {
            (ColorRole::Highlight, ColorRole::HighlightedText)
        } else {
            (ColorRole::Base, ColorRole::Text)
        };
        // Idle/hover keep outline = Text (tracks the text colour); pressed
        // swaps to HighlightedText for legibility under the inverted fill.
        let outline_role_idle = if pressed {
            ColorRole::HighlightedText
        } else {
            ColorRole::Text
        };
        let fill_color = maybe_disabled(palette.color(fill_role, group), enabled);
        let text_color = maybe_disabled(palette.color(text_role, group), enabled);
        let outline_color_idle = maybe_disabled(palette.color(outline_role_idle, group), enabled);

        painter.fill_rect(geom, &Brush::solid(fill_color));
        if w.read_only {
            painter.fill_rect(geom, &Brush::solid(read_only_overlay(palette)));
        }
        // `focused` widens the outline to 2 px FocusRing (full alpha — never alpha-halved).
        let (outline_color, outline_width) = if focused {
            (
                palette.color(ColorRole::FocusRing, ColorGroup::Normal),
                FOCUS_RING_WIDTH,
            )
        } else {
            (outline_color_idle, 1.0)
        };
        painter.draw_rect(
            geom,
            &Pen::new(outline_color, outline_width),
            &Brush::solid(Color::TRANSPARENT),
        );
        // 3-arm text-brush selection ladder — preserved from pre-spec impl with the
        // state-resolved + disabled-wrapped `text_color` substituted for the old
        // `Normal`-group lookup. Placeholder wins when text is empty; read-only
        // dim composes orthogonally on top of the state-resolved text colour.
        let (text_arg, text_brush) = if w.text.is_empty() && !w.placeholder.is_empty() {
            (w.placeholder.as_str(), Brush::solid(disabled(text_color)))
        } else if w.read_only {
            (
                w.text.as_str(),
                Brush::solid(text_color.with_alpha(READ_ONLY_TEXT_ALPHA)),
            )
        } else {
            (w.text.as_str(), Brush::solid(text_color))
        };
        painter.draw_text_in(geom, text_arg, &font, &text_brush, Alignment::Left);
    }
}

/// Resolves the [`ColorGroup`] for a widget given its `pressed` / `hovered` flags.
///
/// `pressed` wins over `hovered`; falls back to [`ColorGroup::Normal`] otherwise.
/// Shared selector for every state-aware `Paint<W>` impl in this module.
#[inline]
pub(super) const fn state_group(pressed: bool, hovered: bool) -> ColorGroup {
    if pressed {
        ColorGroup::Pressed
    } else if hovered {
        ColorGroup::Hover
    } else {
        ColorGroup::Normal
    }
}

/// Returns a solid [`Brush`] using the `Normal`-group colour at `role` in `palette`.
#[inline]
pub(super) const fn brush(palette: &Palette, role: ColorRole) -> Brush {
    Brush::solid(palette.color(role, ColorGroup::Normal))
}

/// Halves the alpha of `color` to signal the "disabled" visual state.
///
/// With the default palette (all roles fully opaque), maps `1.0 → 0.5`.
#[inline]
pub(super) fn disabled(color: Color) -> Color {
    color.with_alpha(color.a() * 0.5)
}

/// Returns the read-only overlay colour for `palette`.
///
/// Tints the editable surface with [`ColorRole::WindowText`] at a low alpha.
/// This guarantees a visible effect on every palette — even when `Window`
/// and `Base` share a colour (as on `Palette::default`) — because
/// `WindowText` always carries contrast against `Window` and `Base`.
#[inline]
pub(super) const fn read_only_overlay(palette: &Palette) -> Color {
    palette
        .color(ColorRole::WindowText, ColorGroup::Normal)
        .with_alpha(READ_ONLY_OVERLAY_ALPHA)
}

/// Returns [`disabled`]`(color)` when `enabled` is `false`; otherwise `color` unchanged.
#[allow(
    clippy::doc_link_code,
    reason = "adjacency-to-(args) pattern: renders disabled(color) with disabled intra-doc-linked; flattening to [disabled](path) would drop the surrounding code styling on (color)"
)]
pub(super) fn maybe_disabled(color: Color, enabled: bool) -> Color {
    if enabled { color } else { disabled(color) }
}

#[cfg(test)]
#[path = "../default_style_tests.rs"]
mod tests;
