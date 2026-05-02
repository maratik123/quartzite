# quartzite

A GUI and object framework for Rust built around signals/slots,
a rich object model, and a declarative UI layer — implemented as idiomatic Rust with no native
dependencies, no foreign ABI, and no code generation outside of proc macros.

## Status

Early development. Core crates are implemented; widget and painting layers are next.

| Crate | Status |
|---|---|
| `quartzite` (facade) | ✅ implemented |
| `quartzite-core` | ✅ implemented |
| `quartzite-macros` | ✅ implemented |
| `quartzite-runtime` | ✅ implemented |
| `quartzite-examples` | ✅ runnable examples: `hello_object`, `signals_slots`, `object_tree`, `timer` |
| `quartzite-geometry` / `quartzite-events` | planned |
| `quartzite-widgets` | planned |
| `quartzite-paint` / `quartzite-style` | planned |

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
cargo build
```

## Test

```bash
cargo test
```

## Lint

```bash
cargo clippy -- -D warnings
```

## Format

```bash
cargo fmt              # apply
cargo fmt -- --check   # verify (CI gate)
```

## Docs

```bash
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace
```

## License

Licensed under the GNU Lesser General Public License v3.0 — see [LICENSE](LICENSE) for details.
