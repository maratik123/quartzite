#![no_std]
//! Shared paint types and [`Painter`] trait for quartzite.
//!
//! This crate is `no_std`-compatible and has no platform dependencies.
//! It defines the thin shared vocabulary used by both `quartzite-paint`
//! (backend-agnostic utilities) and `quartzite-renderer` (vello+wgpu+winit backend).

extern crate alloc;

mod brush;
mod color;
mod error;
mod font;
mod image;
mod painter;
mod path;
mod pen;

pub use brush::{Brush, BrushKind};
pub use color::Color;
pub use error::PaintError;
pub use font::{Font, FontWeight};
pub use image::{Image, ImageError};
pub use painter::Painter;
pub use path::{Path, Segment};
pub use pen::Pen;
