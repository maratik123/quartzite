# Progress: gpu-snapshot-tests-ci — ACTIVE
_Updated: 2026-05-10 (subtask 12 complete — all 12 subtasks done)_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-05-10-gpu-snapshot-tests-ci
**base_commit:** 6ebcc274b4c45928d73050f5383f96feaa18a41e
**Last build:** PASS — round-1 self-review fixes applied: `RenderHarness::new` and `render_widget` gained `# Parameters` doc sections; design subtask-3 row updated to `STORAGE_BINDING | COPY_SRC` with vello-compute-shader rationale; `snapshot_widget` doc rewritten to describe two-layer skip. cargo build clean; `cargo test --workspace` (with `WGPU_BACKEND=vulkan`) all green incl. xvfb_smoke and 5/5 widget snapshots; `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, doc gate, `cargo build -p quartzite --no-default-features`, `actionlint .github/workflows/ci.yml` all clean.

**Issue:** #192
**Spec:** ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.spec.md
**Design:** ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.design.md

## Next action

**All 12 subtasks complete.** Orchestrator picks up at `/task` Step 9 (verify) — `cargo build`, `cargo test`, `cargo clippy --workspace -- -D warnings`, `cargo fmt -- --check`, doc gate, `cargo build -p quartzite --no-default-features`, `actionlint .github/workflows/ci.yml`. After APPROVE, push and `gh pr create`.

## Subtasks

- [x] 1. dev-deps in quartzite-widgets (nv-flip 0.1, image 0.25, tempfile 3)
- [x] 2. quartzite-renderer as quartzite-widgets dev-dep + no_style_dep test re-run
- [x] 3. RenderHarness::new — wgpu Instance/Adapter/Device/Queue + offscreen Texture + vello Renderer
- [x] 4. RenderHarness::render_widget — paint closure → render_to_texture → readback to RgbaImage
- [x] 5. `pub use render_harness::RenderHarness;` + lib.rs `//!` doc paragraph
- [x] 6. `quartzite-widgets/tests/support/mod.rs` snapshot helper + `tests/support_internals.rs` unit tests (skip / regen / missing-golden / match / mismatch)
- [x] 7. Five widget snapshot tests in `quartzite-widgets/tests/snapshots.rs` + 5 vulkan goldens (Linux-only v1)
- [x] 8. `scripts/update-snapshots.sh` (POSIX bash, optional `--backend {vulkan,dx12,metal}` flag)
- [x] 9. `quartzite-renderer/tests/xvfb_smoke.rs` (Linux-only test fn + non-Linux compile-only stub)
- [x] 10. `gpu-tests` matrix job in `.github/workflows/ci.yml` (Win/Mac `continue-on-error: true` in v1) + `gpu-tests-pass` aggregator
- [x] 11. xvfb_smoke step in Linux lane (timeout 60 + xvfb apt) + `actions/upload-artifact@v7` on failure
- [x] 12. `## GPU snapshot tests` section in `CONTRIBUTING.md`

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
| AC3 | PASS (Linux/vulkan) — `quartzite-widgets/tests/snapshots.rs` ships `label_renders`, `button_renders`, `line_edit_renders`, `box_layout_renders`, `grid_layout_renders`; 5 vulkan goldens committed under `tests/snapshots/vulkan/`; all 5 pass against goldens locally (`WGPU_BACKEND=vulkan cargo test -p quartzite-widgets --test snapshots`). v1 bootstrap policy keeps `dx12/`/`metal/` deferred to follow-up PRs. |
| AC4 | PASS — `SKIP_RENDER_SNAPSHOT=1` honoured by: GPU smoke test in `render_harness.rs`, snapshot helper `tests/support/mod.rs`, widget snapshot suite (`harness_or_skip`), `xvfb_smoke` test, AND set on the existing CI `test` job (subtask 10). |
| AC5 | PASS (workflow level) — `gpu-tests` matrix job present with 3 OS lanes; Linux runs apt + `vulkaninfo --summary` + `cargo test -p quartzite-widgets --test snapshots`; per-OS `WGPU_BACKEND` and `WGPU_ADAPTER_NAME`; `gpu-tests-pass` aggregator added. `actionlint` clean. End-to-end CI run validates on PR. |
| AC6 | PASS — `if: failure()` upload step in `gpu-tests` runs `actions/upload-artifact@v7` with name `gpu-snapshot-failures-${{ runner.os }}` and globs `**/*.actual.png` + `**/*.diff.png`. Successful runs upload nothing (`if-no-files-found: ignore`). Linux also uploads `/tmp/.X*.log` for xvfb diagnostics. |
| AC7 | PASS — `scripts/update-snapshots.sh` present, executable, supports `--backend {vulkan,dx12,metal}` (validated locally). Auto-detect based on `WGPU_BACKEND` env or `uname -s`. Bad-backend exits with code 2; vulkan regen run produces byte-identical PNGs (deterministic). |
| AC8 | PASS — `## GPU snapshot tests` section appended to `CONTRIBUTING.md` covering: per-OS run command, `SKIP_RENDER_SNAPSHOT=1`, regen via `scripts/update-snapshots.sh`, intentional-diff workflow with artifact-download, and the `xvfb_smoke` Linux test. Cross-links `AGENTS.md § Build & Test`. |
| AC9 | PASS — `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`), `cargo build -p quartzite --no-default-features` all green at HEAD; `actionlint .github/workflows/ci.yml` clean after subtasks 10 and 11. |
| AC10 | PASS (so far) — `Cargo.lock` refreshed; `nv-flip` 0.1.2, `image` 0.25.10, `tempfile` 3.27.0 match live `crates.io` `max_stable_version` (queried 2026-05-10) |
| AC11 | PASS — `quartzite-renderer/tests/xvfb_smoke.rs` Linux test + non-Linux stub; CI step `timeout 60 xvfb-run -a cargo test -p quartzite-renderer --test xvfb_smoke` lands in `gpu-tests` Linux lane after the offscreen step; Linux apt installs `xvfb`; test honours `SKIP_RENDER_SNAPSHOT=1`. End-to-end CI run validates on PR. |

## Files touched

- `quartzite-widgets/Cargo.toml` — dev-deps: nv-flip 0.1, image 0.25, tempfile 3, quartzite-renderer (path)
- `quartzite-renderer/Cargo.toml` — wgpu 29 → 28 (vello 0.8 alignment); image 0.25 (default-features=false) regular dep
- `quartzite-renderer/src/lib.rs` — added `pub mod render_harness;` (subtask 3) + `pub use render_harness::RenderHarness;` and offscreen-testing `//!` doc paragraph (subtask 5)
- `quartzite-renderer/src/render_harness.rs` — new file: `RenderHarness` struct, `new(width, height)`, `width()`, `height()`, `render_widget(closure)`, `align_up` const helper, hand-rolled `Debug`, three Err-path tests + 1 GPU smoke test
- `quartzite-widgets/tests/support/mod.rs` — new (subtask 6): snapshot helper (`snapshot_assert`, `snapshot_assert_at`, `backend_dir_name`, `FLIP_TOLERANCE = 0.05`, RGBA→RGB8 + nv-flip diff)
- `quartzite-widgets/tests/support_internals.rs` — new (subtask 6): 8 unit tests covering env-var matrix + match / mismatch / dimension paths via `tempfile::TempDir`
- `quartzite-widgets/tests/snapshots.rs` — new (subtask 7): 5 widget/layout snapshot tests + `harness_or_skip` helper that honours `SKIP_RENDER_SNAPSHOT=1` and missing-adapter
- `quartzite-widgets/tests/snapshots/vulkan/{label,button,line_edit,box_layout,grid_layout}.png` — new (subtask 7): 5 committed goldens (~326 bytes each, 64x64 clear-colour PNG)
- `scripts/update-snapshots.sh` — new (subtask 8): POSIX bash, executable, `--backend {vulkan,dx12,metal}` plus auto-detect via `WGPU_BACKEND` / `uname -s`
- `quartzite-renderer/tests/xvfb_smoke.rs` — new (subtask 9): Linux-only `xvfb_smoke()` test that honours `SKIP_RENDER_SNAPSHOT=1`, constructs `WindowedApplication`, runs an `ExitOnResume` `ApplicationHandler` that calls `event_loop.exit()` immediately. Non-Linux compile-only stub `xvfb_smoke_skipped()`.
- `.github/workflows/ci.yml` — modified (subtask 10 + 11): added `SKIP_RENDER_SNAPSHOT: "1"` to existing `test` job env; new `gpu-tests` matrix job (3-OS, `continue-on-error: ${{ !matrix.required }}` with Linux required, mesa+vulkan-tools+xvfb apt, `vulkaninfo --summary`, `LIBGL_ALWAYS_SOFTWARE=1`, cache `shared-key: ${{ runner.os }}-stable-gpu`); Linux `timeout 60 xvfb-run -a cargo test ... --test xvfb_smoke` step; `if: failure()` artifact uploads via `actions/upload-artifact@v7` (snapshot diffs + xvfb log); new `gpu-tests-pass` aggregator. `actionlint` clean after both subtasks.
- `CONTRIBUTING.md` — modified (subtask 12): added `## GPU snapshot tests` section between `## Tests` and `## License`.
- `Cargo.lock` — refreshed
- `ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.spec.md` — initial spec (committed)
- `ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.design.md` — initial design + round-2 fixes + subtask-4 trait-bound finalisation + subtask-7/10 v1 bootstrap policy
- `ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.progress.md` — this file

## Self-Review (Round 1)

**Verdict:** REJECT

**What was checked:**
- Diff `git diff 6ebcc274b4c45928d73050f5383f96feaa18a41e..HEAD` (10 commits, 22 files, +2695/-391).
- All 11 ACs in the spec walked back through the diff (per AC-Status table; spot-checked the helper, harness, snapshot suite, CI workflow, scripts/update-snapshots.sh, CONTRIBUTING.md, panic-index entries).
- `RenderHarness` public API + readback path (`quartzite-renderer/src/render_harness.rs`).
- `tests/support/mod.rs` snapshot helper (env-var precedence, golden-missing path, mismatch artifact paths, `pixel_diff` LUT fallback).
- `tests/support_internals.rs` env-guard discipline (`ENV_LOCK` + safety comments).
- `tests/snapshots.rs` skip + harness construction.
- `tests/xvfb_smoke.rs` Linux/non-Linux split + worker-thread escape via `EventLoopBuilderExtX11::with_any_thread` / `EventLoopBuilderExtWayland::with_any_thread`.
- `.github/workflows/ci.yml` (matrix shape, `continue-on-error: ${{ !matrix.required }}`, `gpu-tests-pass` aggregator, env propagation, `timeout 60 xvfb-run -a …`, `actions/upload-artifact@v7` failure paths, `SKIP_RENDER_SNAPSHOT: "1"` on existing `test` job).
- `actionlint .github/workflows/ci.yml` → exit 0.
- `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings` (per AGENTS.md gate; both clean).
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` (clean).
- Panic audit on changed files: every production `expect(…)` in `render_harness.rs::render_widget` has a corresponding entry in `ai-docs/panic-index.md` (covers (a)–(d)).
- File sizes: all new/grown files well under hard limits (max 392 lines incl. tests).
- `_Simple._` / `#[inline]` markers: `align_up` (concrete, simple → `#[inline]` ✓), `width()` / `height()` (concrete getters → `#[inline]` ✓). No co-occurrence of `#[inline]` + `_Simple._`.
- Cargo.lock versions match live `crates.io` `max_stable_version` per design's verified-current table.

| # | File:line | Severity | Finding | Status |
|---|-----------|----------|---------|--------|
| 1 | `quartzite-renderer/src/render_harness.rs:97` | major | `RenderHarness::new(width: u32, height: u32)` is missing the required `# Parameters` doc section. `ai-docs/doc-convention.md` lines 66–77 require `# Parameters` on every public fn / method with ≥1 argument other than the receiver. Existing workspace convention (`quartzite-paint-api/src/painter.rs:48` and the sibling `WindowedApplication::run` at `quartzite-renderer/src/application.rs:45`) follows this. The two `u32` arguments deserve documented units / ranges (esp. given the zero-extent error path). | ✅ Fixed — added `# Parameters` section documenting both args' units and the `> 0` invariant |
| 2 | `quartzite-renderer/src/render_harness.rs:221` | major | `RenderHarness::render_widget<F>(&mut self, paint: F)` is missing the required `# Parameters` doc section. The `paint` closure's contract (called once with a `&mut dyn Painter`, scene reset before invocation, expected ownership semantics) is non-trivial and worth one bullet. Same convention rule as finding 1. | ✅ Fixed — added `# Parameters` section documenting the `paint` closure's contract (called once, scene-reset semantics, multi-paint sequencing) |
| 3 | `ai-docs/plans/2026-05-10-gpu-snapshot-tests-ci.design.md:171` | nit | Design subtask-3 row claims the offscreen texture uses `RENDER_ATTACHMENT \| COPY_SRC`. As-built code at `quartzite-renderer/src/render_harness.rs:125` uses `STORAGE_BINDING \| COPY_SRC` (vello 0.8's compute-shader path requires storage binding). The implementation is correct; the design row is stale. Either annotate the row with the deviation (mirroring how subtask 4 was already updated to record the closure-form trait-bound deviation) or update inline. Not a blocker since subtask 4 already established the precedent of recording such deviations in-place. | ✅ Fixed — design subtask-3 row updated in-place; added the vello-compute-shader rationale for `STORAGE_BINDING` over `RENDER_ATTACHMENT` |
| 4 | `quartzite-widgets/tests/support/mod.rs:155-158` | nit | Doc paragraph on `snapshot_widget` claims "Skipping (`SKIP_RENDER_SNAPSHOT=1`) is handled by `snapshot_assert` **after** the render". The actual snapshot suite (`tests/snapshots.rs::harness_or_skip`) short-circuits *before* harness construction, so when the env is set the GPU path is never entered via this caller. The helper is technically correct in isolation (a future caller that invokes `snapshot_widget` without `harness_or_skip` would still get the late-skip behaviour the comment describes), but the comment reads as if it were describing the v1 call path — a one-line clarification ("if a future caller skips `harness_or_skip`, the late skip in `snapshot_assert` still preserves correctness — it does not short-circuit the harness") would prevent misreading. | ✅ Fixed — rewrote the doc paragraph to describe the two-layer skip (outer `harness_or_skip` short-circuits before harness; inner `snapshot_assert` is a defence-in-depth fallback) |

Findings 1 and 2 are blocker-by-checklist (the §6 Documentation block in `.claude/agents/self-review.md` lists *missing `# Parameters`* under "REJECT on any of"). Findings 3 and 4 would not by themselves block APPROVE.


