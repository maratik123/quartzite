# Design: GPU snapshot tests in CI

**Issue:** #192
**Date:** 2026-05-10
**Spec:** [`2026-05-10-gpu-snapshot-tests-ci.spec.md`](2026-05-10-gpu-snapshot-tests-ci.spec.md)

## Approach

The plan ships four deliverables that share a single end-to-end pipeline
(`widget tree → RenderHarness → wgpu offscreen texture → readback →
perceptual diff → golden PNG on disk`) plus one orthogonal Linux-only
windowed smoke test. The investigation surfaced four shape-defining
constraints that drive the chosen approach:

1. **The renderer is a no-op skeleton today.** `VelloPainter::draw_*`
   methods are empty stubs and no widget overrides `WidgetExt::paint`
   (`quartzite-renderer/src/vello_painter.rs`,
   `quartzite-widgets/src/widget_ext.rs:319`). The harness must therefore
   produce a deterministic image regardless of widget content — concretely
   the wgpu clear colour. Goldens encode that state and get regenerated
   when real pixels land. This is *not* a future concern: the very first
   PR has to make a deliberate choice about what to clear to.
2. **`Application` is a process-singleton (`OnceLock`).** The existing
   `quartzite-renderer/tests/application.rs` documents this — only one
   `Application::new()` succeeds per process. The offscreen `RenderHarness`
   must therefore not construct an `Application`; otherwise five snapshot
   tests in one binary collide. The `xvfb_smoke.rs` integration test, in
   contrast, *does* boot the full `Application` + `EventLoop` and so must
   live in its own test binary (one fn per file).
3. **`Painter` is object-safe and pass-through (no internal state stack).**
   The 11-method trait in `quartzite-paint-api/src/painter.rs` takes pen
   and brush as call args. The harness can drive `WidgetExt::paint(&mut
   dyn Painter)` against a `VelloPainter` with zero changes to the paint
   vocabulary. No new trait surface needed.
4. **CI patterns are already standardised.** `.github/workflows/ci.yml`
   uses one shape across the four compile jobs: `dorny/paths-filter@v4`
   `changes` gate → `actions/checkout@v6` → `dtolnay/rust-toolchain@stable`
   → `mozilla-actions/sccache-action@v0.0.10` → `Swatinem/rust-cache@v2`
   with `shared-key: ${{ runner.os }}-stable` and
   `save-if: github.ref == 'refs/heads/master'` → `cargo …` → matching
   `<job>-pass` aggregator that treats `skipped` as success. The new
   `gpu-tests` job MUST adopt this exact shape, not invent a fifth.

### Crate-placement decisions

| Concern | Decision | Rationale |
|---|---|---|
| `RenderHarness` lives in | `quartzite-renderer/src/render_harness.rs`, `pub use` from `lib.rs` | Owns wgpu/vello/peniko already; harness is just an offscreen variant of the rendering path that `WindowedApplication` wires for windows. |
| Snapshot helper lives in | `quartzite-widgets/tests/support/mod.rs` (test-only sibling) | Widgets crate is the consumer (per the spec's "Where do snapshot tests live?"). A test-side helper avoids exporting test-only API from the widgets crate's library surface. |
| Snapshot-helper API surface | **Test-only**, NOT `pub` from `quartzite-widgets`'s library | Spec AC2 names the helper `quartzite_widgets::test_support::snapshot_assert` with an "e.g." softener. The design intentionally diverges from the literal path: a test-side helper at `tests/support/mod.rs` is invisible to downstream crates, keeps the widget library's public surface free of test-only items, and avoids needing a `test-support` cargo feature. The helper is reachable from sibling integration-test files (`tests/snapshots.rs`, `tests/support_internals.rs`) via `mod support;` and that is its only consumer. If a future need surfaces the helper to other crates' tests, the helper migrates into a dedicated `quartzite-test-support` dev-only crate — *not* into the widgets crate's public API. |
| Snapshot tests live in | `quartzite-widgets/tests/snapshots.rs` | Exactly as the spec mandates. |
| Goldens live under | `quartzite-widgets/tests/snapshots/<backend>/<test_name>.png` | Per-backend dirs keep PR diffs small when one backend regenerates. |
| `xvfb_smoke` test | `quartzite-renderer/tests/xvfb_smoke.rs` | Constructs `WindowedApplication`; needs its own process for the `Application` singleton. |
| `update-snapshots.sh` | `scripts/update-snapshots.sh` | Sibling of existing `gen-roadmap.sh`. |

### Why the harness owns wgpu directly (vs. wrapping `WindowedApplication`)

Rejected alternative: have `RenderHarness` re-use `WindowedApplication`'s
internals. That would couple offscreen testing to the windowed pipeline
and force the harness to drag in `winit` initialisation that headless CI
on macOS / Windows can't satisfy without a display.

Chosen alternative: `RenderHarness` calls `wgpu::Instance::new` directly,
requests an adapter via `pollster::block_on(instance.request_adapter(...))`,
creates a `wgpu::Texture` (RGBA8 unorm, `RENDER_ATTACHMENT | COPY_SRC`),
hands it to a `vello::Renderer`, then uses a `wgpu::Buffer` +
`buffer.map_async` for readback. The harness never touches `winit`.

### Why `nv-flip` over `image-compare`

The spec resolves this (Q4): `nv-flip` per upstream wgpu/vello use,
acknowledging the staleness deferred-item. The design adopts the resolved
choice unmodified. The fallback path lives at the helper-layer abstraction:
`fn pixel_diff(a: &RgbaImage, b: &RgbaImage) -> DiffReport` is a single
function call site, so swapping `nv-flip` for `image-compare` is a 10-line
change confined to `tests/support/mod.rs`.

### Backend-dir resolution at test time

The helper reads `WGPU_BACKEND` and maps it to a directory name:

| `WGPU_BACKEND` env | Directory | Notes |
|---|---|---|
| `vulkan` | `vulkan` | Linux lavapipe |
| `dx12` | `dx12` | Windows WARP |
| `metal` | `metal` | macOS native Metal |
| (unset / other) | `auto` | Local-dev catch-all; fallback per spec AC2 |

The helper also reads `WGPU_ADAPTER_NAME` for diagnostic output but does
NOT include it in the dir name (would explode test storage with one
golden per GPU vendor).

### `xvfb_smoke` test shape

The test must (a) construct `Application` + `EventLoop`, (b) render one
frame, (c) exit cleanly. Two complications drove the design:

- **Event-loop hangs.** `EventLoop::run_app` blocks until exit-requested;
  if the app never asks to exit, the test runs forever. Mitigation: a
  `winit::application::ApplicationHandler` impl that calls
  `event_loop.exit()` from inside `resumed` (the event fired immediately
  after the loop starts). This guarantees one frame's worth of redraw
  request followed by clean exit.
- **`xvfb-run` does not enforce a timeout on its inner process.** If the
  exit-on-resume logic ever regresses, the test would consume the full CI
  job quota. Mitigation: wrap the test command in `timeout 60` at the CI
  step level (`timeout 60 xvfb-run -a cargo test …`). 60 s is generous
  for one frame (typical run < 5 s) and well below GitHub's job timeout.

### Linux-driver decision tree

The spec's Q5 left a fallback chain for lavapipe; the design pins the
default and the fallbacks:

1. **Default:** `apt install mesa-vulkan-drivers` (already on
   `ubuntu-latest`'s repos). Ships lavapipe ~24.x as of 2026-05.
2. **Fallback 1 (compute-shader gap):** `kisak/kisak-mesa` PPA →
   `add-apt-repository ppa:kisak/kisak-mesa && apt install
   mesa-vulkan-drivers`. Single new step.
3. **Fallback 2 (PPA also lags):** `jakoch/install-vulkan-sdk-action`.
   Pulls upstream SDK; biggest install.

Implementation plan: ship default-only first; gate fallbacks behind a
follow-up if `vulkaninfo --summary` shows the default is too old.

### Aggregator pattern

A new `gpu-tests-pass` job mirrors the existing `*-pass` aggregators:

```yaml
gpu-tests-pass:
  name: GPU tests
  needs: [changes, gpu-tests]
  runs-on: ubuntu-latest
  if: always()
  steps:
    - run: |
        c="${{ needs.changes.result }}"
        r="${{ needs.gpu-tests.result }}"
        if [[ "$c" != "success" ]] || [[ "$r" != "success" && "$r" != "skipped" ]]; then exit 1; fi
```

Identical shape to `build-pass`, `test-pass`, `clippy-pass`. `skipped`
is treated as success so non-Rust PRs don't block on a job that didn't run.

### Decomposition philosophy

12 atomic subtasks. The spec explicitly asks for 8–14, and the four
deliverables decompose cleanly into:

- 1 — env diagnostic (lowest risk, lands first to surface CI driver
  issues before any code depends on them);
- 2–4 — `RenderHarness` infrastructure (deps → struct → public API);
- 5–6 — perceptual-diff helper (deps → helper);
- 7 — five widget snapshot tests + initial goldens;
- 8 — `update-snapshots.sh`;
- 9 — Linux `xvfb_smoke` test;
- 10 — CI `gpu-tests` job (offscreen lanes);
- 11 — CI Linux lane gains the `xvfb_smoke` step + `xvfb` package;
- 12 — `CONTRIBUTING.md` section.

Each subtask is ≤ ~150 lines of diff and reviewable in isolation.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Append dev-dependencies to `quartzite-widgets/Cargo.toml`: `nv-flip` 0.1, `image` 0.25, `tempfile` 3 (for the helper-internal unit tests' isolated file-IO — version pinned per AGENTS.md after live-registry query at implementation time). `pollster` is already present. Update `Cargo.lock` via `cargo build`. Verify `cargo build -p quartzite --no-default-features` still passes (deps are dev-only — should not affect the no-default-features path). | `quartzite-widgets/Cargo.toml`, `Cargo.lock` | — |
| 2 | Add `quartzite-renderer` to `quartzite-widgets/[dev-dependencies]`. Verify `quartzite-widgets/tests/no_style_dep.rs` still passes (renderer transitively pulls `quartzite-paint-api` only — no `quartzite-style` exposure). | `quartzite-widgets/Cargo.toml`, `Cargo.lock` | 1 |
| 3 | Implement `RenderHarness::new(width, height) -> Result<Self, RendererError>` in a new `quartzite-renderer/src/render_harness.rs`. Owns `wgpu::Instance` / `Adapter` / `Device` / `Queue` / offscreen `wgpu::Texture` (RGBA8 unorm, **`STORAGE_BINDING` + `COPY_SRC`** — vello 0.8 writes the target via a compute shader through `STORAGE_BINDING`, not a render-pass attachment; `COPY_SRC` is required for the readback path) / `vello::Renderer`. Constructor uses `pollster::block_on` internally so the test boundary is sync. Errors: adapter-request failure maps to `RendererError::Paint(PaintError::Other("adapter request failed"))` — `DeviceLost` is wrong because the device never existed at adapter-request time, and `SurfaceLost` is wrong because the harness never creates a `wgpu::Surface`. The `&'static str` literal is fixed at `"adapter request failed"` (stable wording, greppable in CI logs). | `quartzite-renderer/src/render_harness.rs`, `quartzite-renderer/src/lib.rs` | 1 |
| 4 | Implement `RenderHarness::render_widget<F: FnOnce(&mut dyn Painter)>(&mut self, paint: F) -> RgbaImage`. **Trait-bound finalisation (per spec AC1's "or equivalent" escape hatch):** the original design's `<W: WidgetExt + ?Sized>(&self, widget: &W)` bound is unreachable because `WidgetExt` lives in `quartzite-widgets` and `quartzite-widgets` is `quartzite-renderer`'s dev-dependency, not a regular dep — taking a `WidgetExt` bound directly would either need a regular dep cycle (forbidden) or move the harness out of the renderer crate (architecturally wrong, since the harness owns wgpu/vello). The closure form is fully equivalent: callers write `harness.render_widget(\|p\| widget.paint(p))`, the widget-specific shorthand is provided by the test-side helper (subtask 6) where both `RenderHarness` and `WidgetExt` are visible. **Mut receiver** because vello's `Renderer::render_to_texture` requires `&mut self`. Internals: reset the vello scene, call `paint(&mut VelloPainter::new())`, run `vello::Renderer::render_to_texture`, submit a copy-to-buffer + map_async readback (256-byte row alignment per `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`), decode RGBA8 row-padded bytes into `image::RgbaImage`. Add `#[cfg(test)]` smoke test asserting a 64×64 harness produces an all-`BASE_COLOR` image (gated behind `SKIP_RENDER_SNAPSHOT=1` to honour the workspace skip env). | `quartzite-renderer/src/render_harness.rs` | 3 |
| 5 | Public re-export: `pub use render_harness::RenderHarness;` in `quartzite-renderer/src/lib.rs`. Update crate `//!` doc to mention offscreen testing path. | `quartzite-renderer/src/lib.rs` | 4 |
| 6 | Implement perceptual-diff helper `quartzite-widgets/tests/support/mod.rs` with `fn snapshot_assert(name: &str, image: RgbaImage)`. Reads `WGPU_BACKEND` (or `auto`) → resolves golden path; reads `SKIP_RENDER_SNAPSHOT` → early-return with `eprintln!`; reads `QUARTZITE_REGENERATE_SNAPSHOTS` → write golden + return; otherwise: load golden via `image::open`, run `nv-flip` diff, on mismatch write `<name>.actual.png` + `<name>.diff.png` next to golden, panic with reviewer-friendly message including artifact paths. Single-number tolerance constant `FLIP_TOLERANCE: f32 = 0.05` documented inline. The helper-internal unit tests (skip / regen / missing-golden / match / mismatch) live in a new sibling top-level integration test `quartzite-widgets/tests/support_internals.rs` that `mod support;`-includes the helper — distinct from `snapshots.rs` so widget-snapshot failures and helper-unit failures stay distinguishable in CI output. Use `tempfile::TempDir` per scenario to isolate file-IO. | `quartzite-widgets/tests/support/mod.rs`, `quartzite-widgets/tests/support_internals.rs` | 1 |
| 7 | Five snapshot tests in `quartzite-widgets/tests/snapshots.rs` (one per primitive: `Label`, `Button`, `LineEdit`, `BoxLayout`, `GridLayout`). Each test: build widget, set canvas-sized geometry (64×64 or 128×128), construct `RenderHarness`, call `render_widget`, call `snapshot_assert`. **v1 goldens (Linux-only):** generate via `WGPU_BACKEND=vulkan QUARTZITE_REGENERATE_SNAPSHOTS=1 cargo test -p quartzite-widgets --test snapshots` on the local Linux dev box; commit `quartzite-widgets/tests/snapshots/vulkan/{label,button,line_edit,box_layout,grid_layout}.png` (5 PNGs). The `dx12/` and `metal/` subdirs are NOT pre-created; their goldens are bootstrapped in follow-up PRs (Windows/macOS contributors run the regen script on their platform, commit). Goldens are 64×64 or 128×128 PNG-compressed all-clear-colour images (a few hundred bytes each). | `quartzite-widgets/tests/snapshots.rs`, `quartzite-widgets/tests/snapshots/vulkan/*.png` | 2, 5, 6 |
| 8 | `scripts/update-snapshots.sh`. POSIX bash (per `gen-roadmap.sh` precedent — same constraints documented at the top of that file). Args: optional `--backend {vulkan,dx12,metal}` (default: detect from current platform via `uname` + `WGPU_BACKEND` env). Sets `QUARTZITE_REGENERATE_SNAPSHOTS=1` and runs `cargo test -p quartzite-widgets --test snapshots`. Does NOT run `xvfb_smoke` (no goldens to regenerate). Add executable bit. | `scripts/update-snapshots.sh` | 7 |
| 9 | `quartzite-renderer/tests/xvfb_smoke.rs`. `#[cfg(target_os = "linux")] #[test] fn xvfb_smoke()`. Honors `SKIP_RENDER_SNAPSHOT=1` (skip + pass). Constructs `WindowedApplication`, runs an `ApplicationHandler` that calls `event_loop.exit()` from `resumed`, exits with code 0. The non-Linux path is `#[cfg(not(target_os = "linux"))] #[test] fn xvfb_smoke_skipped() { /* compile-only stub */ }` so the test binary always exists in `cargo test --workspace` even on Windows / macOS. | `quartzite-renderer/tests/xvfb_smoke.rs` | 5 |
| 10 | New `gpu-tests` matrix job in `.github/workflows/ci.yml`. 3-OS matrix (`ubuntu-latest`, `windows-latest`, `macos-latest`); `fail-fast: false`. **v1 bootstrap policy:** Linux is the only required lane at PR merge time; Windows + macOS lanes are marked `continue-on-error: true` until their per-backend goldens are bootstrapped in follow-up PRs. The matrix uses an `include:` form with a `required` boolean per row (`true` for ubuntu, `false` for windows/macos) wired via `continue-on-error: ${{ !matrix.required }}`. This honours spec AC3 ("at least one backend at PR merge time") while keeping the matrix shape ready for later goldens. Steps mirror existing `test` job shape (`actions/checkout@v6`, `dtolnay/rust-toolchain@stable`, `mozilla-actions/sccache-action@v0.0.10`, `Swatinem/rust-cache@v2` with `shared-key: ${{ runner.os }}-stable-gpu` — distinct from `*-stable` to avoid cache thrash with the existing test job; `save-if` master-only). Per-OS env: Linux sets `WGPU_BACKEND=vulkan`, `WGPU_ADAPTER_NAME=llvmpipe`, `LIBGL_ALWAYS_SOFTWARE=1` and runs `apt install mesa-vulkan-drivers vulkan-tools xvfb`; Windows sets `WGPU_BACKEND=dx12`, `WGPU_ADAPTER_NAME=Microsoft`; macOS sets `WGPU_BACKEND=metal`. Linux runs `vulkaninfo --summary` as a diagnostic step. All runs execute `cargo test -p quartzite-widgets --test snapshots`. New `gpu-tests-pass` aggregator added per the existing `*-pass` shape (`needs: [changes, gpu-tests]`, `if: always()`, `success` or `skipped` passes). Existing `test` job env adds `SKIP_RENDER_SNAPSHOT: "1"`. `actionlint .github/workflows/ci.yml` MUST pass before commit (AGENTS.md axiom). | `.github/workflows/ci.yml` | 7 |
| 11 | Linux lane gains a step **after** the offscreen suite: `timeout 60 xvfb-run -a cargo test -p quartzite-renderer --test xvfb_smoke`. Same env as offscreen Linux step (vulkan/lavapipe). On failure, an `if: failure()` step uploads `/tmp/.X*.log` (xvfb log) for diagnostics. Artifact-upload step (`actions/upload-artifact@v7` — live latest verified 2026-05-10 via `gh api /repos/actions/upload-artifact/releases --jq '.[0].tag_name'`) runs on `if: failure()` and uploads `quartzite-widgets/tests/snapshots/**/*.actual.png` + `quartzite-widgets/tests/snapshots/**/*.diff.png`; this step is shared across all three OS lanes (one matrix-element per artifact name via `name: gpu-snapshot-failures-${{ runner.os }}`). | `.github/workflows/ci.yml` | 9, 10 |
| 12 | New `## GPU snapshot tests` section in `CONTRIBUTING.md`: how to run snapshots locally (`cargo test -p quartzite-widgets --test snapshots`), how to skip GPU work (`SKIP_RENDER_SNAPSHOT=1`), how to regenerate goldens (`scripts/update-snapshots.sh [--backend …]`), what to do when an intentional diff lands (review `*.actual.png` artifact → run regen → commit), and how the `xvfb_smoke` Linux test fits in. Cross-link `AGENTS.md § Build & Test`. | `CONTRIBUTING.md` | 8, 11 |

## Risks

- **Initial-golden bootstrapping problem.** Five tests need committed
  goldens before they can pass; goldens are created by running the tests
  with `QUARTZITE_REGENERATE_SNAPSHOTS=1`. **Mitigation:** subtask 7
  documents the bootstrap procedure (run regen script in each backend
  environment locally OR in a temporary CI workflow run with regen-mode
  env, download artifact, commit). Once the first PR lands, the regen
  loop is self-sustaining via `scripts/update-snapshots.sh`. The
  bootstrap risk is one-time.
- **Per-backend pixel drift.** vulkan/lavapipe vs. dx12/WARP vs.
  metal/native may emit non-identical clear-colour bytes (alpha
  pre-multiplication, RGBA vs. BGRA differences). **Mitigation:**
  per-backend golden dirs already isolate each lane; if drift exceeds
  the FLIP tolerance, the helper writes diff PNGs and fails loudly. The
  spec accepts this in *Deferred*: "Per-test perceptual tolerance
  tuning".
- **`nv-flip` blocking a future wgpu major bump.** Spec acknowledges
  this in *Deferred*. **Mitigation:** the helper isolates `nv-flip`
  behind one `pixel_diff` function; swap to `image-compare` is a 10-line
  change at one call site.
- **Driver-install drift on `ubuntu-latest`.** GitHub may upgrade the
  default mesa version. **Mitigation:** `vulkaninfo --summary`
  diagnostic step makes the version visible in CI logs. Driver-version
  drift will surface as a snapshot mismatch, not a silent failure.
- **`Application` singleton crosses test boundaries.** Multiple
  `tests/*.rs` files each get a fresh process and therefore a fresh
  `OnceLock`. The harness deliberately does NOT construct
  `Application`, so `quartzite-widgets/tests/snapshots.rs` is unaffected
  by this. The `xvfb_smoke.rs` file contains exactly one test fn — same
  reason. **Documented in the design** so future contributors don't add
  a second test fn to `xvfb_smoke.rs` and break it.
- **`xvfb-run` test consuming job quota on regression.** Mitigated by
  `timeout 60` wrapping the `xvfb-run` invocation in CI. 60 s is the
  hard limit; observed run time should be < 5 s.
- **Cache thrash from sharing `shared-key` with existing `test` job.**
  GPU lane uses a distinct `shared-key: ${{ runner.os }}-stable-gpu`
  per the existing `features` job's pattern.
- **No-default-features path regression** — `cargo build -p quartzite
  --no-default-features` is on the gate; subtasks 1 & 2 verify dev-deps
  don't leak into the main build.
- **AGENTS.md API-naming axiom.** `RenderHarness::new` is the
  unsuffixed constructor returning `Result<Self, RendererError>` — safe,
  non-panicking, returns `Result` on failure. No `_unchecked` variant
  needed (no UB-on-failure surface).

## Test Design

### `RenderHarness` (subtask 4)

- **Location:** `quartzite-renderer/src/render_harness.rs` `#[cfg(test)] mod tests`.
- **Entry points:** `RenderHarness::new`, `RenderHarness::render_widget`.
- **Scenarios:**
  - happy path: 64×64 harness → `RgbaImage` returned with width=64,
    height=64, all pixels equal the expected clear-colour bytes;
  - construction with zero dimensions → returns `Err` (wgpu rejects
    zero-extent textures);
  - rendering a `WidgetBase` (no paint override) → output equals
    rendering a separately-constructed `Label::new("test".into())`
    (both are no-op paints today; serves as a regression guard for
    "future widget paint code accidentally leaks across instances").
- **Fixtures:** none beyond `WidgetBase::new()`.

### Snapshot helper (subtask 6)

- **Location:** the helper itself lives in `quartzite-widgets/tests/support/mod.rs`.
  Cargo does **not** compile files under `tests/<dir>/` as standalone
  integration-test binaries — they are `mod`-included from a sibling
  top-level `tests/*.rs` file. The helper is therefore consumed by two
  sibling integration-test binaries:
  - `quartzite-widgets/tests/snapshots.rs` (subtask 7) — declares
    `mod support;` and calls `support::snapshot_assert(...)` from each
    widget snapshot test.
  - `quartzite-widgets/tests/support_internals.rs` (new in subtask 6)
    — declares `mod support;` and hosts the helper-internal unit
    tests (tolerance / regen-mode logic). Kept as a separate top-level
    file so failures here are immediately distinguishable from
    widget-snapshot failures in CI output.
- **Entry point:** `snapshot_assert(name, image)`.
- **Scenarios** (covered by `tests/support_internals.rs`):
  - `SKIP_RENDER_SNAPSHOT=1` → early return, no IO, no GPU;
  - `QUARTZITE_REGENERATE_SNAPSHOTS=1` → writes golden, returns Ok;
  - golden missing → panics with reviewer-friendly message that names
    the regen script;
  - golden present + match within tolerance → returns Ok;
  - golden present + mismatch → writes `*.actual.png` + `*.diff.png`,
    panics with paths in message.
- **Fixtures:** in-memory `RgbaImage::from_pixel(2, 2, …)` for unit
  tests; the widget snapshot tests in `snapshots.rs` exercise the file
  IO path end-to-end. Tolerance constant scenarios use injected
  `RgbaImage` with controlled deltas (single fully-opaque red vs.
  single one-shade-darker red) to assert the threshold logic. Each
  internal test runs against a unique `tempfile::TempDir` so file IO
  scenarios do not collide.

### Snapshot tests (subtask 7)

- **Location:** `quartzite-widgets/tests/snapshots.rs`.
- **Entry points:** five `#[test]` fns — `label_renders`, `button_renders`,
  `line_edit_renders`, `box_layout_renders`, `grid_layout_renders`.
- **Scenarios:** each builds the widget with deterministic content
  (e.g. `Label::new("test".into())`, fixed text), positions a 64×64 or
  128×128 canvas, calls the harness, calls `snapshot_assert`. No
  per-test branching; the helper handles skip / regen.
- **Fixtures:** the goldens themselves (15 PNGs).

### `xvfb_smoke` (subtask 9)

- **Location:** `quartzite-renderer/tests/xvfb_smoke.rs` (Linux-only
  test fn; non-Linux compile-only stub).
- **Entry point:** the test fn itself.
- **Scenarios:** clean startup + clean exit. No pixel comparison (per
  spec).
- **Fixtures:** `ApplicationHandler` impl that exits on `resumed`.
  Defined inline in the test file.

### Existing `quartzite-widgets/tests/no_style_dep.rs` (subtask 2)

- Re-runs unchanged after `quartzite-renderer` lands as a dev-dep.
  Test asserts `quartzite-style` is not in `cargo tree` for
  `quartzite-widgets`; `quartzite-renderer` does not depend on
  `quartzite-style` (verified via `Cargo.toml` — only `quartzite-paint-api`,
  `quartzite-runtime`, `quartzite-geometry`), so the assertion holds.
  Subtask 2 explicitly re-runs this test.

### CI gates (subtask 10/11)

- `actionlint .github/workflows/ci.yml` — CI gate per AGENTS.md axiom;
  runs locally before commit.
- `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`,
  `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps
  --workspace`, `cargo build -p quartzite --no-default-features` — all
  on the AC9 gate, verified locally before each push.

## Verified versions (queried 2026-05-10)

| Dep | Version pinned in design | Source |
|---|---|---|
| `nv-flip` | `0.1` (live `0.1.2`) | `crates.io/api/v1/crates/nv-flip` |
| `image` | `0.25` (live `0.25.10`) | `crates.io/api/v1/crates/image` |
| `image-compare` (fallback only) | `0.5` (live `0.5.0`) | `crates.io/api/v1/crates/image-compare` |
| `actions/checkout` | `@v6` (live `v6.0.2`; node24) | `gh api /repos/actions/checkout/...` |
| `actions/upload-artifact` | `@v7` (live `v7.0.1`, verified 2026-05-10) | `gh api /repos/actions/upload-artifact/releases --jq '.[0].tag_name'` |
| `dorny/paths-filter` | `@v4` (live `v4.0.1`) | `gh api /repos/dorny/paths-filter/...` |
| `Swatinem/rust-cache` | `@v2` (live `v2.9.1`) | `gh api /repos/Swatinem/rust-cache/...` |
| `mozilla-actions/sccache-action` | `@v0.0.10` (live `v0.0.10`) | `gh api /repos/mozilla-actions/sccache-action/...` |
| `dtolnay/rust-toolchain` | `@stable` (composite action; not version-tagged) | `gh api /repos/dtolnay/rust-toolchain/...` |

> Note on `actions/upload-artifact`: the spec's "verify live tag at design
> time" instruction was applied — `gh api /repos/actions/upload-artifact/releases`
> reports the live current major as `v7` (latest tag `v7.0.1`, verified
> 2026-05-10). Per AGENTS.md § *Dependency Versions* (live-current rule),
> the design pins `@v7` and the spec's reference to `@v5` was updated
> in lockstep on line 86. No follow-up bump is deferred.

## Open questions

1. **FLIP tolerance starting value.** Design picks `0.05` as the single
   workspace constant. With today's all-clear-colour goldens, any
   non-zero FLIP score is a real difference, so `0.0` would also work
   but admits no slack for backend rounding. Confirm `0.05` is
   acceptable as the v1 default; the spec's *Deferred* item handles
   later tuning.
2. **Backend-name fallback `auto` for unset `WGPU_BACKEND`.** Spec AC2
   says "falls back to `auto` when unset". Design uses literal `"auto"`
   as the dir name. Local-dev contributors then commit goldens under
   `tests/snapshots/auto/`. Confirm this is desired (vs. e.g.
   refusing-to-run when `WGPU_BACKEND` is unset, which would prevent
   accidental commits of non-CI goldens).
3. **Cache shared-key.** Design uses `${{ runner.os }}-stable-gpu` to
   avoid thrash with the existing `${{ runner.os }}-stable` cache used
   by `build` / `test` / `clippy` / `docs`. Alternative: re-use
   `${{ runner.os }}-stable` (faster cold cache, more thrash on
   master-branch saves). Confirm preference; the spec is silent.
4. **Goldens-on-disk contributor education.** The PNGs are tracked
   binary in git. CONTRIBUTING.md will spell out the regen workflow
   (subtask 12). Open question: should the repo gain a pre-commit
   hint (e.g. a comment in `update-snapshots.sh`) reminding
   contributors that *intentional* visual changes require a regen
   step? Spec is silent; design recommends adding the hint for
   discoverability but flags it as a soft preference.

### Resolved-by-design (no longer open)

- **Behaviour when `WGPU_BACKEND` is set but the matching golden dir
  does not exist.** Resolved by subtask 6: the helper falls through to
  the standard "golden missing" panic, whose reviewer-friendly message
  names `scripts/update-snapshots.sh` and the resolved backend
  directory. CI treats this as a hard failure (a backend the suite
  hasn't been bootstrapped for); local dev is told how to bootstrap
  via the same message. No special-cased behaviour beyond the panic
  text.

## Post-implementation refinements

These changes landed *after* the original Step-10 self-review APPROVE
(at commit `f033deb`) in response to actual CI failures, PR review
comments, and a coverage-gap signal. The decomposition rows above are
preserved for historical accuracy; this section is the canonical record
of how the as-built shape differs.

### Subtask 6 / 7 — shared-fallback golden layout

Original rows: 5 vulkan-only goldens under
`quartzite-widgets/tests/snapshots/vulkan/`; Windows + macOS lanes were
`continue-on-error: ${{ !matrix.required }}` until follow-up bootstrap
PRs landed.

As-built (commit `3739569`): goldens are looked up via "backend
override → shared default → fail":

- `quartzite-widgets/tests/snapshots/shared/<name>.png` — cross-backend
  default. While `VelloPainter` is no-op every backend writes
  byte-identical `Rgba8Unorm` clear-colour pixels, so a single shared
  golden covers Linux + Windows + macOS. The 5 v1 goldens live here.
- `quartzite-widgets/tests/snapshots/<backend>/<name>.png` — per-backend
  override. Used when a backend produces pixels that drift beyond
  `FLIP_TOLERANCE` from the shared default (typical once real
  rasterization lands).

Regeneration always writes to the per-backend dir matching the
`WGPU_BACKEND` env (so `update-snapshots.sh --backend vulkan` always
writes to `tests/snapshots/vulkan/`). Bootstrapping a fresh `shared/`
default is a manual step: regen on one backend, then `mv <backend>/* shared/`.

Two new internals tests cover the new paths
(`shared_fallback_used_when_backend_dir_empty`,
`backend_override_takes_precedence_over_shared`).

`continue-on-error` is dropped — all 3 OS lanes are required at
PR-merge time.

### Subtask 9 — `xvfb_smoke` worker-thread bypass

Original row: test calls `WindowedApplication::new()` and runs
`event_loop.exit()` from `resumed`.

As-built (commit `39e28f7`): `cargo test` runs every `#[test]` on a
worker thread; winit 0.30's default `EventLoop::new()` enforces a
main-thread check on Linux and panics otherwise. The test now
constructs `quartzite_runtime::Application` directly and builds the
`EventLoop` via
`EventLoop::builder()` + `EventLoopBuilderExtX11::with_any_thread(true)`
+ `EventLoopBuilderExtWayland::with_any_thread(true)`. Production
code (`WindowedApplication`) keeps the strict default; the bypass
stays scoped to the test. Behaviour is otherwise identical to the
original row.

### Subtask 10 — Linux apt set + Windows DX12 install dance

Original row: Linux apt installs `mesa-vulkan-drivers vulkan-tools xvfb`;
Windows lane uses the system DX12 / WARP runtime.

As-built:

- **Linux apt set** gains `libxkbcommon-x11-0` (commit `f78d660`). winit's
  X11 backend dlopens `libxkbcommon-x11.so` at runtime via `xkbcommon-dl`;
  the package is not preinstalled on `ubuntu-latest` and the `xvfb_smoke`
  test panics without it.
- **Windows DX12 install dance** (commits `05fa02c`, `ce8b251`,
  `3160f89`) — mirrors `gfx-rs/wgpu`'s own CI. After the rust-cache
  restore, the Windows lane downloads:
  - **WARP 1.0.19** (NuGet `Microsoft.Direct3D.WARP`) — `d3d10warp.dll`
    placed in `target/debug/` and `target/debug/deps/`.
  - **DXC v1.9.2602** (Microsoft DXC GitHub release `dxc_2026_02_20.zip`)
    — `dxc.exe` + `dxcompiler.dll` added to `PATH`.
  - **D3D12 Agility SDK 1.619.2** (NuGet `Microsoft.Direct3D.D3D12`) —
    binaries extracted to `target/agility-sdk/build/native/bin/x64`;
    `WGPU_DX12_AGILITY_SDK_PATH` / `_VERSION` / `_REQUIRE` env vars set.
  - `WGPU_DX12_COMPILER=dxc` env set.

  Uses `7z` for the NuGet `.nupkg` extracts (GNU tar on `windows-latest`
  rejects them as "not a tar archive"). Without the install dance the
  snapshot suite crashes with `STATUS_ACCESS_VIOLATION` on the first
  compute dispatch — vello requires modern D3D12 runtime features the
  bare `windows-latest` image does not ship by default. Drops the
  earlier `continue-on-error: ${{ matrix.os == 'windows-latest' }}`;
  Windows is now a required lane.

### Coverage workflow GPU install

Not in the original 12 subtasks. Codecov flagged 144 uncovered lines in
`render_harness.rs` because `coverage.yml` ran on a bare `ubuntu-latest`
without a Vulkan ICD, so the harness's adapter request failed and every
GPU code path short-circuited via "no GPU adapter; skipping" branches.

As-built (commit `e5d59bf`): `coverage.yml` mirrors the gpu-tests Linux
lane — apt-installs `mesa-vulkan-drivers vulkan-tools xvfb
libxkbcommon-x11-0`, sets `WGPU_BACKEND=vulkan` +
`WGPU_ADAPTER_NAME=llvmpipe` + `LIBGL_ALWAYS_SOFTWARE=1`, runs
`vulkaninfo --summary` for diagnostics, and wraps `cargo llvm-cov` in
`timeout 600 xvfb-run -a`. `render_harness.rs` patch coverage went from
20.87% → 90.65%; project coverage stayed essentially flat against the
master baseline.

### Documentation refinements

- Round-1 self-review (commit `f033deb`): `# Parameters` doc sections
  added on `RenderHarness::new` and `render_widget`; design subtask-3
  texture-usage row corrected to `STORAGE_BINDING | COPY_SRC`
  (vello's compute-shader path requires storage binding);
  `snapshot_widget` doc clarified to describe the two-layer skip.
- `CONTRIBUTING.md` § *GPU snapshot tests* (commits `cfcabc0`,
  `55cde9c`, `3739569`, `3160f89`) gained, beyond what subtask 12
  required: a Required-tooling table with per-distro install commands
  (Debian-Ubuntu / Fedora / Arch / Gentoo) and macOS / Windows
  no-install notes; a "Reproduce the CI Linux lane locally" recipe;
  the shared/<backend>/ goldens-layout explainer.
- `ai-docs/panic-index.md` (commit `39e28f7`) gained an entry for the
  `RenderHarness::render_widget` panic sites (vello render error, wgpu
  buffer mapping failure, channel send/recv failure, `RgbaImage::from_raw`
  returning `None`).
- `.gitignore` (commit `39e28f7`) gained `tests/snapshots/**/*.actual.png`
  + `*.diff.png` patterns so accidental `git add` cannot capture
  test-failure artifacts.
