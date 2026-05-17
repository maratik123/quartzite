mod codegen;
mod parse;

pub(crate) fn expand(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut ir = match parse::parse(attr, item) {
        Ok(ir) => ir,
        Err(e) => return e.to_compile_error(),
    };

    let type_name = ir.self_ty_ident.to_string();
    let methods = std::mem::take(&mut ir.methods);
    let errors = crate::object_impl::accumulator::push(&type_name, methods);
    if !errors.is_empty() {
        return errors;
    }
    codegen::codegen(&ir)
}

#[cfg(test)]
mod tests {
    use quote::quote;

    // AC1+AC2: expand emits only the cleaned impl block and pushes methods to accumulator.
    #[test]
    fn expand_emits_impl_block_and_accumulates() {
        let out = super::expand(
            quote! {},
            quote! {
                impl __TestPart__ {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        );
        let s = out.to_string();
        assert!(!s.contains("compile_error"), "unexpected error: {s}");
        assert!(s.contains("impl __TestPart__"), "missing impl block: {s}");
        assert!(
            !s.contains("META___TestPart__"),
            "unexpected MetaObject: {s}"
        );
        // accumulator now holds the method
        assert!(
            crate::object_impl::accumulator::peek("__TestPart__"),
            "peek should return true after expand"
        );
        // clean up
        crate::object_impl::accumulator::drain("__TestPart__");
    }

    // AC8: non-empty attr → compile_error.
    #[test]
    fn non_empty_attr_produces_error() {
        let out = super::expand(quote! { something }, quote! { impl __TestPartAttr__ {} });
        let s = out.to_string();
        assert!(
            s.contains("compile_error"),
            "expected compile_error for non-empty attr: {s}"
        );
    }

    // AC5: duplicate method name across two object_part blocks → compile_error from second push.
    #[test]
    fn duplicate_across_two_parts_produces_error() {
        let out1 = super::expand(
            quote! {},
            quote! {
                impl __TestPartDup__ {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        );
        assert!(
            !out1.to_string().contains("compile_error"),
            "unexpected error on first part: {out1}"
        );
        let out2 = super::expand(
            quote! {},
            quote! {
                impl __TestPartDup__ {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        );
        assert!(
            out2.to_string().contains("compile_error"),
            "expected compile_error for duplicate across parts: {out2}"
        );
        // clean up (first push succeeded; second failed but did not push again)
        crate::object_impl::accumulator::drain("__TestPartDup__");
    }
}
