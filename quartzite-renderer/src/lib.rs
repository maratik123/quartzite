//! Windowed rendering backend for quartzite using vello, wgpu, and winit.

#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![deny(missing_docs)]

/// Application module (stub; implemented in Task 7).
pub mod application;
mod error;
/// VelloPainter module (stub; implemented in Task 7).
pub mod vello_painter;

pub use error::RendererError;
pub use winit::application::ApplicationHandler;
