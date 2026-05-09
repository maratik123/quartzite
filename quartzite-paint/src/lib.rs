#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![deny(missing_docs)]
//! Backend-agnostic paint utilities for quartzite.
//!
//! This crate re-exports the shared paint vocabulary from [`quartzite_paint_api`]
//! ([`Brush`], [`BrushKind`], [`Color`], [`Font`], [`FontWeight`], [`Image`],
//! [`ImageError`], [`PaintError`], [`Painter`], [`Path`], [`Pen`], [`Segment`])
//! plus [`Alignment`] from [`quartzite_geometry`]. It has no dependency on any
//! GPU backend (winit, wgpu, vello).

pub use quartzite_geometry::Alignment;
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
        let _ = Alignment::default();
        let _ = Font::new("a", 1.0);
    }
}
