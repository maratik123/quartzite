# Graphics Stack Selection

**Source:** user description
**Date:** 2026-05-03
**Tracked in:** #73

## Scope

Decide and document the graphics stack for quartzite's windowed rendering path, and define the resulting crate structure changes.

### Stack (fixed for v1)

- **Windowing / OS events:** `winit`
- **GPU backend:** `wgpu`
- **2D vector rendering:** `vello` (built on wgpu)

### Crate changes

1. **`quartzite-paint-api`** (new, thin) — shared types and `Painter` trait; no platform dep, `no_std`-compatible
2. **`quartzite-paint`** (existing, adjusted) — utility types and higher-level paint abstractions; depends on `quartzite-paint-api` + `quartzite-geometry`; backend-agnostic
3. **`quartzite-renderer`** (new) — vello + wgpu + winit backend; implements `Painter` from `quartzite-paint-api`; owns `Window` and the winit event loop
4. **`quartzite-runtime`** — stays graphics-free; headless `EventLoop` preserved

### EventLoop strategy

- `quartzite-runtime` retains its headless `mpsc`-based `EventLoop` for CLI, daemon, and headless-test use cases
- `quartzite-renderer` owns the winit event loop for windowed apps
- Integration mechanism (trait abstraction vs. replacement) is a design-phase decision

## Out of scope

- Text / font rendering (deferred to a later step)
- Pluggable backend abstraction (single fixed stack for v1)
- `quartzite-style` and `quartzite-widgets` implementation (separate plans)
- Mobile as a primary target (Android / iOS supported by winit + wgpu; treated as a bonus)
- WASM-specific packaging / bundling

## Deferred

- Text layout and font loading | deferred explicitly; vello uses `skrifa`/`parley` — will land with `quartzite-paint` implementation | separate issue needed
- Backend swap-out trait | deferred past v1 | separate issue if ever needed
- Per-thread winit event loops | depends on winit multi-window / per-thread design | issue #51

## Key decisions

| Question | Decision |
|---|---|
| Windowing library | winit — cross-platform (Linux X11/Wayland, macOS, Windows, WASM, Android/iOS) |
| GPU backend | wgpu — same platform matrix as winit |
| 2D drawing layer | vello — GPU-accelerated vector renderer on wgpu |
| Backend pluggability | Single fixed stack for v1; no swap-out trait |
| `quartzite-paint` backend coupling | None — `quartzite-paint` stays backend-agnostic |
| Shared types location | New `quartzite-paint-api` crate; `quartzite-paint` and `quartzite-renderer` are siblings both depending on it |
| `quartzite-runtime` graphics coupling | None — runtime stays graphics-free; suitable for CLI / daemon / headless tests |
| Headless event loop | Preserved in `quartzite-runtime`; winit loop lives in `quartzite-renderer` |
| EventLoop integration mechanism | Deferred to design phase |
| Primary target platforms | Desktop (Linux, macOS, Windows) + WASM |

## Blockers

- **`quartzite-geometry` does not exist yet** (plan: `geometry-events` #45). The `Painter` trait methods (`draw_rect`, `draw_line`, `clip_rect`, `translate`, `draw_image`, `draw_text_in`) all take `Point` / `Rect` / `Size` arguments. `quartzite-paint-api` cannot be fully defined without these types, and `quartzite-renderer` depends on `quartzite-paint-api`.
- `geometry-events` (#45) must land before any implementation work begins on #73.

## Technical constraints

- winit's event loop must own the main thread — it cannot run alongside the existing headless loop without integration work
- wgpu requires a `Surface` tied to a native window handle provided by winit
- vello renders into a wgpu `Texture`; the texture is then presented via the wgpu swap chain
- `quartzite-paint-api` must remain `no_std`-compatible (no platform imports, no `std::io`)
- `Painter` trait must be object-safe (`Box<dyn Painter>` and `&mut dyn Painter` must compile)
- winit does not support headless rendering — CI tests that don't open a window must use the headless loop

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-paint-api` compiles with `no_default_features` (no_std path) |
| AC2 | `quartzite-paint` compiles with no wgpu / vello / winit dependency in its `Cargo.toml` |
| AC3 | `quartzite-renderer` brings in winit, wgpu, and vello as direct dependencies |
| AC4 | `quartzite-runtime` has no direct or transitive dependency on winit, wgpu, or vello |
| AC5 | A mock `Painter` implementation compiles against `quartzite-paint-api` alone (no renderer dep) |
| AC6 | The dependency graph contains no cycles between `quartzite-paint-api`, `quartzite-paint`, and `quartzite-renderer` |

## Open questions

- Should the winit `EventLoop` integration be a trait (`EventDriver`?) in `quartzite-runtime` that both the headless loop and winit loop implement, or simply a separate entry point in `quartzite-renderer`?
- Should `quartzite-renderer` expose a `WindowedApplication` wrapper over `Application`, or extend `Application` via an extension trait?
- Does `quartzite-paint-api` need its own crate or can the types live in a `quartzite-paint::api` module with `pub use` re-exports? (Separate crate avoids the cycle risk; module approach avoids workspace churn.)
