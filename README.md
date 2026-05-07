# quartzite

[![codecov](https://codecov.io/gh/maratik123/quartzite/branch/master/graph/badge.svg)](https://codecov.io/gh/maratik123/quartzite)
[![docs](https://img.shields.io/badge/docs-master-blue)](https://maratik123.github.io/quartzite/)

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
| `examples/` | ✅ runnable examples: `hello_object`, `signals_slots`, `object_tree`, `timer` |
| `quartzite-geometry` / `quartzite-events` / `quartzite-event-types` | ✅ implemented |
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
cargo build --workspace
```

## Test

```bash
cargo test --workspace
```

## Lint

```bash
cargo clippy --workspace -- -D warnings
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
