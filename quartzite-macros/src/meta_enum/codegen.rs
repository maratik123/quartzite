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
