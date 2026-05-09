# Progress — gpu-snapshot-tests-ci

**Branch:** feat/2026-05-10-gpu-snapshot-tests-ci
**base_commit:** 6ebcc274b4c45928d73050f5383f96feaa18a41e
**Issue:** #192
**Spec:** ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.spec.md
**Design:** ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.design.md
**Last build:** 2026-05-10 — cargo build green; `cargo test -p quartzite-renderer --lib` 11/11 green incl. GPU smoke test on Vulkan/RADV; clippy --workspace clean; cargo fmt clean; doc gate clean; cargo build -p quartzite --no-default-features clean

## Subtasks

| # | Title | Status |
|---|-------|--------|
| 1 | dev-deps: nv-flip 0.1, image 0.25, tempfile 3 in quartzite-widgets | ✅ done |
| 2 | Add quartzite-renderer as quartzite-widgets dev-dep; verify no_style_dep test | ✅ done |
| 3 | RenderHarness::new — wgpu Instance/Adapter/Device/Queue + offscreen Texture + vello Renderer | ✅ done |
| 4 | RenderHarness::render_widget — paint, render_to_texture, readback to RgbaImage | ✅ done |
| 5 | Public re-export pub use render_harness::RenderHarness; lib.rs doc | ⬜ pending |
| 6 | Snapshot helper at tests/support/mod.rs + tests/support_internals.rs unit tests | ⬜ pending |
| 7 | Five widget snapshot tests + 5 vulkan goldens (Linux-only v1) | ⬜ pending |
| 8 | scripts/update-snapshots.sh (POSIX bash, --backend flag) | ⬜ pending |
| 9 | quartzite-renderer/tests/xvfb_smoke.rs (Linux test + non-Linux stub) | ⬜ pending |
| 10 | gpu-tests CI matrix job (Win/Mac continue-on-error in v1) + gpu-tests-pass aggregator | ⬜ pending |
| 11 | xvfb_smoke step in Linux lane + xvfb apt + actions/upload-artifact@v7 on failure | ⬜ pending |
| 12 | CONTRIBUTING.md GPU snapshot tests section | ⬜ pending |

## Files touched

- `quartzite-widgets/Cargo.toml` — dev-deps: nv-flip 0.1, image 0.25, tempfile 3, quartzite-renderer (path)
- `quartzite-renderer/Cargo.toml` — wgpu pinned 29 → **28** to match vello 0.8 (vello pulls wgpu 28 transitively, version-mismatch otherwise prevents passing wgpu types into vello APIs); added `image` 0.25 (default-features = false) as regular dep (RgbaImage is part of public API)
- `quartzite-renderer/src/render_harness.rs` — new file: RenderHarness struct, new(width, height), width(), height(), render_widget(closure), align_up helper, three Err-path tests + GPU smoke test
- `Cargo.lock` — refreshed via cargo build
- `ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.design.md` — subtask 4 trait-bound refinement (closure form per AC1 escape hatch); subtask 7 / subtask 10 v1-bootstrap policy (Linux-only goldens; Win/Mac continue-on-error)

## Notes

- v1 bootstrap policy: **Linux-only goldens at PR merge time**; Windows/macOS lanes
  are `continue-on-error: true` and bootstrapped in follow-up PRs (per user
  decision on the bootstrap question).
- Cache `shared-key: ${{ runner.os }}-stable-gpu` (distinct from existing
  `*-stable`) to avoid thrash.

## Next action

Subtask 5 — public re-export of `RenderHarness` from `quartzite-renderer/src/lib.rs` (`pub use render_harness::RenderHarness;`) and update crate `//!` doc to mention the offscreen testing path. Note: `pub mod render_harness;` is **already** in lib.rs (added during subtask 3); this subtask only adds the convenience re-export and the doc paragraph.

## Notes for a fresh-agent handoff

- **Branch:** `feat/2026-05-10-gpu-snapshot-tests-ci` (already checked out).
- **Pre-commit state (NOT yet committed):** subtasks 1–4 are complete in working tree but not yet on the branch. Recommended: run a single commit covering subtasks 1–4 before continuing (one logical unit — adding the dev-deps without the harness, or the harness without the deps, is a broken intermediate state). See **Recommended commit** below.
- **wgpu version pin:** `quartzite-renderer/Cargo.toml` was downgraded `wgpu = "29"` → `wgpu = "28"` because vello 0.8 transitively requires wgpu 28 and the harness passes `wgpu::Texture` etc. into `vello::Renderer::render_to_texture`. **Do not bump wgpu independently of vello** — they must match. If a vello upgrade lands in the future, the wgpu pin moves with it.
- **`render_widget` deviation:** the harness takes `FnOnce(&mut dyn Painter)` not `<W: WidgetExt>`, because `WidgetExt` lives in `quartzite-widgets` (which is renderer's dev-dep). Spec AC1's escape hatch covers this; the design (subtask 4 row) is updated. The widget-specific helper in subtask 6 (`tests/support/mod.rs`) gives callers the ergonomic `render_widget(harness, &widget)` form.
- **v1 bootstrap policy:** Linux-only goldens are committed at PR-merge time. Windows/macOS lanes in the matrix get `continue-on-error: true` until their goldens are bootstrapped in follow-up PRs (per user decision). Subtask 7 commits 5 PNGs under `tests/snapshots/vulkan/`; `dx12/` and `metal/` dirs are NOT created by this PR.
- **GPU smoke test:** `render_widget_no_op_produces_clear_color_image` in `render_harness.rs` honours `SKIP_RENDER_SNAPSHOT=1` (skip + pass) and gracefully reports if no GPU is available. Subtask 10 sets `SKIP_RENDER_SNAPSHOT: "1"` on the existing `test` workflow lane so this test only runs in the new `gpu-tests` lane.
- **Per-subtask gates** to run after each remaining subtask: `cargo build && cargo test -p <crate> [filter] && cargo fmt -- --check && cargo clippy --workspace -- -D warnings`. Doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`) before final verify.
- **Bootstrap recipe (subtask 7):** to generate the 5 vulkan goldens locally, run `WGPU_BACKEND=vulkan QUARTZITE_REGENERATE_SNAPSHOTS=1 cargo test -p quartzite-widgets --test snapshots` once the helper (subtask 6) and tests (subtask 7) are in place. The current dev box has a working Vulkan/RADV adapter (verified by the smoke test passing).
- **Subtask 5 is small** — likely combine with subtask 6 into one commit. Subtasks 7+ are larger and should be one commit each.

## Recommended commit (subtasks 1–4 bundle)

```
feat(renderer): RenderHarness — offscreen wgpu/vello readback for snapshot tests (#192)

- quartzite-widgets dev-deps: nv-flip 0.1, image 0.25, tempfile 3, quartzite-renderer (path)
- quartzite-renderer: wgpu 29 → 28 (align with vello 0.8); add image 0.25 default-features=false
- quartzite-renderer: new `pub mod render_harness` with `RenderHarness::new(w, h)`,
  `render_widget(|p| ...)` taking a closure (deviation from design's WidgetExt bound, per
  AC1 escape hatch — WidgetExt lives in renderer's dev-dep crate)
- 4 new unit tests incl. GPU smoke test gated on SKIP_RENDER_SNAPSHOT
```
