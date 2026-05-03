use proc_macro2::TokenStream;
use quote::quote;

use super::parse::ObjectMetaInput;
use crate::object_impl::{
    codegen::{
        emit_invoke_method, emit_lookup_fns, emit_meta_static, emit_methods_static,
        emit_object_impl,
    },
    parse::MethodItem,
};
use crate::util::hidden_mod_ident;

/// Generates MetaObject statics and `impl Object` for the type, merging accumulated methods.
/// The original empty impl block is intentionally not re-emitted.
pub(crate) fn codegen_meta(ir: ObjectMetaInput, methods: Vec<MethodItem>) -> TokenStream {
    let type_ident = &ir.self_ty_ident;
    let self_ty = &ir.self_ty;
    let mod_ident = hidden_mod_ident(type_ident);

    let methods_static = emit_methods_static(type_ident, &methods);
    let invoke_fn = emit_invoke_method(type_ident, &methods);
    let lookup_fns = emit_lookup_fns(type_ident, &methods);
    let meta_static = emit_meta_static(type_ident, &mod_ident);
    let object_impl = emit_object_impl(self_ty, type_ident, &mod_ident);

    quote! {
        #methods_static
        #invoke_fn
        #lookup_fns
        #meta_static
        #object_impl
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use quote::quote;
    use syn::Ident;

    use super::*;
    use crate::object_impl::parse::{MethodItem, ParamMeta};

    fn make_method(name: &str) -> MethodItem {
        MethodItem {
            ident: Ident::new(name, Span::call_site()),
            params: vec![],
            ret_ty: syn::ReturnType::Default,
        }
    }

    fn make_method_with_param(name: &str, param: &str) -> MethodItem {
        MethodItem {
            ident: Ident::new(name, Span::call_site()),
            params: vec![ParamMeta {
                ident: Ident::new(param, Span::call_site()),
                ty: syn::parse2(quote! { i32 }).unwrap(),
            }],
            ret_ty: syn::ReturnType::Default,
        }
    }

    fn emit_meta(type_ts: proc_macro2::TokenStream, methods: Vec<MethodItem>) -> String {
        let ir = crate::object_meta::parse::parse(type_ts).expect("parse ok");
        codegen_meta(ir, methods).to_string()
    }

    // MetaObject static and impl Object are emitted.
    #[test]
    fn emits_meta_object_and_object_impl() {
        let out = emit_meta(quote! { impl Counter {} }, vec![]);
        assert!(out.contains("META_Counter"), "missing meta: {out}");
        assert!(
            out.contains("impl :: quartzite :: core :: Object for Counter"),
            "missing Object impl: {out}"
        );
    }

    // Methods from accumulator appear in the output.
    #[test]
    fn methods_appear_in_output() {
        let out = emit_meta(
            quote! { impl Counter {} },
            vec![make_method("reset"), make_method_with_param("add", "n")],
        );
        assert!(out.contains("\"reset\""), "missing reset: {out}");
        assert!(out.contains("\"add\""), "missing add: {out}");
    }

    // No bare inherent impl block re-emitted (the empty original is discarded).
    #[test]
    fn no_bare_impl_block_emitted() {
        let out = emit_meta(quote! { impl Counter {} }, vec![]);
        assert!(
            !out.contains("impl Counter {"),
            "unexpected inherent impl block: {out}"
        );
    }
}
