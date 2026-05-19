//! Downstream styling crate for quartzite.
//!
//! Adds the [`Style`] trait and the global [`StyleRegistry`] on top of the
//! [`quartzite-style-types`](quartzite_style_types) leaf crate. The split
//! exists to break the Cargo cycle that would otherwise arise from
//! [`Style::draw_widget`] needing `&dyn AsWidget` (so `quartzite-style`
//! depends on `quartzite-widgets`) **and** widgets re-exporting [`Palette`]
//! (so widgets would depend on `quartzite-style`). [`Palette`] and
//! [`ColorRole`] live in the leaf crate so widgets re-exports them from
//! there; this crate adds the trait and registry on top.
//!
//! # Examples
//!
//! ```no_run
//! use quartzite_paint_api::Painter;
//! use quartzite_style::{Palette, Style, StyleRegistry};
//! use quartzite_widgets::AsWidget;
//!
//! struct NoopStyle;
//!
//! impl Style for NoopStyle {
//!     fn draw_widget(
//!         &self,
//!         _widget: &dyn AsWidget,
//!         _painter: &mut dyn Painter,
//!         _palette: &Palette,
//!     ) {
//!     }
//! }
//!
//! StyleRegistry::set_style(Box::new(NoopStyle));
//! assert!(StyleRegistry::try_style().is_some());
//! ```

mod default_style;
mod paint_widget;
mod registry;
mod style;

pub use default_style::DefaultStyle;
pub use paint_widget::Paint;
pub use quartzite_style_types::{ColorRole, DARK_PALETTE, Palette};
pub use registry::StyleRegistry;
pub use style::Style;
