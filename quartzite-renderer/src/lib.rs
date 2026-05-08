//! Windowed rendering backend for quartzite using vello, wgpu, and winit.

#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![deny(missing_docs)]

pub mod application;
mod error;
pub mod vello_painter;

pub use application::WindowedApplication;
pub use error::RendererError;
pub use vello_painter::VelloPainter;
pub use winit::application::ApplicationHandler;
