//! Backend-agnostic paint utilities for quartzite.
//!
//! This crate re-exports the shared paint vocabulary from [`quartzite_paint_api`]
//! ([`Brush`], [`BrushKind`], [`Color`], [`Font`], [`FontWeight`], [`Image`],
//! [`ImageError`], [`PaintError`], [`Painter`], [`Path`], [`Pen`], [`Segment`])
//! plus [`HAlignment`] and [`VAlignment`] from [`quartzite_geometry`].
//!
//! It also re-exports the peniko gradient types needed to construct
//! [`BrushKind::Custom`] brushes ([`Gradient`], [`GradientKind`],
//! [`ColorStop`], [`Extend`]) so that callers do not need a direct
//! `peniko` dependency.

pub use peniko::{ColorStop, Extend, Gradient, GradientKind};
pub use quartzite_geometry::{HAlignment, VAlignment};
pub use quartzite_paint_api::{
    Brush, BrushKind, Color, Font, FontWeight, Image, ImageError, PaintError, Painter, Path, Pen,
    Segment,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn re_exported_color_accessible() {
        let _ = Color::BLACK;
    }

    #[test]
    fn re_exports_full_vocabulary() {
        let _ = Path::new();
        let _ = HAlignment::default();
        let _ = VAlignment::default();
        let _ = Font::new("a", 1.0);
    }
}
