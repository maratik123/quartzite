//! Windowed rendering backend for quartzite using vello, wgpu, and winit.
//!
//! The crate offers two complementary entry points:
//!
//! - [`WindowedApplication`] — the production windowed pipeline. It owns a
//!   winit [`EventLoop`](winit::event_loop::EventLoop) and renders into a
//!   [`wgpu::Surface`] backed by an OS window.
//! - [`RenderHarness`] — the test-side, headless counterpart. It bypasses
//!   winit entirely, renders into an offscreen [`wgpu::Texture`], and reads
//!   the pixels back into an [`image::RgbaImage`] for snapshot comparison.
//!   The harness deliberately does not construct an
//!   [`Application`](quartzite_runtime::Application), so many snapshot tests
//!   share a single process.

#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![deny(missing_docs)]

pub mod application;
mod error;
pub mod render_harness;
pub mod vello_painter;

pub use application::WindowedApplication;
pub use error::RendererError;
pub use render_harness::RenderHarness;
pub use vello_painter::VelloPainter;
pub use winit::application::ApplicationHandler;
