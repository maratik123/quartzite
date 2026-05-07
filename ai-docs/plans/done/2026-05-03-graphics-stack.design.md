# Design: Graphics Stack — quartzite-paint-api + quartzite-renderer

**Issue:** #73
**Date:** 2026-05-08

## Approach

### Resolved open questions

**1. EventLoop integration — separate entry point in `quartzite-renderer`**

winit 0.30 uses an `ApplicationHandler` callback model and must own the main thread
for its event loop (`EventLoop::run_app`). The existing `quartzite-runtime::EventLoop`
is a headless `mpsc`-based loop that is orthogonal in purpose. Introducing a shared
`EventDriver` trait in `quartzite-runtime` would require that crate to know about
winit concepts or to carry a lowest-common-denominator abstraction that weakens both
implementations. Instead, `quartzite-renderer` exposes a standalone `run_windowed`
free function (or `WindowedApplication::run`) that drives `winit::event_loop::EventLoop`
directly. The headless `quartzite-runtime::EventLoop` is unchanged; `quartzite-runtime`
does not gain any winit dep (preserving AC4).

**2. `quartzite-renderer` surface — `WindowedApplication` newtype wrapper**

`WindowedApplication` is a newtype that wraps `quartzite_runtime::Application` and
owns a `winit::event_loop::EventLoop<()>`. It is constructed via
`WindowedApplication::new() -> Result<Self, ApplicationError>`, which calls
`Application::new()` internally. The `run` method on `WindowedApplication` takes an
`impl ApplicationHandler` (winit trait) and drives the winit loop. This keeps
`Application` in `quartzite-runtime` unchanged and provides a clear, discoverable
API surface. An extension trait was rejected: it would scatter construction and
ownership of the winit loop into a separate impl block, making it unclear who owns
`EventLoop<()>`.

**3. `quartzite-paint-api` — dedicated crate**

A module approach (`quartzite-paint::api`) would require `quartzite-renderer` to
depend on `quartzite-paint`, but `quartzite-paint` should in turn depend on
`quartzite-paint-api` — this creates a cycle. A separate thin crate removes the
cycle entirely, satisfies AC6, and keeps the `no_std` boundary explicit in
`Cargo.toml`. The workspace churn is limited to one `Cargo.toml` entry.

### Architecture overview

```
quartzite-geometry  (no_std, existing)
       │
       ▼
quartzite-paint-api (no_std, new — Color, Painter trait, basic paint types)
       │
       ├──► quartzite-paint   (std, new/adjusted — higher-level abstractions; no renderer dep)
       │
       └──► quartzite-renderer (std, new — winit + wgpu + vello; VelloPainter implements Painter)
                  │
                  └──► quartzite-runtime (std, existing — unchanged; no graphics dep)
```

Dependency graph for the new crates:

| Crate | Deps (direct) |
|---|---|
| `quartzite-paint-api` | `quartzite-geometry` (no_std path; `default-features = false`) |
| `quartzite-paint` | `quartzite-paint-api`, `quartzite-geometry` |
| `quartzite-renderer` | `quartzite-paint-api`, `quartzite-runtime`, `winit 0.30`, `wgpu 29`, `vello 0.8` |

### `Painter` trait design (object-safety constraints)

All methods take `&mut self`. No generic parameters on methods (generic methods are
not object-safe). Associated types are allowed only if they carry a `where Self: Sized`
bound or are absent. The trait exposes:

```rust
pub trait Painter {
    fn draw_rect(&mut self, rect: Rect, pen: &Pen, brush: &Brush);
    fn fill_rect(&mut self, rect: Rect, brush: &Brush);
    fn draw_line(&mut self, from: Point, to: Point, pen: &Pen);
    fn clip_rect(&mut self, rect: Rect);
    fn translate(&mut self, delta: Point);
    fn save(&mut self);
    fn restore(&mut self);
}
```

`draw_image` and `draw_text_in` are deferred (image/font types not yet defined);
they will be added in subsequent plans without breaking the trait (new methods with
provided default impls or behind a feature).

### `quartzite-paint-api` types

Thin, `no_std`-compatible types only; no alloc dependency unless unavoidable:

- `Color` — RGBA `f32` components; derives `Copy`, `Clone`, `Debug`, `PartialEq`
- `Pen` — stroke: `Color` + line width (`f32`); `Copy`
- `Brush` — fill: `BrushKind` enum (`Solid(Color)` for v1; gradient variants later)
- `Painter` trait (object-safe, as above)
- `PaintError` — `thiserror`-derived error type for renderer failures (used by
  `quartzite-renderer`, re-exported for callers). Rendering errors in v1 are
  non-recoverable; `VelloPainter` panics or logs on failure; `PaintError` is
  reserved for a future API version when `Painter` methods gain `Result` return
  types.

`Brush` contains `BrushKind` which is an enum, so `Brush` itself is `Copy`
in v1 (only `Solid` variant). This avoids any alloc dependency.

### `quartzite-paint` (stub crate, v1 scope)

`quartzite-paint` was listed in the spec as "existing, adjusted". Since it does not
exist on disk yet, it is created as a new crate. v1 scope is minimal: re-exports
from `quartzite-paint-api` plus a `Path` type stub (empty for now). Full
implementation is deferred to the paint-style plan (#47).

### `quartzite-renderer` v1 scope

`WindowedApplication` struct:
- Owns `quartzite_runtime::Application`
- Owns `winit::event_loop::EventLoop<()>`
- `new() -> Result<Self, ApplicationError>` — constructs both
- `run(self, app: impl winit::application::ApplicationHandler) -> Result<(), RendererError>`
  — drives winit loop; `RendererError` wraps `winit::event_loop::EventLoopError`

`VelloPainter` struct (implements `Painter`):
- Wraps `vello::Scene` + `wgpu::Device` / `wgpu::Queue` / `wgpu::Surface`
- `VelloPainter::new(window: &winit::window::Window, ...)` — async initialiser;
  wrapped by a sync `block_on` in `WindowedApplication::run`
- Implements `Painter` by translating calls to `vello::Scene` draw commands using
  `peniko` color types and `kurbo` geometry types internally

The `VelloPainter` constructor uses `wgpu` async APIs. The renderer crate pulls in
`pollster` (tiny `block_on` executor) to avoid an async runtime dep.

### Workspace wiring

`quartzite` facade:
- Adds `quartzite-paint-api` as a dep (no_std-gated, no default-features)
- Does NOT add `quartzite-renderer` — renderer is a leaf crate; apps depend on it
  directly; the facade re-exports only headless/core items

Root `Cargo.toml` workspace `members` gains: `quartzite-paint-api`,
`quartzite-paint`, `quartzite-renderer`.

### `no_std` verification

`quartzite-paint-api` carries `#![no_std]` unconditionally (no `std` feature),
matching `quartzite-geometry`. CI gate: `cargo build -p quartzite-paint-api
--no-default-features` (already required by AC1; the existing CI job
`cargo build -p quartzite --no-default-features` does not cover the new crate —
a dedicated step must be added).

### Versions (verified 2026-05-08)

- `winit = "0.30"` (latest stable 0.30.13)
- `wgpu = "29"` (latest stable 29.0.3)
- `vello = "0.8"` (latest stable 0.8.0)
- `peniko = "0.6"` (latest stable 0.6.0 — vello re-exports; also direct dep for color types)
- `pollster = "0.4"` (tiny block_on; latest stable 0.4.0 verified 2026-05-08)

### Rejected alternatives

- **Shared `EventDriver` trait in `quartzite-runtime`**: would couple runtime to
  windowing concepts, violating AC4. Rejected.
- **`quartzite-paint::api` module with `pub use`**: creates a dependency cycle
  (`quartzite-renderer → quartzite-paint → quartzite-paint-api`, but also
  `quartzite-renderer` needs to reference the Painter trait directly without going
  through `quartzite-paint`). Rejected.
- **`winit::application::ApplicationHandler` re-exported by `quartzite-renderer`**:
  considered, but users must implement it themselves; re-exporting avoids version
  mismatch when they already depend on winit. Will re-export.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `quartzite-paint-api` crate scaffold: `Cargo.toml`, `src/lib.rs` with `#![no_std]`, workspace registration | `quartzite-paint-api/Cargo.toml`, `quartzite-paint-api/src/lib.rs`, root `Cargo.toml` | — |
| 2 | Implement `Color`, `Pen`, `Brush`/`BrushKind` types in `quartzite-paint-api` | `quartzite-paint-api/src/color.rs`, `quartzite-paint-api/src/pen.rs`, `quartzite-paint-api/src/brush.rs` | 1 |
| 3 | Define object-safe `Painter` trait and `PaintError` in `quartzite-paint-api` | `quartzite-paint-api/src/painter.rs`, `quartzite-paint-api/src/error.rs` | 2 |
| 4 | Create `quartzite-paint` crate stub: `Cargo.toml`, `src/lib.rs`, re-exports from `quartzite-paint-api`, `Path` stub | `quartzite-paint/Cargo.toml`, `quartzite-paint/src/lib.rs`, `quartzite-paint/src/path.rs`, root `Cargo.toml` | 3 |
| 5 | Add `quartzite-paint-api` dep to the `quartzite` facade and expose in prelude; update `context.md` + `INDEX.md` | root `Cargo.toml`, `src/lib.rs`, `ai-docs/context.md`, `ai-docs/plans/INDEX.md` | 3 |
| 6 | Create `quartzite-renderer` crate scaffold: `Cargo.toml` (winit, wgpu, vello, pollster, quartzite-paint-api, quartzite-runtime deps), `src/lib.rs` | `quartzite-renderer/Cargo.toml`, `quartzite-renderer/src/lib.rs`, root `Cargo.toml` | 3 |
| 7 | Implement `WindowedApplication` struct and `VelloPainter` skeleton in `quartzite-renderer` | `quartzite-renderer/src/application.rs`, `quartzite-renderer/src/vello_painter.rs`, `quartzite-renderer/tests/application.rs` | 6 |

Seven tasks exactly at the limit. If the `VelloPainter` implementation grows
substantially beyond a skeleton, split task 7 into: 7a `WindowedApplication` struct
+ 7b `VelloPainter` skeleton. That makes 8 tasks; propose splitting into two issues
(crate scaffolding + windowed application surface vs. vello painter implementation)
only if the reviewer deems the full `VelloPainter` (with real wgpu surface setup)
too large for one PR.

## Risks

- **winit main-thread requirement:** winit 0.30 `EventLoop::run_app` must be called
  on the main thread on macOS and some platforms. `WindowedApplication::run` must
  document this constraint and callers must not call it from a spawned thread.
  Mitigation: document `# Panics` / platform note; test on CI with
  `cargo test --no-run` (no GPU needed).
- **wgpu async surface creation:** `wgpu::Instance::create_surface` + adapter
  request are async. Using `pollster::block_on` in the sync `WindowedApplication::run`
  introduces a nested-executor risk if a caller already runs an async runtime.
  Mitigation: document that `run` must not be called from inside an async context;
  long-term tracked by #89 (async executor strategy).
- **vello `wgpu` version lock:** vello 0.8 pins a specific wgpu major; both must be
  at the same major. Both are currently at compatible versions (vello 0.8 / wgpu 29).
  Mitigation: verify at implementation time with `cargo tree`; lock versions together
  in `Cargo.toml`.
- **`no_std` regression in `quartzite-paint-api`:** accidental `std` import (e.g.
  `std::fmt::Display`) breaks the guarantee silently if not gated. Mitigation:
  `#![no_std]` crate-level attribute plus a dedicated CI step
  `cargo build -p quartzite-paint-api --no-default-features`.
- **`Painter` object-safety:** adding a generic method or `-> Self` return later
  would break `Box<dyn Painter>`. Mitigation: object-safety is verified by AC5
  (mock Painter compiles); the test explicitly constructs `Box<dyn Painter>`.
- **`quartzite-renderer` headless CI:** wgpu requires a GPU or a software rasteriser.
  CI jobs without a GPU must skip `VelloPainter` integration tests. Mitigation:
  use `#[cfg(test)]` integration tests that compile but are marked `#[ignore]` unless
  a `RUN_GPU_TESTS` env var is set; the mock-Painter tests in `quartzite-paint-api`
  are fully headless.
- **API surface of `quartzite-paint` stays minimal for v1:** the paint-style plan
  (#47) will add `Font`, `Image`, `Path` impl, `Style`, etc. The stub created in
  task 4 must not lock in types that conflict with that plan. Mitigation: task 4
  scope is strictly re-exports + empty `Path` struct; no method bodies.

## Test Design

**Task 2 — Color, Pen, Brush types (`quartzite-paint-api`)**
- Location: `quartzite-paint-api/src/color.rs`, `pen.rs`, `brush.rs` — `#[cfg(test)]` modules
- Entry points: constructors, `Default` impls, field accessors
- Scenarios:
  - `Color::new(r, g, b, a)` round-trips field values
  - `Color::BLACK`, `Color::WHITE` named constructors have correct components
  - `Pen::new(color, width)` stores width and color
  - `BrushKind::Solid(Color::RED)` is `Copy` (compile-time check via `let _ = brush;` twice)
- Fixtures: none needed

**Task 3 — `Painter` object-safety + `PaintError` (`quartzite-paint-api`)**
- Location: `quartzite-paint-api/src/painter.rs` — `#[cfg(test)]` module
- Entry point: `Box<dyn Painter>` construction (AC5 / object-safety)
- Scenarios:
  - `MockPainter` struct implementing `Painter`; all methods record calls in a `Vec<String>`
  - Construct `Box<dyn Painter>` from `MockPainter` — must compile
  - Call each method through the trait object; assert calls recorded
  - `PaintError` implements `std::error::Error` (std path only)
- Fixtures: `MockPainter` defined in the test module

**Task 4 — `quartzite-paint` stub**
- Location: `quartzite-paint/src/lib.rs` — `#[cfg(test)]` module
- Entry point: re-exports are accessible; `Path` struct is constructible
- Scenarios: `use quartzite_paint::Color; let _ = Color::BLACK;` — compile test only
- Fixtures: none

**Task 7 — `WindowedApplication` (`quartzite-renderer`)**
- Location (unit): `quartzite-renderer/src/application.rs` — `#[cfg(test)]` module
- Location (integration): `quartzite-renderer/tests/application.rs` — separate
  integration test binary (matches the pattern in `quartzite-runtime/tests/application.rs`;
  each `tests/*.rs` file is a fresh process, giving a clean `OnceLock` for singleton tests)
- Entry point: `WindowedApplication::new`
- Scenarios:
  - `WindowedApplication::new()` returns `Ok` on first call — compile + unit test
    (no GPU; does not call `run`)
  - Calling `WindowedApplication::new()` after an `Application` is already live
    returns `Err(ApplicationError::AlreadyExists)` — integration test binary in
    `quartzite-renderer/tests/application.rs`; all singleton assertions in one
    `#[test]` fn to guarantee a single `OnceLock` lifecycle per process
- Note: `run()` and `VelloPainter` require GPU; tested manually or via
  `#[ignore]`-gated integration tests

## Open questions

- Should `quartzite-renderer` re-export `winit::application::ApplicationHandler` for
  callers who have no direct `winit` dep? (Convenience re-export vs. letting callers
  control their own `winit` version.) Defer to implementation; document the choice in
  `quartzite-renderer/src/lib.rs`.
- The CI `no_std` gate (`cargo build -p quartzite-paint-api --no-default-features`)
  must be added to `.github/workflows/ci.yml`. Confirm with the user whether this
  should be a new step in the existing `test` job or a separate job, and whether
  macOS/Windows runners should also run it (currently the `no_std` path check only
  runs on Linux).
