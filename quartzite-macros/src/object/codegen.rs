use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Index, Type};

use super::parse::{ObjectInput, PropField, SignalField};
use crate::util::hidden_mod_ident;

pub(crate) fn codegen(ir: ObjectInput) -> TokenStream {
    let type_ident = &ir.ident;
    let mod_ident = hidden_mod_ident(type_ident);

    let props_static = emit_props_static(type_ident, &ir.props);
    let signals_static = emit_signals_static(type_ident, &ir.signals);
    let read_fn = emit_read_property(type_ident, &ir.props);
    let write_fn = emit_write_property(type_ident, &ir.props);
    let connect_fn = emit_connect_signal_dynamic(type_ident, &ir.signals);

    quote! {
        #[doc(hidden)]
        #[allow(non_snake_case, non_upper_case_globals)]
        mod #mod_ident {
            #props_static
            #signals_static
            #read_fn
            #write_fn
            #connect_fn
        }
    }
}

fn props_static_ident(type_ident: &Ident) -> Ident {
    Ident::new(&format!("__PROPS__{type_ident}"), type_ident.span())
}

fn signals_static_ident(type_ident: &Ident) -> Ident {
    Ident::new(&format!("__SIGNALS__{type_ident}"), type_ident.span())
}

fn emit_props_static(type_ident: &Ident, props: &[PropField]) -> TokenStream {
    let static_name = props_static_ident(type_ident);
    let entries = props.iter().map(|p| {
        let name = p.ident.to_string();
        let ty = &p.ty;
        let readable = true;
        let writable = !p.read_only && !p.constant;
        let notify = p.notify.is_some();
        let stored = p.stored;
        let designable = p.designable;
        let user = p.user;
        let constant = p.constant;
        quote! {
            ::quartzite_core::PropertyMeta::new(
                #name,
                ::core::stringify!(#ty),
                ::quartzite_core::PropertyFlags {
                    readable: #readable,
                    writable: #writable,
                    notify: #notify,
                    stored: #stored,
                    designable: #designable,
                    user: #user,
                    constant: #constant,
                },
            )
        }
    });
    quote! {
        pub static #static_name: &'static [::quartzite_core::PropertyMeta] = &[
            #(#entries),*
        ];
    }
}

fn emit_signals_static(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    let static_name = signals_static_ident(type_ident);
    let entries = signals.iter().map(|s| {
        let name = s.ident.to_string();
        let params: Vec<TokenStream> = tuple_elems(&s.args_ty)
            .into_iter()
            .enumerate()
            .map(|(i, ty)| {
                let param_name = format!("arg{i}");
                quote! {
                    ::quartzite_core::ParamMeta::new(#param_name, ::core::stringify!(#ty))
                }
            })
            .collect();
        quote! {
            ::quartzite_core::SignalMeta::new(#name, &[#(#params),*])
        }
    });
    quote! {
        pub static #static_name: &'static [::quartzite_core::SignalMeta] = &[
            #(#entries),*
        ];
    }
}

fn emit_read_property(type_ident: &Ident, props: &[PropField]) -> TokenStream {
    let fn_name = Ident::new(&format!("__read_property_{type_ident}"), type_ident.span());
    let arms = props.iter().map(|p| {
        let name = p.ident.to_string();
        let field = &p.ident;
        quote! {
            #name => ::core::option::Option::Some(
                ::quartzite_core::IntoValue::into_value(this.#field.clone())
            )
        }
    });
    quote! {
        pub fn #fn_name(
            this: &super::#type_ident,
            name: &str,
        ) -> ::core::option::Option<::quartzite_core::Value> {
            match name {
                #(#arms,)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

fn emit_write_property(type_ident: &Ident, props: &[PropField]) -> TokenStream {
    let fn_name = Ident::new(&format!("__write_property_{type_ident}"), type_ident.span());
    let arms = props.iter().map(|p| {
        let name = p.ident.to_string();
        let field = &p.ident;
        if p.read_only || p.constant {
            quote! { #name => false }
        } else {
            let notify_emit = p.notify.as_ref().map(|sig_ident| {
                quote! {
                    let __notify_val = v.clone();
                    this.#sig_ident.emit(&(__notify_val,));
                }
            });
            quote! {
                #name => match ::quartzite_core::FromValue::from_value(val) {
                    ::core::result::Result::Ok(v) => {
                        this.#field = v;
                        #notify_emit
                        true
                    }
                    ::core::result::Result::Err(_) => false,
                }
            }
        }
    });
    quote! {
        pub fn #fn_name(
            this: &mut super::#type_ident,
            name: &str,
            val: ::quartzite_core::Value,
        ) -> bool {
            match name {
                #(#arms,)*
                _ => false,
            }
        }
    }
}

fn emit_connect_signal_dynamic(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    let fn_name = Ident::new(
        &format!("__connect_signal_dynamic_{type_ident}"),
        type_ident.span(),
    );
    let arms = signals.iter().map(|s| {
        let name = s.ident.to_string();
        let field = &s.ident;
        let args_ty = &s.args_ty;
        let conversions: Vec<TokenStream> = tuple_elems(args_ty)
            .into_iter()
            .enumerate()
            .map(|(i, _ty)| {
                let idx = Index::from(i);
                quote! {
                    ::quartzite_core::IntoValue::into_value(args.#idx.clone())
                }
            })
            .collect();
        quote! {
            #name => {
                let cb = ::std::sync::Arc::clone(&cb);
                ::core::option::Option::Some(this.#field.connect(move |args: &#args_ty| {
                    (*cb)(&[#(#conversions),*])
                }))
            }
        }
    });
    quote! {
        pub fn #fn_name(
            this: &mut super::#type_ident,
            name: &str,
            cb: ::std::boxed::Box<dyn ::core::ops::Fn(&[::quartzite_core::Value])>,
        ) -> ::core::option::Option<::quartzite_core::ConnectionId> {
            let cb: ::std::sync::Arc<dyn ::core::ops::Fn(&[::quartzite_core::Value])> = ::std::sync::Arc::from(cb);
            match name {
                #(#arms)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

/// Extracts element types from a tuple type; returns a single-element vec for non-tuples.
fn tuple_elems(ty: &Type) -> Vec<&Type> {
    match ty {
        Type::Tuple(tt) => tt.elems.iter().collect(),
        _ => vec![ty],
    }
}
