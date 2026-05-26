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

    let type_name = ir.self_ty_ident.to_string();

    if accumulator::peek(&type_name) {
        let accumulated = accumulator::drain(&type_name);
        let mut errors = proc_macro2::TokenStream::new();
        for method in &ir.methods {
            if accumulated.iter().any(|m| m.ident == method.ident) {
                errors.extend(crate::util::emit_compile_error(
                    method.ident.span(),
                    &format!(
                        "duplicate method `{}` across `#[object_part]`/`#[object_impl]` blocks",
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
    }
    codegen::codegen(&ir)
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use quote::quote;
    use syn::Ident;

    use crate::object_impl::parse::MethodItem;

    fn make_method(name: &str) -> MethodItem {
        MethodItem {
            ident: Ident::new(name, Span::call_site()),
            params: vec![],
            ret_ty: syn::ReturnType::Default,
            doc_present: false,
            per_item_level: None,
        }
    }

    // AC3: empty accumulator → sole mode → full output with MetaObject and impl Object.
    #[test]
    fn sole_mode_emits_full_output() {
        let out = super::expand(
            quote! {},
            quote! {
                impl __TestSole__ {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        );
        let s = out.to_string();
        assert!(!s.contains("compile_error"), "unexpected error: {s}");
        assert!(s.contains("META___TestSole__"), "missing MetaObject: {s}");
        assert!(
            s.contains("impl :: quartzite :: core :: Object for __TestSole__"),
            "missing Object impl: {s}"
        );
    }

    // AC4: non-empty accumulator → terminal mode → accumulated + current methods in output.
    #[test]
    fn terminal_mode_merges_accumulated_methods() {
        let type_name = "__TestTerminalMerge__";
        super::accumulator::push(type_name, vec![make_method("from_part")]);

        let out = super::expand(
            quote! {},
            quote! {
                impl __TestTerminalMerge__ {
                    #[slot]
                    fn from_impl(&mut self) {}
                }
            },
        );
        let s = out.to_string();
        assert!(!s.contains("compile_error"), "unexpected error: {s}");
        assert!(s.contains("\"from_part\""), "missing from_part: {s}");
        assert!(s.contains("\"from_impl\""), "missing from_impl: {s}");
        assert!(
            s.contains("META___TestTerminalMerge__"),
            "missing MetaObject: {s}"
        );
    }

    // AC6: duplicate method name between object_part accumulated and object_impl terminal.
    #[test]
    fn terminal_mode_duplicate_produces_error() {
        let type_name = "__TestTerminalDup__";
        super::accumulator::push(type_name, vec![make_method("reset")]);

        let out = super::expand(
            quote! {},
            quote! {
                impl __TestTerminalDup__ {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        );
        assert!(
            out.to_string().contains("compile_error"),
            "expected compile_error for terminal duplicate: {out}"
        );
    }

    // AC7: non-empty attr → compile_error mentioning #[object_part].
    #[test]
    fn non_empty_attr_produces_error_with_object_part_hint() {
        let out = super::expand(quote! { partial }, quote! { impl __TestAttrErr__ {} });
        let s = out.to_string();
        assert!(
            s.contains("compile_error"),
            "expected compile_error for non-empty attr: {s}"
        );
        assert!(
            s.contains("object_part"),
            "error should mention object_part: {s}"
        );
    }
}
