# Fix `_Simple._` markers on generic/trait-decl simple fns

**Source:** issue #116
**Date:** 2026-05-07
**Tracked in:** #116

## Scope

Per the marker-form decision tree (AGENTS.md → *Code Style* → `#[inline]` and the `_Simple._` doc tag):

- **Inherent generic method** (inside `impl<T> Foo<T>`): use `/// _Simple._` doc line — NOT `#[inline]`
- **Trait method declaration** (default method or method decl in `pub trait` body): use `/// _Simple._` doc line — NOT `#[inline]`

This spec covers:

1. **Drift fix (7 sites):** inherent generic methods inside `impl<T>` blocks that currently carry `#[inline]` instead of `/// _Simple._`.
2. **Audit fix (5 sites):** trait method declarations in `pub trait` bodies that currently carry `#[inline]` instead of `/// _Simple._`.

Out of scope:
- Concrete fns (no type params, concrete `Self`) — `#[inline]` is correct there
- Methods inside `impl<T> Trait for Foo<T>` blocks — already fixed by PR #121 (`// _Simple._`)
- Codegen-emitted generic fns (covered by issue #117 Part 2)

## Acceptance Criteria

| # | Criterion | Verifiable? |
|---|-----------|-------------|
| AC1 | `Signal::new` in `impl<Args: 'static> Signal<Args>` carries `/// _Simple._` doc line; `#[inline]` removed | `rg '_Simple._' quartzite-core/src/signal.rs` |
| AC2 | `ObjectRef::new` in `impl<T> ObjectRef<T>` carries `/// _Simple._`; `#[inline]` removed | `rg '_Simple._' quartzite-runtime/src/object_ref.rs` |
| AC3 | `ObjectRef::id` in `impl<T> ObjectRef<T>` carries `/// _Simple._`; `#[inline]` removed | same |
| AC4 | `ObjectRef::downgrade` in `impl<T> ObjectRef<T>` carries `/// _Simple._`; `#[inline]` removed | same |
| AC5 | `WeakRef::new` in `impl<T> WeakRef<T>` carries `/// _Simple._`; `#[inline]` removed | same |
| AC6 | `WeakRef::id` in `impl<T> WeakRef<T>` carries `/// _Simple._`; `#[inline]` removed | same |
| AC7 | `WeakRef::is_valid` in `impl<T> WeakRef<T>` carries `/// _Simple._`; `#[inline]` removed | same |
| AC8 | `ObjectExt::id`, `ObjectExt::name`, `ObjectExt::is_on_current_thread` in `pub trait ObjectExt` body each carry `/// _Simple._` doc line; `#[inline]` removed from all three | `rg '_Simple._' quartzite-core/src/traits.rs` |
| AC9 | `ObjectTreeExt::parent_in` and `ObjectTreeExt::children_in` in `pub trait ObjectTreeExt` body each carry `/// _Simple._` doc line; `#[inline]` removed | `rg '_Simple._' quartzite-runtime/src/object_tree_ext.rs` |
| AC10 | `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, and doc gate all pass | CI / local |
| AC11 | No other `#[inline]` inside `impl<T>` or `pub trait` bodies remains in non-codegen source files | `rg '#\[inline\]' --type rust` audit |
