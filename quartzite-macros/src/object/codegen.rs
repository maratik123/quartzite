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
    let lookup_prop_fn = emit_lookup_prop_fn(type_ident, &ir.props);
    let lookup_signal_fn = emit_lookup_signal_fn(type_ident, &ir.signals);
    let emit_wrappers = emit_signal_wrappers(type_ident, &ir.signals);

    quote! {
        #[doc(hidden)]
        #[allow(non_snake_case, non_upper_case_globals)]
        mod #mod_ident {
            #props_static
            #signals_static
            #read_fn
            #write_fn
            #connect_fn
            #lookup_prop_fn
            #lookup_signal_fn
        }

        #emit_wrappers
    }
}

fn props_static_ident(type_ident: &Ident) -> Ident {
    Ident::new(&format!("__PROPS__{type_ident}"), type_ident.span())
}

fn signals_static_ident(type_ident: &Ident) -> Ident {
    Ident::new(&format!("__SIGNALS__{type_ident}"), type_ident.span())
}

/// Extracts element types from a tuple type; returns a single-element vec for non-tuples.
fn tuple_elems(ty: &Type) -> Vec<&Type> {
    match ty {
        Type::Tuple(tt) => tt.elems.iter().collect(),
        _ => vec![ty],
    }
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
            ::quartzite::core::PropertyMeta::new(
                #name,
                ::core::stringify!(#ty),
                ::quartzite::core::PropertyFlags {
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
        pub const #static_name: &[::quartzite::core::PropertyMeta] = &[
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
                    ::quartzite::core::ParamMeta::new(#param_name, ::core::stringify!(#ty))
                }
            })
            .collect();
        quote! {
            ::quartzite::core::SignalMeta::new(#name, &[#(#params),*])
        }
    });
    quote! {
        pub const #static_name: &[::quartzite::core::SignalMeta] = &[
            #(#entries),*
        ];
    }
}

fn emit_lookup_prop_fn(type_ident: &Ident, props: &[PropField]) -> TokenStream {
    let fn_name = Ident::new(
        &format!("__lookup_property_{type_ident}"),
        type_ident.span(),
    );
    let static_name = props_static_ident(type_ident);
    let arms = props.iter().enumerate().map(|(idx, p)| {
        let name = p.ident.to_string();
        let idx_lit = Index::from(idx);
        quote! {
            #name => ::core::option::Option::Some(#static_name[#idx_lit])
        }
    });
    quote! {
        pub fn #fn_name(name: &str) -> ::core::option::Option<::quartzite::core::PropertyMeta> {
            match name {
                #(#arms,)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

fn emit_lookup_signal_fn(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    let fn_name = Ident::new(&format!("__lookup_signal_{type_ident}"), type_ident.span());
    let static_name = signals_static_ident(type_ident);
    let arms = signals.iter().enumerate().map(|(idx, s)| {
        let name = s.ident.to_string();
        let idx_lit = Index::from(idx);
        quote! {
            #name => ::core::option::Option::Some(#static_name[#idx_lit])
        }
    });
    quote! {
        pub fn #fn_name(name: &str) -> ::core::option::Option<::quartzite::core::SignalMeta> {
            match name {
                #(#arms,)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

fn emit_read_property(type_ident: &Ident, props: &[PropField]) -> TokenStream {
    let fn_name = Ident::new(&format!("__read_property_{type_ident}"), type_ident.span());
    let arms = props.iter().map(|p| {
        let name = p.ident.to_string();
        let field = &p.ident;
        quote! {
            #name => ::core::option::Option::Some(
                ::quartzite::core::IntoValue::into_value(this.#field.clone())
            )
        }
    });
    quote! {
        pub fn #fn_name(
            this: &super::#type_ident,
            name: &str,
        ) -> ::core::option::Option<::quartzite::core::Value> {
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
                // CONSTRAINT: the notify signal must be `Signal<(PropType,)>` — one
                // element tuple matching the property's type. Enforced at compile time.
                quote! {
                    if !::quartzite::core::AsObject::object_base(this).signals_blocked() {
                        let __notify_val = v.clone();
                        this.#sig_ident.emit(&(__notify_val,));
                    }
                }
            });
            quote! {
                #name => match ::quartzite::core::FromValue::from_value(val) {
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
            val: ::quartzite::core::Value,
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
                    ::quartzite::core::IntoValue::into_value(args.#idx.clone())
                }
            })
            .collect();
        quote! {
            #name => {
                let cb = ::quartzite::core::__macro::Arc::clone(&cb);
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
            cb: ::quartzite::core::SignalCallback,
        ) -> ::core::option::Option<::quartzite::core::ConnectionId> {
            let cb: ::quartzite::core::__macro::Arc<
                dyn ::core::ops::Fn(&[::quartzite::core::Value]) + Send + Sync,
            > = ::quartzite::core::__macro::Arc::from(cb);
            match name {
                #(#arms)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

fn emit_signal_wrappers(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    if signals.is_empty() {
        return quote! {};
    }
    let methods = signals.iter().map(|s| {
        let field = &s.ident;
        let fn_name = Ident::new(&format!("emit_{field}"), field.span());
        let args_ty = &s.args_ty;
        let elems = tuple_elems(args_ty);
        let params: Vec<TokenStream> = elems
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let arg = Ident::new(&format!("arg{i}"), field.span());
                quote! { #arg: #ty }
            })
            .collect();
        let arg_idents: Vec<Ident> = (0..elems.len())
            .map(|i| Ident::new(&format!("arg{i}"), field.span()))
            .collect();
        quote! {
            /// Emits this signal unless signals are blocked on this object.
            ///
            /// Checks [`quartzite::core::ObjectBase::signals_blocked`] before firing.
            /// Returns immediately without invoking any slots when blocked.
            #[inline]
            pub fn #fn_name(&mut self, #(#params),*) {
                if ::quartzite::core::AsObject::object_base(self).signals_blocked() {
                    return;
                }
                self.#field.emit(&(#(#arg_idents,)*));
            }
        }
    });
    quote! {
        impl #type_ident {
            #(#methods)*
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
        assert!(
            out.contains("__lookup_property_Foo"),
            "missing lookup_property fn: {out}"
        );
        assert!(
            out.contains("__lookup_signal_Foo"),
            "missing lookup_signal fn: {out}"
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

    // write_property notify is guarded by signals_blocked.
    #[test]
    fn write_property_notify_guarded_by_signals_blocked() {
        let out = emit(quote! {
            struct Foo {
                #[prop(notify = changed)]
                pub val: i32,
                #[signal]
                pub changed: Signal<(i32,)>,
            }
        });
        assert!(
            out.contains("signals_blocked"),
            "missing signals_blocked guard in write_property: {out}"
        );
        assert!(out.contains("changed . emit"), "missing notify emit: {out}");
    }

    // write_property without notify does NOT introduce a signals_blocked check.
    #[test]
    fn write_property_no_notify_no_guard() {
        let out = emit(quote! {
            struct Foo {
                #[prop]
                pub val: i32,
            }
        });
        assert!(
            !out.contains("signals_blocked"),
            "unexpected signals_blocked guard for prop without notify: {out}"
        );
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

    // lookup_prop_fn: match arm per property, indexing into __PROPS__ static.
    #[test]
    fn lookup_prop_fn_has_match_arm() {
        let out = emit(quote! {
            struct Foo {
                #[prop]
                pub score: i32,
            }
        });
        assert!(
            out.contains("fn __lookup_property_Foo"),
            "missing lookup fn: {out}"
        );
        assert!(
            out.contains("\"score\"") && out.contains("__PROPS__Foo [0]"),
            "missing match arm or index: {out}"
        );
    }

    // lookup_signal_fn: match arm per signal, indexing into __SIGNALS__ static.
    #[test]
    fn lookup_signal_fn_has_match_arm() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub ticked: Signal<(i32,)>,
            }
        });
        assert!(
            out.contains("fn __lookup_signal_Foo"),
            "missing lookup fn: {out}"
        );
        assert!(
            out.contains("\"ticked\"") && out.contains("__SIGNALS__Foo [0]"),
            "missing match arm or index: {out}"
        );
    }

    // lookup_prop_fn with no props produces a fn that always returns None.
    #[test]
    fn lookup_prop_fn_empty_returns_none() {
        let out = emit(quote! { struct Foo {} });
        assert!(
            out.contains("fn __lookup_property_Foo"),
            "missing lookup fn: {out}"
        );
        assert!(
            !out.contains("__PROPS__Foo ["),
            "unexpected index arm for empty props: {out}"
        );
    }

    // emit_signal_wrappers: wrapper fn present, contains signals_blocked guard.
    #[test]
    fn emit_wrappers_generated_for_signal() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub value_changed: Signal<(i32,)>,
            }
        });
        assert!(
            out.contains("pub fn emit_value_changed"),
            "missing emit wrapper: {out}"
        );
        assert!(
            out.contains("signals_blocked"),
            "missing signals_blocked guard: {out}"
        );
    }

    // No signals → no emit_ wrappers emitted.
    #[test]
    fn emit_wrappers_no_signals_no_block() {
        let out = emit(quote! {
            struct Foo {
                #[prop]
                pub count: i32,
            }
        });
        assert!(!out.contains("emit_"), "unexpected emit_ wrapper: {out}");
    }

    // Multi-arg signal: individual parameters are flattened.
    #[test]
    fn emit_wrappers_multi_arg_parameters_flattened() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub moved: Signal<(i32, bool)>,
            }
        });
        assert!(out.contains("arg0 : i32"), "missing arg0 param: {out}");
        assert!(out.contains("arg1 : bool"), "missing arg1 param: {out}");
    }

    // Zero-arg signal (Signal<()>): wrapper has no extra parameters.
    #[test]
    fn emit_wrappers_zero_arg_signal() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub activated: Signal<()>,
            }
        });
        assert!(
            out.contains("pub fn emit_activated"),
            "missing emit wrapper: {out}"
        );
        assert!(
            !out.contains("arg0"),
            "unexpected arg for zero-arg signal: {out}"
        );
    }

    // emit wrappers carry #[inline].
    #[test]
    fn emit_wrappers_inline_attribute_present() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub ticked: Signal<(i32,)>,
            }
        });
        assert!(
            out.contains("# [inline] pub fn emit_ticked"),
            "missing #[inline] on emit wrapper: {out}"
        );
    }

    // emit wrappers live outside the hidden module.
    // emit wrappers live outside the hidden module (in the outer impl block, not in mod __quartzite_Foo).
    #[test]
    fn emit_wrappers_live_outside_hidden_mod() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub ticked: Signal<(i32,)>,
            }
        });
        let mod_start = out.find("mod __quartzite_Foo").expect("hidden mod not found");
        let impl_start = out.find("impl Foo").expect("outer impl block not found");
        // The outer impl block must follow the hidden mod.
        assert!(
            impl_start > mod_start,
            "impl Foo block not after hidden mod: {out}"
        );
        // The mod section must not contain the wrapper.
        let mod_section = &out[mod_start..impl_start];
        assert!(
            !mod_section.contains("emit_ticked"),
            "emit_ticked found inside hidden mod section: {mod_section}"
        );
        // The outer impl block must contain the wrapper.
        assert!(
            out[impl_start..].contains("emit_ticked"),
            "emit_ticked not found in outer impl block: {out}"
        );
    }
}
