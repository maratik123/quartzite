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

pub mod application;
mod application_builder;
mod error;
pub mod event_convert;
pub mod font;
pub mod render_harness;
pub mod vello_painter;
pub mod window_id;
pub mod window_registry;
pub mod window_root;
mod windowed_app_handler;
mod wrapped_handler;

pub use application::WindowedApplication;
pub use application_builder::AppEvent;
pub use application_builder::WindowedApplicationBuilder;
pub use error::RendererError;
pub use render_harness::RenderHarness;
pub use render_harness::RenderHarnessBuilder;
pub use vello_painter::VelloPainter;
pub use window_id::WindowId;
pub use window_registry::WindowRegistry;
pub use window_root::WidgetRoot;
pub use windowed_app_handler::WindowedAppHandler;
