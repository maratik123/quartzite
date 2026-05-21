//! [`VelloPainter`] — vello + wgpu backed [`Painter`] implementation.

use parley::{
    Alignment as ParleyAlignment, AlignmentOptions, FontFamilyName, FontStyle as ParleyFontStyle,
    FontWeight as ParleyFontWeight, Layout, PositionedLayoutItem, RangedBuilder, StyleProperty,
};
use quartzite_geometry::{Alignment, Point, Rect};
use quartzite_paint_api::{Brush, BrushKind, Font, Image, Painter, Path, Pen, Segment};
use vello::peniko::kurbo;
use vello::peniko::{self, Blob, Fill, ImageAlphaType, ImageData, ImageFormat};
use vello::{Glyph, Scene};

use crate::font::FontCache;

/// A [`Painter`] implementation backed by vello, borrowing a
/// [`vello::Scene`] for a single frame.
///
/// Construct via [`VelloPainter::new`], chain [`with_scale`](VelloPainter::with_scale)
/// and [`with_fonts`](VelloPainter::with_fonts) as needed, then pass to a
/// widget's `paint` method.
///
/// In v1, rendering errors are non-recoverable. Methods skip drawing silently
/// for unsupported brush kinds (e.g. gradient brushes — AC10); text methods
/// panic on font-resolution failure (documented in `# Panics` per those
/// methods).
///
/// # Examples
///
/// ```no_run
/// use vello::Scene;
/// use quartzite_renderer::{VelloPainter, font::FontCache};
///
/// let mut scene = Scene::new();
/// let mut cache = FontCache::new();
/// let mut painter = VelloPainter::new(&mut scene)
///     .with_scale(1.0)
///     .with_fonts(&mut cache);
/// // Use painter as a &mut dyn Painter argument.
/// ```
pub struct VelloPainter<'a> {
    scene: &'a mut Scene,
    fonts: Option<&'a mut FontCache>,
    scale: f32,
    /// Stack of accumulated transforms; initially `[Affine::IDENTITY]`.
    xforms: Vec<kurbo::Affine>,
    /// Per-save-frame count of `push_layer` calls; same length as `xforms`.
    clips: Vec<u32>,
}

/// Private renderer-internal classification of [`BrushKind`] into the four variants the
/// renderer knows how to handle today, plus an explicit `Unknown` bucket that funnels
/// every future upstream variant. Mirrors `quartzite_paint_api::BrushKind` 1:1 by
/// reference; this isolates the single `_ => Unknown` wildcard to one location so
/// neither `brush_to_peniko` nor `brush_color` needs a catch-all arm.
///
/// `quartzite_paint_api::BrushKind` is `#[non_exhaustive]`, meaning downstream crates
/// cannot write an exhaustive match directly. Any future variant added in that crate
/// routes here to `Unknown`, preserving the existing "no brush" fallback semantics
/// without requiring a `_` wildcard in the two consumer call sites.
enum LocalBrushKind<'a> {
    Solid(&'a quartzite_paint_api::Color),
    LinearGradient {
        start: &'a quartzite_geometry::Point,
        end: &'a quartzite_geometry::Point,
        start_color: &'a quartzite_paint_api::Color,
        end_color: &'a quartzite_paint_api::Color,
    },
    RadialGradient {
        centre: &'a quartzite_geometry::Point,
        radius: f32,
        start_color: &'a quartzite_paint_api::Color,
        end_color: &'a quartzite_paint_api::Color,
    },
    Custom(&'a peniko::Gradient),
    /// Forward-compat sink for any future `BrushKind` variant added in
    /// `quartzite-paint-api` (the upstream type is `#[non_exhaustive]`).
    /// The renderer falls back to "no brush" semantics, matching the previous
    /// `_ => None` behaviour. See `quartzite_paint_api::BrushKind`.
    ///
    /// FIXME(after `BrushKind` extension): map the new variant in
    /// `LocalBrushKind::from_brush_kind` when `quartzite_paint_api` adds one.
    Unknown,
}

impl<'a> LocalBrushKind<'a> {
    /// Classifies a `&'a BrushKind` into the renderer-internal `LocalBrushKind`.
    ///
    /// The single `_ => Self::Unknown` arm is the only place in the renderer
    /// that carries a wildcard over `BrushKind`, satisfying AC4.
    #[inline]
    const fn from_brush_kind(k: &'a BrushKind) -> Self {
        match k {
            BrushKind::Solid(c) => Self::Solid(c),
            BrushKind::LinearGradient {
                start,
                end,
                start_color,
                end_color,
            } => Self::LinearGradient {
                start,
                end,
                start_color,
                end_color,
            },
            BrushKind::RadialGradient {
                centre,
                radius,
                start_color,
                end_color,
            } => Self::RadialGradient {
                centre,
                radius: *radius,
                start_color,
                end_color,
            },
            BrushKind::Custom(g) => Self::Custom(g),
            // Upstream `BrushKind` is `#[non_exhaustive]` — keep the sink here so the
            // exhaustive matches in `brush_to_peniko` / `brush_color` never need `_`.
            _ => Self::Unknown,
        }
    }
}

impl<'a> VelloPainter<'a> {
    /// Creates a painter that borrows `scene` for one frame.
    ///
    /// The default scale factor is `1.0` (logical pixels == physical pixels).
    /// Chain [`with_scale`](Self::with_scale) and [`with_fonts`](Self::with_fonts)
    /// before use.
    ///
    /// # Parameters
    ///
    /// - `scene`: the vello scene that draw calls will be appended to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vello::Scene;
    /// use quartzite_renderer::VelloPainter;
    ///
    /// let mut scene = Scene::new();
    /// let painter = VelloPainter::new(&mut scene);
    /// ```
    #[must_use]
    pub fn new(scene: &'a mut Scene) -> Self {
        Self {
            scene,
            fonts: None,
            scale: 1.0,
            xforms: vec![kurbo::Affine::IDENTITY],
            clips: vec![0],
        }
    }

    /// Sets the device-pixel ratio multiplier (logical → physical pixels).
    ///
    /// Defaults to `1.0`. Pass `2.0` for `HiDPI` / Retina rendering.
    ///
    /// # Parameters
    ///
    /// - `scale`: device-pixel ratio; `1.0` for standard displays, `2.0` for `HiDPI`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vello::Scene;
    /// use quartzite_renderer::VelloPainter;
    ///
    /// let mut scene = Scene::new();
    /// let painter = VelloPainter::new(&mut scene).with_scale(2.0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Attaches a [`FontCache`] to enable text rendering.
    ///
    /// # Parameters
    ///
    /// - `fonts`: mutable reference to the frame's font context; borrowed for
    ///   the lifetime of this painter.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vello::Scene;
    /// use quartzite_renderer::{VelloPainter, font::FontCache};
    ///
    /// let mut scene = Scene::new();
    /// let mut cache = FontCache::new();
    /// let painter = VelloPainter::new(&mut scene).with_fonts(&mut cache);
    /// ```
    #[inline]
    #[must_use]
    pub const fn with_fonts(mut self, fonts: &'a mut FontCache) -> Self {
        self.fonts = Some(fonts);
        self
    }

    #[inline]
    fn current_xform(&self) -> kurbo::Affine {
        *self.xforms.last().expect("xforms stack is never empty")
    }

    #[inline]
    fn scale_pt(&self, p: Point) -> kurbo::Point {
        let s = f64::from(self.scale);
        kurbo::Point::new(f64::from(p.x()) * s, f64::from(p.y()) * s)
    }

    #[inline]
    fn scale_rect(&self, r: Rect) -> kurbo::Rect {
        let s = f64::from(self.scale);
        kurbo::Rect::new(
            f64::from(r.left()) * s,
            f64::from(r.top()) * s,
            f64::from(r.right()) * s,
            f64::from(r.bottom()) * s,
        )
    }

    #[inline]
    const fn color_to_peniko(c: quartzite_paint_api::Color) -> peniko::Color {
        peniko::Color::new([c.r(), c.g(), c.b(), c.a()])
    }

    #[inline]
    fn color_to_dynamic(c: quartzite_paint_api::Color) -> peniko::color::DynamicColor {
        peniko::color::DynamicColor::from_alpha_color(Self::color_to_peniko(c))
    }

    fn brush_to_peniko(&self, brush: &Brush) -> Option<peniko::Brush> {
        match LocalBrushKind::from_brush_kind(brush.kind()) {
            LocalBrushKind::Solid(c) => Some(peniko::Brush::Solid(Self::color_to_peniko(*c))),
            LocalBrushKind::LinearGradient {
                start,
                end,
                start_color,
                end_color,
            } => {
                let stop0 = peniko::ColorStop {
                    offset: 0.0,
                    color: Self::color_to_dynamic(*start_color),
                };
                let stop1 = peniko::ColorStop {
                    offset: 1.0,
                    color: Self::color_to_dynamic(*end_color),
                };
                Some(peniko::Brush::Gradient(
                    peniko::Gradient::new_linear(self.scale_pt(*start), self.scale_pt(*end))
                        .with_stops([stop0, stop1]),
                ))
            }
            LocalBrushKind::RadialGradient {
                centre,
                radius,
                start_color,
                end_color,
            } => {
                let stop0 = peniko::ColorStop {
                    offset: 0.0,
                    color: Self::color_to_dynamic(*start_color),
                };
                let stop1 = peniko::ColorStop {
                    offset: 1.0,
                    color: Self::color_to_dynamic(*end_color),
                };
                let scaled_centre = self.scale_pt(*centre);
                let scaled_radius = radius * self.scale;
                Some(peniko::Brush::Gradient(
                    peniko::Gradient::new_radial(scaled_centre, scaled_radius)
                        .with_stops([stop0, stop1]),
                ))
            }
            LocalBrushKind::Custom(gradient) => Some(peniko::Brush::Gradient(gradient.clone())),
            LocalBrushKind::Unknown => None,
        }
    }

    const fn brush_color(brush: &Brush) -> Option<peniko::Color> {
        match LocalBrushKind::from_brush_kind(brush.kind()) {
            LocalBrushKind::Solid(c) => Some(Self::color_to_peniko(*c)),
            LocalBrushKind::LinearGradient { .. }
            | LocalBrushKind::RadialGradient { .. }
            | LocalBrushKind::Custom(_)
            | LocalBrushKind::Unknown => None,
        }
    }

    #[inline]
    const fn pen_color(pen: &Pen) -> peniko::Color {
        Self::color_to_peniko(pen.color())
    }

    fn to_bez_path(&self, path: &Path) -> kurbo::BezPath {
        let mut bez = kurbo::BezPath::new();
        let s = f64::from(self.scale);
        for seg in path.segments() {
            match seg {
                Segment::MoveTo(p) => bez.move_to(self.scale_pt(*p)),
                Segment::LineTo(p) => bez.line_to(self.scale_pt(*p)),
                Segment::CubicTo(c1, c2, p) => {
                    bez.curve_to(self.scale_pt(*c1), self.scale_pt(*c2), self.scale_pt(*p));
                }
                Segment::ArcTo {
                    centre,
                    radii,
                    start_angle,
                    sweep_angle,
                } => {
                    let arc = kurbo::Arc::new(
                        (f64::from(centre.x()) * s, f64::from(centre.y()) * s),
                        (f64::from(radii.width()) * s, f64::from(radii.height()) * s),
                        f64::from(*start_angle),
                        f64::from(*sweep_angle),
                        0.0,
                    );
                    bez.extend(arc.append_iter(0.1));
                }
                Segment::Close => bez.close_path(),
                _ => {}
            }
        }
        bez
    }

    /// Pushes font size, family, weight, and italic style onto a parley builder.
    fn push_font_style(builder: &mut RangedBuilder<'_, [u8; 4]>, font: &Font) {
        builder.push_default(StyleProperty::FontSize(font.size_pt()));
        builder.push_default(StyleProperty::from(FontFamilyName::named(font.family())));
        builder.push_default(StyleProperty::FontWeight(ParleyFontWeight::new(f32::from(
            font.weight() as u16,
        ))));
        if font.italic() {
            builder.push_default(StyleProperty::FontStyle(ParleyFontStyle::Italic));
        }
    }

    /// Iterates `layout` and emits glyph runs (plus underline / strikethrough) into `self.scene`.
    fn emit_layout_glyphs(
        &mut self,
        layout: &Layout<[u8; 4]>,
        px: f64,
        py: f64,
        fill_color: peniko::Color,
        font: &Font,
    ) {
        let xform = self.current_xform();
        let peniko_brush = peniko::Brush::Solid(fill_color);
        for layout_line in layout.lines() {
            for item in layout_line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                let parley_font = run.font();
                let run_size = run.font_size();
                let normalized = run.normalized_coords();
                let baseline = f64::from(glyph_run.baseline());
                let offset = f64::from(glyph_run.offset());

                // positioned_glyphs() already encodes both the centering offset and the
                // per-run baseline into g.x / g.y. The run transform only supplies the
                // text-block origin (px, py); adding offset/baseline a second time here
                // would double-count them.
                let run_xform = xform * kurbo::Affine::translate((px, py));
                self.scene
                    .draw_glyphs(parley_font)
                    .font_size(run_size)
                    .transform(run_xform)
                    .normalized_coords(normalized)
                    .brush(&peniko_brush)
                    .draw(
                        Fill::NonZero,
                        glyph_run.positioned_glyphs().map(|g| Glyph {
                            id: g.id,
                            x: g.x,
                            y: g.y,
                        }),
                    );

                if font.underline() {
                    let m = run.metrics();
                    let uy = py + baseline + f64::from(m.underline_offset);
                    let x1 = px + offset + f64::from(glyph_run.advance());
                    let seg = kurbo::Line::new((px + offset, uy), (x1, uy));
                    let stroke = kurbo::Stroke::new(f64::from(m.underline_size));
                    self.scene.stroke(&stroke, xform, fill_color, None, &seg);
                }
                if font.strikethrough() {
                    let m = run.metrics();
                    let sy = py + baseline + f64::from(m.strikethrough_offset);
                    let x1 = px + offset + f64::from(glyph_run.advance());
                    let seg = kurbo::Line::new((px + offset, sy), (x1, sy));
                    let stroke = kurbo::Stroke::new(f64::from(m.strikethrough_size));
                    self.scene.stroke(&stroke, xform, fill_color, None, &seg);
                }
            }
        }
    }

    /// Returns `(transform_depth, total_active_clip_layers)` for stack tests.
    ///
    /// `transform_depth` is the current save-frame depth (1 at construction,
    /// incremented by each `save()`, decremented by each `restore()`).
    /// `total_active_clip_layers` is the sum of all `push_layer` calls not yet
    /// matched by `pop_layer`.
    #[cfg(test)]
    pub(crate) fn debug_stack_state(&self) -> (usize, u32) {
        (self.xforms.len(), self.clips.iter().sum())
    }
}

impl Painter for VelloPainter<'_> {
    fn draw_rect(&mut self, rect: Rect, pen: &Pen, brush: &Brush) {
        let r = self.scale_rect(rect);
        let xform = self.current_xform();
        if let Some(fill_brush) = self.brush_to_peniko(brush) {
            self.scene.fill(Fill::NonZero, xform, &fill_brush, None, &r);
        }
        let stroke = kurbo::Stroke::new(f64::from(pen.width()) * f64::from(self.scale));
        self.scene
            .stroke(&stroke, xform, Self::pen_color(pen), None, &r);
    }

    fn fill_rect(&mut self, rect: Rect, brush: &Brush) {
        if let Some(fill_brush) = self.brush_to_peniko(brush) {
            let r = self.scale_rect(rect);
            let xform = self.current_xform();
            self.scene.fill(Fill::NonZero, xform, &fill_brush, None, &r);
        }
    }

    fn draw_line(&mut self, from: Point, to: Point, pen: &Pen) {
        let p0 = self.scale_pt(from);
        let p1 = self.scale_pt(to);
        let xform = self.current_xform();
        let stroke = kurbo::Stroke::new(f64::from(pen.width()) * f64::from(self.scale));
        let line = kurbo::Line::new(p0, p1);
        self.scene
            .stroke(&stroke, xform, Self::pen_color(pen), None, &line);
    }

    fn clip_rect(&mut self, rect: Rect) {
        let r = self.scale_rect(rect);
        let xform = self.current_xform();
        self.scene.push_clip_layer(Fill::NonZero, xform, &r);
        *self.clips.last_mut().expect("clips stack is never empty") += 1;
    }

    fn translate(&mut self, delta: Point) {
        let s = f64::from(self.scale);
        let t = kurbo::Affine::translate((f64::from(delta.x()) * s, f64::from(delta.y()) * s));
        let xf = self.xforms.last_mut().expect("xforms stack is never empty");
        *xf *= t;
    }

    fn save(&mut self) {
        let current = self.current_xform();
        self.xforms.push(current);
        self.clips.push(0);
    }

    fn restore(&mut self) {
        if self.xforms.len() <= 1 {
            return;
        }
        if let Some(clip_count) = self.clips.pop() {
            for _ in 0..clip_count {
                self.scene.pop_layer();
            }
        }
        self.xforms.pop();
    }

    fn draw_text(&mut self, pos: Point, text: &str, font: &Font, brush: &Brush) {
        if text.is_empty() {
            return;
        }
        let Some(fill_color) = Self::brush_color(brush) else {
            return;
        };
        let s = f64::from(self.scale);
        let px = f64::from(pos.x()) * s;
        let py = f64::from(pos.y()) * s;

        let layout: Layout<[u8; 4]> = {
            let Some(fonts) = self.fonts.as_mut() else {
                return;
            };
            let FontCache { font_cx, layout_cx } = fonts;
            let mut builder = layout_cx.ranged_builder(font_cx, text, self.scale, false);
            Self::push_font_style(&mut builder, font);
            let mut layout = builder.build(text);
            layout.break_all_lines(None);
            layout.align(ParleyAlignment::Left, AlignmentOptions::default());
            layout
        };

        self.emit_layout_glyphs(&layout, px, py, fill_color, font);
    }

    fn draw_text_in(
        &mut self,
        rect: Rect,
        text: &str,
        font: &Font,
        brush: &Brush,
        alignment: Alignment,
    ) {
        if text.is_empty() {
            return;
        }
        let Some(fill_color) = Self::brush_color(brush) else {
            return;
        };
        let s = f64::from(self.scale);
        let px = f64::from(rect.left()) * s;
        let py = f64::from(rect.top()) * s;
        #[allow(
            clippy::cast_precision_loss,
            reason = "sub-pixel coordinate widening is intentional"
        )]
        let max_advance = rect.size().width() as f32 * self.scale;

        let layout: Layout<[u8; 4]> = {
            let Some(fonts) = self.fonts.as_mut() else {
                return;
            };
            let FontCache { font_cx, layout_cx } = fonts;
            let mut builder = layout_cx.ranged_builder(font_cx, text, self.scale, false);
            Self::push_font_style(&mut builder, font);
            let mut layout = builder.build(text);
            let wrap = if max_advance > 0.0 {
                Some(max_advance)
            } else {
                None
            };
            layout.break_all_lines(wrap);
            let parley_align = match alignment {
                Alignment::Left => ParleyAlignment::Left,
                Alignment::Center => ParleyAlignment::Center,
                Alignment::Right => ParleyAlignment::Right,
                Alignment::Justify => ParleyAlignment::Justify,
            };
            layout.align(parley_align, AlignmentOptions::default());
            layout
        };

        self.emit_layout_glyphs(&layout, px, py, fill_color, font);
    }

    fn draw_image(&mut self, rect: Rect, image: &Image) {
        if image.width() == 0 || image.height() == 0 {
            return;
        }
        let blob: Blob<u8> = Blob::from(image.pixels().to_vec());
        let img_data = ImageData {
            data: blob,
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width: image.width(),
            height: image.height(),
        };
        let s = f64::from(self.scale);
        let sw = f64::from(rect.size().width()) * s / f64::from(image.width());
        let sh = f64::from(rect.size().height()) * s / f64::from(image.height());
        let tx = f64::from(rect.left()) * s;
        let ty = f64::from(rect.top()) * s;
        let local = kurbo::Affine::new([sw, 0.0, 0.0, sh, tx, ty]);
        let img_xform = self.current_xform() * local;
        self.scene.draw_image(&img_data, img_xform);
    }

    fn draw_path(&mut self, path: &Path, pen: &Pen, brush: &Brush) {
        let bez = self.to_bez_path(path);
        let xform = self.current_xform();
        if let Some(fill_brush) = self.brush_to_peniko(brush) {
            self.scene
                .fill(Fill::NonZero, xform, &fill_brush, None, &bez);
        }
        let stroke = kurbo::Stroke::new(f64::from(pen.width()) * f64::from(self.scale));
        self.scene
            .stroke(&stroke, xform, Self::pen_color(pen), None, &bez);
    }
}

#[cfg(test)]
mod tests {
    use quartzite_geometry::{Point, Rect, Size};
    use quartzite_paint_api::{Brush, BrushKind, Color, Font, Image, Painter, Path, Pen};
    use vello::Scene;

    use crate::font::FontCache;

    use super::*;

    fn make_scene_and_cache() -> (Scene, FontCache) {
        (Scene::new(), FontCache::new())
    }

    #[test]
    fn painter_starts_with_identity_transform_and_no_clips() {
        let (mut scene, mut cache) = make_scene_and_cache();
        let p = VelloPainter::new(&mut scene).with_fonts(&mut cache);
        assert_eq!(p.debug_stack_state(), (1, 0));
    }

    #[test]
    fn save_then_restore_round_trip() {
        let (mut scene, mut cache) = make_scene_and_cache();
        let mut p = VelloPainter::new(&mut scene).with_fonts(&mut cache);
        assert_eq!(p.debug_stack_state(), (1, 0));
        p.save();
        assert_eq!(p.debug_stack_state(), (2, 0));
        p.restore();
        assert_eq!(p.debug_stack_state(), (1, 0));
    }

    #[test]
    fn translate_modifies_top_only() {
        let (mut scene, mut cache) = make_scene_and_cache();
        let mut p = VelloPainter::new(&mut scene).with_fonts(&mut cache);
        p.save();
        p.translate(Point::new(10, 0));
        assert_eq!(p.debug_stack_state(), (2, 0));
        p.save();
        p.translate(Point::new(5, 5));
        assert_eq!(p.debug_stack_state(), (3, 0));
        p.restore();
        assert_eq!(p.debug_stack_state(), (2, 0));
        p.restore();
        assert_eq!(p.debug_stack_state(), (1, 0));
    }

    #[test]
    fn clip_rect_increments_active_clip_count() {
        let (mut scene, mut cache) = make_scene_and_cache();
        let mut p = VelloPainter::new(&mut scene).with_fonts(&mut cache);
        let rect = Rect::new(Point::new(0, 0), Size::new(10, 10));
        p.save();
        p.clip_rect(rect);
        assert_eq!(p.debug_stack_state(), (2, 1));
        p.clip_rect(rect);
        assert_eq!(p.debug_stack_state(), (2, 2));
        p.restore();
        assert_eq!(p.debug_stack_state(), (1, 0));
    }

    #[test]
    fn restore_at_base_frame_is_no_op() {
        let (mut scene, mut cache) = make_scene_and_cache();
        let mut p = VelloPainter::new(&mut scene).with_fonts(&mut cache);
        p.restore();
        assert_eq!(p.debug_stack_state(), (1, 0));
        p.restore();
        assert_eq!(p.debug_stack_state(), (1, 0));
    }

    #[test]
    fn all_painter_methods_are_invocable() {
        let (mut scene, mut cache) = make_scene_and_cache();
        let mut p: Box<dyn Painter> =
            Box::new(VelloPainter::new(&mut scene).with_fonts(&mut cache));
        let pen = Pen::new(Color::BLACK, 1.0);
        let brush = Brush::solid(Color::WHITE);
        assert_eq!(brush.kind(), &BrushKind::Solid(Color::WHITE));
        let rect = Rect::new(Point::new(0, 0), Size::new(10, 10));
        let origin = Point::new(0, 0);
        let font = Font::new("Arial", 12.0);
        let image = Image::try_new(1, 1, vec![0u8, 0, 0, 0]).unwrap();
        let path = Path::new();

        p.draw_rect(rect, &pen, &brush);
        p.fill_rect(rect, &brush);
        p.draw_line(origin, origin, &pen);
        p.clip_rect(rect);
        p.translate(origin);
        p.save();
        p.restore();
        p.draw_text(origin, "hi", &font, &brush);
        p.draw_text_in(rect, "hi", &font, &brush, Alignment::Left);
        p.draw_image(rect, &image);
        p.draw_path(&path, &pen, &brush);

        // gradient brushes must not panic
        let linear = Brush::linear_gradient(origin, Point::new(10, 0), Color::RED, Color::BLUE);
        let radial = Brush::radial_gradient(origin, 5.0, Color::WHITE, Color::BLACK);
        let custom = Brush::custom_gradient(
            peniko::Gradient::new_linear((0.0f64, 0.0f64), (10.0f64, 0.0f64)).with_stops([
                peniko::ColorStop {
                    offset: 0.0,
                    color: peniko::color::DynamicColor::from_alpha_color(peniko::Color::new([
                        1.0f32, 0.0, 0.0, 1.0,
                    ])),
                },
                peniko::ColorStop {
                    offset: 1.0,
                    color: peniko::color::DynamicColor::from_alpha_color(peniko::Color::new([
                        0.0f32, 0.0, 1.0, 1.0,
                    ])),
                },
            ]),
        );
        p.fill_rect(rect, &linear);
        p.fill_rect(rect, &radial);
        p.fill_rect(rect, &custom);
        p.draw_rect(rect, &pen, &linear);
        p.draw_path(&path, &pen, &radial);
        p.draw_path(&path, &pen, &custom);
    }

    // R8 — draw_text with italic/underline/strikethrough font flags
    #[test]
    fn draw_text_with_italic_font_does_not_panic() {
        let (mut scene, mut cache) = make_scene_and_cache();
        let mut p = VelloPainter::new(&mut scene).with_fonts(&mut cache);
        let font = Font::new("Arial", 12.0).with_italic(true);
        let brush = Brush::solid(Color::BLACK);
        p.draw_text(Point::new(0, 0), "hello", &font, &brush);
    }

    #[test]
    fn draw_text_with_underline_font_does_not_panic() {
        let (mut scene, mut cache) = make_scene_and_cache();
        let mut p = VelloPainter::new(&mut scene).with_fonts(&mut cache);
        let font = Font::new("Arial", 12.0).with_underline(true);
        let brush = Brush::solid(Color::BLACK);
        p.draw_text(Point::new(0, 0), "hello", &font, &brush);
    }

    #[test]
    fn draw_text_with_strikethrough_font_does_not_panic() {
        let (mut scene, mut cache) = make_scene_and_cache();
        let mut p = VelloPainter::new(&mut scene).with_fonts(&mut cache);
        let font = Font::new("Arial", 12.0).with_strikethrough(true);
        let brush = Brush::solid(Color::BLACK);
        p.draw_text(Point::new(0, 0), "hello", &font, &brush);
    }

    // R8 — LocalBrushKind::from_brush_kind round-trip classification
    #[test]
    fn local_brush_kind_solid_classifies_correctly() {
        let brush = Brush::solid(Color::RED);
        assert!(matches!(
            LocalBrushKind::from_brush_kind(brush.kind()),
            LocalBrushKind::Solid(_)
        ));
    }

    #[test]
    fn local_brush_kind_linear_gradient_classifies_correctly() {
        let brush =
            Brush::linear_gradient(Point::new(0, 0), Point::new(10, 0), Color::RED, Color::BLUE);
        assert!(matches!(
            LocalBrushKind::from_brush_kind(brush.kind()),
            LocalBrushKind::LinearGradient { .. }
        ));
    }

    #[test]
    fn local_brush_kind_radial_gradient_classifies_correctly() {
        let brush = Brush::radial_gradient(Point::new(5, 5), 5.0, Color::WHITE, Color::BLACK);
        assert!(matches!(
            LocalBrushKind::from_brush_kind(brush.kind()),
            LocalBrushKind::RadialGradient { .. }
        ));
    }

    #[test]
    fn local_brush_kind_custom_classifies_correctly() {
        let gradient = peniko::Gradient::new_linear((0.0f64, 0.0f64), (10.0f64, 0.0f64))
            .with_stops([
                peniko::ColorStop {
                    offset: 0.0,
                    color: peniko::color::DynamicColor::from_alpha_color(peniko::Color::new([
                        1.0f32, 0.0, 0.0, 1.0,
                    ])),
                },
                peniko::ColorStop {
                    offset: 1.0,
                    color: peniko::color::DynamicColor::from_alpha_color(peniko::Color::new([
                        0.0f32, 0.0, 1.0, 1.0,
                    ])),
                },
            ]);
        let brush = Brush::custom_gradient(gradient);
        assert!(matches!(
            LocalBrushKind::from_brush_kind(brush.kind()),
            LocalBrushKind::Custom(_)
        ));
    }
}
