//! Helper-internal unit tests for the snapshot helper.
//!
//! Lives in a separate top-level integration-test file (distinct from
//! `tests/snapshots.rs`) so widget-snapshot failures and helper-unit
//! failures stay distinguishable in CI output.
//!
//! Each scenario uses its own `tempfile::TempDir` for file IO isolation
//! and toggles env vars under a process-global mutex (env-var mutation
//! affects the whole process).

mod support;

use std::sync::Mutex;

use image::{Rgba, RgbaImage};
use support::{
    BACKEND_ENV, DEFAULT_BACKEND_DIR, FLIP_TOLERANCE, REGEN_ENV, SHARED_DIR_NAME, SKIP_ENV,
    backend_dir_name, snapshot_assert_at,
};

/// Process-global mutex serialising env-var mutation across these tests.
/// Cargo's default test runner is multi-threaded; without this, one
/// test's `set_var` would race another's `var_os`.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Saves and restores the four env vars the helper reads. Construction
/// clears all four; `Drop` restores the originally-observed values so a
/// later test on the same thread doesn't inherit our state.
struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    skip: Option<std::ffi::OsString>,
    regen: Option<std::ffi::OsString>,
    backend: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn new() -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let skip = std::env::var_os(SKIP_ENV);
        let regen = std::env::var_os(REGEN_ENV);
        let backend = std::env::var_os(BACKEND_ENV);
        // SAFETY: env-var mutation is process-global; `ENV_LOCK` ensures
        // no other helper-internals test runs concurrently. Other test
        // binaries get their own process and are unaffected.
        unsafe {
            std::env::remove_var(SKIP_ENV);
            std::env::remove_var(REGEN_ENV);
            std::env::remove_var(BACKEND_ENV);
        }
        Self {
            _lock: lock,
            skip,
            regen,
            backend,
        }
    }

    #[allow(
        clippy::unused_self,
        reason = "Tying `set` to `&self` ensures callers hold a live `EnvGuard` (and thus `ENV_LOCK`) during env-var mutation; an associated fn would weaken that invariant"
    )]
    fn set(&self, key: &str, value: &str) {
        // SAFETY: see `new` — ENV_LOCK serialises mutation.
        unsafe {
            std::env::set_var(key, value);
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: see `new` — ENV_LOCK serialises mutation.
        unsafe {
            restore(SKIP_ENV, self.skip.take());
            restore(REGEN_ENV, self.regen.take());
            restore(BACKEND_ENV, self.backend.take());
        }
    }
}

unsafe fn restore(key: &str, value: Option<std::ffi::OsString>) {
    // SAFETY: caller holds `ENV_LOCK` via `EnvGuard`; mutation is
    // process-global but serialised across helper-internals tests.
    unsafe {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}

fn solid(width: u32, height: u32, rgba: [u8; 4]) -> RgbaImage {
    RgbaImage::from_pixel(width, height, Rgba(rgba))
}

#[test]
fn backend_dir_name_maps_known_values() {
    let env = EnvGuard::new();
    env.set(BACKEND_ENV, "vulkan");
    assert_eq!(backend_dir_name(), "vulkan");
    env.set(BACKEND_ENV, "dx12");
    assert_eq!(backend_dir_name(), "dx12");
    env.set(BACKEND_ENV, "metal");
    assert_eq!(backend_dir_name(), "metal");
}

#[test]
fn backend_dir_name_falls_back_to_auto() {
    let env = EnvGuard::new();
    // unset → auto
    assert_eq!(backend_dir_name(), DEFAULT_BACKEND_DIR);
    // unknown → auto
    env.set(BACKEND_ENV, "gl");
    assert_eq!(backend_dir_name(), DEFAULT_BACKEND_DIR);
}

#[test]
fn skip_env_returns_without_io() {
    let env = EnvGuard::new();
    env.set(SKIP_ENV, "1");
    let tmp = tempfile::tempdir().unwrap();
    let img = solid(2, 2, [10, 20, 30, 255]);
    // No golden anywhere — would normally panic. Skip env must short-circuit.
    snapshot_assert_at(tmp.path(), "skip_test", &img);
    // Confirm the helper did NOT touch the directory.
    let backend_dir = tmp.path().join(DEFAULT_BACKEND_DIR);
    assert!(
        !backend_dir.exists(),
        "skip should not create backend dir; found {}",
        backend_dir.display()
    );
}

#[test]
fn regen_env_writes_golden() {
    let env = EnvGuard::new();
    env.set(REGEN_ENV, "1");
    env.set(BACKEND_ENV, "vulkan");
    let tmp = tempfile::tempdir().unwrap();
    let img = solid(3, 3, [11, 22, 33, 255]);
    snapshot_assert_at(tmp.path(), "regen_test", &img);
    let golden = tmp.path().join("vulkan").join("regen_test.png");
    assert!(
        golden.exists(),
        "golden not written at {}",
        golden.display()
    );
    // Round-trip: written PNG decodes back to identical pixels.
    let decoded = image::open(&golden).unwrap().into_rgba8();
    assert_eq!(decoded.dimensions(), (3, 3));
    assert_eq!(decoded.get_pixel(0, 0).0, [11, 22, 33, 255]);
}

#[test]
fn missing_golden_panics_with_helpful_message() {
    let env = EnvGuard::new();
    env.set(BACKEND_ENV, "vulkan");
    let tmp = tempfile::tempdir().unwrap();
    let img = solid(2, 2, [0, 0, 0, 255]);
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        snapshot_assert_at(tmp.path(), "missing_test", &img);
    }))
    .unwrap_err();
    let msg = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&'static str>().copied())
        .unwrap_or("");
    assert!(msg.contains("golden missing"), "msg was: {msg}");
    // Both lookup paths are listed in the panic message.
    assert!(
        msg.contains("backend override:"),
        "backend hint missing: {msg}"
    );
    assert!(
        msg.contains("shared fallback:"),
        "shared hint missing: {msg}"
    );
    assert!(
        msg.contains("update-snapshots.sh"),
        "regen hint missing: {msg}"
    );
    // Actual artifact written for reviewer convenience.
    let actual = tmp.path().join("vulkan").join("missing_test.actual.png");
    assert!(
        actual.exists(),
        "actual.png not written at {}",
        actual.display()
    );
}

#[test]
fn shared_fallback_used_when_backend_dir_empty() {
    use std::fs;
    let env = EnvGuard::new();
    env.set(BACKEND_ENV, "vulkan");
    let tmp = tempfile::tempdir().unwrap();
    // Manually pre-populate shared/ as if the contributor had bootstrapped it.
    let shared_dir = tmp.path().join(SHARED_DIR_NAME);
    fs::create_dir_all(&shared_dir).unwrap();
    let img = solid(4, 4, [42, 42, 42, 255]);
    img.save(shared_dir.join("fallback_test.png")).unwrap();
    // No vulkan/<name>.png — the helper should fall back to shared/ and pass.
    snapshot_assert_at(tmp.path(), "fallback_test", &img);
    // Confirm no per-backend artifacts were written (no mismatch happened).
    let actual = tmp.path().join("vulkan").join("fallback_test.actual.png");
    assert!(
        !actual.exists(),
        "actual.png should not appear on a successful shared-fallback match"
    );
}

#[test]
fn backend_override_takes_precedence_over_shared() {
    use std::fs;
    let env = EnvGuard::new();
    env.set(BACKEND_ENV, "vulkan");
    let tmp = tempfile::tempdir().unwrap();
    // Shared has solid black; backend override has solid white.
    let shared_dir = tmp.path().join(SHARED_DIR_NAME);
    let backend_dir = tmp.path().join("vulkan");
    fs::create_dir_all(&shared_dir).unwrap();
    fs::create_dir_all(&backend_dir).unwrap();
    solid(4, 4, [0, 0, 0, 255])
        .save(shared_dir.join("override_test.png"))
        .unwrap();
    solid(4, 4, [255, 255, 255, 255])
        .save(backend_dir.join("override_test.png"))
        .unwrap();
    // Pass solid white. Comparing against the shared (black) golden would
    // explode FLIP_TOLERANCE; comparing against the backend (white)
    // override matches. A successful run proves precedence.
    let actual = solid(4, 4, [255, 255, 255, 255]);
    snapshot_assert_at(tmp.path(), "override_test", &actual);
    let actual_artifact = backend_dir.join("override_test.actual.png");
    assert!(
        !actual_artifact.exists(),
        "no mismatch artifact expected on override match"
    );
}

#[test]
fn matching_golden_passes() {
    let env = EnvGuard::new();
    env.set(BACKEND_ENV, "vulkan");
    let tmp = tempfile::tempdir().unwrap();
    let img = solid(4, 4, [200, 100, 50, 255]);
    // First write the golden via regen mode, then compare.
    env.set(REGEN_ENV, "1");
    snapshot_assert_at(tmp.path(), "match_test", &img);
    // SAFETY: ENV_LOCK held via env guard.
    unsafe {
        std::env::remove_var(REGEN_ENV);
    }
    snapshot_assert_at(tmp.path(), "match_test", &img);
    // No artifact files should appear next to the golden.
    let actual = tmp.path().join("vulkan").join("match_test.actual.png");
    let diff = tmp.path().join("vulkan").join("match_test.diff.png");
    assert!(!actual.exists(), "actual.png should not exist on match");
    assert!(!diff.exists(), "diff.png should not exist on match");
}

#[test]
fn mismatching_golden_writes_artifacts_and_panics() {
    let env = EnvGuard::new();
    env.set(BACKEND_ENV, "vulkan");
    let tmp = tempfile::tempdir().unwrap();
    // Golden is solid black, actual is solid white — mean FLIP score must
    // exceed FLIP_TOLERANCE (0.05) by a wide margin.
    let golden_img = solid(8, 8, [0, 0, 0, 255]);
    let actual_img = solid(8, 8, [255, 255, 255, 255]);
    env.set(REGEN_ENV, "1");
    snapshot_assert_at(tmp.path(), "mismatch_test", &golden_img);
    // SAFETY: ENV_LOCK held via env guard.
    unsafe {
        std::env::remove_var(REGEN_ENV);
    }
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        snapshot_assert_at(tmp.path(), "mismatch_test", &actual_img);
    }))
    .unwrap_err();
    let msg = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&'static str>().copied())
        .unwrap_or("");
    assert!(msg.contains("FLIP mean"), "msg was: {msg}");
    assert!(
        msg.contains(&format!("tolerance {FLIP_TOLERANCE}")),
        "tolerance hint missing: {msg}"
    );
    let actual = tmp.path().join("vulkan").join("mismatch_test.actual.png");
    let diff = tmp.path().join("vulkan").join("mismatch_test.diff.png");
    assert!(
        actual.exists(),
        "actual.png not written at {}",
        actual.display()
    );
    assert!(diff.exists(), "diff.png not written at {}", diff.display());
}

#[test]
fn dimension_mismatch_panics_and_writes_actual() {
    let env = EnvGuard::new();
    env.set(BACKEND_ENV, "vulkan");
    let tmp = tempfile::tempdir().unwrap();
    let golden_img = solid(4, 4, [0, 0, 0, 255]);
    let actual_img = solid(8, 8, [0, 0, 0, 255]);
    env.set(REGEN_ENV, "1");
    snapshot_assert_at(tmp.path(), "dim_test", &golden_img);
    // SAFETY: ENV_LOCK held via env guard.
    unsafe {
        std::env::remove_var(REGEN_ENV);
    }
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        snapshot_assert_at(tmp.path(), "dim_test", &actual_img);
    }))
    .unwrap_err();
    let msg = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&'static str>().copied())
        .unwrap_or("");
    assert!(msg.contains("dimension mismatch"), "msg was: {msg}");
    let actual = tmp.path().join("vulkan").join("dim_test.actual.png");
    assert!(actual.exists());
}
