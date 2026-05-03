pub(crate) mod accumulator;
pub(crate) mod codegen;
pub(crate) mod parse;

pub(crate) fn expand(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut ir = match parse::parse(attr, item) {
        Ok(ir) => ir,
        Err(e) => return e.to_compile_error(),
    };

    match ir.kind {
        parse::MethodKind::Sole => codegen::codegen(ir),
        parse::MethodKind::Partial => {
            let type_name = ir.self_ty_ident.to_string();
            let methods = std::mem::take(&mut ir.methods);
            let errors = accumulator::push(&type_name, methods);
            if !errors.is_empty() {
                return errors;
            }
            codegen::codegen_partial(ir)
        }
        parse::MethodKind::Final => {
            let type_name = ir.self_ty_ident.to_string();
            let accumulated = accumulator::drain(&type_name);
            let mut errors = proc_macro2::TokenStream::new();
            for method in &ir.methods {
                if accumulated.iter().any(|m| m.ident == method.ident) {
                    errors.extend(crate::util::emit_compile_error(
                        method.ident.span(),
                        &format!(
                            "duplicate method `{}` across `#[object_impl]` blocks",
                            method.ident
                        ),
                    ));
                }
            }
            if !errors.is_empty() {
                return errors;
            }
            let mut all_methods = accumulated;
            all_methods.append(&mut ir.methods);
            ir.methods = all_methods;
            codegen::codegen(ir)
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    // AC5: duplicate method name between partial block and final block produces compile_error.
    #[test]
    fn partial_to_final_duplicate_produces_error() {
        let partial_out = super::expand(
            quote! { partial },
            quote! {
                impl __TestPFDup1 {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        );
        assert!(
            !partial_out.to_string().contains("compile_error"),
            "unexpected error in partial expand: {partial_out}"
        );
        let final_attr: proc_macro2::TokenStream = "final".parse().unwrap();
        let final_out = super::expand(
            final_attr,
            quote! {
                impl __TestPFDup1 {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        );
        assert!(
            final_out.to_string().contains("compile_error"),
            "expected compile_error for partial→final duplicate: {final_out}"
        );
    }

    // AC4: methods from partial block appear in the final MetaObject output.
    #[test]
    fn final_merges_partial_methods_into_output() {
        super::expand(
            quote! { partial },
            quote! {
                impl __TestMerge {
                    #[slot]
                    fn from_partial(&mut self) {}
                }
            },
        );
        let final_attr: proc_macro2::TokenStream = "final".parse().unwrap();
        let out = super::expand(
            final_attr,
            quote! {
                impl __TestMerge {
                    #[slot]
                    fn from_final(&mut self) {}
                }
            },
        );
        let s = out.to_string();
        assert!(!s.contains("compile_error"), "unexpected error: {s}");
        assert!(
            s.contains("\"from_partial\""),
            "partial method missing from final output: {s}"
        );
        assert!(
            s.contains("\"from_final\""),
            "final method missing from output: {s}"
        );
        assert!(s.contains("META___TestMerge"), "missing MetaObject: {s}");
    }

    // Final expand with no accumulated methods and a new method produces full output (no error).
    #[test]
    fn final_with_no_accumulated_methods_succeeds() {
        let final_attr: proc_macro2::TokenStream = "final".parse().unwrap();
        let out = super::expand(
            final_attr,
            quote! {
                impl __TestFinalNoAcc {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        );
        let s = out.to_string();
        assert!(!s.contains("compile_error"), "unexpected error: {s}");
        assert!(
            s.contains("META___TestFinalNoAcc"),
            "missing MetaObject: {s}"
        );
    }
}
