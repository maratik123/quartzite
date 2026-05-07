//! Windowed rendering backend for quartzite using vello, wgpu, and winit.

#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![deny(missing_docs)]

pub mod application;
mod error;
pub mod vello_painter;

pub use application::WindowedApplication;
pub use error::RendererError;
pub use vello_painter::VelloPainter;
pub use winit::application::ApplicationHandler;
