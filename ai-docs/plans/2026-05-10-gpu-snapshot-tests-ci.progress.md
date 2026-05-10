# Progress: gpu-snapshot-tests-ci — ACTIVE
_Updated: 2026-05-10 (subtask 6 complete)_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-05-10-gpu-snapshot-tests-ci
**base_commit:** 6ebcc274b4c45928d73050f5383f96feaa18a41e
**Last build:** PASS (cargo build clean; `cargo test -p quartzite-widgets --test support_internals` 8/8; `cargo fmt --check` clean; `cargo clippy --workspace -- -D warnings` clean; per-crate `cargo clippy -p {quartzite-widgets,quartzite-renderer} --tests -- -D warnings` clean; doc gate clean workspace-wide; `cargo build -p quartzite --no-default-features` clean)

**Issue:** #192
**Spec:** ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.spec.md
**Design:** ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.design.md

## Next action

**Do this immediately:** subtask 7 — write 5 widget snapshot tests in `quartzite-widgets/tests/snapshots.rs` (`label_renders`, `button_renders`, `line_edit_renders`, `box_layout_renders`, `grid_layout_renders`) using `RenderHarness` + `support::snapshot_assert`. Bootstrap goldens via `WGPU_BACKEND=vulkan QUARTZITE_REGENERATE_SNAPSHOTS=1 cargo test -p quartzite-widgets --test snapshots`; commit `tests/snapshots/vulkan/{label,button,line_edit,box_layout,grid_layout}.png` (5 PNGs). Do NOT create `dx12/` or `metal/` directories.

## Subtasks

- [x] 1. dev-deps in quartzite-widgets (nv-flip 0.1, image 0.25, tempfile 3)
- [x] 2. quartzite-renderer as quartzite-widgets dev-dep + no_style_dep test re-run
- [x] 3. RenderHarness::new — wgpu Instance/Adapter/Device/Queue + offscreen Texture + vello Renderer
- [x] 4. RenderHarness::render_widget — paint closure → render_to_texture → readback to RgbaImage
- [x] 5. `pub use render_harness::RenderHarness;` + lib.rs `//!` doc paragraph
- [x] 6. `quartzite-widgets/tests/support/mod.rs` snapshot helper + `tests/support_internals.rs` unit tests (skip / regen / missing-golden / match / mismatch)
- [ ] 7. Five widget snapshot tests in `quartzite-widgets/tests/snapshots.rs` + 5 vulkan goldens (Linux-only v1)  ← CURRENT
- [ ] 8. `scripts/update-snapshots.sh` (POSIX bash, optional `--backend {vulkan,dx12,metal}` flag)
- [ ] 9. `quartzite-renderer/tests/xvfb_smoke.rs` (Linux-only test fn + non-Linux compile-only stub)
- [ ] 10. `gpu-tests` matrix job in `.github/workflows/ci.yml` (Win/Mac `continue-on-error: true` in v1) + `gpu-tests-pass` aggregator
- [ ] 11. xvfb_smoke step in Linux lane (timeout 60 + xvfb apt) + `actions/upload-artifact@v7` on failure
- [ ] 12. `## GPU snapshot tests` section in `CONTRIBUTING.md`

## Key discoveries (don't re-investigate)

- **wgpu version pin: 28, NOT 29.** vello 0.8 transitively requires wgpu 28. The previous `wgpu = "29"` in `quartzite-renderer/Cargo.toml` was a latent bug that surfaced as soon as the harness passed `wgpu::Texture` etc. into `vello::Renderer::render_to_texture`. Aligned to `wgpu = "28"`. Do **not** bump wgpu independently of vello.
- **`image` is now a regular dep of `quartzite-renderer`** (`default-features = false`). Required because `RenderHarness::render_widget` returns `image::RgbaImage` as part of the public API per spec AC1.
- **`render_widget` takes a closure, not a `WidgetExt` bound.** `WidgetExt` lives in `quartzite-widgets` (renderer's dev-dep) — taking a generic bound on it would close a regular dep cycle. Per spec AC1 "(or equivalent — design phase finalises the trait bound)" the closure form is acceptable. Design subtask 4 row already updated.
- **`render_widget` takes `&mut self`** (not `&self` as the original design row said). vello's `Renderer::render_to_texture` requires `&mut self`. Already reflected in the implementation.
- **Adapter-failure error variant is `PaintError::Other("adapter request failed")`** (`&'static str`). Round-1 design-review fix: `DeviceLost` was wrong (no device yet at adapter-request time); `SurfaceLost` is wrong (no surface ever).
- **GPU smoke test honours `SKIP_RENDER_SNAPSHOT=1`** (`std::env::var_os` check). Existing `cargo test --workspace` job in CI must set this env (subtask 10 wires it).
- **`RenderHarness` cannot derive `Debug`** because `vello::Renderer` does not impl `Debug`. Hand-rolled `Debug` impl present (`width`, `height`, `..`).
- **v1 bootstrap policy (per user decision):** Linux-only goldens at PR-merge time. Windows/macOS matrix lanes get `continue-on-error: true` and goldens bootstrapped in follow-up PRs. Subtask 7 commits 5 PNGs under `tests/snapshots/vulkan/`; `dx12/`, `metal/` dirs are NOT created by this PR. Spec AC3 ("at least one backend") covers this.
- **Cache `shared-key`:** `${{ runner.os }}-stable-gpu` (distinct from existing `*-stable`) to avoid thrash. Subtask 10.
- **FLIP tolerance default:** `0.05` as a single workspace constant in the helper. Subtask 6.
- **Backend-name fallback:** literal `"auto"` directory name when `WGPU_BACKEND` is unset. Subtask 6.
- **Bootstrap recipe (when running subtask 7):** `WGPU_BACKEND=vulkan QUARTZITE_REGENERATE_SNAPSHOTS=1 cargo test -p quartzite-widgets --test snapshots` once subtasks 6+7 are in place. Local Vulkan/RADV adapter verified working.
- **Live-verified action versions** (queried 2026-05-10): `actions/checkout@v6` (live `v6.0.2`), `actions/upload-artifact@v7` (live `v7.0.1`), `dorny/paths-filter@v4` (live `v4.0.1`), `Swatinem/rust-cache@v2` (live `v2.9.1`), `mozilla-actions/sccache-action@v0.0.10`. Use these in subtasks 10 / 11.

## AC Status

| AC | Status |
|----|--------|
| AC1 | PASS — `RenderHarness::new(width, height) -> Result<Self, RendererError>` + `render_widget` (closure form) implemented; AC1 explicitly allows trait-bound finalisation |
| AC2 | PASS (helper layer) — `tests/support/mod.rs` ships `snapshot_assert` (+ `snapshot_assert_at` for tempdir-driven internals tests). 8/8 internals tests pass: backend-dir mapping, skip env, regen env, missing golden, match, mismatch (writes `*.actual.png` + `*.diff.png`), dimension mismatch. End-to-end exercise with the harness + real goldens lands in subtask 7. |
| AC3 | NOT_TESTED — pending subtask 7 (5 widget snapshot tests + vulkan goldens) |
| AC4 | NOT_TESTED — `SKIP_RENDER_SNAPSHOT=1` honoured by GPU smoke test today; full coverage pending subtasks 6 (helper) + 9 (xvfb smoke) + 10 (CI env) |
| AC5 | NOT_TESTED — pending subtask 10 (`gpu-tests` matrix job) |
| AC6 | NOT_TESTED — pending subtask 11 (artifact upload on failure) |
| AC7 | NOT_TESTED — pending subtask 8 (`scripts/update-snapshots.sh`) |
| AC8 | NOT_TESTED — pending subtask 12 (`CONTRIBUTING.md`) |
| AC9 | PASS (so far) — `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, doc gate, `cargo build -p quartzite --no-default-features` all green at HEAD; `actionlint` not yet exercised (no workflow file modified yet — fires in subtasks 10+11) |
| AC10 | PASS (so far) — `Cargo.lock` refreshed; `nv-flip` 0.1.2, `image` 0.25.10, `tempfile` 3.27.0 match live `crates.io` `max_stable_version` (queried 2026-05-10) |
| AC11 | NOT_TESTED — pending subtasks 9 (test) + 11 (CI step) |

## Files touched

- `quartzite-widgets/Cargo.toml` — dev-deps: nv-flip 0.1, image 0.25, tempfile 3, quartzite-renderer (path)
- `quartzite-renderer/Cargo.toml` — wgpu 29 → 28 (vello 0.8 alignment); image 0.25 (default-features=false) regular dep
- `quartzite-renderer/src/lib.rs` — added `pub mod render_harness;` (subtask 3) + `pub use render_harness::RenderHarness;` and offscreen-testing `//!` doc paragraph (subtask 5)
- `quartzite-renderer/src/render_harness.rs` — new file: `RenderHarness` struct, `new(width, height)`, `width()`, `height()`, `render_widget(closure)`, `align_up` const helper, hand-rolled `Debug`, three Err-path tests + 1 GPU smoke test
- `quartzite-widgets/tests/support/mod.rs` — new (subtask 6): snapshot helper (`snapshot_assert`, `snapshot_assert_at`, `backend_dir_name`, `FLIP_TOLERANCE = 0.05`, RGBA→RGB8 + nv-flip diff)
- `quartzite-widgets/tests/support_internals.rs` — new (subtask 6): 8 unit tests covering env-var matrix + match / mismatch / dimension paths via `tempfile::TempDir`
- `Cargo.lock` — refreshed
- `ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.spec.md` — initial spec (committed)
- `ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.design.md` — initial design + round-2 fixes + subtask-4 trait-bound finalisation + subtask-7/10 v1 bootstrap policy
- `ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.progress.md` — this file
