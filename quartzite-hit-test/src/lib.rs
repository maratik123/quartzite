//! Paint-free point-to-widget hit-testing for quartzite.
//!
//! This crate is the structural inverse of the paint dispatcher in
//! `quartzite-style-dispatch`: where that crate walks a widget subtree
//! front-to-back (parent-before-child, painter's-algorithm z-order) and paints
//! each visible node, this crate walks the same tree in **reverse z-order** to
//! find the visually-topmost widget under a point.
//!
//! It is **paint-free**: it touches no `Painter`, `Style`, `StyleRegistry`, or
//! `Palette` — only the widget tree (resolver + geometry + visibility + clip
//! rect) and a point. The shared read-only [`WidgetResolver`] trait lives here;
//! `quartzite-style-dispatch` re-exports it so its `dispatch_paint` signature is
//! unchanged.

mod resolver;

pub use resolver::WidgetResolver;
