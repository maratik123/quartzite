//! Derive macros and attribute macros for the quartzite object system.
//!
//! - [`Object`](derive_object) / [`object_impl`]: generate `AsObject`/`Object` trait impls
//!   with property access, method dispatch, and signal connection.
//! - [`Extend`](derive_extend): generate `AsObject` delegation via a `#[base]` field,
//!   enabling type-safe single-inheritance hierarchies.
//! - [`MetaEnum`](derive_meta_enum): generate `IntoValue`/`FromValue` and `EnumMeta` for
//!   C-like enums.

use proc_macro::TokenStream;

mod util;

mod extend;
mod meta_enum;
mod object;
mod object_impl;

#[proc_macro_derive(Extend, attributes(root, base, mixin))]
pub fn derive_extend(input: TokenStream) -> TokenStream {
    extend::expand(input.into()).into()
}

#[proc_macro_derive(Object, attributes(prop, signal))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    object::expand(input.into()).into()
}

#[proc_macro_attribute]
pub fn object_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    object_impl::expand(item.into()).into()
}

#[proc_macro_derive(MetaEnum)]
pub fn derive_meta_enum(input: TokenStream) -> TokenStream {
    meta_enum::expand(input.into()).into()
}
