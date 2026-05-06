use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Index, Type};

use super::parse::{ObjectInput, PropField, SignalField};
use crate::util::{crate_root, hidden_mod_ident};

pub(crate) fn codegen(ir: ObjectInput) -> TokenStream {
    let cr = crate_root();
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
    let connect_auto_wrappers = emit_connect_auto_wrappers(type_ident, &ir.signals);
    let connect_queued_wrappers = emit_connect_queued_wrappers(type_ident, &ir.signals);

    quote! {
        #[doc(hidden)]
        #[allow(non_snake_case, non_upper_case_globals)]
        mod #mod_ident {
            use #cr::PropertyFlag;
            #props_static
            #signals_static
            #read_fn
            #write_fn
            #connect_fn
            #lookup_prop_fn
            #lookup_signal_fn
        }

        #emit_wrappers
        #connect_auto_wrappers
        #connect_queued_wrappers
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
    let cr = crate_root();
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
        // Build variant list at proc-macro time; make_bitflags! takes a bare ident (not a
        // qualified path), so PropertyFlag must be in scope via `use` in the hidden module.
        let flag_variants: Vec<TokenStream> = [
            (readable, quote!(Readable)),
            (writable, quote!(Writable)),
            (notify, quote!(Notify)),
            (stored, quote!(Stored)),
            (designable, quote!(Designable)),
            (user, quote!(User)),
            (constant, quote!(Constant)),
        ]
        .into_iter()
        .filter_map(|(active, tok)| active.then_some(tok))
        .collect();
        quote! {
            #cr::PropertyMeta::new(
                #name,
                ::core::stringify!(#ty),
                #cr::enumflags2::make_bitflags!(PropertyFlag::{#(#flag_variants)|*}),
            )
        }
    });
    quote! {
        pub const #static_name: &[#cr::PropertyMeta] = &[
            #(#entries),*
        ];
    }
}

fn emit_signals_static(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    let cr = crate_root();
    let static_name = signals_static_ident(type_ident);
    let entries = signals.iter().map(|s| {
        let name = s.ident.to_string();
        let params: Vec<TokenStream> = tuple_elems(&s.args_ty)
            .into_iter()
            .enumerate()
            .map(|(i, ty)| {
                let param_name = format!("arg{i}");
                quote! {
                    #cr::ParamMeta::new(#param_name, ::core::stringify!(#ty))
                }
            })
            .collect();
        quote! {
            #cr::SignalMeta::new(#name, &[#(#params),*])
        }
    });
    quote! {
        pub const #static_name: &[#cr::SignalMeta] = &[
            #(#entries),*
        ];
    }
}

fn emit_lookup_prop_fn(type_ident: &Ident, props: &[PropField]) -> TokenStream {
    let cr = crate_root();
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
        pub fn #fn_name(name: &str) -> ::core::option::Option<#cr::PropertyMeta> {
            match name {
                #(#arms,)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

fn emit_lookup_signal_fn(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    let cr = crate_root();
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
        pub fn #fn_name(name: &str) -> ::core::option::Option<#cr::SignalMeta> {
            match name {
                #(#arms,)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

fn emit_read_property(type_ident: &Ident, props: &[PropField]) -> TokenStream {
    let cr = crate_root();
    let fn_name = Ident::new(&format!("__read_property_{type_ident}"), type_ident.span());
    let arms = props.iter().map(|p| {
        let name = p.ident.to_string();
        let field = &p.ident;
        quote! {
            #name => ::core::option::Option::Some(
                #cr::IntoValue::into_value(this.#field.clone())
            )
        }
    });
    quote! {
        pub fn #fn_name(
            this: &super::#type_ident,
            name: &str,
        ) -> ::core::option::Option<#cr::Value> {
            match name {
                #(#arms,)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

fn emit_write_property(type_ident: &Ident, props: &[PropField]) -> TokenStream {
    let cr = crate_root();
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
                    let __notify_val = v.clone();
                    #cr::emit!(this.#sig_ident, &(__notify_val,));
                }
            });
            quote! {
                #name => match #cr::FromValue::from_value(val) {
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
            val: #cr::Value,
        ) -> bool {
            match name {
                #(#arms,)*
                _ => false,
            }
        }
    }
}

fn emit_connect_signal_dynamic(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    let cr = crate_root();
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
                    #cr::IntoValue::into_value(args.#idx.clone())
                }
            })
            .collect();
        quote! {
            #name => {
                let cb = #cr::__macro::Arc::clone(&cb);
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
            cb: #cr::SignalCallback,
        ) -> ::core::option::Option<#cr::ConnectionId> {
            let cb: #cr::__macro::Arc<
                dyn ::core::ops::Fn(&[#cr::Value]) + Send + Sync,
            > = #cr::__macro::Arc::from(cb);
            match name {
                #(#arms)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

fn emit_signal_wrappers(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    let cr = crate_root();
    if signals.is_empty() {
        return quote! {};
    }
    let methods = signals.iter().map(|s| {
        let field = &s.ident;
        let fn_name = Ident::new(&format!("emit_{field}"), field.span());
        let fn_name_str = fn_name.to_string();
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
        let parameters_doc = if elems.is_empty() {
            quote! {}
        } else {
            let bullets = (0..elems.len()).map(|i| {
                let bullet = format!(
                    " - `arg{i}`: the {i}-th positional argument forwarded to slots."
                );
                quote! { #[doc = #bullet] }
            });
            quote! {
                #[doc = ""]
                #[doc = " # Parameters"]
                #[doc = ""]
                #(#bullets)*
            }
        };
        let example_args = (0..elems.len())
            .map(|i| format!("arg{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let example_doc = format!(
            " # Examples\n\n```no_run\n# fn example(obj: &mut Emitter) {{\n#     obj.{fn_name_str}({example_args});\n# }}\n```"
        );
        quote! {
            #[doc = " Emits this signal unless signals are blocked on this object."]
            #parameters_doc
            #[doc = ""]
            #[doc = #example_doc]
            #[inline]
            pub fn #fn_name(&mut self, #(#params),*) {
                #cr::emit!(self.#field, &(#(#arg_idents,)*));
            }
        }
    });
    quote! {
        impl #type_ident {
            #(#methods)*
        }
    }
}

fn emit_connect_auto_wrappers(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    let cr = crate_root();
    if signals.is_empty() {
        return quote! {};
    }
    let methods = signals.iter().map(|s| {
        let field = &s.ident;
        let fn_name = Ident::new(&format!("connect_{field}_auto"), field.span());
        let fn_name_str = fn_name.to_string();
        let args_ty = &s.args_ty;
        let example_doc = format!(
            " # Examples\n\n```no_run\n# fn example(obj: &mut Emitter, receiver: &quartzite::core::ObjectBase) {{\n#     obj.{fn_name_str}(receiver, |_| {{}});\n# }}\n```"
        );
        quote! {
            #[doc = " Connects this signal to a slot with `Auto` delivery."]
            #[doc = ""]
            #[doc = " Same-thread emits call `f` directly; cross-thread emits post to the dispatcher."]
            #[doc = " The slot is silently skipped once `receiver` has been dropped."]
            #[doc = ""]
            #[doc = " # Parameters"]
            #[doc = ""]
            #[doc = " - `receiver`: object whose [`quartzite::core::ReceiverGuard`] keeps the slot alive; the slot is silently skipped once the guard is dropped."]
            #[doc = " - `f`: closure invoked on each emit with the signal's argument tuple."]
            #[doc = ""]
            #[doc = #example_doc]
            #[cfg(feature = "std")]
            #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
            #[inline]
            pub fn #fn_name<F>(
                &mut self,
                receiver: &#cr::ObjectBase,
                f: F,
            ) -> #cr::ConnectionId
            where
                F: ::core::ops::Fn(#args_ty) + ::core::marker::Send + ::core::marker::Sync + 'static,
            {
                self.#field.connect_auto(
                    receiver.thread_id,
                    ::std::sync::Arc::downgrade(receiver.receiver_guard()),
                    f,
                )
            }
        }
    });
    quote! {
        // `#[cfg(feature = "std")]` is evaluated against the destination crate.
        // `#[allow(unexpected_cfgs)]` prevents a check-cfg warning in crates
        // that do not declare `std` as an explicit feature.
        #[allow(unexpected_cfgs)]
        impl #type_ident {
            #(#methods)*
        }
    }
}

fn emit_connect_queued_wrappers(type_ident: &Ident, signals: &[SignalField]) -> TokenStream {
    let cr = crate_root();
    if signals.is_empty() {
        return quote! {};
    }
    let methods = signals.iter().map(|s| {
        let field = &s.ident;
        let fn_name = Ident::new(&format!("connect_{field}_queued"), field.span());
        let fn_name_str = fn_name.to_string();
        let args_ty = &s.args_ty;
        let example_doc = format!(
            " # Examples\n\n```no_run\n# fn example(obj: &mut Receiver, receiver: &quartzite::core::ObjectBase) {{\n#     obj.{fn_name_str}(receiver, |_| {{}});\n# }}\n```"
        );
        quote! {
            #[doc = " Connects this signal to a slot with `Queued` delivery."]
            #[doc = ""]
            #[doc = " The slot is always posted to the receiver's dispatcher, even when emitted"]
            #[doc = " from the same thread. The slot is silently skipped once `receiver` has been"]
            #[doc = " dropped."]
            #[doc = ""]
            #[doc = " # Parameters"]
            #[doc = ""]
            #[doc = " - `receiver`: object whose [`quartzite::core::ReceiverGuard`] keeps the slot alive; the slot is silently skipped once the guard is dropped."]
            #[doc = " - `f`: closure posted to the receiver's [`quartzite::core::QueuedDispatcher`] on each emit."]
            #[doc = ""]
            #[doc = #example_doc]
            #[cfg(feature = "std")]
            #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
            #[inline]
            pub fn #fn_name<F>(
                &mut self,
                receiver: &#cr::ObjectBase,
                f: F,
            ) -> #cr::ConnectionId
            where
                F: ::core::ops::Fn(#args_ty) + ::core::marker::Send + ::core::marker::Sync + 'static,
            {
                self.#field.connect_queued(
                    f,
                    ::std::sync::Arc::downgrade(receiver.receiver_guard()),
                )
            }
        }
    });
    quote! {
        // `#[cfg(feature = "std")]` is evaluated against the destination crate.
        // `#[allow(unexpected_cfgs)]` prevents a check-cfg warning in crates
        // that do not declare `std` as an explicit feature.
        #[allow(unexpected_cfgs)]
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
        assert!(
            out.contains("make_bitflags"),
            "missing make_bitflags: {out}"
        );
        assert!(
            out.contains("use :: quartzite :: core :: PropertyFlag"),
            "missing PropertyFlag use import: {out}"
        );
        assert!(out.contains("Readable"), "missing Readable flag: {out}");
        assert!(out.contains("Writable"), "missing Writable flag: {out}");
        assert!(!out.contains("Notify"), "unexpected Notify flag: {out}");
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
            out.contains("make_bitflags"),
            "missing make_bitflags: {out}"
        );
        assert!(!out.contains("Writable"), "expected Writable absent: {out}");
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
            out.contains("make_bitflags"),
            "missing make_bitflags: {out}"
        );
        assert!(!out.contains("Writable"), "expected Writable absent: {out}");
        assert!(out.contains("Constant"), "missing Constant flag: {out}");
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
        assert!(
            out.contains("make_bitflags"),
            "missing make_bitflags: {out}"
        );
        assert!(out.contains("Notify"), "missing Notify flag: {out}");
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
        assert!(
            out.contains("emit !"),
            "missing emit! macro call on notify signal: {out}"
        );
        assert!(out.contains("__notify_val"), "missing notify val: {out}");
    }

    // write_property notify uses emit! macro (not direct .emit with signals_blocked arg).
    #[test]
    fn write_property_notify_uses_emit_macro() {
        let out = emit(quote! {
            struct Foo {
                #[prop(notify = changed)]
                pub val: i32,
                #[signal]
                pub changed: Signal<(i32,)>,
            }
        });
        assert!(
            out.contains("emit !"),
            "missing emit! macro call in write_property notify: {out}"
        );
        assert!(
            !out.contains("signals_blocked"),
            "unexpected signals_blocked in write_property notify (guard is in emit! macro): {out}"
        );
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

    // emit_signal_wrappers: wrapper fn present, uses emit! macro (no signals_blocked arg).
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
            out.contains("emit !"),
            "missing emit! macro call in emit wrapper: {out}"
        );
        assert!(
            !out.contains("signals_blocked"),
            "unexpected signals_blocked in emit wrapper (guard is in emit! macro): {out}"
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
    // connect_auto wrappers: generated for each signal, gated cfg(feature="std"), #[inline].
    #[test]
    fn connect_auto_wrapper_generated_for_signal() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub value_changed: Signal<(i32,)>,
            }
        });
        assert!(
            out.contains("pub fn connect_value_changed_auto"),
            "missing connect_auto wrapper: {out}"
        );
        assert!(
            out.contains("cfg (feature = \"std\")"),
            "missing cfg(feature=\"std\") gate: {out}"
        );
        assert!(
            out.contains("# [inline] pub fn connect_value_changed_auto"),
            "missing #[inline] on connect_auto wrapper: {out}"
        );
        assert!(
            out.contains("receiver : & :: quartzite :: core :: ObjectBase"),
            "missing receiver param: {out}"
        );
        assert!(
            out.contains("connect_auto"),
            "wrapper must delegate to connect_auto: {out}"
        );
        assert!(
            out.contains("receiver . thread_id"),
            "wrapper must pass receiver.thread_id to connect_auto: {out}"
        );
        assert!(
            out.contains("receiver . receiver_guard ()"),
            "wrapper must pass receiver.receiver_guard() to connect_auto: {out}"
        );
    }

    // connect_auto wrappers live outside the hidden module.
    #[test]
    fn connect_auto_wrapper_lives_outside_hidden_mod() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub ticked: Signal<(i32,)>,
            }
        });
        let mod_start = out
            .find("mod __quartzite_Foo")
            .expect("hidden mod not found");
        // Three `impl Foo` blocks: emit wrappers [0], connect_auto wrappers [1],
        // connect_queued wrappers [2]. The auto block is at index 1.
        let positions: Vec<usize> = out.match_indices("impl Foo").map(|(i, _)| i).collect();
        assert_eq!(positions.len(), 3, "expected 3 impl Foo blocks: {out}");
        let auto_impl = positions[1];
        let queued_impl = positions[2];
        assert!(
            auto_impl > mod_start,
            "connect_auto impl Foo block not after hidden mod: {out}"
        );
        let mod_section = &out[mod_start..auto_impl];
        assert!(
            !mod_section.contains("connect_ticked_auto"),
            "connect_ticked_auto found inside hidden mod section: {mod_section}"
        );
        let auto_section = &out[auto_impl..queued_impl];
        assert!(
            auto_section.contains("connect_ticked_auto"),
            "connect_ticked_auto not found in auto impl block: {auto_section}"
        );
    }

    // No signals → no connect_auto wrappers.
    #[test]
    fn connect_auto_wrapper_absent_with_no_signals() {
        let out = emit(quote! {
            struct Foo {
                #[prop]
                pub count: i32,
            }
        });
        assert!(
            !out.contains("connect_auto"),
            "unexpected connect_auto wrapper for no-signal struct: {out}"
        );
    }

    // connect_queued wrappers: generated for each signal, gated cfg(feature="std"), #[inline].
    #[test]
    fn connect_queued_wrapper_generated_for_signal() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub value_changed: Signal<(i32,)>,
            }
        });
        assert!(
            out.contains("pub fn connect_value_changed_queued"),
            "missing connect_queued wrapper: {out}"
        );
        assert!(
            out.contains("cfg (feature = \"std\")"),
            "missing cfg(feature=\"std\") gate: {out}"
        );
        assert!(
            out.contains("# [inline] pub fn connect_value_changed_queued"),
            "missing #[inline] on connect_queued wrapper: {out}"
        );
        assert!(
            out.contains("receiver : & :: quartzite :: core :: ObjectBase"),
            "missing receiver param: {out}"
        );
        assert!(
            out.contains("connect_queued"),
            "wrapper must delegate to connect_queued: {out}"
        );
        assert!(
            out.contains("receiver . receiver_guard ()"),
            "wrapper must pass receiver.receiver_guard() to connect_queued: {out}"
        );
        assert!(
            out.contains("allow (unexpected_cfgs)"),
            "missing #[allow(unexpected_cfgs)] on impl block: {out}"
        );
        assert!(
            out.contains("no_run"),
            "missing # Examples no_run fence in generated doc: {out}"
        );
    }

    // connect_queued wrappers live outside the hidden module.
    #[test]
    fn connect_queued_wrapper_lives_outside_hidden_mod() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub ticked: Signal<(i32,)>,
            }
        });
        let mod_start = out
            .find("mod __quartzite_Foo")
            .expect("hidden mod not found");
        // The connect_queued impl block is the last (third) impl block.
        let last_impl = out.rfind("impl Foo").expect("outer impl block not found");
        assert!(
            last_impl > mod_start,
            "connect_queued impl Foo block not after hidden mod: {out}"
        );
        let mod_section = &out[mod_start..last_impl];
        assert!(
            !mod_section.contains("connect_ticked_queued"),
            "connect_ticked_queued found inside hidden mod section: {mod_section}"
        );
        assert!(
            out[last_impl..].contains("connect_ticked_queued"),
            "connect_ticked_queued not found in outer impl block: {out}"
        );
    }

    // No signals → no connect_queued wrappers.
    #[test]
    fn connect_queued_wrapper_absent_with_no_signals() {
        let out = emit(quote! {
            struct Foo {
                #[prop]
                pub count: i32,
            }
        });
        assert!(
            !out.contains("connect_queued"),
            "unexpected connect_queued wrapper for no-signal struct: {out}"
        );
    }

    // emit wrappers live outside the hidden module (in the outer impl block, not in mod __quartzite_Foo).
    #[test]
    fn emit_wrappers_live_outside_hidden_mod() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub ticked: Signal<(i32,)>,
            }
        });
        let mod_start = out
            .find("mod __quartzite_Foo")
            .expect("hidden mod not found");
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

    // Doc-convention contract (TDD lock for subtask 7): `emit_<sig>` wrappers
    // must carry both `# Parameters` and `# Examples` sections in their docs.
    #[test]
    fn emit_wrapper_doc_contains_parameters_and_examples() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub moved: Signal<(i32, i32)>,
            }
        });
        assert!(
            out.contains("# Parameters"),
            "missing # Parameters in emit_<sig> wrapper doc: {out}"
        );
        assert!(
            out.contains("# Examples"),
            "missing # Examples in emit_<sig> wrapper doc: {out}"
        );
    }

    // Doc-convention contract (TDD lock for subtask 7): `connect_<sig>_auto`
    // wrappers must carry both `# Parameters` and `# Examples` sections.
    #[test]
    fn connect_auto_wrapper_doc_contains_parameters_and_examples() {
        let out = emit(quote! {
            struct Foo {
                #[signal]
                pub ticked: Signal<(i32,)>,
            }
        });
        // Locate the connect_auto impl block to scope the assertion: there are
        // three `impl Foo` blocks (emit, connect_auto, connect_queued); the
        // connect_queued block already contains `# Examples`, so a workspace-
        // wide `out.contains` would pass on it. Restrict to the auto block.
        let positions: Vec<usize> = out.match_indices("impl Foo").map(|(i, _)| i).collect();
        assert_eq!(
            positions.len(),
            3,
            "expected 3 impl Foo blocks (emit, connect_auto, connect_queued): {out}"
        );
        let auto_section = &out[positions[1]..positions[2]];
        assert!(
            auto_section.contains("# Parameters"),
            "missing # Parameters in connect_<sig>_auto wrapper doc: {auto_section}"
        );
        assert!(
            auto_section.contains("# Examples"),
            "missing # Examples in connect_<sig>_auto wrapper doc: {auto_section}"
        );
    }
}
