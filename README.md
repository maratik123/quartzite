# quartzite

[![CI](https://github.com/maratik123/quartzite/actions/workflows/ci.yml/badge.svg)](https://github.com/maratik123/quartzite/actions/workflows/ci.yml)
[![docs](https://img.shields.io/badge/docs-master-blue)](https://maratik123.github.io/quartzite/)
[![codecov](https://codecov.io/gh/maratik123/quartzite/branch/master/graph/badge.svg)](https://codecov.io/gh/maratik123/quartzite)
[![license](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue)](#license)

A GUI and object framework for Rust drawing on Qt's signals/slots and
property/reflection model — implemented in idiomatic Rust with no native
dependencies, no foreign ABI, and no codegen outside proc-macros.

## Current scope

- **Object model:** `ObjectBase`, parent/child trees, named lookup, reflection metadata.
- **Signals/slots:** typed `Signal<Args>`, dynamic dispatch via `Object::invoke_method`, cross-thread queued connections.
- **Event loop:** `Application` singleton, per-thread `EventLoop`, queued dispatcher.
- **Timers:** `Timer` object with `AppDriver` / `PoolDriver` / `ThreadDriver` execution contexts.
- **Painting API** (`quartzite-paint-api`) — 11-method `Painter` trait (rect/line/text/image/path/transform/state), `Color`, `Pen`, `Brush`/`BrushKind` (solid + 2-stop linear/radial gradient + `Custom(peniko::Gradient)` escape hatch), `Font`, `Image`, `Path` — `no_std`-compatible.
- **Renderer** (`quartzite-renderer`) — `WindowedApplication` (multi-window, configurable last-window-quit policy, proxy-based `AppEvent::Exit`) + `WindowRegistry` (per-window event fan-out via `WindowedAppHandler`) + `VelloPainter` (full 11-method `Painter` impl — rect/line/path/image/text via parley+skrifa; transform/clip stack; vello + wgpu + winit) + `RenderHarness` / `RenderHarnessBuilder` offscreen test harness for snapshot testing.
- **Widgets** (`quartzite-widgets`) — `WidgetBase`, `WidgetExt`, layouts (`BoxLayout`, `GridLayout`), and built-in widgets (`Label`, `Button`, `LineEdit`, `TextEdit`, `ScrollArea`, `Container`).
- **Snapshot / save-restore** (`serde` feature) — `capture_object` / `restore_object` / `capture_tree` / `restore_tree` with JSON and bincode support; `Value::Custom` round-trips via `typetag`.

## Forward scope

- **Style system** (`quartzite-style`) — declarative styling on top of widgets.

## Non-goals

- Not a Qt port or a Qt binding — Qt is design lineage, not API surface.
- No FFI / native dependencies — pure-Rust toolchain.

## Status

Early development. Core crates and the widget system are implemented; full painting and theming layers are next.

| Crate | Status |
|---|---|
| `quartzite` (facade) | ✅ implemented |
| `quartzite-core` | ✅ implemented |
| `quartzite-macros` | ✅ implemented |
| `quartzite-runtime` | ✅ implemented |
| `examples/` | ✅ runnable examples: `hello_object`, `signals_slots`, `object_tree`, `timer` |
| `quartzite-geometry` / `quartzite-events` / `quartzite-event-types` | ✅ implemented |
| `quartzite-paint-api` | ✅ implemented (Color, Pen, Brush/BrushKind incl. LinearGradient/RadialGradient/Custom gradient, Font, Image, Path, 11-method Painter trait) |
| `quartzite-paint` | ✅ implemented (re-export shell over paint-api + Alignment from geometry + peniko gradient re-exports) |
| `quartzite-renderer` | ✅ implemented (WindowedApplication + multi-window WindowRegistry + WindowedAppHandler + VelloPainter full 11-method impl with parley/skrifa text; RenderHarness/RenderHarnessBuilder snapshot harness) |
| `quartzite-widgets` | ✅ implemented (#46) |
| `quartzite-style-types` | ✅ implemented (#47, leaf: Palette, ColorRole; #488 DARK_PALETTE; #402 ColorGroup axis + FocusRing) |
| `quartzite-style` | ✅ implemented (#47, downstream: Style trait, StyleRegistry; #290 DefaultStyle concrete impl; #297 GPU snapshot tests; #318 Container+LineEdit arms; #458 read-only overlay fix; #402 ColorGroup palette migration) |
| `quartzite-style-dispatch` | ✅ implemented (#312, widget-tree paint dispatcher: `dispatch_paint` + `WidgetResolver`) |

## Usage

Add `quartzite` to your `Cargo.toml` — no sub-crate deps required:

```toml
[dependencies]
quartzite = { git = "https://github.com/maratik123/quartzite" }
```

Import the prelude for typical usage, or use explicit module paths:

```rust
// one glob covers the object model, signals, derive macros, and runtime
use quartzite::prelude::*;

// explicit paths when you want legibility
use quartzite::core::ObjectBase;
use quartzite::macros::MetaEnum;   // requires `derive` feature (on by default)
use quartzite::runtime::Application;
```

Disable the `derive` feature to skip proc-macro compilation:

```toml
quartzite = { git = "...", default-features = false, features = ["std"] }
```

## Prerequisites

- Rust stable (≥ 1.95)
- Cargo (comes with Rust)

## Build

```bash
cargo build --workspace
```

## Test

```bash
cargo test --workspace
```

## Lint

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

## Format

```bash
cargo fmt --all              # apply
cargo fmt --all -- --check   # verify (CI gate)
```

## Docs

```bash
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace
```

## License

Dual-licensed under either of:

- [MIT License](LICENSE-MIT) ([https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))
- [Apache License, Version 2.0](LICENSE-APACHE) ([https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))

at your option. This is the standard Rust ecosystem dual-license.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.
