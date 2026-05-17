//! GPU snapshot tests for built-in widgets and layouts.
//!
//! Each test builds a deterministic widget (or layout) or drives the
//! [`VelloPainter`] directly, renders through a [`RenderHarness`], and
//! either compares to a committed golden via [`support::snapshot_assert`]
//! or asserts pixel-level properties directly.
//!
//! v1 goldens for the layout/widget tests encode the wgpu clear-colour
//! baseline — widget `WidgetExt::paint` overrides are still no-ops.
//! `VelloPainter` draw methods are now real and exercised via the
//! painter-method tests in this file (AC1–AC9, AC11–AC12; issue #281
//! gradient AC7–AC10).

mod support;

use quartzite_core::ObjectId;
use quartzite_geometry::{Alignment, Point, Rect, Size};
use quartzite_paint_api::{Brush, Color, Font, Image, Path, Pen};
use quartzite_renderer::RenderHarnessBuilder;
use quartzite_widgets::{BoxLayout, Button, Direction, GridLayout, Label, LineEdit};

use support::{harness_or_skip, snapshot_assert, snapshot_widget};

/// Canvas size shared by the standard snapshot tests. 64 keeps committed
/// PNGs small.
const CANVAS: u32 = 64;

/// Background colour used by [`RenderHarness`] as the clear colour.
const BG: [u8; 4] = [0, 0, 0, 255];

/// Constructs a harness from `builder` or skips the test when no GPU adapter
/// is available locally or `SKIP_RENDER_SNAPSHOT=1` is set.
///
/// For the standard 64×64 canvas use [`support::harness_or_skip`] instead.
/// This variant is kept for HiDPI tests that need a custom [`RenderHarnessBuilder`].
fn harness_or_skip_with(
    name: &str,
    builder: RenderHarnessBuilder,
) -> Option<quartzite_renderer::RenderHarness> {
    if std::env::var_os(support::SKIP_ENV).is_some_and(|v| !v.is_empty()) {
        eprintln!("{name}: {} set; skipping", support::SKIP_ENV);
        return None;
    }
    match builder.build() {
        Ok(h) => Some(h),
        Err(e) => {
            eprintln!("{name}: no GPU adapter available ({e}); skipping");
            None
        }
    }
}

/// Count pixels that differ from the harness background colour (black).
fn count_non_background(image: &image::RgbaImage) -> u32 {
    image.pixels().filter(|px| px.0 != BG).count() as u32
}

// ── widget / layout tests ──────────────────────────────────────────────────

#[test]
fn label_renders() {
    let Some(mut harness) = harness_or_skip("label_renders") else {
        return;
    };
    let label = Label::new("test".into());
    snapshot_widget(&mut harness, "label", &label);
}

#[test]
fn button_renders() {
    let Some(mut harness) = harness_or_skip("button_renders") else {
        return;
    };
    let button = Button::new("OK".into());
    snapshot_widget(&mut harness, "button", &button);
}

#[test]
fn line_edit_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_renders") else {
        return;
    };
    let mut edit = LineEdit::new();
    edit.set_text("hello".into());
    snapshot_widget(&mut harness, "line_edit", &edit);
}

#[test]
fn box_layout_renders() {
    // BoxLayout / GridLayout are not WidgetExt impls — they distribute
    // geometry across child widgets via a resolver. The snapshot tests
    // exercise the layout's construction + child registration path,
    // then capture the harness's clear-colour image (the renderer is a
    // no-op for v1 regardless of which widgets are in the layout).
    let Some(mut harness) = harness_or_skip("box_layout_renders") else {
        return;
    };
    let mut layout = BoxLayout::new(Direction::Horizontal);
    layout.add_child(ObjectId::new(), 1);
    layout.add_child(ObjectId::new(), 1);
    let image = harness.render_widget(|_painter| {});
    snapshot_assert("box_layout", &image);
}

#[test]
fn grid_layout_renders() {
    let Some(mut harness) = harness_or_skip("grid_layout_renders") else {
        return;
    };
    let mut layout = GridLayout::new();
    layout.add_cell(ObjectId::new(), 0, 0);
    layout.add_cell(ObjectId::new(), 0, 1);
    layout.add_cell(ObjectId::new(), 1, 0);
    layout.add_cell(ObjectId::new(), 1, 1);
    let image = harness.render_widget(|_painter| {});
    snapshot_assert("grid_layout", &image);
}

// ── painter-method tests (AC1–AC9, AC11–AC12) ─────────────────────────────

/// AC1 — `fill_rect` paints solid red pixels over the given rect.
#[test]
fn fill_rect_paints_red() {
    let Some(mut harness) = harness_or_skip("fill_rect_paints_red") else {
        return;
    };
    let brush = Brush::solid(Color::RED);
    let rect = Rect::new(Point::new(8, 8), Size::new(48, 48));
    let image = harness.render_widget(|p| p.fill_rect(rect, &brush));
    let px = image.get_pixel(32, 32);
    assert_eq!(
        px.0,
        [255, 0, 0, 255],
        "fill_rect: pixel at (32,32) should be red"
    );
}

/// AC2 — `draw_rect` with a non-zero pen produces pixels different from
/// `fill_rect` alone (the stroke outline is rendered, not dropped).
#[test]
fn draw_rect_outline_differs_from_fill() {
    let Some(mut harness) = harness_or_skip("draw_rect_outline_differs_from_fill") else {
        return;
    };
    let brush = Brush::solid(Color::WHITE);
    let pen = Pen::new(Color::RED, 2.0);
    let rect = Rect::new(Point::new(8, 8), Size::new(48, 48));
    let fill_img = harness.render_widget(|p| p.fill_rect(rect, &brush));
    let rect_img = harness.render_widget(|p| p.draw_rect(rect, &pen, &brush));
    assert_ne!(
        fill_img.into_raw(),
        rect_img.into_raw(),
        "draw_rect with a red pen must produce different pixels from fill_rect alone"
    );
}

/// AC3 — `draw_path` with a line + cubic + arc produces non-background pixels.
#[test]
fn draw_path_emits_curve() {
    let Some(mut harness) = harness_or_skip("draw_path_emits_curve") else {
        return;
    };
    let mut path = Path::new();
    path.move_to(Point::new(8, 8))
        .line_to(Point::new(32, 8))
        .cubic_to(Point::new(50, 8), Point::new(56, 40), Point::new(48, 56))
        .arc_to(
            Point::new(32, 32),
            Size::new(16, 16),
            0.0,
            std::f32::consts::PI,
        )
        .close();
    let pen = Pen::new(Color::WHITE, 1.0);
    let brush = Brush::solid(Color::TRANSPARENT);
    let image = harness.render_widget(|p| p.draw_path(&path, &pen, &brush));
    let non_bg = count_non_background(&image);
    assert!(
        non_bg > 0,
        "draw_path should produce non-background pixels, got {non_bg}"
    );
}

/// AC4 — `draw_image` maps a 2×2 RGBA source into a 64×64 canvas.
/// Each source pixel occupies a 32×32 block; sampled near the corners of
/// each block the dominant channel matches the source colour.
#[test]
fn draw_image_quadrants() {
    let Some(mut harness) = harness_or_skip("draw_image_quadrants") else {
        return;
    };
    // Row-major 2×2: (0,0)=RED  (1,0)=GREEN  (0,1)=BLUE  (1,1)=WHITE
    let pixels = vec![
        255u8, 0, 0, 255, // (0,0) RED
        0, 255, 0, 255, // (1,0) GREEN
        0, 0, 255, 255, // (0,1) BLUE
        255, 255, 255, 255, // (1,1) WHITE
    ];
    let image = Image::try_new(2, 2, pixels).expect("valid 2×2 RGBA buffer");
    let canvas = CANVAS as i32;
    let dest = Rect::new(Point::new(0, 0), Size::new(canvas, canvas));
    let rendered = harness.render_widget(|p| p.draw_image(dest, &image));

    // Each source pixel maps to a 32×32 destination block. Sampling at
    // 1/8 of the canvas from each corner keeps us deep inside each block,
    // where the dominant channel of the bilinear blend still matches the
    // source pixel.
    let tl = rendered.get_pixel(4, 4); // top-left  → RED
    let tr = rendered.get_pixel(60, 4); // top-right → GREEN
    let bl = rendered.get_pixel(4, 60); // btm-left  → BLUE
    let br = rendered.get_pixel(60, 60); // btm-right → WHITE

    assert!(
        tl.0[0] > tl.0[1] && tl.0[0] > tl.0[2],
        "top-left quadrant should be red-dominant, got {tl:?}"
    );
    assert!(
        tr.0[1] > tr.0[0] && tr.0[1] > tr.0[2],
        "top-right quadrant should be green-dominant, got {tr:?}"
    );
    assert!(
        bl.0[2] > bl.0[0] && bl.0[2] > bl.0[1],
        "bottom-left quadrant should be blue-dominant, got {bl:?}"
    );
    assert!(
        br.0[0] > 200 && br.0[1] > 200 && br.0[2] > 200,
        "bottom-right quadrant should be near-white, got {br:?}"
    );
}

/// AC5 — `save` / `translate` / `restore` positions two rectangles correctly.
#[test]
fn translate_save_restore() {
    let Some(mut harness) = harness_or_skip("translate_save_restore") else {
        return;
    };
    let brush = Brush::solid(Color::WHITE);
    let r = Rect::new(Point::new(0, 0), Size::new(20, 20));
    let image = harness.render_widget(|p| {
        // Shifted rect: covers [20, 40) × [0, 20) in physical space
        p.save();
        p.translate(Point::new(20, 0));
        p.fill_rect(r, &brush);
        p.restore();
        // Original rect: covers [0, 20) × [0, 20)
        p.fill_rect(r, &brush);
    });
    // Within [0, 20) — painted by the un-shifted rect
    assert_eq!(
        image.get_pixel(5, 5).0,
        [255, 255, 255, 255],
        "origin pixel should be white"
    );
    // Within [20, 40) — painted by the shifted rect
    assert_eq!(
        image.get_pixel(30, 5).0,
        [255, 255, 255, 255],
        "shifted pixel should be white"
    );
    // Outside both rects — should remain black
    assert_eq!(
        image.get_pixel(50, 5).0,
        [0, 0, 0, 255],
        "pixel outside both rects should be black"
    );
}

/// AC6 — `save` / `clip_rect` / `fill_rect` / `restore` fills only inside the clip.
#[test]
fn clip_rect_save_restore() {
    let Some(mut harness) = harness_or_skip("clip_rect_save_restore") else {
        return;
    };
    let brush = Brush::solid(Color::WHITE);
    let clip = Rect::new(Point::new(8, 8), Size::new(20, 20));
    let canvas = CANVAS as i32;
    let large = Rect::new(Point::new(0, 0), Size::new(canvas, canvas));
    let image = harness.render_widget(|p| {
        p.save();
        p.clip_rect(clip);
        p.fill_rect(large, &brush);
        p.restore();
    });
    // Inside [8, 28) × [8, 28) — clipped fill should paint white
    assert_eq!(
        image.get_pixel(15, 15).0,
        [255, 255, 255, 255],
        "inside clip should be white"
    );
    // Outside clip: left edge, top edge, and bottom-right
    assert_eq!(
        image.get_pixel(4, 15).0,
        [0, 0, 0, 255],
        "outside clip (left) should be black"
    );
    assert_eq!(
        image.get_pixel(15, 4).0,
        [0, 0, 0, 255],
        "outside clip (top) should be black"
    );
    assert_eq!(
        image.get_pixel(32, 32).0,
        [0, 0, 0, 255],
        "outside clip (bottom-right) should be black"
    );
}

// AC7 — object-safety: the existing `all_painter_methods_are_invocable` test
// in `quartzite-renderer/src/vello_painter.rs` covers this AC.

/// AC8 — `draw_text` with a system sans-serif font produces at least one
/// non-background pixel along the rendered baseline.
#[test]
fn draw_text_basic() {
    let Some(mut harness) = harness_or_skip("draw_text_basic") else {
        return;
    };
    let font = Font::new("sans-serif", 16.0);
    let brush = Brush::solid(Color::WHITE);
    let image = harness.render_widget(|p| {
        p.draw_text(Point::new(8, 24), "Hello", &font, &brush);
    });
    let non_bg = count_non_background(&image);
    assert!(
        non_bg > 0,
        "draw_text should produce non-background pixels, got {non_bg}"
    );
}

/// AC9 — `draw_text_in` with `Alignment::Center` horizontally centres the
/// rendered glyphs within the rect (midpoint of leftmost .. rightmost
/// non-background pixel within ±2px of the rect centre).
#[test]
fn draw_text_in_center() {
    let Some(mut harness) = harness_or_skip("draw_text_in_center") else {
        return;
    };
    let canvas = CANVAS as i32;
    let rect = Rect::new(Point::new(0, 0), Size::new(canvas, canvas));
    let font = Font::new("sans-serif", 16.0);
    let brush = Brush::solid(Color::WHITE);
    let image = harness.render_widget(|p| {
        p.draw_text_in(rect, "wrap me", &font, &brush, Alignment::Center);
    });
    // Scan for leftmost / rightmost non-background pixel across all rows.
    let mut left_x = u32::MAX;
    let mut right_x = 0u32;
    for y in 0..CANVAS {
        for x in 0..CANVAS {
            if image.get_pixel(x, y).0 != BG {
                left_x = left_x.min(x);
                right_x = right_x.max(x);
            }
        }
    }
    // If font resolution produced no glyphs (extremely rare), skip the
    // metric assertion rather than failing the suite.
    if left_x == u32::MAX {
        eprintln!(
            "draw_text_in_center: no non-background pixels; font may be unavailable — skipping"
        );
        return;
    }
    let mid = left_x.midpoint(right_x);
    let expected = CANVAS / 2;
    assert!(
        mid.abs_diff(expected) <= 2,
        "centred text midpoint should be within ±2px of {expected}, \
         got mid={mid} (left={left_x}, right={right_x})"
    );
}

/// AC11 — `scale_factor(2.0)` maps a 10×10 logical rect to ~20×20 physical
/// pixels: pixel (18, 18) must be filled; pixel (22, 22) must still be
/// background.
#[test]
fn hidpi_2x_extent() {
    let Some(mut harness) = harness_or_skip_with(
        "hidpi_2x_extent",
        RenderHarnessBuilder::new(40, 40).scale_factor(2.0),
    ) else {
        return;
    };
    let brush = Brush::solid(Color::WHITE);
    let logical_rect = Rect::new(Point::new(0, 0), Size::new(10, 10));
    let image = harness.render_widget(|p| p.fill_rect(logical_rect, &brush));
    assert_eq!(
        image.get_pixel(18, 18).0,
        [255, 255, 255, 255],
        "hidpi 2×: pixel (18,18) should be within the 20×20 physical fill"
    );
    assert_eq!(
        image.get_pixel(22, 22).0,
        [0, 0, 0, 255],
        "hidpi 2×: pixel (22,22) should be outside the 20×20 physical fill"
    );
}

/// AC12 — `scale_factor(1.0)` maps a 10×10 logical rect to exactly 10×10
/// physical pixels: pixel (8, 8) must be filled; pixel (12, 12) must still
/// be background.
#[test]
fn dpr_1_0_default_extent() {
    let Some(mut harness) = harness_or_skip_with(
        "dpr_1_0_default_extent",
        RenderHarnessBuilder::new(40, 40).scale_factor(1.0),
    ) else {
        return;
    };
    let brush = Brush::solid(Color::WHITE);
    let logical_rect = Rect::new(Point::new(0, 0), Size::new(10, 10));
    let image = harness.render_widget(|p| p.fill_rect(logical_rect, &brush));
    assert_eq!(
        image.get_pixel(8, 8).0,
        [255, 255, 255, 255],
        "dpr 1.0: pixel (8,8) should be within the 10×10 physical fill"
    );
    assert_eq!(
        image.get_pixel(12, 12).0,
        [0, 0, 0, 255],
        "dpr 1.0: pixel (12,12) should be outside the 10×10 physical fill"
    );
}

// ── gradient brush tests (issue #281: AC7–AC10) ────────────────────────────

/// Gradient-AC7 (issue #281) — `fill_rect` with a `LinearGradient` brush
/// produces a red-dominant left pixel and a blue-dominant right pixel.
#[test]
fn fill_rect_linear_gradient() {
    let Some(mut harness) = harness_or_skip_with(
        "fill_rect_linear_gradient",
        RenderHarnessBuilder::new(20, 1),
    ) else {
        return;
    };
    let brush =
        Brush::linear_gradient(Point::new(0, 0), Point::new(20, 0), Color::RED, Color::BLUE);
    let rect = Rect::new(Point::new(0, 0), Size::new(20, 1));
    let image = harness.render_widget(|p| p.fill_rect(rect, &brush));
    let left = image.get_pixel(0, 0);
    let right = image.get_pixel(19, 0);
    assert!(
        left.0[0] > 200,
        "left pixel should be red-dominant, got {left:?}"
    );
    assert!(
        right.0[2] > 200,
        "right pixel should be blue-dominant, got {right:?}"
    );
}

/// Gradient-AC8 (issue #281) — `fill_rect` with a `RadialGradient` brush
/// produces a near-white centre and a darker edge.
#[test]
fn fill_rect_radial_gradient() {
    let Some(mut harness) = harness_or_skip_with(
        "fill_rect_radial_gradient",
        RenderHarnessBuilder::new(21, 21),
    ) else {
        return;
    };
    let brush = Brush::radial_gradient(Point::new(10, 10), 9.0, Color::WHITE, Color::BLACK);
    let rect = Rect::new(Point::new(0, 0), Size::new(21, 21));
    let image = harness.render_widget(|p| p.fill_rect(rect, &brush));
    let centre = image.get_pixel(10, 10);
    let edge = image.get_pixel(0, 10);
    assert!(
        centre.0[0] > 200 && centre.0[1] > 200 && centre.0[2] > 200,
        "centre pixel should be near-white, got {centre:?}"
    );
    assert!(
        edge.0[0] < 100 || edge.0[1] < 100 || edge.0[2] < 100,
        "edge pixel should be dark (at least one channel < 100), got {edge:?}"
    );
}

/// Gradient-AC9 (issue #281) — `fill_rect` with a 3-stop `Custom` gradient
/// (RED → GREEN → BLUE) produces a green-dominant middle pixel.
#[test]
fn fill_rect_custom_gradient() {
    let Some(mut harness) = harness_or_skip_with(
        "fill_rect_custom_gradient",
        RenderHarnessBuilder::new(21, 1),
    ) else {
        return;
    };
    let gradient = peniko::Gradient::new_linear((0.0f64, 0.0f64), (21.0f64, 0.0f64)).with_stops([
        (0.0f32, peniko::Color::new([1.0, 0.0, 0.0, 1.0])),
        (0.5f32, peniko::Color::new([0.0, 1.0, 0.0, 1.0])),
        (1.0f32, peniko::Color::new([0.0, 0.0, 1.0, 1.0])),
    ]);
    let brush = Brush::custom_gradient(gradient);
    let rect = Rect::new(Point::new(0, 0), Size::new(21, 1));
    let image = harness.render_widget(|p| p.fill_rect(rect, &brush));
    let mid = image.get_pixel(10, 0);
    let [r, g, b, _] = mid.0;
    assert!(
        g as u16 > r as u16 + 51 && g as u16 > b as u16 + 51,
        "middle pixel should be green-dominant (G > R+51, G > B+51), got {mid:?}"
    );
}

/// Gradient-AC10 (issue #281) — `draw_rect` with a gradient fill and a
/// white solid pen renders a near-white stroke border.
#[test]
fn draw_rect_gradient_fill_solid_stroke() {
    let Some(mut harness) = harness_or_skip("draw_rect_gradient_fill_solid_stroke") else {
        return;
    };
    let brush =
        Brush::linear_gradient(Point::new(0, 0), Point::new(64, 0), Color::RED, Color::BLUE);
    let pen = Pen::new(Color::WHITE, 3.0);
    let rect = Rect::new(Point::new(8, 8), Size::new(48, 48));
    let image = harness.render_widget(|p| p.draw_rect(rect, &pen, &brush));
    let border = image.get_pixel(8, 32);
    assert!(
        border.0[0] > 200 && border.0[1] > 200 && border.0[2] > 200,
        "left border pixel should be near-white (white pen), got {border:?}"
    );
}
