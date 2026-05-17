use proc_macro2::TokenStream;
use quote::quote;

use super::parse::MetaEnumInput;
use crate::util::{crate_root, inline_if_concrete};

pub(crate) fn codegen(ir: &MetaEnumInput) -> TokenStream {
    let cr = crate_root();
    let type_ident = &ir.ident;
    let inline = inline_if_concrete(&ir.generics);
    let enum_static_name =
        proc_macro2::Ident::new(&format!("__ENUM_{type_ident}"), type_ident.span());
    let lookup_by_name_fn = proc_macro2::Ident::new(
        &format!("__lookup_entry_by_name_{type_ident}"),
        type_ident.span(),
    );
    let lookup_by_value_fn = proc_macro2::Ident::new(
        &format!("__lookup_entry_by_value_{type_ident}"),
        type_ident.span(),
    );
    let entries_static_name =
        proc_macro2::Ident::new(&format!("__ENTRIES_{type_ident}"), type_ident.span());
    let type_name_str = type_ident.to_string();

    let entries = ir.variants.iter().map(|v| {
        let name = v.ident.to_string();
        let value = v.value;
        quote! {
            #cr::EnumEntry::new(#name, #value)
        }
    });

    let by_name_arms = ir.variants.iter().enumerate().map(|(idx, v)| {
        let name = v.ident.to_string();
        let idx_lit = syn::Index::from(idx);
        quote! {
            #name => ::core::option::Option::Some(#entries_static_name[#idx_lit])
        }
    });

    let by_value_arms = ir.variants.iter().enumerate().map(|(idx, v)| {
        let value = v.value;
        let idx_lit = syn::Index::from(idx);
        quote! {
            #value => ::core::option::Option::Some(#entries_static_name[#idx_lit])
        }
    });

    let from_int_arms = ir.variants.iter().map(|v| {
        let variant_ident = &v.ident;
        let value = v.value;
        quote! { #value => ::core::result::Result::Ok(#type_ident::#variant_ident) }
    });

    quote! {
        #[allow(non_upper_case_globals)]
        static #entries_static_name: &[#cr::EnumEntry] =
            &[#(#entries),*];

        #[allow(non_snake_case)]
        fn #lookup_by_name_fn(name: &str) -> ::core::option::Option<#cr::EnumEntry> {
            match name {
                #(#by_name_arms,)*
                _ => ::core::option::Option::None,
            }
        }

        #[allow(non_snake_case)]
        fn #lookup_by_value_fn(
            value: i64,
        ) -> ::core::option::Option<#cr::EnumEntry> {
            match value {
                #(#by_value_arms,)*
                _ => ::core::option::Option::None,
            }
        }

        #[allow(non_upper_case_globals)]
        static #enum_static_name: #cr::EnumMeta = #cr::EnumMeta::new(
            #type_name_str,
            #entries_static_name,
            #lookup_by_name_fn,
            #lookup_by_value_fn,
        );

        impl #cr::IntoValue for #type_ident {
            #inline
            fn into_value(self) -> #cr::Value {
                #cr::Value::Int(self as i64)
            }
        }

        impl #cr::FromValue for #type_ident {
            fn from_value(
                val: #cr::Value,
            ) -> ::core::result::Result<Self, #cr::TypeError> {
                if let #cr::Value::Int(n) = val {
                    match n {
                        #(#from_int_arms,)*
                        _ => ::core::result::Result::Err(#cr::TypeError {
                            expected: #type_name_str,
                            got: "Int",
                        }),
                    }
                } else {
                    ::core::result::Result::Err(#cr::TypeError {
                        expected: #type_name_str,
                        got: val.type_name(),
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    fn emit(ts: TokenStream) -> String {
        let ir = crate::meta_enum::parse::parse(ts).expect("parse ok");
        super::codegen(&ir).to_string()
    }

    // EnumMeta static: correct type name and variant entries.
    #[test]
    fn enum_static_emits_name_and_entries() {
        let out = emit(quote! { enum Color { Red, Green, Blue } });
        assert!(out.contains("__ENUM_Color"), "missing static name: {out}");
        // New API uses EnumMeta::new("Color", ...) instead of struct literal
        assert!(
            out.contains("EnumMeta :: new (\"Color\""),
            "missing type name: {out}"
        );
        assert!(out.contains("\"Red\""), "missing Red entry: {out}");
        assert!(out.contains("\"Green\""), "missing Green entry: {out}");
        assert!(out.contains("\"Blue\""), "missing Blue entry: {out}");
        // Lookup functions are emitted
        assert!(
            out.contains("__lookup_entry_by_name_Color"),
            "missing by-name lookup: {out}"
        );
        assert!(
            out.contains("__lookup_entry_by_value_Color"),
            "missing by-value lookup: {out}"
        );
    }

    // EnumEntry values follow auto-increment from 0.
    #[test]
    fn entries_have_correct_auto_discriminants() {
        let out = emit(quote! { enum Color { Red, Green, Blue } });
        assert!(
            out.contains("EnumEntry :: new (\"Red\" , 0i64)"),
            "wrong Red value: {out}"
        );
        assert!(
            out.contains("EnumEntry :: new (\"Green\" , 1i64)"),
            "wrong Green value: {out}"
        );
        assert!(
            out.contains("EnumEntry :: new (\"Blue\" , 2i64)"),
            "wrong Blue value: {out}"
        );
    }

    // Explicit discriminants are used as-is.
    #[test]
    fn explicit_discriminants_in_entries() {
        let out = emit(quote! { enum Status { Ok = 200, NotFound = 404 } });
        assert!(
            out.contains("EnumEntry :: new (\"Ok\" , 200i64)"),
            "wrong Ok value: {out}"
        );
        assert!(
            out.contains("EnumEntry :: new (\"NotFound\" , 404i64)"),
            "wrong NotFound: {out}"
        );
    }

    // IntoValue impl: casts self to i64 via Value::Int.
    #[test]
    fn into_value_impl_emits_cast_to_int() {
        let out = emit(quote! { enum Color { Red } });
        assert!(
            out.contains("impl :: quartzite :: core :: IntoValue for Color"),
            "missing impl: {out}"
        );
        assert!(
            out.contains("Value :: Int (self as i64)"),
            "missing cast: {out}"
        );
    }

    // FromValue impl: known discriminant arm returns Ok(Variant).
    #[test]
    fn from_value_impl_has_match_arms() {
        let out = emit(quote! { enum Color { Red, Green } });
        assert!(
            out.contains("impl :: quartzite :: core :: FromValue for Color"),
            "missing impl: {out}"
        );
        assert!(
            out.contains("=> :: core :: result :: Result :: Ok (Color :: Red)"),
            "missing Red arm: {out}"
        );
        assert!(
            out.contains("=> :: core :: result :: Result :: Ok (Color :: Green)"),
            "missing Green arm: {out}"
        );
    }

    // FromValue impl: unknown discriminant and non-Int value produce TypeError.
    #[test]
    fn from_value_emits_type_error_for_unknown() {
        let out = emit(quote! { enum Color { Red } });
        assert!(out.contains("TypeError"), "missing TypeError: {out}");
        assert!(
            out.contains("expected : \"Color\""),
            "missing expected field: {out}"
        );
        // fallthrough arm for unrecognised Int
        assert!(out.contains("got : \"Int\""), "missing got Int: {out}");
        // non-Int branch
        assert!(
            out.contains("val . type_name ()"),
            "missing type_name call: {out}"
        );
    }

    // Empty enum: entries static is empty, match functions have only wildcard arms.
    #[test]
    fn empty_enum_emits_empty_entries() {
        let out = emit(quote! { enum Empty {} });
        // Entries static contains an empty slice
        assert!(
            out.contains("__ENTRIES_Empty : & [:: quartzite :: core :: EnumEntry] = & []"),
            "expected empty entries: {out}"
        );
        // Lookup functions are emitted even for an empty enum
        assert!(
            out.contains("__lookup_entry_by_name_Empty"),
            "missing by-name lookup: {out}"
        );
        assert!(
            out.contains("__lookup_entry_by_value_Empty"),
            "missing by-value lookup: {out}"
        );
    }

    // AC3: concrete enum — IntoValue::into_value carries #[inline]; FromValue::from_value does not.
    #[test]
    fn into_value_concrete_has_inline_from_value_does_not() {
        let out = emit(quote! { enum Color { Red, Green } });
        let count = out.matches("# [inline]").count();
        assert!(
            count == 1,
            "expected exactly 1 #[inline] (IntoValue::into_value only), got {count}: {out}"
        );
    }

    // AC3: generic enum — neither IntoValue::into_value nor FromValue::from_value carries #[inline].
    #[test]
    fn into_value_generic_has_no_inline_from_value_does_not() {
        let out = emit(quote! { enum Color<T> { Red, Green } });
        let count = out.matches("# [inline]").count();
        assert!(
            count == 0,
            "unexpected #[inline] in generic enum output, got {count}: {out}"
        );
    }
}
