#![no_std]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![deny(missing_docs)]
//! Shared paint types and [`Painter`] trait for quartzite.
//!
//! This crate is `no_std`-compatible and has no platform dependencies.
//! It defines the thin shared vocabulary used by both `quartzite-paint`
//! (backend-agnostic utilities) and `quartzite-renderer` (vello+wgpu+winit backend).

extern crate alloc;

mod brush;
mod color;
mod error;
mod painter;
mod pen;

pub use brush::{Brush, BrushKind};
pub use color::Color;
pub use error::PaintError;
pub use painter::Painter;
pub use pen::Pen;
