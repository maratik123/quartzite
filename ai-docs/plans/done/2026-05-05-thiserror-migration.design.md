# Design: Migrate manual Display/Error impls to `#[derive(thiserror::Error)]`

**Issue:** #86
**Date:** 2026-05-05

## Approach

Replace four manually written `Display` + `Error` impl blocks with `#[derive(thiserror::Error)]`
and `#[error("...")]` annotations. No public API changes: variant/field names, error message
strings, and derive lists (e.g. `Debug`, `Clone`, `PartialEq`, `Eq`) are all preserved verbatim.

**Why thiserror?** Already used in `quartzite-runtime` for `TreeAccessError`. Using it uniformly
removes boilerplate `Display` impls, reduces the diff surface for future message changes, and is
the idiomatic Rust approach for library error types.

**thiserror 2.x and no_std.** `thiserror` 2.x generates `core::error::Error` impls when `std` is
not in scope, making it compatible with the `no_std + alloc` configuration of `quartzite-core`.
The only requirement is that `thiserror = "2"` is added to `quartzite-core`'s `[dependencies]`.

**`DispatcherAlreadySet` cfg-gating.** The struct and its `Display`/`Error` impls are currently
all wrapped in `#[cfg(feature = "std")]`. With `thiserror`, the struct definition stays under
`#[cfg(feature = "std")]`, and `#[derive(thiserror::Error)]` is applied directly to it — the
derive itself is inside the `cfg` block, so nothing leaks into no_std builds. No separate
`cfg(feature = "std")` guards on the impl blocks are needed: removing those two standalone impl
blocks is part of the migration.

**`TypeError` — adding `Error` impl.** `TypeError` currently has `Display` but no `Error` impl.
The spec mandates adding one via `#[derive(thiserror::Error)]`. This is a purely additive change;
no callers are broken.

**Rejected alternatives:**
- Manual impls: status quo; discarded to reduce boilerplate.
- `snafu` or `anyhow`: heavier deps; not appropriate for leaf error types in a library crate.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `thiserror = "2"` to `quartzite-core` `[dependencies]` | `quartzite-core/Cargo.toml` | — |
| 2 | Migrate `DispatcherAlreadySet` | `quartzite-core/src/signal.rs` | 1 |
| 3 | Migrate `TypeError` | `quartzite-core/src/value.rs` | 1 |
| 4 | Migrate `ApplicationError` | `quartzite-runtime/src/application.rs` | — |
| 5 | Migrate `FactoryAlreadySet` | `quartzite-runtime/src/factory.rs` | — |
| 6 | Full gate check: `cargo build -p quartzite --no-default-features` | — | 2, 3 |
| 7 | CI gate: `cargo test`, `cargo clippy -- -D warnings`, `cargo doc` | — | 4, 5, 6 |

Tasks 4 and 5 depend only on an already-present `thiserror = "2"` in `quartzite-runtime/Cargo.toml`
and can be done before or in parallel with tasks 2–3.

### Task details

**Task 1** — `quartzite-core/Cargo.toml`

Add to `[dependencies]`:
```toml
thiserror = { version = "2", default-features = false }
```
`default-features = false` is required so that thiserror's `std` feature is not pulled in
unconditionally — thiserror 2.x enables its `std` feature by default, which would break the
no_std build path. With `default-features = false`, thiserror auto-detects the environment
via its own `#[cfg(feature = "std")]` detection inside the generated code.

**Task 2** — `quartzite-core/src/signal.rs`

Inside the `#[cfg(feature = "std")]` block that declares `DispatcherAlreadySet`:
- Add `thiserror::Error` to the existing `#[derive(...)]`.
- Add `#[error("queued dispatcher is already installed")]` to the struct.
- Remove the two impl blocks: `impl std::fmt::Display for DispatcherAlreadySet` and
  `impl std::error::Error for DispatcherAlreadySet`.

Before (lines 135–147 of `signal.rs`):
```rust
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatcherAlreadySet;

#[cfg(feature = "std")]
impl std::fmt::Display for DispatcherAlreadySet { ... }

#[cfg(feature = "std")]
impl std::error::Error for DispatcherAlreadySet {}
```

After:
```rust
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("queued dispatcher is already installed")]
pub struct DispatcherAlreadySet;
```

**Task 3** — `quartzite-core/src/value.rs`

`TypeError` has two public fields (`expected`, `got`) used in the format string.

Before (lines 215–231):
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub expected: &'static str,
    pub got: &'static str,
}

impl core::fmt::Display for TypeError { ... }
// No Error impl
```

After:
```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("type error: expected {expected}, got {got}")]
pub struct TypeError {
    pub expected: &'static str,
    pub got: &'static str,
}
```

Note: `thiserror` uses named-field interpolation in `#[error(...)]`. The format
`{expected}` and `{got}` refer to the struct fields directly — this is idiomatic thiserror
and produces the same message as the current `write!(f, "type error: expected {}, got {}", self.expected, self.got)`.

**Task 4** — `quartzite-runtime/src/application.rs`

`ApplicationError` is an enum with one variant `AlreadyExists`. No source fields.

Before (lines 19–33):
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    AlreadyExists,
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::AlreadyExists => write!(f, "Application already exists"),
        }
    }
}

impl std::error::Error for ApplicationError {}
```

After:
```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApplicationError {
    /// An `Application` instance already exists in this process.
    #[error("Application already exists")]
    AlreadyExists,
}
```

**Task 5** — `quartzite-runtime/src/factory.rs`

`FactoryAlreadySet` is a unit struct. The `Display` impl has `#[inline]` — thiserror generates
its own non-inline `fmt`; the `#[inline]` annotation is dropped (it was on a trivial impl that
thiserror replaces entirely, so no behaviour change).

Before (lines 24–34):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactoryAlreadySet;

impl std::fmt::Display for FactoryAlreadySet {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObjectFactory is already installed")
    }
}

impl std::error::Error for FactoryAlreadySet {}
```

After:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("ObjectFactory is already installed")]
pub struct FactoryAlreadySet;
```

## Risks

- **no_std breakage from `thiserror` std feature:** Mitigated by using
  `thiserror = { version = "2", default-features = false }` in `quartzite-core/Cargo.toml`.
  Verified by task 6 (`cargo build -p quartzite --no-default-features`).
- **`DispatcherAlreadySet` leaking outside cfg gate:** The struct definition is inside
  `#[cfg(feature = "std")]` and `#[derive(thiserror::Error)]` is part of the same definition —
  no code is generated outside the gate.
- **`TypeError` field interpolation:** thiserror 2.x supports named-field interpolation in struct
  `#[error(...)]`; the generated message is identical. Low risk.
- **`#[inline]` on Display removed for `FactoryAlreadySet`:** The inline annotation was on a
  trivial `write!` call in a manually written impl. thiserror generates a non-inlined fmt. For a
  unit struct with a constant string this makes no observable difference. No risk.
- **Doc example in `DispatcherAlreadySet` uses `format!("{err}")`:** The message string is
  preserved verbatim — doctest remains valid.

## Test Design

All four error types already have coverage through existing tests and doctests. No new test logic
is required — the migration is purely mechanical (same observable behaviour, different
implementation). The test plan confirms nothing regresses:

### Existing coverage that validates AC5 (message strings unchanged)

- `quartzite-core/src/signal.rs` — doctest on `DispatcherAlreadySet`:
  `assert_eq!(format!("{err}"), "queued dispatcher is already installed");`
- `quartzite-core/src/value.rs` — doctest on `TypeError` via `FromValue` example.
- `quartzite-runtime/src/application.rs` — doctest on `ApplicationError`.
- `quartzite-runtime/src/factory.rs` — doctest on `FactoryAlreadySet` checks `err == FactoryAlreadySet`.

### Gate checks (tasks 6 & 7)

- `cargo build -p quartzite --no-default-features` — no_std path compiles (AC6).
- `cargo test` — all doctests and unit tests pass (AC7).
- `cargo clippy -- -D warnings` — no new warnings (AC8).
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` — doc gate passes.

No new test fixtures or test modules needed: the change is non-algorithmic.

## Open questions

- None.
