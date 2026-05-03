//! Derive macros and attribute macros for the quartzite object system.
//!
//! - [`Object`](derive_object) / [`object_impl`]: generate `AsObject`/`Object` trait impls
//!   with property access, method dispatch, and signal connection.
//! - [`Extend`](derive_extend): generate `AsObject` delegation via a `#[base]` field,
//!   enabling type-safe single-inheritance hierarchies.
//! - [`MetaEnum`](derive_meta_enum): generate `IntoValue`/`FromValue` and `EnumMeta` for
//!   C-like enums.
#![deny(missing_docs)]

use proc_macro::TokenStream;

macro_rules! make_expand {
    () => {
        pub(crate) fn expand(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
            match parse::parse(input) {
                Ok(ir) => codegen::codegen(ir),
                Err(e) => e.to_compile_error(),
            }
        }
    };
}
pub(crate) use make_expand;

mod util;

mod extend;
mod meta_enum;
mod object;
pub(crate) mod object_impl;
mod object_meta;

/// Derive macro that generates `As{TypeName}` trait impls and delegation chains for
/// single-inheritance object hierarchies.
///
/// # Attributes
///
/// - `#[root]` — marks the struct as the root of a hierarchy; generates the `As{TypeName}`
///   trait and a self-returning impl. The `#[base]` field must hold an `ObjectBase` or
///   any type that already implements `AsObject`.
/// - `#[base]` — the field that holds the parent object. Generates delegation impls for
///   `As{ParentTypeName}` and all its ancestors.
/// - `#[mixin]` — additional delegation targets; generates `As{MixinTypeName}` impls
///   without joining the primary inheritance chain.
///
/// # Examples
///
/// ```no_run
/// use quartzite::core::ObjectBase;
/// use quartzite_macros::Extend;
///
/// // Root of the hierarchy: generates AsWidget + AsObject delegation.
/// #[derive(Extend)]
/// #[root]
/// struct Widget {
///     #[base]
///     object_base: ObjectBase,
/// }
///
/// // Derived type: extends Widget, so AsButton, AsWidget, and AsObject are all available.
/// #[derive(Extend)]
/// struct Button {
///     #[base]
///     widget: Widget,
/// }
/// ```
#[proc_macro_derive(Extend, attributes(root, base, mixin))]
pub fn derive_extend(input: TokenStream) -> TokenStream {
    extend::expand(input.into()).into()
}

/// Derive macro that generates property metadata, signal metadata, and partial `Object`
/// implementation helpers (property read/write functions and dynamic signal connections).
///
/// Must be used together with [`object_impl`] on a matching `impl` block to produce a
/// complete [`Object`] implementation.
///
/// # Attributes on fields
///
/// - `#[prop]` — marks a field as a readable/writable property. Supports sub-options:
///   - `notify = signal_name` — emit `signal_name` after every successful write
///   - `read_only` — disallow property writes
///   - `constant` — value never changes (implies `read_only`)
///   - `stored = false` — exclude from serialisation
///   - `designable = false` — hide from visual editors
///   - `user` — mark as the primary user-editable property
/// - `#[signal]` — marks a `Signal<Args>` field so its
///   parameter types are recorded in the `MetaObject`.
///
/// # Examples
///
/// ```ignore
/// use quartzite::core::{ObjectBase, Signal};
/// use quartzite_macros::{Extend, Object};
///
/// #[derive(Extend, Object)]
/// #[root]
/// struct Counter {
///     #[base]
///     object_base: ObjectBase,
///     #[prop(notify = count_changed)]
///     pub count: i32,
///     #[signal]
///     pub count_changed: Signal<(i32,)>,
/// }
/// ```
#[proc_macro_derive(Object, attributes(prop, signal))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    object::expand(input.into()).into()
}

/// Attribute macro applied to an `impl` block that finalises the `Object` implementation.
///
/// Generates the `MetaObject` static (class name, property list, signal list, method list)
/// and the complete `Object` trait impl. Methods inside the block can be annotated with:
///
/// - `#[slot]` — callable via `Object::invoke_method`; return type must be `()`
/// - `#[invokable]` — callable via `Object::invoke_method` with a return value converted
///   via `IntoValue`
///
/// The struct must already derive both [`Extend`] and [`Object`].
///
/// ## Multi-block mode: `partial` and `final`
///
/// When the impl is split across multiple blocks, annotate every block but the last with
/// `#[object_impl(partial)]` and the last with `#[object_impl(final)]`
/// (or use [`#[object_meta]`](macro@object_meta) instead of a `final` block).
///
/// **A terminal block is required.** If all blocks use `partial` and no `final` /
/// `#[object_meta]` is ever written, the accumulated methods are silently discarded and
/// the type will not implement `Object`.  The resulting compile error
/// ("the trait `Object` is not implemented for `T`") will not mention the missing terminal.
///
/// # Examples
///
/// ```ignore
/// use quartzite::core::{ObjectBase, Signal};
/// use quartzite_macros::{Extend, Object, object_impl};
///
/// #[derive(Extend, Object)]
/// #[root]
/// struct Counter {
///     #[base]
///     object_base: ObjectBase,
///     #[prop(notify = count_changed)]
///     pub count: i32,
///     #[signal]
///     pub count_changed: Signal<(i32,)>,
/// }
///
/// #[object_impl]
/// impl Counter {
///     #[slot]
///     fn reset(&mut self) {
///         self.count = 0;
///     }
///
///     #[invokable]
///     fn doubled(&self) -> i32 {
///         self.count * 2
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn object_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    object_impl::expand(attr.into(), item.into()).into()
}

/// Attribute macro that finalises the `Object` implementation after multiple
/// `#[object_impl(partial)]` blocks.
///
/// Place `#[object_meta]` on an empty `impl Counter {}` block **after all**
/// `#[object_impl(partial)]` blocks for the same type — ordering matters.
/// Proc-macros expand in source order: an `#[object_meta]` that appears
/// before any `#[object_impl(partial)]` block drains an empty accumulator and
/// produces a `MetaObject` with no methods.
///
/// It reads the accumulated `#[slot]`/`#[invokable]` methods, generates the
/// `MetaObject` static and the full `impl Object`, then discards the empty
/// impl block.
///
/// # Examples
///
/// ```ignore
/// use quartzite_macros::{Extend, Object, object_impl, object_meta};
///
/// #[derive(Extend, Object)]
/// #[root]
/// struct Counter { /* ... */ }
///
/// #[object_impl(partial)]
/// impl Counter {
///     #[slot]
///     fn reset(&mut self) { /* ... */ }
/// }
///
/// #[object_meta]
/// impl Counter {}
/// ```
#[proc_macro_attribute]
pub fn object_meta(_attr: TokenStream, item: TokenStream) -> TokenStream {
    object_meta::expand(item.into()).into()
}

/// Derive macro for C-like enums that generates `IntoValue` / `FromValue` conversions
/// and an `EnumMeta` descriptor for runtime reflection.
///
/// Only unit variants (with optional explicit discriminants) are supported. Variants with
/// tuple or struct fields are rejected at compile time.
///
/// `IntoValue`, `FromValue`, and `EnumMeta` are types from `quartzite_core`.
///
/// # Examples
///
/// ```no_run
/// use quartzite_macros::MetaEnum;
///
/// #[derive(Debug, Clone, Copy, PartialEq, MetaEnum)]
/// enum Direction {
///     North = 0,
///     East  = 1,
///     South = 2,
///     West  = 3,
/// }
/// ```
#[proc_macro_derive(MetaEnum)]
pub fn derive_meta_enum(input: TokenStream) -> TokenStream {
    meta_enum::expand(input.into()).into()
}
