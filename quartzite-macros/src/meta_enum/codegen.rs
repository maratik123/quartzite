use proc_macro2::TokenStream;
use quote::quote;

use super::parse::MetaEnumInput;

pub(crate) fn codegen(ir: MetaEnumInput) -> TokenStream {
    let type_ident = &ir.ident;
    let enum_static_name =
        proc_macro2::Ident::new(&format!("__ENUM_{type_ident}"), type_ident.span());
    let type_name_str = type_ident.to_string();

    let entries = ir.variants.iter().map(|v| {
        let name = v.ident.to_string();
        let value = v.value;
        quote! {
            ::quartzite_core::EnumEntry::new(#name, #value)
        }
    });

    let from_int_arms = ir.variants.iter().map(|v| {
        let variant_ident = &v.ident;
        let value = v.value;
        quote! { #value => ::core::result::Result::Ok(#type_ident::#variant_ident) }
    });

    quote! {
        static #enum_static_name: ::quartzite_core::EnumMeta = ::quartzite_core::EnumMeta {
            name: #type_name_str,
            entries: &[#(#entries),*],
        };

        impl ::quartzite_core::IntoValue for #type_ident {
            fn into_value(self) -> ::quartzite_core::Value {
                ::quartzite_core::Value::Int(self as i64)
            }
        }

        impl ::quartzite_core::FromValue for #type_ident {
            fn from_value(
                val: ::quartzite_core::Value,
            ) -> ::core::result::Result<Self, ::quartzite_core::TypeError> {
                if let ::quartzite_core::Value::Int(n) = val {
                    match n {
                        #(#from_int_arms,)*
                        _ => ::core::result::Result::Err(::quartzite_core::TypeError {
                            expected: #type_name_str,
                            got: "Int",
                        }),
                    }
                } else {
                    ::core::result::Result::Err(::quartzite_core::TypeError {
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
        super::codegen(ir).to_string()
    }

    // EnumMeta static: correct type name and variant entries.
    #[test]
    fn enum_static_emits_name_and_entries() {
        let out = emit(quote! { enum Color { Red, Green, Blue } });
        assert!(out.contains("__ENUM_Color"), "missing static name: {out}");
        assert!(out.contains("name : \"Color\""), "missing type name: {out}");
        assert!(out.contains("\"Red\""), "missing Red entry: {out}");
        assert!(out.contains("\"Green\""), "missing Green entry: {out}");
        assert!(out.contains("\"Blue\""), "missing Blue entry: {out}");
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
            out.contains("impl :: quartzite_core :: IntoValue for Color"),
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
            out.contains("impl :: quartzite_core :: FromValue for Color"),
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

    // Empty enum: static has empty entries slice, match has no arms (only wildcard).
    #[test]
    fn empty_enum_emits_empty_entries() {
        let out = emit(quote! { enum Empty {} });
        assert!(
            out.contains("entries : & []"),
            "expected empty entries: {out}"
        );
    }
}
