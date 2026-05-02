# Design: Comprehensive documentation and quartzite facade crate

**Issue:** user description
**Date:** 2026-05-02

## Approach

The work splits into two largely independent tracks: (A) metadata and docs-rs hygiene across all
four crates, and (B) creating a new `quartzite` facade crate.

**Track A — metadata + docs-rs hygiene**

Add `authors`, `license`, `repository` to `[workspace.package]` so all crates inherit them via
`workspace = true`. Add per-crate `description` fields. Add `[package.metadata.docs.rs]` to every
crate (features + rustdoc/rustc cfg flags for the `docsrs` gate). Add `document-features` to
`quartzite-core` (and later the facade); annotate the `std` feature with a `##` comment; inject
the macro call at the top of both `lib.rs` files with `#![doc = document_features::document_features!()]`.

`quartzite-macros` is a `proc-macro = true` crate and cannot use `document-features` (proc-macro
crates have special linking constraints; a dependency chain through `document-features`'s own
proc-macro would either conflict or be silently dropped). The spec explicitly excludes it.

`quartzite-runtime` has no feature flags, so `document-features` adds nothing there and is also
excluded.

**Track B — facade crate**

Create `quartzite/` as a new workspace member. It re-exports a curated `prelude` module containing
the most commonly used public types from the three existing crates. The crate has a `std` feature
that forwards to `quartzite-core/std`. The `prelude` guards its `std`-only re-exports with
`#[cfg(feature = "std")]`.

**`cfg_attr(docsrs, doc(cfg(...)))` placement**

The following items in `quartzite-core` are only present when `std` is active and must receive the
`#[cfg_attr(docsrs, doc(cfg(feature = "std")))]` attribute so docs.rs shows the feature badge:

- `signal::QueuedDispatcher` (trait)
- `signal::DispatcherAlreadySet` (struct)
- `signal::set_queued_dispatcher` (fn)
- `signal::queued_dispatcher` (fn)
- `signal::ConnectionType::Queued` (variant)
- `signal::ConnectionType::Auto` (variant)
- `object_base::ObjectBase::thread_id` (field — inside a struct; annotation goes on the field)
- `object_base::ObjectBase::is_on_current_thread` (method)
- `traits::ObjectExt::is_on_current_thread` (method)

These are the *public* std-gated items that appear in the rendered documentation. Internal
`struct`/`trait` items (`SlotEntry`, `DynQueuedSlot`, `QueuedSlotInner`, `DynAutoSlot`,
`AutoSlotInner`) are private and do not need the annotation.

**Rejected alternative — wildcard re-exports at crate root**

Full `pub use quartzite_core::*` etc. at the facade root is simpler but clutters the public
surface and makes it harder for users to understand origin. A `prelude` module is standard Rust
practice (e.g. `std::prelude`) and matches the spec's "curated" requirement.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `authors`, `license`, `repository` to `[workspace.package]`; add `workspace = true` inherits to all three existing crates | `Cargo.toml`, `quartzite-core/Cargo.toml`, `quartzite-macros/Cargo.toml`, `quartzite-runtime/Cargo.toml` | — |
| 2 | Add `description` to all three existing crates' `Cargo.toml` | `quartzite-core/Cargo.toml`, `quartzite-macros/Cargo.toml`, `quartzite-runtime/Cargo.toml` | 1 |
| 3 | Add `[package.metadata.docs.rs]` to all three existing crates | `quartzite-core/Cargo.toml`, `quartzite-macros/Cargo.toml`, `quartzite-runtime/Cargo.toml` | 1 |
| 4 | Add `document-features` dependency to `quartzite-core`; annotate `std` feature with `##` comment; add `#![doc = document_features::document_features!()]` to `quartzite-core/src/lib.rs` | `quartzite-core/Cargo.toml`, `quartzite-core/src/lib.rs` | 3 |
| 5 | Add `#[cfg_attr(docsrs, doc(cfg(feature = "std")))]` to std-gated public items in `quartzite-core` | `quartzite-core/src/signal.rs`, `quartzite-core/src/object_base.rs`, `quartzite-core/src/traits.rs` | 4 |
| 6 | Create `quartzite/` facade crate: `Cargo.toml`, `src/lib.rs` with `pub mod prelude`, register in workspace | `quartzite/Cargo.toml`, `quartzite/src/lib.rs`, `Cargo.toml` | 4 |
| 7 | Add `document-features` to facade; annotate `std` feature; inject `#![doc = ...]`; add `[package.metadata.docs.rs]` | `quartzite/Cargo.toml`, `quartzite/src/lib.rs` | 6 |

Seven tasks — within the limit; no split needed.

## File inventory

**Modified files**
- `Cargo.toml` — workspace members + workspace.package fields
- `quartzite-core/Cargo.toml` — workspace inherits, description, document-features dep, docs.rs metadata
- `quartzite-macros/Cargo.toml` — workspace inherits, description, docs.rs metadata
- `quartzite-runtime/Cargo.toml` — workspace inherits, description, docs.rs metadata
- `quartzite-core/src/lib.rs` — `#![doc = ...]` line at top
- `quartzite-core/src/signal.rs` — `cfg_attr` annotations on 6 std-gated public items
- `quartzite-core/src/object_base.rs` — `cfg_attr` on `thread_id` field and `is_on_current_thread`
- `quartzite-core/src/traits.rs` — `cfg_attr` on `ObjectExt::is_on_current_thread`

**New files**
- `quartzite/Cargo.toml`
- `quartzite/src/lib.rs`

## Key content details

### `quartzite-core/Cargo.toml` — feature annotation
```toml
[features]
default = ["std"]
## Enable standard-library support (threading, queued dispatch, `is_on_current_thread`).
## Disable to build in `no_std + alloc` environments.
std = []
```

### `quartzite/Cargo.toml` sketch
```toml
[package]
name = "quartzite"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Quartzite object system — facade crate with curated prelude"

[features]
default = ["std"]
## Enable standard-library support (inherits `quartzite-core/std`).
std = ["quartzite-core/std"]

[dependencies]
quartzite-core    = { path = "../quartzite-core", default-features = false }
quartzite-macros  = { path = "../quartzite-macros" }
quartzite-runtime = { path = "../quartzite-runtime" }
document-features = "~0.2"

[package.metadata.docs.rs]
features = ["std"]
rustdoc-args = ["--cfg", "docsrs"]
rustc-args   = ["--cfg", "docsrs"]
```

`quartzite-core` is declared with `default-features = false` so the facade's own `std` feature
gates it cleanly.

### `quartzite/src/lib.rs` prelude content

```rust
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = document_features::document_features!()]

pub mod prelude {
    // quartzite-core
    pub use quartzite_core::{
        AsObject, Object, ObjectBase, ObjectExt, ObjectId, SignalCallback, Value, WeakObjectRef,
    };
    pub use quartzite_core::signal::{ConnectionType, Signal};
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub use quartzite_core::{
        DispatcherAlreadySet, QueuedDispatcher, queued_dispatcher, set_queued_dispatcher,
    };

    // quartzite-macros
    pub use quartzite_macros::{Extend, Object as DeriveObject, object_impl};

    // quartzite-runtime
    pub use quartzite_runtime::{Application, EventLoop, ObjectRef, ObjectTree, Timer, WeakRef};
}
```

Note: `quartzite_macros::Object` is re-exported as `DeriveObject` to avoid a name collision with
`quartzite_core::Object` (the trait). Both live in the prelude but under distinct names.

### `[package.metadata.docs.rs]` shape per crate

`quartzite-core` and `quartzite`:
```toml
[package.metadata.docs.rs]
features      = ["std"]
rustdoc-args  = ["--cfg", "docsrs"]
rustc-args    = ["--cfg", "docsrs"]
```

`quartzite-macros` and `quartzite-runtime` (no features to enable):
```toml
[package.metadata.docs.rs]
rustdoc-args  = ["--cfg", "docsrs"]
rustc-args    = ["--cfg", "docsrs"]
```

## Risks

- **Derive macro name collision in prelude:** `quartzite_macros::derive_object` is exposed as
  `Object` from the crate root; `quartzite_core::Object` is a trait. Both re-exported into the
  same module would shadow each other. Mitigation: re-export the derive macro under `DeriveObject`
  as shown above. The implementor should verify the actual exported name from `quartzite-macros`
  (`pub use quartzite_macros::{Object as DeriveObject, ...}`).

- **`document-features` requires `##` comments directly adjacent to feature entries:** any blank
  line between the comment and the feature line causes the comment to be silently ignored.
  Mitigation: follow the `## comment\nfeature = [...]` format exactly, verified by checking
  rendered docs locally with `cargo doc --features std`.

- **`quartzite-runtime` transitively enables `quartzite-core/std`** because its `Cargo.toml`
  declares `quartzite-core = { path = "../quartzite-core" }` without `default-features = false`,
  so the core `std` feature is always on when runtime is in the graph. This is intentional and
  consistent with the spec decision to skip a `std` feature on `quartzite-runtime`.

- **`cfg_attr(docsrs, feature(doc_cfg))` needs `#![cfg_attr(docsrs, feature(doc_cfg))]` at
  crate root:** required to make the `doc(cfg(...))` attribute take effect on nightly (docs.rs
  uses nightly). Both `quartzite-core/src/lib.rs` and `quartzite/src/lib.rs` need this inner
  attribute. `quartzite-core` already has `#![cfg_attr(not(feature = "std"), no_std)]` at the
  top; the new attribute is added alongside it.

- **`cargo build` includes `Cargo.lock` update:** adding a new crate and new dependency
  (`document-features`) will modify `Cargo.lock`. Per project workflow, `cargo build` must be run
  before committing so the updated lock file is included.

## Test Design

This task contains no non-trivial runtime logic — it is purely configuration, metadata, and
re-exports. The acceptance tests are build-and-lint checks:

- `cargo build` must compile clean with zero errors (AC9)
- `cargo clippy -- -D warnings` must pass clean (AC10)

The facade's `prelude` module is a set of `pub use` statements; there is no branching logic to
unit-test. A smoke `#[cfg(test)]` block in `quartzite/src/lib.rs` that imports from
`crate::prelude::*` and uses at least one type is sufficient to confirm the re-exports resolve:

```rust
#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn prelude_compiles() {
        let _: ObjectId = ObjectBase::new().id();
    }
}
```

Location: `quartzite/src/lib.rs` inline `#[cfg(test)]` module.
Entry point: `prelude_compiles` — exercises that key types are accessible via prelude.
Fixtures: none needed.

## Open questions

- None
