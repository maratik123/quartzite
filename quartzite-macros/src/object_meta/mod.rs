mod codegen;
mod parse;

pub(crate) fn expand(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let ir = match parse::parse(item) {
        Ok(ir) => ir,
        Err(e) => return e.to_compile_error(),
    };
    let type_name = ir.self_ty_ident.to_string();
    let methods = crate::object_impl::accumulator::drain(&type_name);
    codegen::codegen_meta(ir, methods)
}

#[cfg(test)]
mod tests {
    use quote::quote;

    // Full path: partial block accumulates, then #[object_meta] drains and emits MetaObject.
    #[test]
    fn object_meta_emits_accumulated_methods() {
        crate::object_impl::expand(
            quote! { partial },
            quote! {
                impl __TestOMeta {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        );
        let out = super::expand(quote! { impl __TestOMeta {} });
        let s = out.to_string();
        assert!(!s.contains("compile_error"), "unexpected error: {s}");
        assert!(s.contains("META___TestOMeta"), "missing MetaObject: {s}");
        assert!(
            s.contains("\"reset\""),
            "accumulated method missing from output: {s}"
        );
        assert!(
            !s.contains("impl __TestOMeta {}"),
            "bare impl block should not be re-emitted: {s}"
        );
    }

    // object_meta with empty accumulator emits an empty MetaObject.
    #[test]
    fn object_meta_empty_accumulator_succeeds() {
        let out = super::expand(quote! { impl __TestOMetaEmpty {} });
        let s = out.to_string();
        assert!(!s.contains("compile_error"), "unexpected error: {s}");
        assert!(
            s.contains("META___TestOMetaEmpty"),
            "missing MetaObject: {s}"
        );
    }
}
