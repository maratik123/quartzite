//! Offscreen [`RenderHarness`] for headless GPU rendering tests.
//!
//! The harness bypasses winit and renders a widget tree to an offscreen
//! [`wgpu::Texture`] which can be read back into an [`image::RgbaImage`]
//! for snapshot comparison. It is the test-side counterpart to
//! [`WindowedApplication`](crate::WindowedApplication): both run the same
//! rendering pipeline, but the harness terminates at a memory texture
//! instead of a window surface.
//!
//! The harness deliberately does **not** construct an
//! [`Application`](quartzite_runtime::Application). Many snapshot tests can
//! therefore share one process; the singleton is reserved for the windowed
//! pipeline.

use std::sync::mpsc;

use image::RgbaImage;
use pollster::block_on;
use quartzite_paint_api::{PaintError, Painter};
use vello::{
    AaConfig, AaSupport, RenderParams, Renderer as VelloRenderer, RendererOptions, Scene, peniko,
};
use wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

use crate::error::RendererError;
use crate::font::FontCache;
use crate::vello_painter::VelloPainter;

/// Offscreen render-target format. RGBA8 unorm matches the format vello
/// produces and is the simplest format to read back into an `RgbaImage`.
pub(crate) const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Background color of the offscreen target.
pub(crate) const BASE_COLOR: peniko::Color = peniko::Color::BLACK;

/// Builder for [`RenderHarness`].
///
/// Construct via [`RenderHarnessBuilder::new`], configure with the setter
/// methods, then call [`build`](RenderHarnessBuilder::build) to allocate the
/// GPU resources.
///
/// # Examples
///
/// ```no_run
/// use quartzite_renderer::RenderHarnessBuilder;
///
/// let harness = RenderHarnessBuilder::new(64, 64)
///     .scale_factor(1.0)
///     .build()
///     .expect("GPU available");
/// ```
#[derive(Debug)]
pub struct RenderHarnessBuilder {
    width: u32,
    height: u32,
    scale_factor: f32,
}

impl RenderHarnessBuilder {
    /// Creates a new builder for a `width × height` offscreen target.
    ///
    /// The default scale factor is `1.0` (logical pixels == physical pixels).
    /// Call [`scale_factor`](RenderHarnessBuilder::scale_factor) before
    /// [`build`](RenderHarnessBuilder::build) to override it.
    ///
    /// # Parameters
    ///
    /// - `width`: render-target width in pixels; must be `> 0` (enforced in
    ///   [`build`](RenderHarnessBuilder::build)).
    /// - `height`: render-target height in pixels; must be `> 0` (enforced in
    ///   [`build`](RenderHarnessBuilder::build)).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::RenderHarnessBuilder;
    ///
    /// let builder = RenderHarnessBuilder::new(128, 128);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            scale_factor: 1.0,
        }
    }

    /// Sets the device-pixel ratio used to convert logical to physical pixels.
    ///
    /// Defaults to `1.0`. Pass `2.0` for `HiDPI` / Retina testing.
    ///
    /// # Parameters
    ///
    /// - `scale`: the DPR multiplier; typically `1.0` or `2.0` for tests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::RenderHarnessBuilder;
    ///
    /// let harness = RenderHarnessBuilder::new(40, 40)
    ///     .scale_factor(2.0)
    ///     .build()
    ///     .expect("GPU available");
    /// ```
    #[inline]
    #[must_use]
    pub const fn scale_factor(mut self, scale: f32) -> Self {
        self.scale_factor = scale;
        self
    }

    /// Allocates GPU resources and returns a [`RenderHarness`].
    ///
    /// Initialisation is synchronous at the test boundary: wgpu's async
    /// adapter and device requests are wrapped in [`pollster::block_on`].
    ///
    /// # Errors
    ///
    /// Returns [`RendererError::Paint`] wrapping [`PaintError::Other`] if:
    /// - `width == 0` or `height == 0` (wgpu rejects zero-extent textures);
    /// - no GPU adapter is available (no driver installed, or the configured
    ///   software renderer cannot be loaded);
    /// - device creation fails (requested limits exceed adapter support);
    /// - vello renderer initialisation fails (shader compilation error).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::RenderHarnessBuilder;
    ///
    /// let harness = RenderHarnessBuilder::new(64, 64).build().expect("GPU available");
    /// ```
    pub fn build(self) -> Result<RenderHarness, RendererError> {
        let Self {
            width,
            height,
            scale_factor,
        } = self;

        if width == 0 || height == 0 {
            return Err(RendererError::Paint(PaintError::Other(
                "zero-extent render target",
            )));
        }
        let instance = wgpu::Instance::default();
        let adapter =
            block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .map_err(|_| RendererError::Paint(PaintError::Other("adapter request failed")))?;

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("quartzite RenderHarness device"),
            ..Default::default()
        }))
        .map_err(|_| RendererError::Paint(PaintError::Other("device request failed")))?;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("quartzite RenderHarness target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let renderer = VelloRenderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .map_err(|_| RendererError::Paint(PaintError::Other("vello renderer init failed")))?;

        Ok(RenderHarness {
            device,
            queue,
            texture,
            texture_view,
            width,
            height,
            scale_factor,
            renderer,
            scene: Scene::new(),
            fonts: FontCache::new(),
        })
    }
}

/// Offscreen GPU render harness for headless rendering tests.
///
/// Constructed via [`RenderHarnessBuilder`]: request a wgpu adapter and device,
/// allocate an offscreen RGBA8 texture, and initialise a `vello` renderer.
/// Subsequent [`render_widget`](RenderHarness::render_widget) calls drive the
/// widget's `paint` method against a [`VelloPainter`], render the resulting
/// scene into the texture, and read the pixels back into an
/// [`image::RgbaImage`].
///
/// Construction is synchronous at the test boundary — wgpu's async adapter
/// and device requests are wrapped in [`pollster::block_on`] inside
/// [`RenderHarnessBuilder::build`].
///
/// # Examples
///
/// ```no_run
/// use quartzite_renderer::RenderHarnessBuilder;
///
/// // 64x64 RGBA8 offscreen target.
/// let _harness = RenderHarnessBuilder::new(64, 64).build().expect("GPU available");
/// ```
pub struct RenderHarness {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) texture: wgpu::Texture,
    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) scale_factor: f32,
    pub(crate) renderer: VelloRenderer,
    pub(crate) scene: Scene,
    pub(crate) fonts: FontCache,
}

impl core::fmt::Debug for RenderHarness {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RenderHarness")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale_factor", &self.scale_factor)
            .finish_non_exhaustive()
    }
}

impl RenderHarness {
    /// Returns the offscreen render target's width in pixels.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::RenderHarnessBuilder;
    ///
    /// let h = RenderHarnessBuilder::new(64, 32).build().unwrap();
    /// assert_eq!(h.width(), 64);
    /// ```
    #[inline]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the offscreen render target's height in pixels.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::RenderHarnessBuilder;
    ///
    /// let h = RenderHarnessBuilder::new(64, 32).build().unwrap();
    /// assert_eq!(h.height(), 32);
    /// ```
    #[inline]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the active device-pixel ratio.
    ///
    /// `1.0` by default (logical == physical pixels). Override via
    /// [`RenderHarnessBuilder::scale_factor`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::RenderHarnessBuilder;
    ///
    /// let h = RenderHarnessBuilder::new(40, 40).scale_factor(2.0).build().unwrap();
    /// assert_eq!(h.scale_factor(), 2.0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Drives `paint` against an internal [`VelloPainter`], renders the
    /// resulting scene to the offscreen texture, and reads the pixels back.
    ///
    /// The harness takes a closure rather than a widget bound, because
    /// widget types live in `quartzite-widgets` and that crate is the
    /// renderer's *dev-dependency* — taking the bound directly would close
    /// a regular dependency cycle. Callers wrap the widget at the call site
    /// (e.g. via `Style::draw_widget`):
    ///
    /// ```ignore
    /// harness.render_widget(|p| style.draw_widget(&label, p, &palette));
    /// ```
    ///
    /// The widget-specific shorthand is provided by the test-side helper.
    ///
    /// # Parameters
    ///
    /// - `paint`: closure invoked exactly once with a `&mut dyn Painter`
    ///   pointing at this harness's [`VelloPainter`]. The vello scene is
    ///   reset before `paint` runs, so callers should not rely on prior
    ///   render state. To render multiple widgets in a single image, the
    ///   closure can drive several `paint` calls sequentially.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `vello::Renderer::render_to_texture` returns an error (treated as a
    ///   non-recoverable v1 failure per [`VelloPainter`]'s same v1 policy);
    /// - `wgpu` buffer mapping for the readback path fails (signals a GPU
    ///   driver fault — the test harness cannot meaningfully recover).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::RenderHarnessBuilder;
    ///
    /// let mut h = RenderHarnessBuilder::new(64, 64).build().unwrap();
    /// let img = h.render_widget(|_painter| { /* drive paint here */ });
    /// assert_eq!(img.width(), 64);
    /// assert_eq!(img.height(), 64);
    /// ```
    pub fn render_widget<F>(&mut self, paint: F) -> RgbaImage
    where
        F: FnOnce(&mut dyn Painter),
    {
        self.scene.reset();
        {
            let mut painter = VelloPainter::new(&mut self.scene)
                .with_scale(self.scale_factor)
                .with_fonts(&mut self.fonts);
            paint(&mut painter);
        } // painter drops here, releasing borrows on scene and fonts

        self.renderer
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.scene,
                &self.texture_view,
                &RenderParams {
                    base_color: BASE_COLOR,
                    width: self.width,
                    height: self.height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .expect("vello render_to_texture should succeed");

        let unpadded_bytes_per_row = 4 * self.width;
        let bytes_per_row = align_up(unpadded_bytes_per_row, COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = u64::from(bytes_per_row) * u64::from(self.height);
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RenderHarness readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RenderHarness encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender
                .send(result)
                .expect("readback channel still open at map_async callback time");
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device poll should succeed");
        receiver
            .recv()
            .expect("readback channel sender alive after poll")
            .expect("buffer mapping should succeed");

        let data = buffer_slice.get_mapped_range();
        let mut pixels = Vec::with_capacity((unpadded_bytes_per_row * self.height) as usize);
        for row in 0..self.height {
            let row_start = (row * bytes_per_row) as usize;
            let row_end = row_start + unpadded_bytes_per_row as usize;
            pixels.extend_from_slice(&data[row_start..row_end]);
        }
        drop(data);
        buffer.unmap();

        RgbaImage::from_raw(self.width, self.height, pixels)
            .expect("buffer length matches width * height * 4")
    }
}

/// Rounds `value` up to the next multiple of `align` (assumes `align > 0`).
#[inline]
const fn align_up(value: u32, align: u32) -> u32 {
    value.div_ceil(align) * align
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn builder_zero_width_returns_err_from_build() {
        let err = RenderHarnessBuilder::new(0, 64).build().unwrap_err();
        assert_matches!(
            err,
            RendererError::Paint(PaintError::Other("zero-extent render target")),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn builder_zero_height_returns_err_from_build() {
        let err = RenderHarnessBuilder::new(64, 0).build().unwrap_err();
        assert_matches!(
            err,
            RendererError::Paint(PaintError::Other("zero-extent render target")),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn builder_zero_both_returns_err_from_build() {
        let err = RenderHarnessBuilder::new(0, 0).build().unwrap_err();
        assert_matches!(
            err,
            RendererError::Paint(PaintError::Other("zero-extent render target"))
        );
    }

    #[test]
    #[allow(
        clippy::float_cmp,
        reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
    )]
    fn builder_default_scale_factor_is_1_0() {
        // Validates builder default without GPU — zero-extent error fires first.
        let harness = RenderHarnessBuilder::new(0, 64).build().unwrap_err();
        // Confirm it's the zero-extent path, not a scale-factor problem.
        assert_matches!(
            harness,
            RendererError::Paint(PaintError::Other("zero-extent render target"))
        );
        // Check the in-memory scale_factor field before GPU init via the builder.
        let builder = RenderHarnessBuilder::new(64, 64);
        assert_eq!(builder.scale_factor, 1.0);
    }

    #[test]
    #[allow(
        clippy::float_cmp,
        reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
    )]
    fn builder_explicit_scale_factor_round_trips() {
        let builder = RenderHarnessBuilder::new(64, 64).scale_factor(2.0);
        assert_eq!(builder.scale_factor, 2.0);
    }

    // R10 — accessor + Debug tests (GPU-gated like the snapshot test below)
    #[test]
    #[allow(
        clippy::float_cmp,
        reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction"
    )]
    fn render_harness_accessors_and_debug() {
        if std::env::var_os("SKIP_RENDER_SNAPSHOT").is_some() {
            eprintln!(
                "render_harness_accessors_and_debug: \
                 SKIP_RENDER_SNAPSHOT set; skipping GPU work"
            );
            return;
        }
        let harness = match RenderHarnessBuilder::new(64, 32).scale_factor(1.5).build() {
            Ok(h) => h,
            Err(e) => {
                eprintln!(
                    "render_harness_accessors_and_debug: \
                     no GPU adapter available ({e}); skipping"
                );
                return;
            }
        };
        assert_eq!(harness.width(), 64);
        assert_eq!(harness.height(), 32);
        assert_eq!(harness.scale_factor(), 1.5);
        let dbg = format!("{harness:?}");
        assert!(dbg.contains("RenderHarness"), "debug: {dbg}");
        assert!(dbg.contains("width: 64"), "debug: {dbg}");
        assert!(dbg.contains("height: 32"), "debug: {dbg}");
        assert!(dbg.contains("scale_factor: 1.5"), "debug: {dbg}");
    }

    /// Smoke test: a 64x64 harness rendered against a no-op paint closure
    /// produces an all-`BASE_COLOR` (opaque black) image. This ensures the
    /// readback path (texture → buffer copy → row-aligned decode) is wired
    /// correctly. Skipped when `SKIP_RENDER_SNAPSHOT=1` so the workspace
    /// `cargo test` job can run without GPU.
    #[test]
    fn render_widget_no_op_produces_clear_color_image() {
        if std::env::var_os("SKIP_RENDER_SNAPSHOT").is_some() {
            eprintln!(
                "render_widget_no_op_produces_clear_color_image: \
                 SKIP_RENDER_SNAPSHOT set; skipping GPU work"
            );
            return;
        }
        let mut harness = match RenderHarnessBuilder::new(64, 64).build() {
            Ok(h) => h,
            Err(e) => {
                eprintln!(
                    "render_widget_no_op_produces_clear_color_image: \
                     no GPU adapter available ({e}); skipping"
                );
                return;
            }
        };
        let image = harness.render_widget(|_painter| {});
        assert_eq!(image.width(), 64);
        assert_eq!(image.height(), 64);
        // BASE_COLOR is `peniko::Color::BLACK` = sRGB (0,0,0,1) → RGBA8 (0,0,0,255).
        let expected: [u8; 4] = [0, 0, 0, 255];
        for (i, pixel) in image.pixels().enumerate() {
            assert_eq!(
                pixel.0, expected,
                "pixel {i} is {:?}, expected {expected:?}",
                pixel.0
            );
        }
    }
}
