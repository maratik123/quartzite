use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Index, ReturnType};

use super::parse::{MethodItem, ObjectImplInput};
use crate::util::hidden_mod_ident;

pub(crate) fn codegen(ir: ObjectImplInput) -> TokenStream {
    let type_ident = &ir.self_ty_ident;
    let self_ty = &ir.self_ty;
    let other_items = &ir.other_items;
    let mod_ident = hidden_mod_ident(type_ident);

    let methods_static = emit_methods_static(type_ident, &ir.methods);
    let invoke_fn = emit_invoke_method(type_ident, &ir.methods);
    let meta_static = emit_meta_static(type_ident, &mod_ident);
    let object_impl = emit_object_impl(self_ty, type_ident, &mod_ident);

    quote! {
        impl #self_ty {
            #(#other_items)*
        }

        #methods_static
        #invoke_fn
        #meta_static
        #object_impl
    }
}

fn emit_methods_static(type_ident: &Ident, methods: &[MethodItem]) -> TokenStream {
    let static_name = Ident::new(&format!("__METHODS__{type_ident}"), type_ident.span());
    let entries = methods.iter().map(|m| {
        let name = m.ident.to_string();
        let params: Vec<TokenStream> = m
            .params
            .iter()
            .map(|p| {
                let param_name = p.ident.to_string();
                let ty = &p.ty;
                quote! {
                    ::quartzite_core::ParamMeta::new(#param_name, ::core::stringify!(#ty))
                }
            })
            .collect();
        let ret_str = match &m.ret_ty {
            ReturnType::Default => quote! { "()" },
            ReturnType::Type(_, ty) => quote! { ::core::stringify!(#ty) },
        };
        quote! {
            ::quartzite_core::MethodMeta::new(#name, &[#(#params),*], #ret_str)
        }
    });
    quote! {
        static #static_name: &'static [::quartzite_core::MethodMeta] = &[
            #(#entries),*
        ];
    }
}

fn emit_invoke_method(type_ident: &Ident, methods: &[MethodItem]) -> TokenStream {
    let fn_name = Ident::new(&format!("__invoke_method_{type_ident}"), type_ident.span());
    let arms = methods.iter().map(|m| {
        let name = m.ident.to_string();
        let method_ident = &m.ident;
        let n_params = m.params.len();
        let arg_bindings: Vec<TokenStream> = m
            .params
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let idx = Index::from(i);
                let binding = Ident::new(&format!("__arg{i}"), method_ident.span());
                quote! {
                    let #binding = match ::quartzite_core::FromValue::from_value(args[#idx].clone()) {
                        ::core::result::Result::Ok(v) => v,
                        ::core::result::Result::Err(_) => return ::core::option::Option::None,
                    };
                }
            })
            .collect();
        let arg_idents: Vec<Ident> = (0..n_params)
            .map(|i| Ident::new(&format!("__arg{i}"), method_ident.span()))
            .collect();
        let call_and_return = match &m.ret_ty {
            ReturnType::Default => quote! {
                this.#method_ident(#(#arg_idents),*);
                ::core::option::Option::Some(::quartzite_core::Value::Null)
            },
            ReturnType::Type(_, _) => quote! {
                ::core::option::Option::Some(
                    ::quartzite_core::IntoValue::into_value(this.#method_ident(#(#arg_idents),*))
                )
            },
        };
        quote! {
            #name => {
                if args.len() != #n_params {
                    return ::core::option::Option::None;
                }
                #(#arg_bindings)*
                #call_and_return
            }
        }
    });
    quote! {
        fn #fn_name(
            this: &mut #type_ident,
            name: &str,
            args: &[::quartzite_core::Value],
        ) -> ::core::option::Option<::quartzite_core::Value> {
            match name {
                #(#arms)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

fn emit_meta_static(type_ident: &Ident, mod_ident: &Ident) -> TokenStream {
    let meta_static_name = Ident::new(&format!("META_{type_ident}"), type_ident.span());
    let meta_init_fn = Ident::new(&format!("__meta_init_{type_ident}"), type_ident.span());
    let props_name = Ident::new(&format!("__PROPS__{type_ident}"), type_ident.span());
    let signals_name = Ident::new(&format!("__SIGNALS__{type_ident}"), type_ident.span());
    let methods_name = Ident::new(&format!("__METHODS__{type_ident}"), type_ident.span());
    quote! {
        static #meta_static_name: ::std::sync::OnceLock<::quartzite_core::MetaObject> =
            ::std::sync::OnceLock::new();

        fn #meta_init_fn() -> &'static ::quartzite_core::MetaObject {
            #meta_static_name.get_or_init(|| ::quartzite_core::MetaObject {
                class_name: ::core::stringify!(#type_ident),
                properties: #mod_ident::#props_name,
                signals: #mod_ident::#signals_name,
                methods: &#methods_name,
                enums: &[],
            })
        }
    }
}

fn emit_object_impl(self_ty: &syn::Type, type_ident: &Ident, mod_ident: &Ident) -> TokenStream {
    let read_fn = Ident::new(&format!("__read_property_{type_ident}"), type_ident.span());
    let write_fn = Ident::new(&format!("__write_property_{type_ident}"), type_ident.span());
    let connect_fn = Ident::new(
        &format!("__connect_signal_dynamic_{type_ident}"),
        type_ident.span(),
    );
    let invoke_fn = Ident::new(&format!("__invoke_method_{type_ident}"), type_ident.span());
    let meta_init = Ident::new(&format!("__meta_init_{type_ident}"), type_ident.span());
    quote! {
        impl ::quartzite_core::Object for #self_ty {
            fn meta_object(&self) -> &'static ::quartzite_core::MetaObject {
                #meta_init()
            }
            fn read_property(&self, name: &str) -> ::core::option::Option<::quartzite_core::Value> {
                #mod_ident::#read_fn(self, name)
            }
            fn write_property(&mut self, name: &str, val: ::quartzite_core::Value) -> bool {
                #mod_ident::#write_fn(self, name, val)
            }
            fn invoke_method(
                &mut self,
                name: &str,
                args: &[::quartzite_core::Value],
            ) -> ::core::option::Option<::quartzite_core::Value> {
                #invoke_fn(self, name, args)
            }
            fn connect_signal(
                &mut self,
                signal: &str,
                callback: ::quartzite_core::SignalCallback,
            ) -> ::core::option::Option<::quartzite_core::ConnectionId> {
                #mod_ident::#connect_fn(self, signal, callback)
            }
        }
    }
}
