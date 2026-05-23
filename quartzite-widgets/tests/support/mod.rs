//! Test-side support for GPU snapshot tests.
//!
//! **Sync group:** kept in lock-step with
//! `quartzite-style/tests/support/mod.rs` (snapshot-helper group).
//! A change to one MUST be mirrored to the other in the same PR.
//!
//! Provides [`snapshot_assert`], [`harness_or_skip`], and [`snapshot_widget`]
//! used by the widget snapshot suite (`tests/snapshots.rs`) and by the
//! helper-internal unit tests (`tests/support_internals.rs`).
//!
//! The helper is intentionally **test-only** — `mod`-included from sibling
//! integration-test files via `mod support;`. It is not exported from the
//! `quartzite_widgets` library surface, which keeps test-only API out of
//! the crate's public API. Spec AC2's literal path
//! `quartzite_widgets::test_support::snapshot_assert` has an "e.g."
//! softener that the design takes advantage of.
//!
//! Workflow:
//! - `SKIP_RENDER_SNAPSHOT=1` in env → early return with an `eprintln!`
//!   notice. No GPU work, no file IO.
//! - `QUARTZITE_REGENERATE_SNAPSHOTS=1` in env → write the supplied image
//!   as the golden and return.
//! - Otherwise: load the golden, run an `nv-flip` perceptual diff, fail
//!   with `<name>.actual.png` + `<name>.diff.png` written next to the
//!   golden when the per-pixel mean error exceeds [`FLIP_TOLERANCE`].
//!
//! ## Per-backend dir + shared fallback
//!
//! Goldens live under `tests/snapshots/`:
//! - `tests/snapshots/<backend>/<name>.png` — per-backend override. Used
//!   when the backend produces pixels that drift from the cross-backend
//!   default (typical once real rasterization lands).
//! - `tests/snapshots/shared/<name>.png` — cross-backend default. Used
//!   when no per-backend override exists. Today's all-clear-colour
//!   goldens live here because the renderer is no-op and every backend
//!   produces byte-identical output.
//!
//! Lookup order is "backend override → shared default → fail".
//! Regeneration always writes to the per-backend dir matching the
//! `WGPU_BACKEND` env (so `update-snapshots.sh --backend vulkan` always
//! writes to `tests/snapshots/vulkan/`). Bootstrapping a fresh `shared/`
//! default is a manual step: regen on one backend, then `mv <backend>/* shared/`.
//!
//! The backend directory name is derived from the `WGPU_BACKEND` env var
//! (`vulkan` / `dx12` / `metal`) and falls back to `auto` when unset
//! (per spec AC2). The `shared/` fallback fires regardless of the resolved
//! backend dir.

#![allow(dead_code)] // not every sibling test calls every helper

use std::path::{Path, PathBuf};

use image::{Rgb, RgbImage, RgbaImage};
use nv_flip::{DEFAULT_PIXELS_PER_DEGREE, FlipImageRgb8, FlipPool, flip, magma_lut};
use quartzite_renderer::{RenderHarness, RenderHarnessBuilder};
use quartzite_style::{DefaultStyle, Style};
use quartzite_style_types::Palette;
use quartzite_widgets::AsWidget;

/// Workspace-wide perceptual-diff tolerance. The mean FLIP score across
/// the image must be at or below this value for a snapshot to pass.
///
/// `0.05` is the v1 default per the design's open question 1: any non-zero
/// FLIP score is a real difference for today's all-clear-colour goldens,
/// but `0.05` admits slack for backend-rounding drift.
pub const FLIP_TOLERANCE: f32 = 0.05;

/// Env var that, when set to any non-empty value, causes snapshot tests
/// (and the `xvfb-run` smoke test) to skip with a clear notice and pass.
pub const SKIP_ENV: &str = "SKIP_RENDER_SNAPSHOT";

/// Env var that, when set to any non-empty value, causes [`snapshot_assert`]
/// to *write* the supplied image as the new golden instead of comparing.
/// Driven by `scripts/update-snapshots.sh`.
pub const REGEN_ENV: &str = "QUARTZITE_REGENERATE_SNAPSHOTS";

/// Env var read by the helper to pick the per-backend snapshot directory.
pub const BACKEND_ENV: &str = "WGPU_BACKEND";

/// Default backend-dir name when `WGPU_BACKEND` is unset (spec AC2).
pub const DEFAULT_BACKEND_DIR: &str = "auto";

/// Cross-backend fallback dir name. Goldens here apply to every backend
/// that does not have a per-backend override at `<root>/<backend>/`.
pub const SHARED_DIR_NAME: &str = "shared";

/// Constructs a 64×64 [`RenderHarness`] or skips the test when no GPU
/// adapter is available or [`SKIP_ENV`] is set. Returns `None` on skip
/// so the caller can `let Some(mut harness) = harness_or_skip(…) else { return; }`.
pub fn harness_or_skip(name: &str) -> Option<RenderHarness> {
    if std::env::var_os(SKIP_ENV).is_some_and(|v| !v.is_empty()) {
        eprintln!("{name}: {SKIP_ENV} set; skipping");
        return None;
    }
    match RenderHarnessBuilder::new(64, 64).build() {
        Ok(h) => Some(h),
        Err(e) => {
            eprintln!("{name}: no GPU adapter available ({e}); skipping");
            None
        }
    }
}

/// Compares `image` against the golden at
/// `<root>/<backend_dir>/<name>.png`, falling back to
/// `<root>/shared/<name>.png` when no per-backend override exists. Writes
/// artifact PNGs and panics on mismatch.
///
/// `root` is the directory containing the per-backend subdirs (typically
/// `quartzite-widgets/tests/snapshots`). The widget snapshot suite calls
/// [`snapshot_assert`] (which uses the workspace default root); the
/// internals tests call this directly with a `tempfile::TempDir` root.
///
/// Errors are surfaced via `panic!` so the wrapping `#[test]` fn fails
/// with a reviewer-friendly message that names the artifact paths.
pub fn snapshot_assert_at(root: &Path, name: &str, image: &RgbaImage) {
    if std::env::var_os(SKIP_ENV).is_some_and(|v| !v.is_empty()) {
        eprintln!("snapshot_assert({name}): {SKIP_ENV} set; skipping");
        return;
    }
    let backend_dir = backend_dir_name();
    let backend_path = root.join(&backend_dir).join(format!("{name}.png"));
    let shared_path = root.join(SHARED_DIR_NAME).join(format!("{name}.png"));

    if std::env::var_os(REGEN_ENV).is_some_and(|v| !v.is_empty()) {
        // Regen always writes to the per-backend dir. Bootstrapping a new
        // shared default is a manual step (regen, then mv <backend>/* shared/).
        let backend_dir_path = root.join(&backend_dir);
        std::fs::create_dir_all(&backend_dir_path)
            .unwrap_or_else(|e| panic!("create snapshot dir {}: {e}", backend_dir_path.display()));
        image
            .save(&backend_path)
            .unwrap_or_else(|e| panic!("write golden {}: {e}", backend_path.display()));
        eprintln!(
            "snapshot_assert({name}): wrote golden {} (regen mode)",
            backend_path.display()
        );
        return;
    }

    let golden_path: &Path = if backend_path.exists() {
        &backend_path
    } else if shared_path.exists() {
        &shared_path
    } else {
        let actual_path = root.join(&backend_dir).join(format!("{name}.actual.png"));
        let actual_dir = actual_path.parent().unwrap_or(root);
        std::fs::create_dir_all(actual_dir)
            .unwrap_or_else(|e| panic!("create snapshot dir {}: {e}", actual_dir.display()));
        let _ = image.save(&actual_path);
        panic!(
            "snapshot_assert({name}): golden missing\n\
             tried backend override: {}\n\
             tried shared fallback:  {}\n\
             actual saved to:        {}\n\
             regenerate via: scripts/update-snapshots.sh --backend {backend_dir}",
            backend_path.display(),
            shared_path.display(),
            actual_path.display(),
        );
    };

    let golden = image::open(golden_path)
        .unwrap_or_else(|e| panic!("open golden {}: {e}", golden_path.display()))
        .into_rgba8();

    // Mismatch artifacts always land in the per-backend dir so it's
    // obvious which backend produced the diverging output, even when the
    // golden being compared against was the shared fallback.
    let backend_dir_path = root.join(&backend_dir);
    let actual_path = backend_dir_path.join(format!("{name}.actual.png"));
    let diff_path = backend_dir_path.join(format!("{name}.diff.png"));

    if golden.dimensions() != image.dimensions() {
        std::fs::create_dir_all(&backend_dir_path)
            .unwrap_or_else(|e| panic!("create snapshot dir {}: {e}", backend_dir_path.display()));
        let _ = image.save(&actual_path);
        panic!(
            "snapshot_assert({name}): dimension mismatch — golden {:?}, actual {:?}\n\
             golden:  {}\n\
             actual:  {}\n\
             regenerate via: scripts/update-snapshots.sh --backend {backend_dir}",
            golden.dimensions(),
            image.dimensions(),
            golden_path.display(),
            actual_path.display(),
        );
    }

    let report = pixel_diff(&golden, image);
    if report.mean > FLIP_TOLERANCE {
        std::fs::create_dir_all(&backend_dir_path)
            .unwrap_or_else(|e| panic!("create snapshot dir {}: {e}", backend_dir_path.display()));
        let _ = image.save(&actual_path);
        let _ = report.diff.save(&diff_path);
        panic!(
            "snapshot_assert({name}): FLIP mean {:.6} exceeds tolerance {FLIP_TOLERANCE}\n\
             golden:  {}\n\
             actual:  {}\n\
             diff:    {}\n\
             regenerate via: scripts/update-snapshots.sh --backend {backend_dir}",
            report.mean,
            golden_path.display(),
            actual_path.display(),
            diff_path.display(),
        );
    }
}

/// Convenience wrapper over [`snapshot_assert_at`] that resolves the
/// snapshot root to `quartzite-widgets/tests/snapshots`.
pub fn snapshot_assert(name: &str, image: &RgbaImage) {
    snapshot_assert_at(&default_snapshot_root(), name, image);
}

/// Renders `widget` into `harness` via [`DefaultStyle::draw_widget`] and
/// asserts against the committed golden for `name`.
///
/// The closure form of [`RenderHarness::render_widget`] is wrapped here
/// so the widget snapshot tests don't repeat the
/// `DefaultStyle.draw_widget(widget, p, &Palette::default())` call on
/// every line.
///
/// **Skip-env layering** (v1 call path, see `tests/snapshots.rs`):
/// each `#[test]` fn calls a `harness_or_skip` helper that checks
/// `SKIP_RENDER_SNAPSHOT=1` *before* constructing a [`RenderHarness`] —
/// so when the env is set, this fn is never reached. As a defence-in-depth
/// check, the inner [`snapshot_assert`] also early-returns on the same
/// env, which protects callers that skip the outer guard (e.g. tests
/// that opt to construct a harness for unrelated reasons and only later
/// decide to snapshot).
pub fn snapshot_widget(harness: &mut RenderHarness, name: &str, widget: &dyn AsWidget) {
    let image =
        harness.render_widget(|p| DefaultStyle::new().draw_widget(widget, p, &Palette::default()));
    snapshot_assert(name, &image);
}

/// Resolves the on-disk root for committed goldens — `<crate>/tests/snapshots`.
pub fn default_snapshot_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
}

/// Resolves the per-backend directory name from `WGPU_BACKEND`.
///
/// `vulkan` / `dx12` / `metal` map to themselves; anything else
/// (including unset) maps to [`DEFAULT_BACKEND_DIR`] (`auto`).
pub fn backend_dir_name() -> String {
    match std::env::var(BACKEND_ENV).as_deref() {
        Ok("vulkan") => "vulkan".to_owned(),
        Ok("dx12") => "dx12".to_owned(),
        Ok("metal") => "metal".to_owned(),
        _ => DEFAULT_BACKEND_DIR.to_owned(),
    }
}

/// Result of a single perceptual diff.
pub struct DiffReport {
    /// Mean per-pixel FLIP score across the image (0.0 = identical).
    pub mean: f32,
    /// LUT-coloured visualisation of the FLIP error map. Saved as the
    /// `*.diff.png` artifact on mismatch.
    pub diff: RgbImage,
}

/// Runs an `nv-flip` perceptual diff between `reference` and `actual`.
///
/// `nv-flip` operates on RGB8; the alpha channel of the input images is
/// dropped before comparison. The two images must have identical
/// dimensions (the caller checks this earlier and panics with a friendlier
/// message; this fn would also panic via `nv_flip::flip`).
fn pixel_diff(reference: &RgbaImage, actual: &RgbaImage) -> DiffReport {
    let width = reference.width();
    let height = reference.height();
    let ref_rgb = rgba_to_rgb8(reference);
    let act_rgb = rgba_to_rgb8(actual);
    let ref_flip = FlipImageRgb8::with_data(width, height, &ref_rgb);
    let act_flip = FlipImageRgb8::with_data(width, height, &act_rgb);
    let error_map = flip(ref_flip, act_flip, DEFAULT_PIXELS_PER_DEGREE);
    let pool = FlipPool::from_image(&error_map);
    let mean = pool.mean();
    let lut = magma_lut();
    let visualised = error_map.apply_color_lut(&lut);
    let diff = RgbImage::from_raw(width, height, visualised.to_vec()).unwrap_or_else(|| {
        // `apply_color_lut` always produces width*height*3 bytes, but
        // construct a plausible empty fallback rather than panicking
        // through `unwrap` — the caller's mismatch panic already covers
        // the failure path with full context.
        RgbImage::from_pixel(width, height, Rgb([0, 0, 0]))
    });
    DiffReport { mean, diff }
}

/// Drops the alpha channel of `img` into a tightly-packed RGB8 buffer.
fn rgba_to_rgb8(img: &RgbaImage) -> Vec<u8> {
    let mut out = Vec::with_capacity((img.width() * img.height() * 3) as usize);
    for px in img.pixels() {
        out.extend_from_slice(&px.0[..3]);
    }
    out
}
