#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![deny(missing_docs)]
//! Backend-agnostic paint utilities for quartzite.
//!
//! This crate re-exports the shared paint types from [`quartzite_paint_api`] and
//! provides higher-level abstractions on top.  It has no dependency on any GPU
//! backend (winit, wgpu, vello).

mod path;

pub use path::Path;
pub use quartzite_paint_api::{Brush, BrushKind, Color, PaintError, Painter, Pen};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn re_exported_color_accessible() {
        let _ = Color::BLACK;
    }
}
