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

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    fn emit(ts: TokenStream) -> String {
        let ir = crate::object::parse::parse(ts).expect("parse ok");
        super::codegen(ir).to_string()
    }

    // Top-level: everything lives inside the hidden __quartzite_Foo module.
    #[test]
    fn codegen_wraps_in_hidden_mod() {
        let out = emit(quote! { struct Foo {} });
        assert!(
            out.contains("mod __quartzite_Foo"),
            "missing hidden mod: {out}"
        );
        assert!(out.contains("__PROPS__Foo"), "missing props static: {out}");
        assert!(
            out.contains("__SIGNALS__Foo"),
            "missing signals static: {out}"
        );
        assert!(
            out.contains("__read_property_Foo"),
            "missing read fn: {out}"
        );
        assert!(
            out.contains("__write_property_Foo"),
            "missing write fn: {out}"
        );
        assert!(
            out.contains("__connect_signal_dynamic_Foo"),
            "missing connect fn: {out}"
        );
    }

    // Props static: readable=true always; writable obeys read_only and constant flags.
    #[test]
    fn writable_prop_flags() {
        let out = emit(quote! {
            struct Foo {
                #[prop]
                pub count: i32,
            }
        });
        assert!(out.contains("readable : true"), "missing readable: {out}");
        assert!(out.contains("writable : true"), "missing writable: {out}");
        assert!(out.contains("notify : false"), "unexpected notify: {out}");
    }

    #[test]
    fn read_only_prop_has_writable_false() {
        let out = emit(quote! {
            struct Foo {
                #[prop(read_only)]
                pub val: i32,
            }
        });
        assert!(
            out.contains("writable : false"),
            "expected writable false: {out}"
        );
    }

    #[test]
    fn constant_prop_has_writable_false() {
        let out = emit(quote! {
            struct Foo {
                #[prop(constant)]
                pub val: i32,
            }
        });
        assert!(
            out.contains("writable : false"),
            "expected writable false: {out}"
        );
        assert!(out.contains("constant : true"), "missing constant: {out}");
    }

    #[test]
    fn notify_prop_has_notify_true() {
        let out = emit(quote! {
            struct Foo {
                #[prop(notify = changed)]
                pub val: i32,
                #[signal]
                pub changed: Signal<(i32,)>,
            }
        });
        assert!(out.contains("notify : true"), "missing notify flag: {out}");
    }

    // Signals static: name, auto-named params (arg0, arg1…), type stringify.
    #[test]
    fn signal_static_includes_name_and_params() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub value_changed: Signal<(i32,)>,
            }
        });
        assert!(
            out.contains("\"value_changed\""),
            "missing signal name: {out}"
        );
        assert!(out.contains("\"arg0\""), "missing param name: {out}");
        assert!(
            out.contains("stringify ! (i32)"),
            "missing param type: {out}"
        );
    }

    #[test]
    fn signal_multi_arg_names_incremented() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub moved: Signal<(i32, i32)>,
            }
        });
        assert!(out.contains("\"arg0\""), "missing arg0: {out}");
        assert!(out.contains("\"arg1\""), "missing arg1: {out}");
    }

    // read_property: known field → IntoValue; unknown → None.
    #[test]
    fn read_property_emits_into_value_arm() {
        let out = emit(quote! {
            struct Foo {
                #[prop]
                pub score: i32,
            }
        });
        assert!(out.contains("\"score\""), "missing prop arm: {out}");
        assert!(
            out.contains("IntoValue :: into_value"),
            "missing IntoValue: {out}"
        );
        assert!(
            out.contains("this . score . clone"),
            "missing field access: {out}"
        );
    }

    // write_property: writable uses FromValue; read_only arm returns false literal.
    #[test]
    fn write_property_writable_uses_from_value() {
        let out = emit(quote! {
            struct Foo {
                #[prop]
                pub score: i32,
            }
        });
        assert!(
            out.contains("FromValue :: from_value"),
            "missing FromValue: {out}"
        );
    }

    #[test]
    fn write_property_read_only_arm_is_false() {
        let out = emit(quote! {
            struct Foo {
                #[prop(read_only)]
                pub version: i32,
            }
        });
        assert!(
            out.contains("\"version\" => false"),
            "missing false arm: {out}"
        );
        assert!(
            !out.contains("FromValue"),
            "unexpected FromValue for read_only: {out}"
        );
    }

    #[test]
    fn write_property_notify_emits_signal_call() {
        let out = emit(quote! {
            struct Foo {
                #[prop(notify = changed)]
                pub val: i32,
                #[signal]
                pub changed: Signal<(i32,)>,
            }
        });
        assert!(out.contains("changed . emit"), "missing signal emit: {out}");
        assert!(out.contains("__notify_val"), "missing notify val: {out}");
    }

    // connect_signal: signal name in match, Arc::from, per-arm Arc::clone, conversion closure.
    #[test]
    fn connect_signal_emits_arc_from_and_clone() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub ticked: Signal<(i32,)>,
            }
        });
        assert!(out.contains("\"ticked\""), "missing signal arm: {out}");
        assert!(out.contains("Arc :: from"), "missing Arc::from: {out}");
        assert!(out.contains("Arc :: clone"), "missing Arc::clone: {out}");
        assert!(
            out.contains("IntoValue :: into_value"),
            "missing conversion: {out}"
        );
    }
}

/// Extracts element types from a tuple type; returns a single-element vec for non-tuples.
fn tuple_elems(ty: &Type) -> Vec<&Type> {
    match ty {
        Type::Tuple(tt) => tt.elems.iter().collect(),
        _ => vec![ty],
    }
}
