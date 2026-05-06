use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Index, ReturnType};

use super::parse::{MethodItem, ObjectImplInput};
use crate::util::{crate_root, hidden_mod_ident};

pub(crate) fn codegen(ir: ObjectImplInput) -> TokenStream {
    let type_ident = &ir.self_ty_ident;
    let self_ty = &ir.self_ty;
    let other_items = &ir.other_items;
    let mod_ident = hidden_mod_ident(type_ident);

    let methods_static = emit_methods_static(type_ident, &ir.methods);
    let invoke_fn = emit_invoke_method(type_ident, &ir.methods);
    let lookup_fns = emit_lookup_fns(type_ident, &ir.methods);
    let meta_static = emit_meta_static(type_ident, &mod_ident);
    let object_impl = emit_object_impl(self_ty, type_ident, &mod_ident);

    let impl_block = emit_impl_block(&ir.trait_path, self_ty, other_items);

    quote! {
        #impl_block

        #methods_static
        #invoke_fn
        #lookup_fns
        #meta_static
        #object_impl
    }
}

pub(crate) fn emit_impl_block(
    trait_path: &Option<syn::Path>,
    self_ty: &syn::Type,
    other_items: &[syn::ImplItem],
) -> TokenStream {
    if let Some(tp) = trait_path {
        quote! { impl #tp for #self_ty { #(#other_items)* } }
    } else {
        quote! { impl #self_ty { #(#other_items)* } }
    }
}

pub(crate) fn emit_methods_static(type_ident: &Ident, methods: &[MethodItem]) -> TokenStream {
    let cr = crate_root();
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
                    #cr::ParamMeta::new(#param_name, ::core::stringify!(#ty))
                }
            })
            .collect();
        let ret_str = match &m.ret_ty {
            ReturnType::Default => quote! { "()" },
            ReturnType::Type(_, ty) => quote! { ::core::stringify!(#ty) },
        };
        quote! {
            #cr::MethodMeta::new(#name, &[#(#params),*], #ret_str)
        }
    });
    quote! {
        #[allow(non_upper_case_globals)]
        const #static_name: &[#cr::MethodMeta] = &[
            #(#entries),*
        ];
    }
}

pub(crate) fn emit_invoke_method(type_ident: &Ident, methods: &[MethodItem]) -> TokenStream {
    let cr = crate_root();
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
                    let #binding = match #cr::FromValue::from_value(args[#idx].clone()) {
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
                ::core::option::Option::Some(#cr::Value::Null)
            },
            ReturnType::Type(_, _) => quote! {
                ::core::option::Option::Some(
                    #cr::IntoValue::into_value(this.#method_ident(#(#arg_idents),*))
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
        #[allow(non_snake_case)]
        fn #fn_name(
            this: &mut #type_ident,
            name: &str,
            args: &[#cr::Value],
        ) -> ::core::option::Option<#cr::Value> {
            match name {
                #(#arms)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

pub(crate) fn emit_lookup_fns(type_ident: &Ident, methods: &[MethodItem]) -> TokenStream {
    let cr = crate_root();
    let lookup_method_fn = Ident::new(&format!("__lookup_method_{type_ident}"), type_ident.span());
    let lookup_enum_fn = Ident::new(&format!("__lookup_enum_{type_ident}"), type_ident.span());
    let methods_name = Ident::new(&format!("__METHODS__{type_ident}"), type_ident.span());

    // Each arm indexes into the 'static __METHODS__ slice — no temporaries.
    // Property and signal lookup fns live in the hidden mod generated by #[derive(Object)].
    let method_arms = methods.iter().enumerate().map(|(idx, m)| {
        let name = m.ident.to_string();
        let idx_lit = syn::Index::from(idx);
        quote! {
            #name => ::core::option::Option::Some(#methods_name[#idx_lit])
        }
    });

    quote! {
        #[allow(non_snake_case)]
        fn #lookup_method_fn(name: &str) -> ::core::option::Option<#cr::MethodMeta> {
            match name {
                #(#method_arms,)*
                _ => ::core::option::Option::None,
            }
        }

        #[allow(non_snake_case)]
        fn #lookup_enum_fn(_name: &str) -> ::core::option::Option<#cr::EnumMeta> {
            ::core::option::Option::None
        }
    }
}

pub(crate) fn emit_meta_static(type_ident: &Ident, mod_ident: &Ident) -> TokenStream {
    let cr = crate_root();
    let meta_static_name = Ident::new(&format!("META_{type_ident}"), type_ident.span());
    let meta_init_fn = Ident::new(&format!("__meta_init_{type_ident}"), type_ident.span());
    let props_name = Ident::new(&format!("__PROPS__{type_ident}"), type_ident.span());
    let signals_name = Ident::new(&format!("__SIGNALS__{type_ident}"), type_ident.span());
    let methods_name = Ident::new(&format!("__METHODS__{type_ident}"), type_ident.span());
    let lookup_prop_fn = Ident::new(
        &format!("__lookup_property_{type_ident}"),
        type_ident.span(),
    );
    let lookup_sig_fn = Ident::new(&format!("__lookup_signal_{type_ident}"), type_ident.span());
    let lookup_method_fn = Ident::new(&format!("__lookup_method_{type_ident}"), type_ident.span());
    let lookup_enum_fn = Ident::new(&format!("__lookup_enum_{type_ident}"), type_ident.span());
    // MetaObject::new is const fn and all its arguments are const — no OnceLock needed.
    // Property and signal lookup fns live in the hidden mod (generated by #[derive(Object)]).
    quote! {
        #[allow(non_upper_case_globals)]
        static #meta_static_name: #cr::MetaObject =
            #cr::MetaObject::new(
                ::core::stringify!(#type_ident),
                #mod_ident::#props_name,
                #mod_ident::#signals_name,
                #methods_name,
                &[],
                #mod_ident::#lookup_prop_fn,
                #mod_ident::#lookup_sig_fn,
                #lookup_method_fn,
                #lookup_enum_fn,
            );

        #[allow(non_snake_case)]
        #[inline]
        fn #meta_init_fn() -> &'static #cr::MetaObject {
            &#meta_static_name
        }
    }
}

pub(crate) fn emit_object_impl(
    self_ty: &syn::Type,
    type_ident: &Ident,
    mod_ident: &Ident,
) -> TokenStream {
    let cr = crate_root();
    let read_fn = Ident::new(&format!("__read_property_{type_ident}"), type_ident.span());
    let write_fn = Ident::new(&format!("__write_property_{type_ident}"), type_ident.span());
    let connect_fn = Ident::new(
        &format!("__connect_signal_dynamic_{type_ident}"),
        type_ident.span(),
    );
    let emit_signal_fn = Ident::new(&format!("__emit_signal_{type_ident}"), type_ident.span());
    let invoke_fn = Ident::new(&format!("__invoke_method_{type_ident}"), type_ident.span());
    let meta_init = Ident::new(&format!("__meta_init_{type_ident}"), type_ident.span());
    quote! {
        impl #cr::Object for #self_ty {
            #[inline]
            fn meta_object(&self) -> &'static #cr::MetaObject {
                #meta_init()
            }
            #[inline]
            fn read_property(&self, name: &str) -> ::core::option::Option<#cr::Value> {
                #mod_ident::#read_fn(self, name)
            }
            #[inline]
            fn write_property(&mut self, name: &str, val: #cr::Value) -> bool {
                #mod_ident::#write_fn(self, name, val)
            }
            #[inline]
            fn invoke_method(
                &mut self,
                name: &str,
                args: &[#cr::Value],
            ) -> ::core::option::Option<#cr::Value> {
                #invoke_fn(self, name, args)
            }
            #[inline]
            fn connect_signal(
                &mut self,
                signal: &str,
                callback: #cr::SignalCallback,
                conn_type: #cr::signal::ConnectionType,
            ) -> ::core::option::Option<#cr::ConnectionId> {
                #mod_ident::#connect_fn(self, signal, callback, conn_type)
            }
            #[inline]
            fn emit_signal(
                &mut self,
                signal: &str,
                args: &[#cr::Value],
            ) -> ::core::option::Option<()> {
                #mod_ident::#emit_signal_fn(self, signal, args)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    fn emit(ts: TokenStream) -> String {
        let ir = crate::object_impl::parse::parse(quote! {}, ts).expect("parse ok");
        super::codegen(ir).to_string()
    }

    // No methods — methods static is empty, invoke fn has no arms.
    #[test]
    fn no_methods_emits_empty_static_and_passthrough() {
        let out = emit(quote! {
            impl Foo {
                fn helper(&self) -> i32 { 42 }
            }
        });
        assert!(out.contains("impl Foo"), "missing impl block: {out}");
        assert!(
            out.contains("__METHODS__Foo"),
            "missing methods static: {out}"
        );
        assert!(
            out.contains("impl :: quartzite :: core :: Object for Foo"),
            "missing Object impl: {out}"
        );
        assert!(!out.contains("__arg0"), "unexpected arg binding: {out}");
    }

    // Void-return slot: invoke returns Some(Value::Null).
    #[test]
    fn void_slot_emits_null_return() {
        let out = emit(quote! {
            impl Foo {
                #[slot]
                fn reset(&mut self) {}
            }
        });
        assert!(out.contains("\"reset\""), "missing method name: {out}");
        assert!(out.contains("Value :: Null"), "missing Null return: {out}");
        assert!(
            !out.contains("IntoValue"),
            "unexpected IntoValue for void: {out}"
        );
    }

    // Typed-return invokable: invoke wraps result in IntoValue::into_value.
    #[test]
    fn typed_invokable_emits_into_value() {
        let out = emit(quote! {
            impl Foo {
                #[invokable]
                fn doubled(&self) -> i32 { 0 }
            }
        });
        assert!(out.contains("\"doubled\""), "missing method name: {out}");
        assert!(
            out.contains("IntoValue :: into_value"),
            "missing IntoValue: {out}"
        );
        assert!(
            !out.contains("Value :: Null"),
            "unexpected Null for typed return: {out}"
        );
    }

    // Multi-param method: arity guard and arg bindings present.
    #[test]
    fn multi_param_emits_arg_bindings_and_arity_check() {
        let out = emit(quote! {
            impl Foo {
                #[invokable]
                fn add(&self, a: i32, b: i32) -> i32 { a + b }
            }
        });
        assert!(
            out.contains("args . len () != 2"),
            "missing arity check: {out}"
        );
        assert!(out.contains("__arg0"), "missing __arg0: {out}");
        assert!(out.contains("__arg1"), "missing __arg1: {out}");
        assert!(
            out.contains("FromValue :: from_value"),
            "missing FromValue: {out}"
        );
    }

    // Methods static includes name, param names, and return type string.
    #[test]
    fn methods_static_includes_metadata() {
        let out = emit(quote! {
            impl Foo {
                #[invokable]
                fn compute(&self, x: i32) -> bool { false }
            }
        });
        assert!(
            out.contains("\"compute\""),
            "missing method name in meta: {out}"
        );
        assert!(out.contains("\"x\""), "missing param name in meta: {out}");
        assert!(
            out.contains("stringify ! (i32)"),
            "missing param type: {out}"
        );
        assert!(
            out.contains("stringify ! (bool)"),
            "missing return type: {out}"
        );
    }

    // Default return type is emitted as literal "()".
    #[test]
    fn void_return_type_in_meta_is_unit_str() {
        let out = emit(quote! {
            impl Foo {
                #[slot]
                fn reset(&mut self) {}
            }
        });
        assert!(
            out.contains("\"()\""),
            "missing unit return type string: {out}"
        );
    }

    // Meta static: plain static (no OnceLock), meta_init fn, class_name stringify, mod references.
    #[test]
    fn meta_static_emits_static_and_init_fn() {
        let out = emit(quote! {
            impl Foo {}
        });
        assert!(out.contains("META_Foo"), "missing META_Foo: {out}");
        assert!(
            !out.contains("OnceLock"),
            "unexpected OnceLock — should use plain static: {out}"
        );
        assert!(
            out.contains("__meta_init_Foo"),
            "missing meta_init fn: {out}"
        );
        assert!(
            out.contains("stringify ! (Foo)"),
            "missing class_name stringify: {out}"
        );
        assert!(out.contains("__PROPS__Foo"), "missing props ref: {out}");
        assert!(out.contains("__SIGNALS__Foo"), "missing signals ref: {out}");
        assert!(out.contains("__METHODS__Foo"), "missing methods ref: {out}");
        // Lookup fns are passed to MetaObject::new
        assert!(
            out.contains("__lookup_property_Foo"),
            "missing lookup_property_Foo: {out}"
        );
        assert!(
            out.contains("__lookup_signal_Foo"),
            "missing lookup_signal_Foo: {out}"
        );
        assert!(
            out.contains("__lookup_method_Foo"),
            "missing lookup_method_Foo: {out}"
        );
        assert!(
            out.contains("__lookup_enum_Foo"),
            "missing lookup_enum_Foo: {out}"
        );
    }

    // Method/enum lookup fns are emitted as top-level fns.
    // Property/signal lookup fns live in the hidden mod (generated by #[derive(Object)]).
    #[test]
    fn lookup_fns_are_emitted() {
        let out = emit(quote! {
            impl Foo {
                #[slot]
                fn reset(&mut self) {}
                #[invokable]
                fn doubled(&self) -> i32 { 0 }
            }
        });
        assert!(
            !out.contains("fn __lookup_property_Foo"),
            "prop lookup fn must be in hidden mod, not outer scope: {out}"
        );
        assert!(
            !out.contains("fn __lookup_signal_Foo"),
            "signal lookup fn must be in hidden mod, not outer scope: {out}"
        );
        assert!(
            out.contains("fn __lookup_method_Foo"),
            "missing lookup_method fn: {out}"
        );
        assert!(
            out.contains("fn __lookup_enum_Foo"),
            "missing lookup_enum fn: {out}"
        );
    }

    // MetaObject::new receives prop/signal lookup fns via the hidden mod qualified path.
    #[test]
    fn meta_static_references_hidden_mod_lookup_fns() {
        let out = emit(quote! { impl Foo {} });
        assert!(
            out.contains("__quartzite_Foo :: __lookup_property_Foo"),
            "missing hidden mod prop lookup ref: {out}"
        );
        assert!(
            out.contains("__quartzite_Foo :: __lookup_signal_Foo"),
            "missing hidden mod signal lookup ref: {out}"
        );
    }

    // Lookup method fn contains correct match arms for each registered method.
    #[test]
    fn lookup_method_fn_has_correct_arms() {
        let out = emit(quote! {
            impl Foo {
                #[slot]
                fn reset(&mut self) {}
                #[invokable]
                fn doubled(&self) -> i32 { 0 }
            }
        });
        // Arms should match on method names and index into __METHODS__ static
        assert!(
            out.contains("\"reset\" => :: core :: option :: Option :: Some (__METHODS__Foo [0])"),
            "missing reset arm: {out}"
        );
        assert!(
            out.contains("\"doubled\" => :: core :: option :: Option :: Some (__METHODS__Foo [1])"),
            "missing doubled arm: {out}"
        );
    }

    // impl Object: all six trait method delegations are present.
    #[test]
    fn object_impl_emits_all_six_delegations() {
        let out = emit(quote! {
            impl Foo {}
        });
        assert!(out.contains("fn meta_object"), "missing meta_object: {out}");
        assert!(
            out.contains("fn read_property"),
            "missing read_property: {out}"
        );
        assert!(
            out.contains("fn write_property"),
            "missing write_property: {out}"
        );
        assert!(
            out.contains("fn invoke_method"),
            "missing invoke_method: {out}"
        );
        assert!(
            out.contains("fn connect_signal"),
            "missing connect_signal: {out}"
        );
        assert!(out.contains("fn emit_signal"), "missing emit_signal: {out}");
    }

    // Trait impl block is re-emitted with `impl Trait for Type { … }` header.
    #[test]
    fn trait_impl_reemitted_with_correct_header() {
        let out = emit(quote! {
            impl MyTrait for Foo {
                fn foo(&self) {}
            }
        });
        assert!(
            out.contains("impl MyTrait for Foo"),
            "missing trait impl header: {out}"
        );
        assert!(out.contains("fn foo"), "missing trait method: {out}");
    }

    // Multiple methods (simulating Final merge) all appear in the codegen output.
    #[test]
    fn multiple_methods_all_in_output() {
        let out = emit(quote! {
            impl Foo {
                #[slot]
                fn reset(&mut self) {}
                #[invokable]
                fn doubled(&self) -> i32 { 0 }
            }
        });
        assert!(out.contains("\"reset\""), "missing reset in output: {out}");
        assert!(
            out.contains("\"doubled\""),
            "missing doubled in output: {out}"
        );
        assert!(out.contains("__METHODS__Foo [0]"), "missing index 0: {out}");
        assert!(out.contains("__METHODS__Foo [1]"), "missing index 1: {out}");
        assert!(
            out.contains("impl :: quartzite :: core :: Object for Foo"),
            "missing Object impl: {out}"
        );
    }

    // Non-annotated methods are re-emitted in the impl block (not discarded).
    #[test]
    fn non_annotated_methods_reemitted() {
        let out = emit(quote! {
            impl Foo {
                fn helper(&self) -> i32 { 42 }
            }
        });
        assert!(out.contains("fn helper"), "helper not re-emitted: {out}");
    }

    // AC9: Object trait shims and __meta_init carry #[inline].
    #[test]
    fn object_impl_shims_are_inline() {
        let out = emit(quote! { impl Foo {} });
        let count = out.matches("# [inline]").count();
        // 5 Object trait shims + 1 __meta_init_Foo
        assert!(
            count >= 6,
            "expected >=6 #[inline] tokens, got {count}: {out}"
        );
    }

    // AC9: __meta_init specifically carries #[inline].
    #[test]
    fn meta_init_fn_is_inline() {
        let out = emit(quote! { impl Foo {} });
        assert!(
            out.contains("# [inline] fn __meta_init_Foo"),
            "missing #[inline] on __meta_init_Foo: {out}"
        );
    }
}
