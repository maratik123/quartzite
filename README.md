# quartzite

A GUI and object framework for Rust. Inspired by the design principles of Qt — signals/slots,
a rich object model, and a declarative UI layer — but implemented as idiomatic Rust with no C++
dependency, no foreign ABI, and no code generation outside of proc macros.

## Status

Early development. Core crates are implemented; widget and painting layers are next.

| Crate | Status |
|---|---|
| `quartzite-core` | ✅ implemented |
| `quartzite-macros` | ✅ implemented |
| `quartzite-runtime` | ✅ implemented |
| `quartzite-geometry` / `quartzite-events` | planned |
| `quartzite-widgets` | planned |
| `quartzite-paint` / `quartzite-style` | planned |

## Prerequisites

- Rust stable (≥ 1.85, for edition 2024 support)
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

## License

Licensed under the GNU Lesser General Public License v3.0 — see [LICENSE](LICENSE) for details.
