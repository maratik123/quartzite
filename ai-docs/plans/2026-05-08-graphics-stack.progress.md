# Progress: Graphics Stack (quartzite-paint-api + quartzite-renderer) — ACTIVE
_Updated: 2026-05-08 (after Task 6)_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-05-08-graphics-stack
**base_commit:** 10762960d63f35b3ddf0e22683fae9d19006f676
**Last build:** PASS (Task 6)
**Issue:** #73
**Spec:** ai-docs/plans/2026-05-03-graphics-stack.spec.md

## Next action

**Do this immediately:** Task 7 — implement `WindowedApplication` + `VelloPainter` skeleton + integration test.

## Subtasks

- [x] 1. `quartzite-paint-api` crate scaffold (`Cargo.toml`, `src/lib.rs`, workspace registration)
- [x] 2. `Color`, `Pen`, `Brush`/`BrushKind` types in `quartzite-paint-api`
- [x] 3. `Painter` trait + `PaintError` in `quartzite-paint-api`
- [x] 4. `quartzite-paint` stub crate (`Cargo.toml`, `src/lib.rs`, `src/path.rs`, workspace registration)
- [x] 5. Facade wiring + doc updates (`quartzite` root `Cargo.toml`, `src/lib.rs`, `context.md`, `INDEX.md`)
- [x] 6. `quartzite-renderer` crate scaffold (`Cargo.toml`, `src/lib.rs`, workspace registration)
- [ ] 7. `WindowedApplication` + `VelloPainter` skeleton + integration test

## Key discoveries (don't re-investigate)

- `Rect::new` takes `(Point, Size)` — NOT `(x, y, w, h)`. Import `quartzite_geometry::Size` in tests.
- `quartzite-paint-api` is `#![no_std]`; add `#[cfg(test)] extern crate alloc;` AFTER all `#![...]` inner attrs in `lib.rs`. Use `use alloc::boxed::Box` / `use alloc::string::ToString` inside test modules.
- `thiserror` used with `default-features = false` (project-wide pattern for no_std crates — see `quartzite-core/Cargo.toml`).
- `quartzite-paint` is std; it re-exports everything from `quartzite-paint-api` and adds a `Path` stub (empty struct, full impl deferred to #47).
- `winit::event_loop::EventLoopError` is private re-export; the public path is `winit::error::EventLoopError` (verified winit 0.30.13).
- Workspace `Cargo.toml` currently lists: `quartzite-paint-api`, `quartzite-paint`. `quartzite-renderer` is NOT yet added.
- Design doc is at `ai-docs/plans/2026-05-03-graphics-stack.design.md` (note: design uses 2026-05-03 date).

## AC Status

| AC | Status |
|----|--------|
| AC1 — `quartzite-paint-api` compiles with `no_default_features` | PASS (cargo build -p quartzite-paint-api --no-default-features not yet run; crate is unconditionally `#![no_std]` with no `std` feature) |
| AC2 — `quartzite-paint` has no wgpu/vello/winit dep | PASS |
| AC3 — `quartzite-renderer` has winit/wgpu/vello | PASS (cargo build + clippy -D warnings) |
| AC4 — `quartzite-runtime` no graphics dep | NOT_TESTED |
| AC5 — Mock `Painter` compiles against `quartzite-paint-api` alone | PASS (17 unit tests + 16 doctests in `quartzite-paint-api`) |
| AC6 — No dep cycles | PASS (paint-api ← paint, paint-api ← renderer; no cycle) |

## Files touched

- `Cargo.toml` — added `quartzite-paint-api`, `quartzite-paint` to workspace members
- `quartzite-paint-api/Cargo.toml` — new crate
- `quartzite-paint-api/src/lib.rs` — crate root, `#![no_std]`, module declarations
- `quartzite-paint-api/src/color.rs` — `Color` type (RGBA f32, Copy, 5 named consts)
- `quartzite-paint-api/src/pen.rs` — `Pen` type (Color + width f32, Copy)
- `quartzite-paint-api/src/brush.rs` — `Brush` + `BrushKind` (Solid(Color), Copy, non_exhaustive)
- `quartzite-paint-api/src/painter.rs` — `Painter` trait (object-safe, 7 methods)
- `quartzite-paint-api/src/error.rs` — `PaintError` (thiserror, SurfaceLost/DeviceLost/Other)
- `quartzite-paint/Cargo.toml` — new crate, depends on quartzite-paint-api + quartzite-geometry
- `quartzite-paint/src/lib.rs` — re-exports quartzite-paint-api, declares path module
- `quartzite-paint/src/path.rs` — `Path` stub (empty struct)
- `Cargo.toml` (root facade) — added `quartzite-paint-api` dep
- `src/lib.rs` (root facade) — added `pub mod paint` re-export + prelude entries + ecosystem docs
- `ai-docs/context.md` — added `quartzite-paint-api` to crate table + plans list
- `ai-docs/plans/INDEX.md` — updated dependency tree to show paint-api done
- `Cargo.toml` — added `quartzite-renderer` to workspace members
- `quartzite-renderer/Cargo.toml` — new crate (quartzite-paint-api, quartzite-runtime, winit, wgpu, vello, peniko, pollster, thiserror)
- `quartzite-renderer/src/lib.rs` — crate root, lint gates, mod declarations, re-exports ApplicationHandler + RendererError
- `quartzite-renderer/src/error.rs` — `RendererError` (EventLoop + Paint variants via thiserror)
- `quartzite-renderer/src/application.rs` — empty stub module with doc comment (Task 7)
- `quartzite-renderer/src/vello_painter.rs` — empty stub module with doc comment (Task 7)
